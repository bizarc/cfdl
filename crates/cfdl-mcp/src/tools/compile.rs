//! `compile`: source -> IR or structured diagnostics (docs/08).

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Defaults;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct CompileParams {
    /// Directory containing `model.cfdl` on this machine. Alternative to `files`.
    #[serde(default)]
    pub model_dir: Option<String>,
    /// In-memory sources: root-relative path -> source text. Must contain `root_file`.
    #[serde(default)]
    pub files: Option<BTreeMap<String, String>>,
    /// Entry module within `files` (default `model.cfdl`).
    #[serde(default)]
    pub root_file: Option<String>,
    /// Pack directory override. Default: the server's packs directory, else
    /// `<model_dir>/packs`, else the embedded registry.
    #[serde(default)]
    pub packs_dir: Option<String>,
    /// Return the compiled IR JSON in the response (can be large). The `run`
    /// tool compiles on its own, so most loops never need the IR text.
    #[serde(default)]
    pub include_ir: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CompileResult {
    pub ok: bool,
    /// Structured diagnostics per docs/08. Empty when `ok`.
    pub diagnostics: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ir: Option<String>,
}

/// Compile to IR JSON: `Err` is a caller mistake; `Ok(Err(diags))` is the
/// model's diagnostics — data for the repair loop, not a protocol error.
pub(crate) fn compile_ir(
    model_dir: Option<&str>,
    files: Option<&BTreeMap<String, String>>,
    root_file: Option<&str>,
    packs_dir: Option<&str>,
    defaults: &Defaults,
) -> Result<Result<String, Vec<cfdl_compile::Diagnostic>>, String> {
    let options = cfdl_compile::CompileOptions {
        packs_dir: super::resolve_packs_dir(packs_dir, defaults),
    };
    match (model_dir, files) {
        (Some(dir), None) => Ok(cfdl_compile::compile_to_json_with_options(
            std::path::Path::new(dir),
            &options,
        )),
        (None, Some(files)) => Ok(cfdl_compile::compile_sources_to_json(
            files,
            root_file.unwrap_or("model.cfdl"),
            &options,
        )),
        (Some(_), Some(_)) => Err("pass `model_dir` or `files`, not both".to_string()),
        (None, None) => Err("compile requires `model_dir` or `files`".to_string()),
    }
}

pub fn compile(params: &CompileParams, defaults: &Defaults) -> Result<CompileResult, String> {
    let outcome = compile_ir(
        params.model_dir.as_deref(),
        params.files.as_ref(),
        params.root_file.as_deref(),
        params.packs_dir.as_deref(),
        defaults,
    )?;
    Ok(match outcome {
        Ok(ir) => CompileResult {
            ok: true,
            diagnostics: Vec::new(),
            ir: params.include_ir.then_some(ir),
        },
        Err(diags) => CompileResult {
            ok: false,
            diagnostics: diags
                .iter()
                .map(|d| serde_json::to_value(d).unwrap_or(serde_json::Value::Null))
                .collect(),
            ir: None,
        },
    })
}
