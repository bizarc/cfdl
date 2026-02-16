//! CFDL import resolver and deterministic module graph ordering.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Component, Path, PathBuf};

use cfdl_parser::{parse, CompilationUnit, Span, Stmt, StreamStmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveDiagnostic {
    pub code: String,
    pub message: String,
    pub file: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveOutput {
    pub compilation_unit: CompilationUnit,
    pub module_order: Vec<String>,
    pub source_statements: Vec<SourceStatement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatement {
    pub file: String,
    pub statement: Stmt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolEntry {
    pub name: String,
    pub file: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SymbolTables {
    pub entities: BTreeMap<String, SymbolEntry>,
    pub streams: BTreeMap<String, SymbolEntry>,
    pub phases: BTreeMap<String, SymbolEntry>,
    pub contracts: BTreeMap<String, SymbolEntry>,
    pub options: BTreeMap<String, SymbolEntry>,
    pub events: BTreeMap<String, SymbolEntry>,
    pub metrics: BTreeMap<String, SymbolEntry>,
    pub assumptions: BTreeMap<String, SymbolEntry>,
}

#[derive(Debug, Clone)]
pub struct RootModule {
    pub relative_path: String,
    pub full_path: PathBuf,
    pub ast: CompilationUnit,
}

#[derive(Debug, Clone)]
struct ModuleEntry {
    full_path: PathBuf,
    ast: CompilationUnit,
}

#[derive(Debug, Clone)]
struct ImportEdge {
    to_rel: String,
    span: Span,
}

pub fn resolve_imports(
    model_root: &Path,
    root_module: RootModule,
) -> Result<ResolveOutput, Vec<ResolveDiagnostic>> {
    let root_canon = match fs::canonicalize(model_root) {
        Ok(path) => path,
        Err(_) => {
            return Err(vec![ResolveDiagnostic {
                code: "E1202_IMPORT_NOT_FOUND".to_string(),
                message: "Model root does not exist.".to_string(),
                file: root_module.relative_path.clone(),
                span: root_module.ast.span,
            }]);
        }
    };

    let mut modules: BTreeMap<String, ModuleEntry> = BTreeMap::new();
    let mut imports: BTreeMap<String, Vec<ImportEdge>> = BTreeMap::new();
    let mut diagnostics = Vec::new();
    let mut queue = VecDeque::new();

    let root_rel = root_module.relative_path.clone();
    modules.insert(
        root_rel.clone(),
        ModuleEntry {
            full_path: root_module.full_path,
            ast: root_module.ast,
        },
    );
    queue.push_back(root_rel.clone());

    while let Some(current_rel) = queue.pop_front() {
        let current = modules
            .get(&current_rel)
            .expect("queued module exists")
            .clone();
        let current_imports = extract_imports(&current.ast);
        let mut resolved_edges = Vec::new();

        for import in current_imports {
            match resolve_import_target(
                &root_canon,
                &current.full_path,
                &import.path,
                &current_rel,
                import.span,
            ) {
                Ok(target_rel) => {
                    resolved_edges.push(ImportEdge {
                        to_rel: target_rel.clone(),
                        span: import.span,
                    });
                    if !modules.contains_key(&target_rel) {
                        match load_module(&root_canon, &target_rel) {
                            Ok(module) => {
                                modules.insert(target_rel.clone(), module);
                                queue.push_back(target_rel);
                            }
                            Err(errs) => diagnostics.extend(errs),
                        }
                    }
                }
                Err(diag) => diagnostics.push(diag),
            }
        }

        resolved_edges.sort_by(|a, b| a.to_rel.cmp(&b.to_rel));
        imports.insert(current_rel, resolved_edges);
    }

    if !diagnostics.is_empty() {
        diagnostics.sort_by(|a, b| {
            a.file
                .cmp(&b.file)
                .then(a.span.start_line.cmp(&b.span.start_line))
                .then(a.span.start_col.cmp(&b.span.start_col))
                .then(a.code.cmp(&b.code))
        });
        return Err(diagnostics);
    }

    if let Some(diag) = detect_cycle(&root_rel, &imports) {
        return Err(vec![diag]);
    }

    let order = deterministic_topological_order(&modules, &imports);
    let source_statements = collect_source_statements(&order, &modules);
    let merged = merge_modules(&source_statements);

    Ok(ResolveOutput {
        compilation_unit: merged,
        module_order: order,
        source_statements,
    })
}

pub fn resolve_symbols(output: &ResolveOutput) -> Result<SymbolTables, Vec<ResolveDiagnostic>> {
    let mut tables = SymbolTables::default();
    let mut diagnostics = Vec::new();
    let mut stream_decls = Vec::new();
    let mut contract_decls = Vec::new();

    for source_stmt in &output.source_statements {
        match &source_stmt.statement {
            Stmt::Entity(entity) => {
                let symbol = entity.symbol();
                if tables.entities.contains_key(&symbol) {
                    diagnostics.push(ResolveDiagnostic {
                        code: "E1001_DUPLICATE_ENTITY".to_string(),
                        message: format!("Duplicate entity '{symbol}'."),
                        file: source_stmt.file.clone(),
                        span: entity.span,
                    });
                } else {
                    tables.entities.insert(
                        symbol.clone(),
                        SymbolEntry {
                            name: symbol,
                            file: source_stmt.file.clone(),
                            span: entity.span,
                        },
                    );
                }
            }
            Stmt::Stream(stream) => {
                if tables.streams.contains_key(&stream.name) {
                    diagnostics.push(ResolveDiagnostic {
                        code: "E1003_DUPLICATE_STREAM".to_string(),
                        message: format!("Duplicate stream '{}'.", stream.name),
                        file: source_stmt.file.clone(),
                        span: stream.span,
                    });
                } else {
                    tables.streams.insert(
                        stream.name.clone(),
                        SymbolEntry {
                            name: stream.name.clone(),
                            file: source_stmt.file.clone(),
                            span: stream.span,
                        },
                    );
                }
                stream_decls.push(StreamDecl {
                    file: source_stmt.file.clone(),
                    stream: stream.clone(),
                });
            }
            Stmt::Contract(contract) => {
                contract_decls.push(ContractDecl {
                    file: source_stmt.file.clone(),
                    name: contract.name.clone(),
                    subject_entity: contract.subject_entity.clone(),
                    span: contract.span,
                });
            }
            _ => {}
        }
    }

    for stream_decl in stream_decls {
        if !is_valid_entity_ref(&stream_decl.stream.attached_entity) {
            diagnostics.push(ResolveDiagnostic {
                code: "E1306_INVALID_ENTITY_REF_FORMAT".to_string(),
                message: format!(
                    "Stream '{}' has invalid entity reference '{}'; expected a qualified name with at least two segments.",
                    stream_decl.stream.name, stream_decl.stream.attached_entity
                ),
                file: stream_decl.file,
                span: stream_decl.stream.span,
            });
            continue;
        }
        if !tables
            .entities
            .contains_key(&stream_decl.stream.attached_entity)
        {
            diagnostics.push(ResolveDiagnostic {
                code: "E1301_UNRESOLVED_ENTITY_REF".to_string(),
                message: format!(
                    "Stream '{}' references unknown entity '{}'.",
                    stream_decl.stream.name, stream_decl.stream.attached_entity
                ),
                file: stream_decl.file,
                span: stream_decl.stream.span,
            });
        }
    }

    for contract_decl in contract_decls {
        let Some(subject_entity) = contract_decl.subject_entity else {
            continue;
        };
        if !is_valid_entity_ref(&subject_entity) {
            diagnostics.push(ResolveDiagnostic {
                code: "E1306_INVALID_ENTITY_REF_FORMAT".to_string(),
                message: format!(
                    "Contract '{}' has invalid entity reference '{}'; expected a qualified name with at least two segments.",
                    contract_decl.name, subject_entity
                ),
                file: contract_decl.file,
                span: contract_decl.span,
            });
            continue;
        }
        if !tables.entities.contains_key(&subject_entity) {
            diagnostics.push(ResolveDiagnostic {
                code: "E1301_UNRESOLVED_ENTITY_REF".to_string(),
                message: format!(
                    "Contract '{}' references unknown entity '{}'.",
                    contract_decl.name, subject_entity
                ),
                file: contract_decl.file,
                span: contract_decl.span,
            });
        }
    }

    if diagnostics.is_empty() {
        Ok(tables)
    } else {
        sort_diagnostics(&mut diagnostics);
        Err(diagnostics)
    }
}

#[derive(Debug, Clone)]
struct ImportStmtRef {
    path: String,
    span: Span,
}

fn extract_imports(module: &CompilationUnit) -> Vec<ImportStmtRef> {
    let mut result = Vec::new();
    for stmt in &module.statements {
        if let Stmt::Import(import) = stmt {
            result.push(ImportStmtRef {
                path: import.path.clone(),
                span: import.span,
            });
        }
    }
    result
}

fn resolve_import_target(
    root_canon: &Path,
    importer_file: &Path,
    import_path: &str,
    importer_rel: &str,
    span: Span,
) -> Result<String, ResolveDiagnostic> {
    let importer_dir = importer_file.parent().unwrap_or(root_canon);
    let joined = importer_dir.join(import_path);

    if import_escapes_root(root_canon, &joined) {
        return Err(ResolveDiagnostic {
            code: "E1203_IMPORT_OUTSIDE_MODEL_ROOT".to_string(),
            message: format!("Import path '{import_path}' escapes the model root."),
            file: importer_rel.to_string(),
            span,
        });
    }

    let target_canon = match fs::canonicalize(&joined) {
        Ok(path) => path,
        Err(_) => {
            return Err(ResolveDiagnostic {
                code: "E1202_IMPORT_NOT_FOUND".to_string(),
                message: format!("Imported module '{import_path}' was not found."),
                file: importer_rel.to_string(),
                span,
            });
        }
    };

    if !target_canon.starts_with(root_canon) {
        return Err(ResolveDiagnostic {
            code: "E1203_IMPORT_OUTSIDE_MODEL_ROOT".to_string(),
            message: format!("Import path '{import_path}' resolves outside the model root."),
            file: importer_rel.to_string(),
            span,
        });
    }

    Ok(path_relative_to(root_canon, &target_canon))
}

fn load_module(
    root_canon: &Path,
    relative_path: &str,
) -> Result<ModuleEntry, Vec<ResolveDiagnostic>> {
    let full_path = root_canon.join(relative_path);
    let source = match fs::read_to_string(&full_path) {
        Ok(src) => src,
        Err(_) => {
            return Err(vec![ResolveDiagnostic {
                code: "E1202_IMPORT_NOT_FOUND".to_string(),
                message: format!("Imported module '{relative_path}' was not found."),
                file: relative_path.to_string(),
                span: Span {
                    start_line: 1,
                    start_col: 1,
                    end_line: 1,
                    end_col: 1,
                },
            }]);
        }
    };

    let (tokens, lex_diags) = cfdl_lexer::lex(&source);
    if !lex_diags.is_empty() {
        return Err(lex_diags
            .into_iter()
            .map(|diag| ResolveDiagnostic {
                code: diag.code.to_string(),
                message: diag.message,
                file: relative_path.to_string(),
                span: diag.span,
            })
            .collect());
    }

    let parse_result = parse(relative_path, &tokens);
    if !parse_result.diagnostics.is_empty() {
        return Err(parse_result
            .diagnostics
            .into_iter()
            .map(|diag| ResolveDiagnostic {
                code: diag.code.to_string(),
                message: diag.message,
                file: diag.file,
                span: diag.span,
            })
            .collect());
    }

    let ast = parse_result
        .ast
        .expect("parser returns AST when diagnostics are empty");
    Ok(ModuleEntry { full_path, ast })
}

fn import_escapes_root(root_canon: &Path, import_target: &Path) -> bool {
    if import_target.is_absolute() && !import_target.starts_with(root_canon) {
        return true;
    }
    match normalize_without_fs(import_target) {
        Some(path) => !path.starts_with(root_canon),
        None => true,
    }
}

fn normalize_without_fs(path: &Path) -> Option<PathBuf> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return None;
                }
            }
            Component::Normal(part) => normalized.push(part),
        }
    }
    Some(normalized)
}

fn path_relative_to(root_canon: &Path, absolute_path: &Path) -> String {
    absolute_path
        .strip_prefix(root_canon)
        .expect("path is under root")
        .to_string_lossy()
        .replace('\\', "/")
}

fn detect_cycle(
    root_rel: &str,
    imports: &BTreeMap<String, Vec<ImportEdge>>,
) -> Option<ResolveDiagnostic> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Color {
        White,
        Gray,
        Black,
    }

    let mut colors: BTreeMap<String, Color> = BTreeMap::new();
    for key in imports.keys() {
        colors.insert(key.clone(), Color::White);
    }

    fn dfs(
        node: &str,
        imports: &BTreeMap<String, Vec<ImportEdge>>,
        colors: &mut BTreeMap<String, Color>,
    ) -> Option<ResolveDiagnostic> {
        colors.insert(node.to_string(), Color::Gray);
        if let Some(edges) = imports.get(node) {
            for edge in edges {
                let next_color = colors.get(&edge.to_rel).copied().unwrap_or(Color::White);
                if next_color == Color::Gray {
                    return Some(ResolveDiagnostic {
                        code: "E1201_IMPORT_CYCLE".to_string(),
                        message: format!(
                            "Import cycle detected: '{node}' depends on '{}' through a cycle.",
                            edge.to_rel
                        ),
                        file: node.to_string(),
                        span: edge.span,
                    });
                }
                if next_color == Color::White {
                    if let Some(diag) = dfs(&edge.to_rel, imports, colors) {
                        return Some(diag);
                    }
                }
            }
        }
        colors.insert(node.to_string(), Color::Black);
        None
    }

    dfs(root_rel, imports, &mut colors)
}

fn deterministic_topological_order(
    modules: &BTreeMap<String, ModuleEntry>,
    imports: &BTreeMap<String, Vec<ImportEdge>>,
) -> Vec<String> {
    let mut indegree: BTreeMap<String, usize> = modules.keys().map(|k| (k.clone(), 0)).collect();
    let mut adjacency: BTreeMap<String, BTreeSet<String>> = modules
        .keys()
        .map(|k| (k.clone(), BTreeSet::new()))
        .collect();

    for (from, edges) in imports {
        for edge in edges {
            adjacency
                .entry(edge.to_rel.clone())
                .or_default()
                .insert(from.clone());
            *indegree.entry(from.clone()).or_insert(0) += 1;
        }
    }

    let mut available: BTreeSet<String> = indegree
        .iter()
        .filter_map(|(node, degree)| {
            if *degree == 0 {
                Some(node.clone())
            } else {
                None
            }
        })
        .collect();
    let mut order = Vec::with_capacity(modules.len());

    while let Some(next) = available.iter().next().cloned() {
        available.remove(&next);
        order.push(next.clone());
        if let Some(dependents) = adjacency.get(&next) {
            for dependent in dependents {
                if let Some(degree) = indegree.get_mut(dependent) {
                    *degree = degree.saturating_sub(1);
                    if *degree == 0 {
                        available.insert(dependent.clone());
                    }
                }
            }
        }
    }

    order
}

#[derive(Debug, Clone)]
struct StreamDecl {
    file: String,
    stream: StreamStmt,
}

#[derive(Debug, Clone)]
struct ContractDecl {
    file: String,
    name: String,
    subject_entity: Option<String>,
    span: Span,
}

fn collect_source_statements(
    order: &[String],
    modules: &BTreeMap<String, ModuleEntry>,
) -> Vec<SourceStatement> {
    let mut statements = Vec::new();
    for module in order {
        if let Some(entry) = modules.get(module) {
            statements.extend(entry.ast.statements.iter().cloned().map(|statement| {
                SourceStatement {
                    file: module.clone(),
                    statement,
                }
            }));
        }
    }
    statements
}

fn merge_modules(source_statements: &[SourceStatement]) -> CompilationUnit {
    let mut statements = Vec::new();
    for source_stmt in source_statements {
        statements.push(source_stmt.statement.clone());
    }

    let span = if let Some(first) = statements.first() {
        let start = statement_span(first);
        let end = statement_span(statements.last().expect("non-empty"));
        Span {
            start_line: start.start_line,
            start_col: start.start_col,
            end_line: end.end_line,
            end_col: end.end_col,
        }
    } else {
        Span {
            start_line: 1,
            start_col: 1,
            end_line: 1,
            end_col: 1,
        }
    };

    CompilationUnit { statements, span }
}

fn statement_span(stmt: &Stmt) -> Span {
    match stmt {
        Stmt::Version(s) => s.span,
        Stmt::Model(s) => s.span,
        Stmt::UsePack(s) => s.span,
        Stmt::Import(s) => s.span,
        Stmt::Time(s) => s.span,
        Stmt::Phase(s) => s.span,
        Stmt::Entity(s) => s.span,
        Stmt::Assume(s) => s.span,
        Stmt::Contract(s) => s.span,
        Stmt::Stream(s) => s.span,
        Stmt::Event(s) => s.span,
        Stmt::Option(s) => s.span,
        Stmt::Metric(s) => s.span,
    }
}

fn sort_diagnostics(diagnostics: &mut [ResolveDiagnostic]) {
    diagnostics.sort_by(|a, b| {
        a.file
            .cmp(&b.file)
            .then(a.span.start_line.cmp(&b.span.start_line))
            .then(a.span.start_col.cmp(&b.span.start_col))
            .then(a.code.cmp(&b.code))
    });
}

fn is_valid_entity_ref(value: &str) -> bool {
    let mut parts = value.split('.');
    let Some(first) = parts.next() else {
        return false;
    };
    if first.is_empty() {
        return false;
    }
    let mut count = 1usize;
    for part in parts {
        if part.is_empty() {
            return false;
        }
        count += 1;
    }
    count >= 2
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn span() -> Span {
        Span {
            start_line: 1,
            start_col: 1,
            end_line: 1,
            end_col: 1,
        }
    }

    #[test]
    fn reports_invalid_entity_ref_format_for_streams() {
        let output = ResolveOutput {
            compilation_unit: CompilationUnit {
                statements: vec![],
                span: span(),
            },
            module_order: vec!["model.cfdl".to_string()],
            source_statements: vec![
                SourceStatement {
                    file: "model.cfdl".to_string(),
                    statement: Stmt::Entity(cfdl_parser::EntityStmt {
                        namespace: "legal".to_string(),
                        name: "borrower".to_string(),
                        span: span(),
                    }),
                },
                SourceStatement {
                    file: "model.cfdl".to_string(),
                    statement: Stmt::Stream(cfdl_parser::StreamStmt {
                        name: "rent".to_string(),
                        attached_entity: "borrower".to_string(),
                        direction: Some("inflow".to_string()),
                        currency: Some("USD".to_string()),
                        schedule: None,
                        amount: None,
                        active_when: None,
                        span: span(),
                    }),
                },
            ],
        };

        let diags = resolve_symbols(&output).expect_err("expected resolve diagnostics");
        assert!(diags
            .iter()
            .any(|diag| diag.code == "E1306_INVALID_ENTITY_REF_FORMAT"));
    }

    #[test]
    fn reports_invalid_entity_ref_format_for_contracts() {
        let output = ResolveOutput {
            compilation_unit: CompilationUnit {
                statements: vec![],
                span: span(),
            },
            module_order: vec!["model.cfdl".to_string()],
            source_statements: vec![
                SourceStatement {
                    file: "model.cfdl".to_string(),
                    statement: Stmt::Entity(cfdl_parser::EntityStmt {
                        namespace: "legal".to_string(),
                        name: "borrower".to_string(),
                        span: span(),
                    }),
                },
                SourceStatement {
                    file: "model.cfdl".to_string(),
                    statement: Stmt::Contract(cfdl_parser::ContractStmt {
                        name: "lease.core.primary".to_string(),
                        subject_entity: Some("borrower".to_string()),
                        has_term: true,
                        has_effects: false,
                        term_start: None,
                        term_end: None,
                        terms: BTreeMap::new(),
                        span: span(),
                    }),
                },
            ],
        };

        let diags = resolve_symbols(&output).expect_err("expected resolve diagnostics");
        assert!(diags
            .iter()
            .any(|diag| diag.code == "E1306_INVALID_ENTITY_REF_FORMAT"));
    }
}
