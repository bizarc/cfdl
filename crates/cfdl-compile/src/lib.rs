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
    let (resolve_output, symbols) = pipeline(model_root)?;

    let validation_diags = cfdl_validate::validate(&resolve_output, &symbols);
    if !validation_diags.is_empty() {
        return Err(validation_diags
            .into_iter()
            .map(|diag| Diagnostic {
                code: diag.code.to_string(),
                severity: "error".to_string(),
                message: diag.message,
                file: Some(diag.file),
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
            .collect());
    }

    Err(vec![not_implemented_diag("compile")])
}

/// Validate a model directory without emitting IR.
///
pub fn validate_only(model_root: &Path) -> Result<(), Vec<Diagnostic>> {
    let (resolve_output, symbols) = pipeline(model_root)?;
    let diagnostics = cfdl_validate::validate(&resolve_output, &symbols);
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics
            .into_iter()
            .map(|diag| Diagnostic {
                code: diag.code.to_string(),
                severity: "error".to_string(),
                message: diag.message,
                file: Some(diag.file),
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
            .collect())
    }
}

fn pipeline(
    model_root: &Path,
) -> Result<(cfdl_resolver::ResolveOutput, cfdl_resolver::SymbolTables), Vec<Diagnostic>> {
    let model_file = model_root.join("model.cfdl");
    let source = std::fs::read_to_string(&model_file).map_err(|_| {
        vec![Diagnostic {
            code: "E1202_IMPORT_NOT_FOUND".to_string(),
            severity: "error".to_string(),
            message: "Model root is missing required file 'model.cfdl'.".to_string(),
            file: Some(PathBuf::from("model.cfdl").to_string_lossy().to_string()),
            span: None,
            path: None,
            hint: None,
            notes: vec![],
        }]
    })?;

    let (tokens, lex_diags) = cfdl_lexer::lex(&source);
    if !lex_diags.is_empty() {
        return Err(lex_diags
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
            .collect());
    }

    let parse_result = cfdl_parser::parse("model.cfdl", &tokens);
    if !parse_result.diagnostics.is_empty() {
        return Err(parse_result
            .diagnostics
            .into_iter()
            .map(|diag| Diagnostic {
                code: diag.code.to_string(),
                severity: "error".to_string(),
                message: diag.message,
                file: Some(diag.file),
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
            .collect());
    }

    let root_ast = parse_result
        .ast
        .expect("parser returns AST when diagnostics are empty");
    let root_module = cfdl_resolver::RootModule {
        relative_path: "model.cfdl".to_string(),
        full_path: std::fs::canonicalize(&model_file).unwrap_or(model_file),
        ast: root_ast,
    };
    let resolve_output = match cfdl_resolver::resolve_imports(model_root, root_module) {
        Ok(output) => output,
        Err(resolve_diags) => {
            return Err(resolve_diags
                .into_iter()
                .map(|diag| Diagnostic {
                    code: diag.code,
                    severity: "error".to_string(),
                    message: diag.message,
                    file: Some(diag.file),
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
                .collect())
        }
    };

    let symbols = match cfdl_resolver::resolve_symbols(&resolve_output) {
        Ok(symbols) => symbols,
        Err(symbol_diags) => {
            return Err(symbol_diags
                .into_iter()
                .map(|diag| Diagnostic {
                    code: diag.code,
                    severity: "error".to_string(),
                    message: diag.message,
                    file: Some(diag.file),
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
                .collect())
        }
    };

    Ok((resolve_output, symbols))
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
