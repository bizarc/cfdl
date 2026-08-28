//! `run`: source or IR + run configuration -> results per docs/06,
//! enriched (domain metrics, statements) through the shared cfdl-run facade.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Defaults;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct RunParams {
    /// Directory containing `model.cfdl`. Alternative to `files` / `ir`.
    #[serde(default)]
    pub model_dir: Option<String>,
    /// In-memory sources: root-relative path -> source text.
    #[serde(default)]
    pub files: Option<BTreeMap<String, String>>,
    /// Entry module within `files` (default `model.cfdl`).
    #[serde(default)]
    pub root_file: Option<String>,
    /// Pre-compiled IR JSON. When absent, sources are compiled first.
    #[serde(default)]
    pub ir: Option<String>,
    /// Run configuration, same shape as a `run.json` (docs/schemas/run.schema.json).
    #[serde(default)]
    pub config: Option<serde_json::Value>,
    /// Path to a `run.json` file. Alternative to `config`.
    #[serde(default)]
    pub config_path: Option<String>,
    /// Fallback annual discount rate when the config omits one.
    #[serde(default)]
    pub rate: Option<f64>,
    /// Domain pack for post-engine metrics and statements (e.g. `"cre"`).
    #[serde(default)]
    pub pack: Option<String>,
    /// Pack directory override (as in `compile`).
    #[serde(default)]
    pub packs_dir: Option<String>,
    /// Write the full results JSON here. When set, the response carries the
    /// summary only, which is what a diff/explain loop needs.
    #[serde(default)]
    pub out: Option<String>,
    /// Force the full results document into the response even with `out` set.
    #[serde(default)]
    pub include_results: Option<bool>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct RunResult {
    pub ok: bool,
    /// Compile diagnostics (docs/08), when compilation failed.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<serde_json::Value>,
    /// Engine or run-config error, when the run itself failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Engine warnings (a warned run is suspect: the benchmark harness fails on them).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    /// Grid periods, from the first series index.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub periods: Option<u64>,
    /// Every series key in `deterministic.series` (what `diff`/`explain` address).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub series: Vec<String>,
    /// `deterministic.metrics` merged with `domain_metrics.metrics`.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub metrics: BTreeMap<String, serde_json::Value>,
    /// Where the full results were written, when `out` was set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out: Option<String>,
    /// The full results document (docs/06), when requested or no `out` given.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<serde_json::Value>,
}

impl RunResult {
    fn failed(diagnostics: Vec<serde_json::Value>, error: Option<String>) -> Self {
        Self {
            ok: false,
            diagnostics,
            error,
            warnings: Vec::new(),
            periods: None,
            series: Vec::new(),
            metrics: BTreeMap::new(),
            out: None,
            results: None,
        }
    }
}

pub fn run(params: &RunParams, defaults: &Defaults) -> Result<RunResult, String> {
    // Obtain IR: use provided IR, else compile.
    let ir_json = match &params.ir {
        Some(ir) => ir.clone(),
        None => match super::compile::compile_ir(
            params.model_dir.as_deref(),
            params.files.as_ref(),
            params.root_file.as_deref(),
            params.packs_dir.as_deref(),
            defaults,
        )
        .map_err(|err| format!("run requires `ir`, `model_dir`, or `files` ({err})"))?
        {
            Ok(ir) => ir,
            Err(diags) => {
                let diagnostics = diags
                    .iter()
                    .map(|d| serde_json::to_value(d).unwrap_or(serde_json::Value::Null))
                    .collect();
                return Ok(RunResult::failed(diagnostics, None));
            }
        },
    };

    // Run configuration: a file, an inline value, or the fallback rate alone.
    let rate = params.rate.unwrap_or(0.0);
    let raw_config = match (&params.config, &params.config_path) {
        (Some(_), Some(_)) => return Err("pass `config` or `config_path`, not both".to_string()),
        (Some(value), None) => Some(value.to_string()),
        (None, Some(path)) => Some(
            std::fs::read_to_string(path)
                .map_err(|err| format!("cannot read run config '{path}': {err}"))?,
        ),
        (None, None) => None,
    };
    let config = match raw_config {
        Some(raw) => match cfdl_engine::run_config_from_json_str(&raw, rate, None) {
            Ok(config) => config,
            Err(err) => {
                return Ok(RunResult::failed(
                    Vec::new(),
                    Some(format!("invalid run configuration: {err}")),
                ))
            }
        },
        None => cfdl_engine::RunConfig {
            discount_rate: rate,
            ..Default::default()
        },
    };

    let registry = params.pack.as_ref().and_then(|_| {
        super::load_registry(
            super::resolve_packs_dir(params.packs_dir.as_deref(), defaults).as_deref(),
        )
        .ok()
    });
    let results =
        match cfdl_run::run_enriched(&ir_json, config, params.pack.as_deref(), registry.as_ref()) {
            Ok(results) => results,
            Err(err) => return Ok(RunResult::failed(Vec::new(), Some(err.to_string()))),
        };
    let value =
        serde_json::to_value(&results).map_err(|err| format!("serializing results: {err}"))?;

    if let Some(out) = &params.out {
        let pretty = serde_json::to_string_pretty(&value)
            .map_err(|err| format!("serializing results: {err}"))?;
        std::fs::write(out, pretty)
            .map_err(|err| format!("cannot write results '{out}': {err}"))?;
    }

    Ok(summarize(value, params))
}

fn summarize(value: serde_json::Value, params: &RunParams) -> RunResult {
    let warnings = value["warnings"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|w| w.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let series_map = value["deterministic"]["series"].as_object();
    let series: Vec<String> = series_map
        .map(|map| map.keys().cloned().collect())
        .unwrap_or_default();
    let periods = series_map
        .and_then(|map| map.values().next())
        .and_then(|s| s["index"]["periods"].as_u64());
    let mut metrics = BTreeMap::new();
    for source in [
        value["deterministic"]["metrics"].as_object(),
        value["domain_metrics"]["metrics"].as_object(),
    ]
    .into_iter()
    .flatten()
    {
        for (key, metric) in source {
            metrics.insert(key.clone(), metric.clone());
        }
    }
    let include_results = params.include_results.unwrap_or(params.out.is_none());
    RunResult {
        ok: true,
        diagnostics: Vec::new(),
        error: None,
        warnings,
        periods,
        series,
        metrics,
        out: params.out.clone(),
        results: include_results.then_some(value),
    }
}
