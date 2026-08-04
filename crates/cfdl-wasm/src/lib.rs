//! WASM bindings for the CFDL playground: compile and run models entirely in
//! the browser over the embedded pack registry (no filesystem, no network).
//!
//! Both entry points take/return JSON strings so the JS side stays simple:
//! - [`compile`] takes a JSON object of `{ "path": "source", ... }` plus the
//!   entry file name, and returns `{"ok":true,"ir":<ir>}` or
//!   `{"ok":false,"diagnostics":[...]}`.
//! - [`run`] takes IR JSON (plus optional run-config JSON and pack name) and
//!   returns `{"ok":true,"results":<results>}` or `{"ok":false,"error":...}`.

use std::collections::BTreeMap;

use serde_json::json;
use wasm_bindgen::prelude::*;

fn parse_files(files_json: &str) -> Result<BTreeMap<String, String>, String> {
    serde_json::from_str(files_json).map_err(|e| format!("invalid files JSON: {e}"))
}

/// Compile an in-memory file map to IR. See module docs for the JSON shapes.
#[wasm_bindgen]
pub fn compile(files_json: &str, root_file: &str) -> String {
    let files = match parse_files(files_json) {
        Ok(files) => files,
        Err(err) => return json!({"ok": false, "error": err}).to_string(),
    };
    match cfdl_compile::compile_sources_to_json(
        &files,
        root_file,
        &cfdl_compile::CompileOptions::default(),
    ) {
        Ok(ir) => {
            let ir_value: serde_json::Value =
                serde_json::from_str(&ir).unwrap_or(serde_json::Value::Null);
            json!({"ok": true, "ir": ir_value}).to_string()
        }
        Err(diags) => {
            let diagnostics: Vec<serde_json::Value> = diags
                .iter()
                .map(|d| serde_json::to_value(d).unwrap_or(serde_json::Value::Null))
                .collect();
            json!({"ok": false, "diagnostics": diagnostics}).to_string()
        }
    }
}

/// Run compiled IR. `config_json` is an optional run-config; `pack` optionally
/// applies that pack's domain metrics from the embedded registry.
#[wasm_bindgen]
pub fn run(ir_json: &str, config_json: Option<String>, pack: Option<String>) -> String {
    let config = match config_json {
        Some(raw) if !raw.trim().is_empty() => {
            match cfdl_engine::run_config_from_json_str(&raw, 0.0, None) {
                Ok(config) => config,
                Err(err) => return json!({"ok": false, "error": err.to_string()}).to_string(),
            }
        }
        _ => cfdl_engine::RunConfig::default(),
    };

    let mut results = match cfdl_engine::run_from_json_str(ir_json, config) {
        Ok(results) => results,
        Err(err) => return json!({"ok": false, "error": err.to_string()}).to_string(),
    };
    if let Some(pack_name) = pack.filter(|p| !p.is_empty()) {
        let registry = cfdl_pack::PackRegistry::load_embedded().ok();
        let specs = registry
            .as_ref()
            .map(|reg| reg.metric_specs(&pack_name))
            .unwrap_or_default();
        results.domain_metrics = cfdl_metrics::compute(&pack_name, &specs, &results);
        // Statements read a stream's CATEGORY, which lives on the IR rather
        // than in results. Parsing it back here keeps the results document from
        // carrying a field only this consumer wants.
        let statement_specs = registry
            .as_ref()
            .map(|reg| reg.statement_specs(&pack_name))
            .unwrap_or_default();
        let categories = serde_json::from_str::<serde_json::Value>(ir_json)
            .ok()
            .map(|ir| cfdl_statement::stream_categories(&ir))
            .unwrap_or_default();
        results.statements =
            cfdl_statement::compute(&pack_name, &statement_specs, &categories, &results);
    }
    match serde_json::to_value(&results) {
        Ok(value) => json!({"ok": true, "results": value}).to_string(),
        Err(err) => json!({"ok": false, "error": err.to_string()}).to_string(),
    }
}

/// One-shot compile + run from sources (convenience for the playground).
#[wasm_bindgen]
pub fn compile_and_run(
    files_json: &str,
    root_file: &str,
    config_json: Option<String>,
    pack: Option<String>,
) -> String {
    let compiled = compile(files_json, root_file);
    let parsed: serde_json::Value =
        serde_json::from_str(&compiled).unwrap_or(serde_json::Value::Null);
    if parsed.get("ok").and_then(|v| v.as_bool()) != Some(true) {
        return compiled; // surface the compile diagnostics unchanged
    }
    let ir = parsed.get("ir").map(|v| v.to_string()).unwrap_or_default();
    run(&ir, config_json, pack)
}
