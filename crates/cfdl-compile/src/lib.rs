use cfdl_parser::{Cadence, ScheduleKind, Stmt};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct CompileOptions {
    pub packs_dir: Option<PathBuf>,
}

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
pub fn compile_to_file(model_root: &Path, out_path: &Path) -> Result<(), Vec<Diagnostic>> {
    compile_to_file_with_options(model_root, out_path, &CompileOptions::default())
}

/// Compile a model directory to an IR JSON file with options.
pub fn compile_to_file_with_options(
    model_root: &Path,
    out_path: &Path,
    options: &CompileOptions,
) -> Result<(), Vec<Diagnostic>> {
    let (resolve_output, symbols) = pipeline(model_root)?;

    let active_pack = resolve_active_pack(model_root, &resolve_output, options)?;

    let validation_diags = filter_pack_aware_validation(
        cfdl_validate::validate(&resolve_output, &symbols),
        &resolve_output,
        active_pack.as_ref(),
    );
    if !validation_diags.is_empty() {
        return Err(validation_diags
            .into_iter()
            .map(map_validation_diag)
            .collect());
    }

    let expr_diags = validate_expressions(&resolve_output);
    if !expr_diags.is_empty() {
        return Err(expr_diags);
    }

    let ir = build_ir(&resolve_output, active_pack.as_ref())?;

    let json = serde_json::to_string_pretty(&ir).map_err(|err| {
        vec![Diagnostic {
            code: "E5003_IR_EMIT_FAILED".to_string(),
            severity: "error".to_string(),
            message: format!("IR emission failed during serialization: {err}"),
            file: Some("model.cfdl".to_string()),
            span: None,
            path: None,
            hint: None,
            notes: vec![],
        }]
    })?;

    std::fs::write(out_path, json).map_err(|err| {
        vec![Diagnostic {
            code: "E5003_IR_EMIT_FAILED".to_string(),
            severity: "error".to_string(),
            message: format!(
                "IR emission failed while writing '{}': {err}",
                out_path.display()
            ),
            file: Some("model.cfdl".to_string()),
            span: None,
            path: None,
            hint: None,
            notes: vec![],
        }]
    })?;

    Ok(())
}

/// Validate a model directory without emitting IR.
///
pub fn validate_only(model_root: &Path) -> Result<(), Vec<Diagnostic>> {
    let (resolve_output, symbols) = pipeline(model_root)?;
    let diagnostics = cfdl_validate::validate(&resolve_output, &symbols);
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics.into_iter().map(map_validation_diag).collect())
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
        return Err(lex_diags.into_iter().map(map_lex_diag).collect());
    }

    let parse_result = cfdl_parser::parse("model.cfdl", &tokens);
    if !parse_result.diagnostics.is_empty() {
        return Err(parse_result
            .diagnostics
            .into_iter()
            .map(map_parse_diag)
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
            return Err(resolve_diags.into_iter().map(map_resolve_diag).collect())
        }
    };

    let symbols = match cfdl_resolver::resolve_symbols(&resolve_output) {
        Ok(symbols) => symbols,
        Err(symbol_diags) => return Err(symbol_diags.into_iter().map(map_resolve_diag).collect()),
    };

    Ok((resolve_output, symbols))
}

fn map_lex_diag(diag: cfdl_lexer::LexDiagnostic) -> Diagnostic {
    Diagnostic {
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
    }
}

fn map_parse_diag(diag: cfdl_parser::ParseDiagnostic) -> Diagnostic {
    Diagnostic {
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
    }
}

fn map_resolve_diag(diag: cfdl_resolver::ResolveDiagnostic) -> Diagnostic {
    Diagnostic {
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
    }
}

fn map_validation_diag(diag: cfdl_validate::ValidationDiagnostic) -> Diagnostic {
    Diagnostic {
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
    }
}

#[derive(Debug, Clone)]
struct ActivePackContext {
    name: String,
    version: String,
    lowering_rules: Vec<cfdl_pack::LoweringRule>,
}

struct PackLoweringOutput {
    streams: Vec<((String, String), IrStream)>,
    diagnostics: Vec<Diagnostic>,
}

struct LoweringContext<'a> {
    id_seed: &'a str,
    model_currency: &'a str,
    time_calendar: &'a str,
    time_start: &'a str,
    timeline_end: &'a str,
    default_owner: &'a str,
}

#[derive(Debug, Serialize)]
struct Ir {
    ir_version: String,
    model: IrModel,
    time: IrTime,
    phases: Vec<IrPhase>,
    entities: Vec<IrEntity>,
    assumptions: IrAssumptions,
    contracts: Vec<IrContract>,
    streams: Vec<IrStream>,
    events: Vec<serde_json::Value>,
    options: Vec<serde_json::Value>,
    runs: Vec<IrRun>,
    metrics: Vec<serde_json::Value>,
    required_observables: Vec<String>,
    required_refs: Vec<String>,
    provenance: IrProvenance,
}

#[derive(Debug, Serialize)]
struct IrModel {
    name: String,
    currency: String,
}

#[derive(Debug, Serialize)]
struct IrTime {
    calendar: String,
    start: String,
    periods: u32,
}

#[derive(Debug, Serialize)]
struct IrDateRange {
    start: String,
    end: String,
}

#[derive(Debug, Serialize)]
struct IrNodeProvenance {
    source_file: String,
    source_span: Span,
    #[serde(skip_serializing_if = "Option::is_none")]
    generated_by: Option<IrGeneratedBy>,
}

#[derive(Debug, Serialize)]
struct IrGeneratedBy {
    pack: IrPackRef,
    rule_id: String,
}

#[derive(Debug, Serialize)]
struct IrPackRef {
    name: String,
    version: String,
}

#[derive(Debug, Serialize)]
struct IrEntityRef {
    symbol: String,
}

#[derive(Debug, Serialize)]
struct IrPhase {
    id: String,
    range: IrDateRange,
}

#[derive(Debug, Serialize)]
struct IrEntity {
    id: String,
    symbol: String,
    r#type: String,
    attrs: BTreeMap<String, serde_json::Value>,
    state: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct IrAssumptions {
    constants: BTreeMap<String, serde_json::Value>,
    random: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct IrExpr {
    lang: String,
    src: String,
}

#[derive(Debug, Serialize)]
struct IrEffects {
    streams: Vec<IrStream>,
}

#[derive(Debug, Serialize)]
struct IrContract {
    id: String,
    name: String,
    r#type: String,
    subject: IrEntityRef,
    term: IrDateRange,
    currency: String,
    terms: BTreeMap<String, serde_json::Value>,
    effects: IrEffects,
    provenance: IrNodeProvenance,
}

#[derive(Debug, Serialize)]
struct IrOnRule {
    kind: String,
    day: i32,
}

#[derive(Debug, Serialize)]
struct IrSchedule {
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    on: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    every: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    on_rule: Option<IrOnRule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    phase: Option<String>,
}

#[derive(Debug, Serialize)]
struct IrStream {
    id: String,
    name: String,
    owner: IrEntityRef,
    direction: String,
    currency: String,
    schedule: IrSchedule,
    amount: IrExpr,
    active_when: IrExpr,
    provenance: IrNodeProvenance,
}

#[derive(Debug, Serialize)]
struct IrRun {
    kind: String,
}

#[derive(Debug, Serialize)]
struct IrProvenanceCompiler {
    name: String,
    version: String,
    hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct IrProvenance {
    sources: Vec<String>,
    compiler: IrProvenanceCompiler,
}

fn build_ir(
    resolve_output: &cfdl_resolver::ResolveOutput,
    active_pack: Option<&ActivePackContext>,
) -> Result<Ir, Vec<Diagnostic>> {
    let model_name = find_model_name(resolve_output).unwrap_or_else(|| "model".to_string());
    let model_currency = "USD".to_string();
    let (time_calendar, time_start, time_periods) = find_time(resolve_output)
        .unwrap_or_else(|| ("monthly".to_string(), "1970-01-01".to_string(), 1));
    let timeline_end = add_periods_for_timeline_end(&time_start, &time_calendar, time_periods);
    let compiler_version = env!("CARGO_PKG_VERSION").to_string();
    let pack_seed = active_pack
        .map(|pack| format!("{}@{}", pack.name, pack.version))
        .unwrap_or_default();
    let compiler_hash = hash_hex(&format!("cfdl:{compiler_version}:{pack_seed}"));
    let id_seed = if pack_seed.is_empty() {
        format!("cfdl:{compiler_version}:{compiler_hash}")
    } else {
        format!("cfdl:{compiler_version}:{compiler_hash}:{pack_seed}")
    };

    let mut phases: Vec<((String, String), IrPhase)> = resolve_output
        .source_statements
        .iter()
        .filter_map(|source_stmt| {
            let Stmt::Phase(phase) = &source_stmt.statement else {
                return None;
            };
            let name = phase.name.clone();
            let stable_key = stable_key(&source_stmt.file, &name);
            let ir_phase = IrPhase {
                id: deterministic_id("Phase", &stable_key, &id_seed),
                range: IrDateRange {
                    start: normalize_date(&phase.from),
                    end: normalize_date(&phase.to),
                },
            };
            Some(((name, source_stmt.file.clone()), ir_phase))
        })
        .collect();
    phases.sort_by(|a, b| a.0.cmp(&b.0));

    let mut entities: Vec<((String, String), IrEntity)> = resolve_output
        .source_statements
        .iter()
        .filter_map(|source_stmt| {
            let Stmt::Entity(entity) = &source_stmt.statement else {
                return None;
            };
            let symbol = entity.symbol();
            let stable_key = stable_key(&source_stmt.file, &symbol);
            let ir_entity = IrEntity {
                id: deterministic_id("Entity", &stable_key, &id_seed),
                symbol: symbol.clone(),
                r#type: "core.Entity".to_string(),
                attrs: BTreeMap::new(),
                state: BTreeMap::new(),
            };
            Some(((symbol, source_stmt.file.clone()), ir_entity))
        })
        .collect();
    entities.sort_by(|a, b| a.0.cmp(&b.0));

    let first_entity_symbol = entities
        .first()
        .map(|(_, entity)| entity.symbol.clone())
        .unwrap_or_else(|| "entity.placeholder".to_string());

    let mut contracts: Vec<((String, String), IrContract)> = resolve_output
        .source_statements
        .iter()
        .filter_map(|source_stmt| {
            let Stmt::Contract(contract) = &source_stmt.statement else {
                return None;
            };
            let name = contract.name.clone();
            let stable_key = stable_key(&source_stmt.file, &name);
            let ir_contract = IrContract {
                id: deterministic_id("Contract", &stable_key, &id_seed),
                name: name.clone(),
                r#type: "core.Contract".to_string(),
                subject: IrEntityRef {
                    symbol: first_entity_symbol.clone(),
                },
                term: IrDateRange {
                    start: time_start.clone(),
                    end: timeline_end.clone(),
                },
                currency: model_currency.clone(),
                terms: BTreeMap::new(),
                effects: IrEffects { streams: vec![] },
                provenance: IrNodeProvenance {
                    source_file: source_stmt.file.clone(),
                    source_span: map_span(contract.span),
                    generated_by: None,
                },
            };
            Some(((name, source_stmt.file.clone()), ir_contract))
        })
        .collect();
    contracts.sort_by(|a, b| a.0.cmp(&b.0));

    let mut streams: Vec<((String, String), IrStream)> = resolve_output
        .source_statements
        .iter()
        .filter_map(|source_stmt| {
            let Stmt::Stream(stream) = &source_stmt.statement else {
                return None;
            };
            let stable_key = stable_key(&source_stmt.file, &stream.name);
            let schedule = lower_schedule(
                stream.schedule.as_ref(),
                &time_calendar,
                &time_start,
                &timeline_end,
            );
            let ir_stream = IrStream {
                id: deterministic_id("Stream", &stable_key, &id_seed),
                name: stream.name.clone(),
                owner: IrEntityRef {
                    symbol: stream.attached_entity.clone(),
                },
                direction: "outflow".to_string(),
                currency: model_currency.clone(),
                schedule,
                amount: IrExpr {
                    lang: stream
                        .amount
                        .as_ref()
                        .map(|expr| expr.lang.clone())
                        .unwrap_or_else(|| "cel".to_string()),
                    src: stream
                        .amount
                        .as_ref()
                        .map(|expr| expr.src.clone())
                        .unwrap_or_else(|| "0".to_string()),
                },
                active_when: IrExpr {
                    lang: stream
                        .active_when
                        .as_ref()
                        .map(|expr| expr.lang.clone())
                        .unwrap_or_else(|| "cel".to_string()),
                    src: stream
                        .active_when
                        .as_ref()
                        .map(|expr| expr.src.clone())
                        .unwrap_or_else(|| "true".to_string()),
                },
                provenance: IrNodeProvenance {
                    source_file: source_stmt.file.clone(),
                    source_span: map_span(stream.span),
                    generated_by: None,
                },
            };
            Some(((stream.name.clone(), source_stmt.file.clone()), ir_stream))
        })
        .collect();
    let lowered = lower_contract_streams(
        resolve_output,
        active_pack,
        LoweringContext {
            id_seed: &id_seed,
            model_currency: &model_currency,
            time_calendar: &time_calendar,
            time_start: &time_start,
            timeline_end: &timeline_end,
            default_owner: &first_entity_symbol,
        },
    );
    if lowered
        .diagnostics
        .iter()
        .any(|diag| diag.severity == "error")
    {
        let mut diagnostics = lowered.diagnostics;
        sort_compile_diagnostics(&mut diagnostics);
        return Err(diagnostics);
    }
    streams.extend(lowered.streams);
    streams.sort_by(|a, b| a.0.cmp(&b.0));

    let mut sources = resolve_output.module_order.clone();
    sources.sort();

    Ok(Ir {
        ir_version: "0.1".to_string(),
        model: IrModel {
            name: model_name,
            currency: model_currency,
        },
        time: IrTime {
            calendar: time_calendar,
            start: time_start,
            periods: time_periods,
        },
        phases: phases.into_iter().map(|(_, phase)| phase).collect(),
        entities: entities.into_iter().map(|(_, entity)| entity).collect(),
        assumptions: IrAssumptions {
            constants: BTreeMap::new(),
            random: BTreeMap::new(),
        },
        contracts: contracts
            .into_iter()
            .map(|(_, contract)| contract)
            .collect(),
        streams: streams.into_iter().map(|(_, stream)| stream).collect(),
        events: vec![],
        options: vec![],
        runs: vec![IrRun {
            kind: "deterministic".to_string(),
        }],
        metrics: vec![],
        required_observables: vec![],
        required_refs: vec![],
        provenance: IrProvenance {
            sources,
            compiler: IrProvenanceCompiler {
                name: "cfdl".to_string(),
                version: compiler_version,
                hash: compiler_hash,
                notes: active_pack
                    .map(|pack| vec![format!("active_pack={}@{}", pack.name, pack.version)]),
            },
        },
    })
}

fn resolve_active_pack(
    model_root: &Path,
    resolve_output: &cfdl_resolver::ResolveOutput,
    options: &CompileOptions,
) -> Result<Option<ActivePackContext>, Vec<Diagnostic>> {
    let use_pack_stmt = resolve_output
        .source_statements
        .iter()
        .find_map(|source_stmt| match &source_stmt.statement {
            Stmt::UsePack(stmt) => Some((source_stmt.file.clone(), stmt.clone())),
            _ => None,
        });

    let Some((file, use_pack)) = use_pack_stmt else {
        return Ok(None);
    };

    let packs_dir = options
        .packs_dir
        .clone()
        .unwrap_or_else(|| model_root.join("packs"));
    let registry = cfdl_pack::PackRegistry::load_from_dir(&packs_dir).map_err(|err| {
        vec![Diagnostic {
            code: "E4004_MISSING_PACK".to_string(),
            severity: "error".to_string(),
            message: err.message,
            file: Some(file.clone()),
            span: Some(map_span(use_pack.span)),
            path: None,
            hint: None,
            notes: vec![format!("pack root: {}", packs_dir.display())],
        }]
    })?;
    let Some(active) = registry.active_pack(&use_pack.name, &use_pack.version) else {
        return Err(vec![Diagnostic {
            code: "E4004_MISSING_PACK".to_string(),
            severity: "error".to_string(),
            message: format!(
                "Pack '{}@{}' was not found under '{}'.",
                use_pack.name,
                use_pack.version,
                packs_dir.display()
            ),
            file: Some(file),
            span: Some(map_span(use_pack.span)),
            path: None,
            hint: Some("Add a matching pack manifest or pass --packs <dir>.".to_string()),
            notes: vec![],
        }]);
    };

    Ok(Some(ActivePackContext {
        name: active.name.clone(),
        version: active.version.clone(),
        lowering_rules: registry.lowering_rules(&active.name),
    }))
}

fn filter_pack_aware_validation(
    diagnostics: Vec<cfdl_validate::ValidationDiagnostic>,
    resolve_output: &cfdl_resolver::ResolveOutput,
    active_pack: Option<&ActivePackContext>,
) -> Vec<cfdl_validate::ValidationDiagnostic> {
    let Some(pack) = active_pack else {
        return diagnostics;
    };
    let lowered_contract_anchors: Vec<(String, cfdl_parser::Span)> = resolve_output
        .source_statements
        .iter()
        .filter_map(|source_stmt| {
            let Stmt::Contract(contract) = &source_stmt.statement else {
                return None;
            };
            if pack
                .lowering_rules
                .iter()
                .any(|rule| rule.contract_name == contract.name)
            {
                Some((source_stmt.file.clone(), contract.span))
            } else {
                None
            }
        })
        .collect();

    diagnostics
        .into_iter()
        .filter(|diag| {
            if diag.code != "E2002_CONTRACT_MISSING_EFFECTS" {
                return true;
            }
            !lowered_contract_anchors.iter().any(|(file, span)| {
                *file == diag.file
                    && span.start_line == diag.span.start_line
                    && span.start_col == diag.span.start_col
            })
        })
        .collect()
}

fn lower_contract_streams(
    resolve_output: &cfdl_resolver::ResolveOutput,
    active_pack: Option<&ActivePackContext>,
    ctx: LoweringContext<'_>,
) -> PackLoweringOutput {
    let Some(pack) = active_pack else {
        return PackLoweringOutput {
            streams: vec![],
            diagnostics: vec![],
        };
    };
    let mut rules = pack.lowering_rules.clone();
    rules.sort_by(|a, b| a.id.cmp(&b.id));

    let mut lowered = Vec::new();
    let mut diagnostics = Vec::new();
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Contract(contract) = &source_stmt.statement else {
            continue;
        };
        let before = diagnostics.len();
        diagnostics.extend(validate_pack_contract(
            pack,
            source_stmt,
            contract,
            ctx.time_start,
            ctx.timeline_end,
        ));
        if diagnostics[before..]
            .iter()
            .any(|diag| diag.severity == "error")
        {
            continue;
        }
        for rule in &rules {
            if rule.contract_name != contract.name {
                continue;
            }

            let stable_key = format!("{}::{}::{}", source_stmt.file, contract.name, rule.id);
            let owner_symbol = if rule.owner_entity.is_empty() {
                ctx.default_owner.to_string()
            } else {
                rule.owner_entity.clone()
            };
            let schedule =
                lower_pack_rule_schedule(rule, ctx.time_calendar, ctx.time_start, ctx.timeline_end);

            lowered.push((
                (rule.stream_name.clone(), stable_key.clone()),
                IrStream {
                    id: deterministic_id("Stream", &stable_key, ctx.id_seed),
                    name: rule.stream_name.clone(),
                    owner: IrEntityRef {
                        symbol: owner_symbol,
                    },
                    direction: if rule.direction.is_empty() {
                        "outflow".to_string()
                    } else {
                        rule.direction.clone()
                    },
                    currency: if rule.currency.is_empty() {
                        ctx.model_currency.to_string()
                    } else {
                        rule.currency.clone()
                    },
                    schedule,
                    amount: IrExpr {
                        lang: "cel".to_string(),
                        src: rule.amount_cel.clone(),
                    },
                    active_when: IrExpr {
                        lang: "cel".to_string(),
                        src: "true".to_string(),
                    },
                    provenance: IrNodeProvenance {
                        source_file: source_stmt.file.clone(),
                        source_span: map_span(contract.span),
                        generated_by: Some(IrGeneratedBy {
                            pack: IrPackRef {
                                name: pack.name.clone(),
                                version: pack.version.clone(),
                            },
                            rule_id: rule.id.clone(),
                        }),
                    },
                },
            ));
        }
    }
    PackLoweringOutput {
        streams: lowered,
        diagnostics,
    }
}

fn validate_pack_contract(
    pack: &ActivePackContext,
    source_stmt: &cfdl_resolver::SourceStatement,
    contract: &cfdl_parser::ContractStmt,
    timeline_start: &str,
    timeline_end: &str,
) -> Vec<Diagnostic> {
    if pack.name != "cre" {
        return vec![];
    }
    let mut diagnostics = Vec::new();
    match contract.name.as_str() {
        "cre_lease" => {
            if !contract.terms.contains_key("base_rent") {
                diagnostics.push(cre_pack_diag(
                    "E6001_CRE_LEASE_MISSING_BASE_RENT",
                    "CRE lease is missing required term 'base_rent'.",
                    source_stmt,
                    contract.span,
                ));
            }
            if !valid_contract_term_range(contract, timeline_start, timeline_end) {
                diagnostics.push(cre_pack_diag(
                    "E6002_CRE_LEASE_INVALID_TERM_RANGE",
                    "CRE lease term range is missing, invalid, or outside model timeline.",
                    source_stmt,
                    contract.span,
                ));
            }
            let lease_up_enabled = contract
                .terms
                .keys()
                .any(|key| key == "lease_up" || key.starts_with("lease_up."));
            if lease_up_enabled {
                let months_ok = contract
                    .terms
                    .get("lease_up.months")
                    .and_then(|term| term.value.parse::<i32>().ok())
                    .map(|months| months > 0)
                    .unwrap_or(false);
                if !months_ok {
                    diagnostics.push(cre_pack_diag(
                        "E6003_CRE_LEASE_UP_MISSING_MONTHS",
                        "CRE lease_up requires term 'lease_up.months' > 0 when lease_up is enabled.",
                        source_stmt,
                        contract.span,
                    ));
                }
                let start_occ = contract
                    .terms
                    .get("lease_up.start_occupancy")
                    .and_then(|term| term.value.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let end_occ = contract
                    .terms
                    .get("lease_up.end_occupancy")
                    .and_then(|term| term.value.parse::<f64>().ok())
                    .unwrap_or(1.0);
                if !(0.0..=1.0).contains(&start_occ) || !(0.0..=1.0).contains(&end_occ) {
                    diagnostics.push(cre_pack_diag(
                        "E6004_CRE_LEASE_UP_INVALID_OCCUPANCY",
                        "CRE lease_up occupancy must be in [0, 1] for start/end occupancy.",
                        source_stmt,
                        contract.span,
                    ));
                }
            }
        }
        "cre_exit_cap" => {
            let exit_cap = contract
                .terms
                .get("exit_cap")
                .and_then(|term| term.value.parse::<f64>().ok());
            if exit_cap.is_none() {
                diagnostics.push(cre_pack_diag(
                    "E6010_CRE_EXIT_MISSING_EXIT_CAP",
                    "CRE exit contract is missing required term 'exit_cap'.",
                    source_stmt,
                    contract.span,
                ));
            } else if exit_cap.unwrap_or(0.0) <= 0.0 {
                diagnostics.push(cre_pack_diag(
                    "E6011_CRE_EXIT_INVALID_EXIT_CAP",
                    "CRE exit 'exit_cap' must be greater than 0.",
                    source_stmt,
                    contract.span,
                ));
            }
            let has_noi = contract.terms.contains_key("noi_ref")
                || contract.terms.contains_key("noi_value")
                || contract.terms.contains_key("noi");
            if !has_noi {
                diagnostics.push(cre_pack_diag(
                    "E6012_CRE_EXIT_MISSING_NOI_REF_OR_VALUE",
                    "CRE exit requires either 'noi_ref' or 'noi_value'.",
                    source_stmt,
                    contract.span,
                ));
            }
        }
        "cre_ops_revenue" | "cre_ops_expense" => {
            if !contract.terms.contains_key("amount") {
                diagnostics.push(cre_pack_diag(
                    "E6020_CRE_OPS_MISSING_AMOUNT",
                    "CRE ops contract is missing required term 'amount'.",
                    source_stmt,
                    contract.span,
                ));
            }
            if !valid_contract_term_range(contract, timeline_start, timeline_end) {
                diagnostics.push(cre_pack_diag(
                    "E6021_CRE_OPS_INVALID_SCHEDULE",
                    "CRE ops term range is missing, invalid, or outside model timeline.",
                    source_stmt,
                    contract.span,
                ));
            }
        }
        _ => {}
    }
    diagnostics
}

fn cre_pack_diag(
    code: &str,
    message: &str,
    source_stmt: &cfdl_resolver::SourceStatement,
    span: cfdl_parser::Span,
) -> Diagnostic {
    Diagnostic {
        code: code.to_string(),
        severity: "error".to_string(),
        message: message.to_string(),
        file: Some(source_stmt.file.clone()),
        span: Some(map_span(span)),
        path: None,
        hint: None,
        notes: vec![],
    }
}

fn valid_contract_term_range(
    contract: &cfdl_parser::ContractStmt,
    timeline_start: &str,
    timeline_end: &str,
) -> bool {
    let Some(start) = contract.term_start.as_ref() else {
        return false;
    };
    let Some(end) = contract.term_end.as_ref() else {
        return false;
    };
    let start = normalize_date(start);
    let end = normalize_date(end);
    if parse_ymd(&start).is_none() || parse_ymd(&end).is_none() {
        return false;
    }
    if start > end {
        return false;
    }
    start.as_str() >= timeline_start && end.as_str() <= timeline_end
}

fn lower_pack_rule_schedule(
    rule: &cfdl_pack::LoweringRule,
    time_calendar: &str,
    time_start: &str,
    timeline_end: &str,
) -> IrSchedule {
    if rule.schedule_kind.eq_ignore_ascii_case("on_date") {
        IrSchedule {
            kind: "OnDate".to_string(),
            on: Some(normalize_date(&rule.schedule_from)),
            every: None,
            from: None,
            to: None,
            on_rule: None,
            phase: None,
        }
    } else {
        IrSchedule {
            kind: "Every".to_string(),
            on: None,
            every: Some(time_calendar.to_string()),
            from: Some(normalize_date(if rule.schedule_from.is_empty() {
                time_start
            } else {
                &rule.schedule_from
            })),
            to: Some(normalize_date(if rule.schedule_to.is_empty() {
                timeline_end
            } else {
                &rule.schedule_to
            })),
            on_rule: None,
            phase: None,
        }
    }
}

fn validate_expressions(resolve_output: &cfdl_resolver::ResolveOutput) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Stream(stream) = &source_stmt.statement else {
            continue;
        };
        if let Some(amount) = &stream.amount {
            if let Err(err) = cfdl_expr::compile_expr(&amount.src) {
                diags.push(Diagnostic {
                    code: err.code.to_string(),
                    severity: "error".to_string(),
                    message: err.message,
                    file: Some(source_stmt.file.clone()),
                    span: Some(map_span(amount.span)),
                    path: None,
                    hint: None,
                    notes: vec![format!("stream '{}', amount expression", stream.name)],
                });
            }
        }
        if let Some(active_when) = &stream.active_when {
            if let Err(err) = cfdl_expr::compile_expr(&active_when.src) {
                diags.push(Diagnostic {
                    code: err.code.to_string(),
                    severity: "error".to_string(),
                    message: err.message,
                    file: Some(source_stmt.file.clone()),
                    span: Some(map_span(active_when.span)),
                    path: None,
                    hint: None,
                    notes: vec![format!("stream '{}', active_when expression", stream.name)],
                });
            }
        }
    }
    sort_compile_diagnostics(&mut diags);
    diags
}

fn sort_compile_diagnostics(diags: &mut [Diagnostic]) {
    diags.sort_by(|a, b| {
        a.file
            .cmp(&b.file)
            .then(
                a.span
                    .as_ref()
                    .map(|s| s.start_line)
                    .cmp(&b.span.as_ref().map(|s| s.start_line)),
            )
            .then(
                a.span
                    .as_ref()
                    .map(|s| s.start_col)
                    .cmp(&b.span.as_ref().map(|s| s.start_col)),
            )
            .then(a.code.cmp(&b.code))
    });
}

fn lower_schedule(
    schedule: Option<&cfdl_parser::ScheduleSpec>,
    time_calendar: &str,
    time_start: &str,
    timeline_end: &str,
) -> IrSchedule {
    let Some(schedule) = schedule else {
        return IrSchedule {
            kind: "OnDate".to_string(),
            on: Some(time_start.to_string()),
            every: None,
            from: None,
            to: None,
            on_rule: None,
            phase: None,
        };
    };

    let on_rule = schedule.day_of_month.map(|day| IrOnRule {
        kind: "DayOfMonth".to_string(),
        day,
    });
    match &schedule.kind {
        ScheduleKind::OnDate => IrSchedule {
            kind: "OnDate".to_string(),
            on: Some(normalize_date(
                schedule.from.as_deref().unwrap_or(time_start),
            )),
            every: None,
            from: None,
            to: None,
            on_rule: None,
            phase: None,
        },
        ScheduleKind::Every => IrSchedule {
            kind: "Every".to_string(),
            on: None,
            every: Some(time_calendar.to_string()),
            from: Some(normalize_date(
                schedule.from.as_deref().unwrap_or(time_start),
            )),
            to: Some(normalize_date(
                schedule.to.as_deref().unwrap_or(timeline_end),
            )),
            on_rule,
            phase: None,
        },
        ScheduleKind::PhaseEnter { phase } => IrSchedule {
            kind: "PhaseEnter".to_string(),
            on: None,
            every: None,
            from: None,
            to: None,
            on_rule: None,
            phase: Some(phase.clone()),
        },
        ScheduleKind::EveryPhase { phase } => IrSchedule {
            kind: "EveryPhase".to_string(),
            on: None,
            every: Some(time_calendar.to_string()),
            from: None,
            to: None,
            on_rule,
            phase: Some(phase.clone()),
        },
    }
}

fn find_model_name(resolve_output: &cfdl_resolver::ResolveOutput) -> Option<String> {
    resolve_output
        .source_statements
        .iter()
        .find_map(|source_stmt| {
            if let Stmt::Model(model) = &source_stmt.statement {
                Some(model.name.clone())
            } else {
                None
            }
        })
}

fn find_time(resolve_output: &cfdl_resolver::ResolveOutput) -> Option<(String, String, u32)> {
    resolve_output
        .source_statements
        .iter()
        .find_map(|source_stmt| {
            if let Stmt::Time(time) = &source_stmt.statement {
                Some((
                    cadence_to_frequency(time.cadence).to_string(),
                    normalize_date(&time.from),
                    time.periods,
                ))
            } else {
                None
            }
        })
}

fn cadence_to_frequency(cadence: Cadence) -> &'static str {
    match cadence {
        Cadence::Daily => "daily",
        Cadence::Monthly => "monthly",
        Cadence::Quarterly => "quarterly",
        Cadence::Annual => "annual",
    }
}

fn normalize_date(raw: &str) -> String {
    let parts: Vec<&str> = raw.split('-').collect();
    match parts.as_slice() {
        [year, month] => format!("{year}-{month}-01"),
        [_, _, _] => raw.to_string(),
        _ => raw.to_string(),
    }
}

fn add_periods_for_timeline_end(start: &str, calendar: &str, periods: u32) -> String {
    let Some((year, month, day)) = parse_ymd(start) else {
        return start.to_string();
    };
    if periods == 0 {
        return format!("{year:04}-{month:02}-{day:02}");
    }
    let offset = periods.saturating_sub(1);
    match calendar {
        "daily" => format!("{year:04}-{month:02}-{day:02}"),
        "monthly" => add_months(year, month, day, offset as i32),
        "quarterly" => add_months(year, month, day, (offset as i32) * 3),
        "annual" => add_months(year, month, day, (offset as i32) * 12),
        _ => format!("{year:04}-{month:02}-{day:02}"),
    }
}

fn add_months(year: i32, month: u32, day: u32, months: i32) -> String {
    let total = (year * 12 + (month as i32 - 1)) + months;
    let out_year = total.div_euclid(12);
    let out_month = (total.rem_euclid(12) + 1) as u32;
    let out_day = day.min(days_in_month(out_year, out_month));
    format!("{out_year:04}-{out_month:02}-{out_day:02}")
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn parse_ymd(value: &str) -> Option<(i32, u32, u32)> {
    let parts: Vec<&str> = value.split('-').collect();
    match parts.as_slice() {
        [year, month, day] => Some((year.parse().ok()?, month.parse().ok()?, day.parse().ok()?)),
        [year, month] => Some((year.parse().ok()?, month.parse().ok()?, 1)),
        _ => None,
    }
}

fn deterministic_id(kind: &str, stable_key: &str, seed: &str) -> String {
    hash_hex(&format!("{kind}:{stable_key}:{seed}"))
}

fn hash_hex(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    let mut out = String::with_capacity(32);
    for byte in digest.iter().take(16) {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn map_span(span: cfdl_parser::Span) -> Span {
    Span {
        start_line: span.start_line,
        start_col: span.start_col,
        end_line: span.end_line,
        end_col: span.end_col,
    }
}

fn stable_key(source_file: &str, symbol_or_name: &str) -> String {
    format!("{source_file}::{symbol_or_name}")
}
