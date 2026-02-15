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

#[pyfunction]
#[pyo3(signature = (ir_json, packs_dir=None, config_json=None))]
#[allow(clippy::useless_conversion)]
fn run_ir(
    ir_json: String,
    packs_dir: Option<String>,
    config_json: Option<String>,
) -> PyResult<String> {
    if let Some(pack_dir) = packs_dir {
        if let Err(err) = cfdl_pack::PackRegistry::load_from_dir(Path::new(&pack_dir)) {
            return Err(PyRuntimeError::new_err(err.message));
        }
    }

    let run_config = if let Some(config) = config_json {
        let config_path = Path::new(&config);
        let parsed = if config_path.is_file() {
            cfdl_engine::run_config_from_json_file(config_path, 0.0, None)
        } else {
            cfdl_engine::run_config_from_json_str(&config, 0.0, None)
        };
        match parsed {
            Ok(value) => value,
            Err(err) => return Err(PyRuntimeError::new_err(err.to_string())),
        }
    } else {
        cfdl_engine::RunConfig::default()
    };

    let results = match cfdl_engine::run_from_json_str(&ir_json, run_config) {
        Ok(value) => value,
        Err(err) => return Err(PyRuntimeError::new_err(err.to_string())),
    };
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
