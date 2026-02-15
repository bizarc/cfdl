use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Clone)]
pub struct Diagnostic {
    pub code: String,
    pub severity: String, // "error" | "warning" | "info"
    pub message: String,
    pub file: Option<String>,
    pub span: Option<Span>,
    pub path: Option<String>,
    pub hint: Option<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Span {
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

/// Compile a model directory to an IR JSON file.
///
/// v0.1 scaffolding: returns an explicit diagnostic until the compiler is implemented.
pub fn compile_to_file(model_root: &Path, _out_path: &Path) -> Result<(), Vec<Diagnostic>> {
    let model_file = model_root.join("model.cfdl");
    if let Ok(source) = std::fs::read_to_string(&model_file) {
        let (_tokens, lex_diags) = cfdl_lexer::lex(&source);
        if !lex_diags.is_empty() {
            let diagnostics = lex_diags
                .into_iter()
                .map(|diag| Diagnostic {
                    code: diag.code.to_string(),
                    severity: "error".to_string(),
                    message: diag.message,
                    file: Some(PathBuf::from("model.cfdl").to_string_lossy().to_string()),
                    span: Some(Span {
                        start_line: diag.span.start_line,
                        start_col: diag.span.start_col,
                        end_line: diag.span.end_line,
                        end_col: diag.span.end_col,
                    }),
                    path: None,
                    hint: None,
                    notes: vec![],
                })
                .collect();
            return Err(diagnostics);
        }
    }
    Err(vec![not_implemented_diag("compile")])
}

/// Validate a model directory without emitting IR.
///
/// v0.1 scaffolding: returns an explicit diagnostic until validation is implemented.
pub fn validate_only(_model_root: &Path) -> Result<(), Vec<Diagnostic>> {
    Err(vec![not_implemented_diag("validate")])
}

fn not_implemented_diag(stage: &str) -> Diagnostic {
    Diagnostic {
        code: "E9999_NOT_IMPLEMENTED".to_string(),
        severity: "error".to_string(),
        message: format!("CFDL compiler stage '{stage}' is not implemented yet."),
        file: Some(PathBuf::from("model.cfdl").to_string_lossy().to_string()),
        span: None,
        path: None,
        hint: Some(
            "Implement lexer/parser/resolver/validate/compile per @docs/compiler_spec_v0_1.md"
                .to_string(),
        ),
        notes: vec![],
    }
}
