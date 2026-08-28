//! The six tools, as plain functions over typed params/results.
//!
//! The MCP layer in `service.rs` is glue only; tests call these directly, so
//! the end-to-end gate exercises the same code paths the wire does.

pub mod compile;
pub mod diff;
pub mod explain;
pub mod lookup;
pub mod run;
pub mod skeleton;

use std::path::{Path, PathBuf};

use cfdl_pack::PackRegistry;

/// Where the server looks when a call does not say: set by `--packs` /
/// `--benchmarks` (or `--repo`), else auto-detected from the working
/// directory, else the embedded pack registry.
#[derive(Debug, Clone, Default)]
pub struct Defaults {
    pub packs_dir: Option<PathBuf>,
    pub benchmarks_dir: Option<PathBuf>,
}

impl Defaults {
    /// Detect `packs/` and `benchmarks/` under a root (a CFDL checkout).
    pub fn from_root(root: &Path) -> Self {
        let dir = |name: &str| {
            let candidate = root.join(name);
            candidate.is_dir().then_some(candidate)
        };
        Self {
            packs_dir: dir("packs"),
            benchmarks_dir: dir("benchmarks"),
        }
    }
}

pub(crate) fn resolve_packs_dir(packs_dir: Option<&str>, defaults: &Defaults) -> Option<PathBuf> {
    packs_dir
        .map(PathBuf::from)
        .or_else(|| defaults.packs_dir.clone())
}

/// A registry from an explicit directory, else the embedded one.
pub(crate) fn load_registry(packs_dir: Option<&Path>) -> Result<PackRegistry, String> {
    match packs_dir {
        Some(dir) => PackRegistry::load_from_dir(dir),
        None => PackRegistry::load_embedded(),
    }
    .map_err(|err| err.message)
}

/// A results document from an inline value or a file path (exactly one).
pub(crate) fn load_results(
    results: Option<serde_json::Value>,
    results_path: Option<&str>,
) -> Result<serde_json::Value, String> {
    match (results, results_path) {
        (Some(value), None) => Ok(value),
        (None, Some(path)) => {
            let raw = std::fs::read_to_string(path)
                .map_err(|err| format!("cannot read results '{path}': {err}"))?;
            serde_json::from_str(&raw)
                .map_err(|err| format!("results '{path}' is not valid JSON: {err}"))
        }
        (Some(_), Some(_)) => Err("pass `results` or `results_path`, not both".to_string()),
        (None, None) => Err("pass `results` (inline) or `results_path`".to_string()),
    }
}

/// A series point as a number, or `None` where it is genuinely undefined.
/// `None` is not zero — the engine publishes JSON null to say a value does
/// not exist, and coercing that to 0.0 would let a caller "match" a number
/// that was never computed. Mirrors the benchmark runner's `scalar`.
pub(crate) fn scalar(value: &serde_json::Value) -> Option<f64> {
    match value {
        serde_json::Value::Null => None,
        serde_json::Value::Object(map) => map.get("amount").and_then(serde_json::Value::as_f64),
        other => other.as_f64(),
    }
}
