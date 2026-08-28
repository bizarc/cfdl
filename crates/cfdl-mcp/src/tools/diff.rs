//! `diff`: results x expectations -> first divergence and per-period deltas.
//!
//! A Rust port of the comparison in `tools/benchmark-runner.py`, kept
//! semantically identical: same column resolution, same absolute tolerances,
//! same null discipline (a stated cell against a null series value fails; a
//! blank cell asserts nothing). One deliberate difference: the runner stops
//! printing after six failures, this reports up to [`MAX_FAILURES`] with a
//! `truncated` flag — pass/fail is unaffected.

use std::collections::BTreeMap;
use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const MAX_FAILURES: usize = 50;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct DiffParams {
    /// The results document (docs/06), inline. Alternative to `results_path`.
    #[serde(default)]
    pub results: Option<serde_json::Value>,
    /// Path to a results JSON file.
    #[serde(default)]
    pub results_path: Option<String>,
    /// A benchmark case directory: uses its `expected.csv`,
    /// `expected_metrics.json`, optional `expected_scenarios.json` /
    /// `expected_monte_carlo.json`, and `case.toml` tolerances.
    #[serde(default)]
    pub case_dir: Option<String>,
    /// Standalone per-period expectations CSV (header row: `period` or `year`
    /// plus series columns). Alternative to `case_dir`.
    #[serde(default)]
    pub expected_csv_path: Option<String>,
    /// Standalone metric expectations: `{metric: {value, tolerance}}`.
    #[serde(default)]
    pub expected_metrics_path: Option<String>,
    /// Per-period tolerance when `case.toml` / `tolerances` do not say (default 0.01).
    #[serde(default)]
    pub default_tolerance: Option<f64>,
    /// Per-column tolerance overrides (merged over the case's `[tolerance]` table).
    #[serde(default)]
    pub tolerances: Option<BTreeMap<String, f64>>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DiffFailure {
    /// `resolve` | `warnings` | `period` | `null` | `range` | `metric` |
    /// `scenario` | `monte_carlo`
    pub kind: String,
    /// The column, metric, or scenario the failure is about.
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub got: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tolerance: Option<f64>,
    pub message: String,
}

#[derive(Debug, Default, Serialize, JsonSchema)]
pub struct DiffChecked {
    pub rows: u64,
    pub columns: Vec<String>,
    pub metrics: u64,
    pub scenarios: u64,
    pub monte_carlo: u64,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct DiffResult {
    pub pass: bool,
    /// The first per-period divergence in row order — where to start reading.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_divergence: Option<DiffFailure>,
    pub failures: Vec<DiffFailure>,
    pub truncated: bool,
    pub checked: DiffChecked,
}

struct Comparison {
    failures: Vec<DiffFailure>,
    truncated: bool,
    checked: DiffChecked,
}

impl Comparison {
    fn push(&mut self, failure: DiffFailure) -> bool {
        if self.failures.len() >= MAX_FAILURES {
            self.truncated = true;
            return false;
        }
        self.failures.push(failure);
        true
    }
}

pub fn diff(params: &DiffParams, defaults: &super::Defaults) -> Result<DiffResult, String> {
    let _ = defaults;
    let results = super::load_results(params.results.clone(), params.results_path.as_deref())?;
    let case_dir = params.case_dir.as_ref().map(Path::new);

    // Tolerances: case.toml supplies the default and the per-column table;
    // explicit params override both.
    let mut default_tolerance = 0.01_f64;
    let mut per_column: BTreeMap<String, f64> = BTreeMap::new();
    let mut case_pack_note = None;
    if let Some(dir) = case_dir {
        let case_toml = dir.join("case.toml");
        let raw = std::fs::read_to_string(&case_toml)
            .map_err(|err| format!("cannot read '{}': {err}", case_toml.display()))?;
        let case: toml::Value = raw
            .parse()
            .map_err(|err| format!("invalid case.toml: {err}"))?;
        if let Some(tol) = case.get("period_tolerance").and_then(toml_f64) {
            default_tolerance = tol;
        }
        if let Some(table) = case.get("tolerance").and_then(|t| t.as_table()) {
            for (key, value) in table {
                if let Some(tol) = toml_f64(value) {
                    per_column.insert(key.clone(), tol);
                }
            }
        }
        case_pack_note = case
            .get("pack")
            .and_then(|p| p.as_str())
            .map(str::to_string);
    }
    if let Some(tol) = params.default_tolerance {
        default_tolerance = tol;
    }
    if let Some(overrides) = &params.tolerances {
        per_column.extend(overrides.iter().map(|(k, v)| (k.clone(), *v)));
    }
    let _ = case_pack_note;

    let mut cmp = Comparison {
        failures: Vec::new(),
        truncated: false,
        checked: DiffChecked::default(),
    };

    // A warned run is a failed comparison, as in the harness.
    if let Some(warnings) = results["warnings"].as_array() {
        if !warnings.is_empty() {
            let sample: Vec<String> = warnings
                .iter()
                .take(3)
                .filter_map(|w| w.as_str().map(str::to_string))
                .collect();
            cmp.push(DiffFailure {
                kind: "warnings".to_string(),
                label: "engine".to_string(),
                period: None,
                got: None,
                expected: None,
                tolerance: None,
                message: format!("engine warnings: {sample:?}"),
            });
        }
    }

    let expected_csv = params
        .expected_csv_path
        .clone()
        .or_else(|| case_dir.map(|d| d.join("expected.csv").to_string_lossy().into_owned()));
    if let Some(csv_path) = expected_csv {
        compare_periods(
            &mut cmp,
            &results,
            &csv_path,
            default_tolerance,
            &per_column,
        )?;
    }

    if let Some(dir) = case_dir {
        let scenarios = dir.join("expected_scenarios.json");
        if scenarios.exists() {
            compare_scenarios(&mut cmp, &results, &scenarios)?;
        }
        let monte_carlo = dir.join("expected_monte_carlo.json");
        if monte_carlo.exists() {
            compare_monte_carlo(&mut cmp, &results, &monte_carlo)?;
        }
    }

    let expected_metrics = params.expected_metrics_path.clone().or_else(|| {
        case_dir.and_then(|d| {
            let path = d.join("expected_metrics.json");
            path.exists().then(|| path.to_string_lossy().into_owned())
        })
    });
    if let Some(metrics_path) = expected_metrics {
        compare_metrics(&mut cmp, &results, &metrics_path)?;
    }

    let first_divergence = cmp
        .failures
        .iter()
        .find(|f| matches!(f.kind.as_str(), "period" | "null" | "range"))
        .cloned();
    Ok(DiffResult {
        pass: cmp.failures.is_empty(),
        first_divergence,
        failures: cmp.failures,
        truncated: cmp.truncated,
        checked: cmp.checked,
    })
}

fn toml_f64(value: &toml::Value) -> Option<f64> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|i| i as f64))
}

/// Map expected.csv headers onto result series, as the harness does: a header
/// naming a series verbatim wins; `net_cash_flow` is the model total; anything
/// else is a stream id, matched against `stream.<column>`.
fn resolve_column(column: &str, series: &serde_json::Map<String, serde_json::Value>) -> String {
    if series.contains_key(column) {
        column.to_string()
    } else if column == "net_cash_flow" {
        "model.net_cash_flow".to_string()
    } else {
        format!("stream.{column}")
    }
}

fn compare_periods(
    cmp: &mut Comparison,
    results: &serde_json::Value,
    csv_path: &str,
    default_tolerance: f64,
    per_column: &BTreeMap<String, f64>,
) -> Result<(), String> {
    let Some(series) = results["deterministic"]["series"].as_object() else {
        return Err("results carry no `deterministic.series`".to_string());
    };
    let raw = std::fs::read_to_string(csv_path)
        .map_err(|err| format!("cannot read '{csv_path}': {err}"))?;
    // The expectation files are plain comma-separated numbers with a header
    // row — no quoting, which is why this does not need a CSV crate.
    let mut lines = raw.lines().filter(|line| !line.trim().is_empty());
    let Some(header) = lines.next() else {
        return Err(format!("'{csv_path}' is empty"));
    };
    let fields: Vec<&str> = header.split(',').map(str::trim).collect();
    let Some(index_pos) = fields.iter().position(|f| *f == "period" || *f == "year") else {
        cmp.push(DiffFailure {
            kind: "resolve".to_string(),
            label: "expected.csv".to_string(),
            period: None,
            got: None,
            expected: None,
            tolerance: None,
            message: "expected.csv: need a 'period' or 'year' column".to_string(),
        });
        return Ok(());
    };
    let by_period = fields[index_pos] == "period";

    let mut columns: Vec<(usize, String, String)> = Vec::new(); // (csv pos, column, series key)
    for (pos, column) in fields.iter().enumerate() {
        if pos == index_pos {
            continue;
        }
        let key = resolve_column(column, series);
        if !series.contains_key(&key) {
            cmp.push(DiffFailure {
                kind: "resolve".to_string(),
                label: (*column).to_string(),
                period: None,
                got: None,
                expected: None,
                tolerance: None,
                message: format!("expected.csv column '{column}': no series '{key}' in results"),
            });
            continue;
        }
        columns.push((pos, (*column).to_string(), key));
    }
    if columns.is_empty() {
        cmp.push(DiffFailure {
            kind: "resolve".to_string(),
            label: "expected.csv".to_string(),
            period: None,
            got: None,
            expected: None,
            tolerance: None,
            message: "expected.csv: no columns to check".to_string(),
        });
        return Ok(());
    }
    cmp.checked.columns = columns.iter().map(|(_, c, _)| c.clone()).collect();

    let values: BTreeMap<&str, Vec<Option<f64>>> = columns
        .iter()
        .map(|(_, _, key)| {
            let points = series[key]["values"]
                .as_array()
                .map(|vals| vals.iter().map(super::scalar).collect())
                .unwrap_or_default();
            (key.as_str(), points)
        })
        .collect();

    for (row_no, line) in lines.enumerate() {
        cmp.checked.rows += 1;
        let cells: Vec<&str> = line.split(',').map(str::trim).collect();
        let t: u64 = if by_period {
            cells
                .get(index_pos)
                .and_then(|c| c.parse().ok())
                .ok_or_else(|| format!("'{csv_path}' row {}: bad period index", row_no + 1))?
        } else {
            row_no as u64
        };
        for (pos, column, key) in &columns {
            let cell = cells.get(*pos).copied().unwrap_or("");
            if cell.is_empty() {
                continue; // blank means "not asserted"
            }
            let expected: f64 = cell.parse().map_err(|_| {
                format!("'{csv_path}' row {}: '{cell}' is not a number", row_no + 1)
            })?;
            let actual = &values[key.as_str()];
            if t as usize >= actual.len() {
                cmp.push(DiffFailure {
                    kind: "range".to_string(),
                    label: column.clone(),
                    period: Some(t),
                    got: None,
                    expected: Some(expected),
                    tolerance: None,
                    message: format!(
                        "{column} period {t}: beyond the {}-period timeline",
                        actual.len()
                    ),
                });
                return Ok(());
            }
            let tolerance = per_column.get(column).copied().unwrap_or(default_tolerance);
            match actual[t as usize] {
                None => {
                    // The CSV states a value the results say is undefined.
                    if !cmp.push(DiffFailure {
                        kind: "null".to_string(),
                        label: column.clone(),
                        period: Some(t),
                        got: None,
                        expected: Some(expected),
                        tolerance: Some(tolerance),
                        message: format!(
                            "{column} period {t}: series is null (undefined here) but {expected} was expected"
                        ),
                    }) {
                        return Ok(());
                    }
                }
                Some(got) if (got - expected).abs() > tolerance => {
                    if !cmp.push(DiffFailure {
                        kind: "period".to_string(),
                        label: column.clone(),
                        period: Some(t),
                        got: Some(got),
                        expected: Some(expected),
                        tolerance: Some(tolerance),
                        message: format!(
                            "{column} period {t}: {got:.6} vs expected {expected:.6} (|diff| {:.6} > {tolerance})",
                            (got - expected).abs()
                        ),
                    }) {
                        return Ok(());
                    }
                }
                Some(_) => {}
            }
        }
    }
    Ok(())
}

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let raw = std::fs::read_to_string(path)
        .map_err(|err| format!("cannot read '{}': {err}", path.display()))?;
    serde_json::from_str(&raw).map_err(|err| format!("'{}': {err}", path.display()))
}

fn compare_metrics(
    cmp: &mut Comparison,
    results: &serde_json::Value,
    metrics_path: &str,
) -> Result<(), String> {
    let expected = read_json(Path::new(metrics_path))?;
    let mut metrics = serde_json::Map::new();
    for source in [
        results["deterministic"]["metrics"].as_object(),
        results["domain_metrics"]["metrics"].as_object(),
    ]
    .into_iter()
    .flatten()
    {
        metrics.extend(source.clone());
    }
    let Some(wanted) = expected.as_object() else {
        return Err(format!("'{metrics_path}': expected an object"));
    };
    for (key, spec) in wanted {
        cmp.checked.metrics += 1;
        compare_spec(cmp, "metric", key, metrics.get(key), spec);
    }
    Ok(())
}

fn compare_scenarios(
    cmp: &mut Comparison,
    results: &serde_json::Value,
    path: &Path,
) -> Result<(), String> {
    let expected = read_json(path)?;
    let mut summaries: BTreeMap<&str, &serde_json::Value> = BTreeMap::new();
    if let Some(items) = results["scenarios"]["summaries"].as_array() {
        for item in items {
            if let Some(name) = item["name"].as_str() {
                summaries.insert(name, &item["metrics"]);
            }
        }
    }
    for (name, wanted) in expected.as_object().into_iter().flatten() {
        let Some(metrics) = summaries.get(name.as_str()) else {
            cmp.push(DiffFailure {
                kind: "scenario".to_string(),
                label: name.clone(),
                period: None,
                got: None,
                expected: None,
                tolerance: None,
                message: format!("scenario '{name}': not in results"),
            });
            continue;
        };
        for (key, spec) in wanted.as_object().into_iter().flatten() {
            cmp.checked.scenarios += 1;
            let label = format!("{name}.{key}");
            compare_spec(cmp, "scenario", &label, metrics.get(key), spec);
        }
    }
    Ok(())
}

fn compare_monte_carlo(
    cmp: &mut Comparison,
    results: &serde_json::Value,
    path: &Path,
) -> Result<(), String> {
    let expected = read_json(path)?;
    let mc = &results["monte_carlo"];
    if mc["status"].as_str() != Some("ok") {
        cmp.push(DiffFailure {
            kind: "monte_carlo".to_string(),
            label: "status".to_string(),
            period: None,
            got: None,
            expected: None,
            tolerance: None,
            message: format!("monte carlo: run status {:?}", mc["status"].as_str()),
        });
    }
    for (key, wanted) in expected.as_object().into_iter().flatten() {
        for (agg, spec) in wanted.as_object().into_iter().flatten() {
            cmp.checked.monte_carlo += 1;
            let label = format!("{key}.{agg}");
            compare_spec(
                cmp,
                "monte_carlo",
                &label,
                mc["metrics"][key].get(agg),
                spec,
            );
        }
    }
    Ok(())
}

/// One `{value, tolerance}` expectation against a result value.
fn compare_spec(
    cmp: &mut Comparison,
    kind: &str,
    label: &str,
    got: Option<&serde_json::Value>,
    spec: &serde_json::Value,
) {
    let (Some(value), Some(tolerance)) = (spec["value"].as_f64(), spec["tolerance"].as_f64())
    else {
        cmp.push(DiffFailure {
            kind: kind.to_string(),
            label: label.to_string(),
            period: None,
            got: None,
            expected: None,
            tolerance: None,
            message: format!("{kind} {label}: expectation needs `value` and `tolerance`"),
        });
        return;
    };
    let Some(got) = got.and_then(super::scalar) else {
        cmp.push(DiffFailure {
            kind: kind.to_string(),
            label: label.to_string(),
            period: None,
            got: None,
            expected: Some(value),
            tolerance: Some(tolerance),
            message: format!("{kind} {label}: missing from results"),
        });
        return;
    };
    if (got - value).abs() > tolerance {
        cmp.push(DiffFailure {
            kind: kind.to_string(),
            label: label.to_string(),
            period: None,
            got: Some(got),
            expected: Some(value),
            tolerance: Some(tolerance),
            message: format!("{kind} {label}: {got} vs expected {value} (tolerance {tolerance})"),
        });
    }
}
