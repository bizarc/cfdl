#![allow(clippy::useless_conversion)]

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::path::{Path, PathBuf};

#[pyfunction]
#[pyo3(signature = (model_dir, packs_dir=None))]
#[allow(clippy::useless_conversion)]
fn compile_model(model_dir: String, packs_dir: Option<String>) -> PyResult<String> {
    let options = cfdl_compile::CompileOptions {
        packs_dir: packs_dir.map(PathBuf::from),
    };
    match cfdl_compile::compile_to_json_with_options(Path::new(&model_dir), &options) {
        Ok(json) => Ok(json),
        Err(diags) => {
            let rendered = serde_json::to_string_pretty(&diags).unwrap_or_else(|_| {
                format!("Compilation failed with {} diagnostic(s).", diags.len())
            });
            Err(PyRuntimeError::new_err(rendered))
        }
    }
}

/// Run compiled IR. Mirrors `cfdl run`: `rate`/`as_of` are the fallback
/// discount rate and valuation date when the config omits them; `pack`
/// applies that pack's declarative domain metrics to the results.
#[pyfunction]
#[pyo3(signature = (ir_json, packs_dir=None, config_json=None, rate=0.0, as_of=None, pack=None))]
#[allow(clippy::useless_conversion)]
fn run_ir(
    ir_json: String,
    packs_dir: Option<String>,
    config_json: Option<String>,
    rate: f64,
    as_of: Option<String>,
    pack: Option<String>,
) -> PyResult<String> {
    let mut registry: Option<cfdl_pack::PackRegistry> = None;
    if let Some(pack_dir) = packs_dir {
        match cfdl_pack::PackRegistry::load_from_dir(Path::new(&pack_dir)) {
            Ok(loaded) => registry = Some(loaded),
            Err(err) => return Err(PyRuntimeError::new_err(err.message)),
        }
    }

    let parsed_as_of = match as_of {
        Some(raw) => match cfdl_engine::Date::parse(&raw) {
            Ok(date) => Some(date),
            Err(_) => {
                return Err(PyRuntimeError::new_err(format!(
                    "Invalid as_of value '{raw}', expected YYYY-MM-DD."
                )))
            }
        },
        None => None,
    };

    let run_config = if let Some(config) = config_json {
        let config_path = Path::new(&config);
        let parsed = if config_path.is_file() {
            cfdl_engine::run_config_from_json_file(config_path, rate, parsed_as_of)
        } else {
            cfdl_engine::run_config_from_json_str(&config, rate, parsed_as_of)
        };
        match parsed {
            Ok(value) => value,
            Err(err) => return Err(PyRuntimeError::new_err(err.to_string())),
        }
    } else {
        cfdl_engine::RunConfig {
            discount_rate: rate,
            as_of: parsed_as_of,
            ..Default::default()
        }
    };

    let mut results = match cfdl_engine::run_from_json_str(&ir_json, run_config) {
        Ok(value) => value,
        Err(err) => return Err(PyRuntimeError::new_err(err.to_string())),
    };
    if let Some(pack_name) = pack {
        let specs = registry
            .as_ref()
            .map(|reg| reg.metric_specs(&pack_name))
            .unwrap_or_default();
        results.domain_metrics = cfdl_metrics::compute(&pack_name, &specs, &results);
    }
    match serde_json::to_string_pretty(&results) {
        Ok(json) => Ok(json),
        Err(err) => Err(PyRuntimeError::new_err(format!(
            "Failed to serialize results JSON: {err}"
        ))),
    }
}

#[pymodule]
fn _native(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compile_model, m)?)?;
    m.add_function(wrap_pyfunction!(run_ir, m)?)?;
    Ok(())
}
