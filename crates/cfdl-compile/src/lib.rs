mod pack_validation;

use cfdl_parser::{Cadence, ScheduleKind, Stmt};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct CompileOptions {
    pub packs_dir: Option<PathBuf>,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Clone, PartialEq)]
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

/// Compile a model directory to an IR JSON string with options.
pub fn compile_to_json_with_options(
    model_root: &Path,
    options: &CompileOptions,
) -> Result<String, Vec<Diagnostic>> {
    let provider = cfdl_resolver::FsProvider::new(model_root.to_path_buf());
    // Preserve the historical filesystem default: <model_root>/packs when the
    // caller does not specify a pack directory.
    let effective = CompileOptions {
        packs_dir: Some(
            options
                .packs_dir
                .clone()
                .unwrap_or_else(|| model_root.join("packs")),
        ),
    };
    compile_json_from_provider(&provider, "model.cfdl", &effective)
}

/// Compile a model from an in-memory file map (root-relative path -> source).
///
/// `root_file` names the entry module (typically `"model.cfdl"`). Pack
/// resolution uses `options.packs_dir` when set, else the embedded pack
/// registry (see [`resolve_active_pack`]). This is the filesystem-free entry
/// point used by the WASM playground and the API server.
pub fn compile_sources_to_json(
    files: &std::collections::BTreeMap<String, String>,
    root_file: &str,
    options: &CompileOptions,
) -> Result<String, Vec<Diagnostic>> {
    let provider = cfdl_resolver::MemoryProvider::new(files.clone());
    compile_json_from_provider(&provider, root_file, options)
}

fn compile_json_from_provider(
    provider: &dyn cfdl_resolver::SourceProvider,
    root_file: &str,
    options: &CompileOptions,
) -> Result<String, Vec<Diagnostic>> {
    let (resolve_output, symbols) = pipeline_with(provider, root_file)?;

    let active_pack = resolve_active_pack_from(&resolve_output, options)?;

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
    serde_json::to_string_pretty(&ir).map_err(|err| {
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
    })
}

/// Compile a model directory to an IR JSON file with options.
pub fn compile_to_file_with_options(
    model_root: &Path,
    out_path: &Path,
    options: &CompileOptions,
) -> Result<(), Vec<Diagnostic>> {
    let json = compile_to_json_with_options(model_root, options)?;

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
    validate_only_with(model_root, &CompileOptions::default())
}

/// Validate a model without emitting IR.
///
/// Takes the same options as `compile` so a pack-based model validates the
/// way it compiles: contracts lowered by pack rules legitimately have no
/// `effects` block, and without the pack registry every one of them would be
/// reported as `E2002_CONTRACT_MISSING_EFFECTS`.
pub fn validate_only_with(
    model_root: &Path,
    options: &CompileOptions,
) -> Result<(), Vec<Diagnostic>> {
    // Validation is a compile that discards the IR. Running the same pipeline
    // is what guarantees `validate` and `compile` can never disagree: a
    // pack-lowered contract legitimately has no `effects` block (so a
    // pack-blind check reports E2002 on every valid pack model), and pack
    // domain validations only run during lowering.
    compile_to_json_with_options(model_root, options).map(|_| ())
}

/// Validate a model from an in-memory file map without emitting IR.
pub fn validate_sources(
    files: &std::collections::BTreeMap<String, String>,
    root_file: &str,
) -> Result<(), Vec<Diagnostic>> {
    let provider = cfdl_resolver::MemoryProvider::new(files.clone());
    let (resolve_output, symbols) = pipeline_with(&provider, root_file)?;
    let diagnostics = cfdl_validate::validate(&resolve_output, &symbols);
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics.into_iter().map(map_validation_diag).collect())
    }
}

fn pipeline_with(
    provider: &dyn cfdl_resolver::SourceProvider,
    root_file: &str,
) -> Result<(cfdl_resolver::ResolveOutput, cfdl_resolver::SymbolTables), Vec<Diagnostic>> {
    let source = provider.read(root_file).ok_or_else(|| {
        vec![Diagnostic {
            code: "E1202_IMPORT_NOT_FOUND".to_string(),
            severity: "error".to_string(),
            message: format!("Model root is missing required file '{root_file}'."),
            file: Some(root_file.to_string()),
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

    let parse_result = cfdl_parser::parse(root_file, &source, &tokens);
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
        relative_path: root_file.to_string(),
        full_path: PathBuf::from(root_file),
        ast: root_ast,
    };
    let resolve_output = match cfdl_resolver::resolve_imports_with(provider, root_module) {
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
    validations: Vec<cfdl_pack::PackValidation>,
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
    time_periods: u32,
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    curves: Vec<IrCurve>,
    contracts: Vec<IrContract>,
    streams: Vec<IrStream>,
    events: Vec<serde_json::Value>,
    options: Vec<serde_json::Value>,
    runs: Vec<IrRun>,

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
    #[serde(skip_serializing_if = "is_zero_u32")]
    projection: u32,
}

fn is_zero_u32(v: &u32) -> bool {
    *v == 0
}

#[derive(Debug, Serialize)]
struct IrDateRange {
    start: String,
    end: String,
}

#[derive(Debug, Serialize)]
struct IrCurve {
    name: String,
    /// "step" (flat-forward) or "linear".
    interpolation: String,
    /// Points sorted ascending by date.
    points: Vec<IrCurvePoint>,
}

#[derive(Debug, Serialize)]
struct IrCurvePoint {
    date: String,
    value: f64,
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
    name: String,
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
    /// Annuity due: payment at the start of each interval. Omitted for an
    /// ordinary annuity, which is the default and the common case.
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    due: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    on_rule: Option<IrOnRule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    phase: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    convention: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    calendar: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    except_dates: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    also_dates: Vec<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    trials: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
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
    let (time_calendar, time_start, time_periods, time_projection) = find_time(resolve_output)
        .unwrap_or_else(|| ("monthly".to_string(), "1970-01-01".to_string(), 1, 0));
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
                name: name.clone(),
                range: IrDateRange {
                    start: normalize_date(&phase.from),
                    end: normalize_date(&phase.to),
                },
            };
            Some(((name, source_stmt.file.clone()), ir_phase))
        })
        .collect();
    phases.sort_by(|a, b| a.0.cmp(&b.0));

    let phase_map: BTreeMap<String, (String, String)> = phases
        .iter()
        .map(|((name, _file), ir_phase)| {
            (
                name.clone(),
                (ir_phase.range.start.clone(), ir_phase.range.end.clone()),
            )
        })
        .collect();

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
                    symbol: contract
                        .subject_entity
                        .clone()
                        .unwrap_or_else(|| first_entity_symbol.clone()),
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

    let mut streams: Vec<((String, String), IrStream)> = Vec::new();
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Stream(stream) = &source_stmt.statement else {
            continue;
        };
        let stable_key = stable_key(&source_stmt.file, &stream.name);
        let schedule = lower_schedule(
            stream.schedule.as_ref(),
            &time_calendar,
            &time_start,
            &timeline_end,
            &phase_map,
        )
        .map_err(|msg| {
            vec![Diagnostic {
                code: "E5005_PHASE_NOT_FOUND".to_string(),
                severity: "error".to_string(),
                message: msg,
                file: Some(source_stmt.file.clone()),
                span: Some(map_span(stream.span)),
                path: None,
                hint: None,
                notes: vec![],
            }]
        })?;
        let ir_stream = IrStream {
            id: deterministic_id("Stream", &stable_key, &id_seed),
            name: stream.name.clone(),
            owner: IrEntityRef {
                symbol: stream.attached_entity.clone(),
            },
            direction: stream.direction.as_deref().unwrap_or("outflow").to_string(),
            currency: stream
                .currency
                .as_ref()
                .cloned()
                .unwrap_or_else(|| model_currency.clone()),
            schedule,
            amount: IrExpr {
                lang: stream
                    .amount
                    .as_ref()
                    .map(|expr| expr.lang.clone())
                    .unwrap_or_else(|| "cfdl".to_string()),
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
                    .unwrap_or_else(|| "cfdl".to_string()),
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
        streams.push(((stream.name.clone(), source_stmt.file.clone()), ir_stream));
    }
    let lowered = lower_contract_streams(
        resolve_output,
        active_pack,
        LoweringContext {
            id_seed: &id_seed,
            model_currency: &model_currency,
            time_calendar: &time_calendar,
            time_start: &time_start,
            time_periods,
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
    {
        // Two lowered streams with one name would silently merge in results
        // reporting; make it a hard error instead.
        let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
        for ((name, key), _stream) in &lowered.streams {
            if let Some(previous) = seen.insert(name.as_str(), key.as_str()) {
                return Err(vec![Diagnostic {
                    code: "E5007_DUPLICATE_LOWERED_STREAM".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "Multiple contracts lower to stream '{name}' ({previous} and {key}); \
                         use suffixed contract instances with a templated stream_name \
                         ({{{{contract.dot_suffix}}}}) so each emits a distinct stream."
                    ),
                    file: None,
                    span: None,
                    path: None,
                    hint: None,
                    notes: vec![],
                }]);
            }
        }
    }
    streams.extend(lowered.streams);
    streams.sort_by(|a, b| a.0.cmp(&b.0));

    let (assume_constants, assume_random, assume_diags) = lower_assumptions(resolve_output);
    if !assume_diags.is_empty() {
        let mut diagnostics = assume_diags;
        sort_compile_diagnostics(&mut diagnostics);
        return Err(diagnostics);
    }

    let (ir_curves, curve_diags) = lower_curves(resolve_output);
    if !curve_diags.is_empty() {
        let mut diagnostics = curve_diags;
        sort_compile_diagnostics(&mut diagnostics);
        return Err(diagnostics);
    }

    let (ir_events, ir_options, event_diags) = lower_events_options(resolve_output, &id_seed);
    if !event_diags.is_empty() {
        let mut diagnostics = event_diags;
        sort_compile_diagnostics(&mut diagnostics);
        return Err(diagnostics);
    }

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
            projection: time_projection,
        },
        phases: phases.into_iter().map(|(_, phase)| phase).collect(),
        entities: entities.into_iter().map(|(_, entity)| entity).collect(),
        assumptions: IrAssumptions {
            constants: assume_constants,
            random: assume_random,
        },
        curves: ir_curves,
        contracts: contracts
            .into_iter()
            .map(|(_, contract)| contract)
            .collect(),
        streams: streams
            .into_iter()
            .map(|(_, stream)| {
                let mut s = stream;
                s.amount.src = coerce_numeric_literals(&s.amount.src);
                s.active_when.src = coerce_numeric_literals(&s.active_when.src);
                s
            })
            .collect(),
        events: ir_events,
        options: ir_options,
        runs: {
            let declared: Vec<IrRun> = resolve_output
                .source_statements
                .iter()
                .filter_map(|source_stmt| match &source_stmt.statement {
                    Stmt::Run(run) => Some(IrRun {
                        kind: run.kind.clone(),
                        trials: run.trials,
                        seed: run.seed,
                    }),
                    _ => None,
                })
                .collect();
            if declared.is_empty() {
                vec![IrRun {
                    kind: "deterministic".to_string(),
                    trials: None,
                    seed: None,
                }]
            } else {
                declared
            }
        },

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

/// Pack resolution: use `options.packs_dir` if set, else
/// the embedded pack registry (WASM/server). Requires the `embedded-packs`
/// feature for the embedded fallback.
fn resolve_active_pack_from(
    resolve_output: &cfdl_resolver::ResolveOutput,
    options: &CompileOptions,
) -> Result<Option<ActivePackContext>, Vec<Diagnostic>> {
    resolve_active_pack_inner(resolve_output, options.packs_dir.clone())
}

fn resolve_active_pack_inner(
    resolve_output: &cfdl_resolver::ResolveOutput,
    packs_dir: Option<PathBuf>,
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

    let pack_diag = |message: String, hint: Option<String>, notes: Vec<String>| {
        vec![Diagnostic {
            code: "E4004_MISSING_PACK".to_string(),
            severity: "error".to_string(),
            message,
            file: Some(file.clone()),
            span: Some(map_span(use_pack.span)),
            path: None,
            hint,
            notes,
        }]
    };

    let registry = match packs_dir.as_ref() {
        Some(dir) => cfdl_pack::PackRegistry::load_from_dir(dir).map_err(|err| {
            pack_diag(
                err.message,
                None,
                vec![format!("pack root: {}", dir.display())],
            )
        })?,
        None => load_embedded_registry(&pack_diag)?,
    };
    let Some(active) = registry.active_pack(&use_pack.name, &use_pack.version) else {
        let where_ = match packs_dir.as_ref() {
            Some(dir) => format!("under '{}'", dir.display()),
            None => "in the embedded pack registry".to_string(),
        };
        return Err(pack_diag(
            format!(
                "Pack '{}@{}' was not found {where_}.",
                use_pack.name, use_pack.version
            ),
            Some("Add a matching pack manifest or pass --packs <dir>.".to_string()),
            vec![],
        ));
    };

    Ok(Some(ActivePackContext {
        name: active.name.clone(),
        version: active.version.clone(),
        lowering_rules: registry.lowering_rules(&active.name),
        validations: registry.validations(&active.name),
    }))
}

#[cfg(feature = "embedded-packs")]
fn load_embedded_registry(
    pack_diag: &dyn Fn(String, Option<String>, Vec<String>) -> Vec<Diagnostic>,
) -> Result<cfdl_pack::PackRegistry, Vec<Diagnostic>> {
    cfdl_pack::PackRegistry::load_embedded().map_err(|err| {
        pack_diag(
            err.message,
            None,
            vec!["embedded pack registry".to_string()],
        )
    })
}

#[cfg(not(feature = "embedded-packs"))]
fn load_embedded_registry(
    pack_diag: &dyn Fn(String, Option<String>, Vec<String>) -> Vec<Diagnostic>,
) -> Result<cfdl_pack::PackRegistry, Vec<Diagnostic>> {
    Err(pack_diag(
        "No pack directory was provided and this build has no embedded packs.".to_string(),
        Some("Pass a packs directory, or build with the `embedded-packs` feature.".to_string()),
        vec![],
    ))
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
                .any(|rule| rule_matches_contract(&rule.contract_name, &contract.name))
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

    // Terms may defer to a declared input. Collect the declared names first so
    // a term naming an input that does not exist is a compile error rather
    // than an expression that quietly resolves to nothing at runtime.
    let declared_inputs: BTreeSet<String> = resolve_output
        .source_statements
        .iter()
        .filter_map(|stmt| match &stmt.statement {
            Stmt::Assume(assume) => Some(assume.name.clone()),
            _ => None,
        })
        .collect();

    let mut lowered = Vec::new();
    let mut diagnostics = Vec::new();
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Contract(contract) = &source_stmt.statement else {
            continue;
        };
        for (key, term) in &contract.terms {
            if let Some(name) = term.input_name() {
                if !declared_inputs.contains(name) {
                    diagnostics.push(lowering_rule_diag(
                        "E5010_TERM_UNKNOWN_INPUT",
                        &format!(
                            "Contract '{}' term '{}' references input '{}', which is not declared. Add `assume {} = <value>` or `assume {} ~ <Dist>(...)`.",
                            contract.name, key, name, name, name
                        ),
                        source_stmt,
                        term.span,
                    ));
                }
            }
        }
        let before = diagnostics.len();
        diagnostics.extend(validate_pack_contract(
            pack,
            source_stmt,
            contract,
            ctx.time_calendar,
            ctx.time_start,
            ctx.time_periods,
            ctx.timeline_end,
        ));
        if diagnostics[before..]
            .iter()
            .any(|diag| diag.severity == "error")
        {
            continue;
        }
        for rule in &rules {
            if !rule_matches_contract(&rule.contract_name, &contract.name) {
                continue;
            }
            if !rule.stream_name.contains("{{") && !is_qualified_name(&rule.stream_name) {
                diagnostics.push(lowering_rule_diag(
                    "E5004_INVALID_LOWERING_RULE",
                    &format!(
                        "Pack lowering rule '{}' generated invalid stream_name '{}'; expected dotted qualified name.",
                        rule.id, rule.stream_name
                    ),
                    source_stmt,
                    contract.span,
                ));
                continue;
            }

            let stable_key = format!("{}::{}::{}", source_stmt.file, contract.name, rule.id);
            let owner_symbol = if rule.owner_entity.is_empty() || rule.owner_entity == "${subject}"
            {
                contract
                    .subject_entity
                    .clone()
                    .unwrap_or_else(|| ctx.default_owner.to_string())
            } else {
                rule.owner_entity.clone()
            };
            if !is_qualified_name(&owner_symbol) {
                diagnostics.push(lowering_rule_diag(
                    "E5004_INVALID_LOWERING_RULE",
                    &format!(
                        "Pack lowering rule '{}' resolved invalid owner_entity '{}'; expected dotted qualified entity symbol.",
                        rule.id, owner_symbol
                    ),
                    source_stmt,
                    contract.span,
                ));
                continue;
            }
            // Template expansion: resolve {{contract.<key>}} placeholders from
            // contract terms (term_start/term_end from the term range), then
            // rule defaults. Missing keys are compile errors.
            let resolve = |key: &str| -> Option<String> {
                let from_contract = match key {
                    "term_start" => contract.term_start.as_deref().map(normalize_date),
                    "term_end" => contract.term_end.as_deref().map(normalize_date),
                    // Full contract name / the suffix beyond the rule's
                    // contract_name (e.g. "tenant_a") for per-instance
                    // stream naming.
                    "name" => Some(contract.name.clone()),
                    "suffix" => Some(
                        contract
                            .name
                            .strip_prefix(&rule.contract_name)
                            .map(|rest| rest.trim_start_matches('.').to_string())
                            .unwrap_or_default(),
                    ),
                    // Like `suffix` but with its leading dot kept (empty for
                    // an exact-name match) so stream names stay valid for
                    // both suffixed and unsuffixed contracts.
                    "dot_suffix" => Some(
                        contract
                            .name
                            .strip_prefix(&rule.contract_name)
                            .unwrap_or_default()
                            .to_string(),
                    ),
                    _ => contract.terms.get(key).map(|term| term.value.clone()),
                };
                from_contract.or_else(|| rule.defaults.get(key).cloned())
            };
            let mut expanded_rule = rule.clone();
            let mut missing_keys: Vec<String> = Vec::new();
            for (slot, target) in [
                (&rule.amount_expr, &mut expanded_rule.amount_expr),
                (&rule.schedule_from, &mut expanded_rule.schedule_from),
                (&rule.schedule_to, &mut expanded_rule.schedule_to),
                (&rule.stream_name, &mut expanded_rule.stream_name),
            ] {
                match cfdl_pack::expand_rule_template(slot, &resolve) {
                    Ok(expanded) => *target = expanded,
                    Err(missing) => {
                        for key in missing {
                            if !missing_keys.contains(&key) {
                                missing_keys.push(key);
                            }
                        }
                    }
                }
            }
            if !missing_keys.is_empty() {
                for key in &missing_keys {
                    diagnostics.push(lowering_rule_diag(
                        "E5006_MISSING_CONTRACT_TERM",
                        &format!(
                            "Pack lowering rule '{}' requires contract term '{}' (no default declared); contract '{}' does not provide it.",
                            rule.id, key, contract.name
                        ),
                        source_stmt,
                        contract.span,
                    ));
                }
                continue;
            }
            if !is_qualified_name(&expanded_rule.stream_name) {
                diagnostics.push(lowering_rule_diag(
                    "E5004_INVALID_LOWERING_RULE",
                    &format!(
                        "Pack lowering rule '{}' expanded to invalid stream_name '{}' for contract '{}'; expected dotted qualified name (suffix the contract, e.g. {}.unit_a).",
                        rule.id, expanded_rule.stream_name, contract.name, rule.contract_name
                    ),
                    source_stmt,
                    contract.span,
                ));
                continue;
            }
            let rule = &expanded_rule;

            let schedule =
                lower_pack_rule_schedule(rule, ctx.time_calendar, ctx.time_start, ctx.timeline_end);
            let amount_src = rule.amount_expr.clone();
            // Pack terms are applied declaratively via rule templates; the
            // legacy hardcoded paths (CRE, then OpCo) were removed with the
            // v1 rule migrations.

            // Template expansion is a textual splice, so a term can produce an
            // expression the parser rejects. Catch it here: the engine's
            // fallback is to evaluate a failed expression as zero and carry on
            // with a warning, which turns a malformed model into a silently
            // empty stream.
            if let Err(err) = cfdl_expr::compile_expr(&amount_src) {
                diagnostics.push(lowering_rule_diag(
                    "E5009_LOWERED_EXPR_INVALID",
                    &format!(
                        "Pack lowering rule '{}' produced an invalid amount expression for contract '{}' [{}]: {}. Expanded to: {}",
                        rule.id, contract.name, err.code, err.message, amount_src
                    ),
                    source_stmt,
                    contract.span,
                ));
                continue;
            }

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
                        lang: "cfdl".to_string(),
                        src: amount_src,
                    },
                    active_when: IrExpr {
                        lang: "cfdl".to_string(),
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
    _timeline_calendar: &str,
    timeline_start: &str,
    _timeline_periods: u32,
    timeline_end: &str,
) -> Vec<Diagnostic> {
    // Domain constraints are declared by the pack in validations.toml; the
    // compiler supplies only what a pack cannot see — the source span and
    // whether the contract's term sits inside the model timeline.
    pack_validation::evaluate(
        &pack.validations,
        contract,
        valid_contract_term_range(contract, timeline_start, timeline_end),
        |code, message, severity, span| {
            let mut diag = pack_diag(code, message, source_stmt, span);
            diag.severity = severity.as_str().to_string();
            diag
        },
    )
}

fn pack_diag(
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

fn lowering_rule_diag(
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

/// A rule matches its exact contract name, or any suffixed instance of it
/// (`cre.lease_unit` matches `cre.lease_unit.tenant_a`) so one rule can lower
/// many per-tenant/per-unit contracts.
fn rule_matches_contract(rule_contract: &str, contract_name: &str) -> bool {
    contract_name == rule_contract
        || contract_name
            .strip_prefix(rule_contract)
            .is_some_and(|rest| rest.starts_with('.'))
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
            due: false,
            on: Some(normalize_date(&rule.schedule_from)),
            every: None,
            from: None,
            to: None,
            on_rule: None,
            phase: None,
            convention: None,
            calendar: None,
            except_dates: Vec::new(),
            also_dates: Vec::new(),
        }
    } else {
        IrSchedule {
            kind: "Every".to_string(),
            due: rule.schedule_due,
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
            convention: None,
            calendar: None,
            except_dates: Vec::new(),
            also_dates: Vec::new(),
        }
    }
}

type EventOptionMaps = (
    Vec<serde_json::Value>,
    Vec<serde_json::Value>,
    Vec<Diagnostic>,
);

/// Lower event/option statements into IR per $defs Event / Option / Action.
fn lower_events_options(
    resolve_output: &cfdl_resolver::ResolveOutput,
    id_seed: &str,
) -> EventOptionMaps {
    let mut events = Vec::new();
    let mut options = Vec::new();
    let mut diags = Vec::new();

    for source_stmt in &resolve_output.source_statements {
        match &source_stmt.statement {
            Stmt::Event(event) => {
                let diag = |code: &str, message: String| Diagnostic {
                    code: code.to_string(),
                    severity: "error".to_string(),
                    message,
                    file: Some(source_stmt.file.clone()),
                    span: Some(map_span(event.span)),
                    path: None,
                    hint: None,
                    notes: vec![format!("event '{}'", event.name)],
                };
                if let Err(err) = cfdl_expr::compile_expr(&event.when) {
                    diags.push(diag(&err.code, err.message));
                    continue;
                }
                let mut actions = Vec::new();
                let mut bad = false;
                for action in &event.actions {
                    use cfdl_parser::EventAction as A;
                    let value = match action {
                        A::SetEntityField {
                            entity,
                            field,
                            value,
                        } => {
                            if let Err(err) = cfdl_expr::compile_expr(value) {
                                diags.push(diag(&err.code, err.message));
                                bad = true;
                                continue;
                            }
                            serde_json::json!({
                                "kind": "SetEntityField",
                                "entity": { "symbol": entity },
                                "field": field,
                                "value": { "lang": "cfdl", "src": value },
                            })
                        }
                        A::ActivateStream(name) => {
                            serde_json::json!({ "kind": "ActivateStream", "stream": name })
                        }
                        A::DeactivateStream(name) => {
                            serde_json::json!({ "kind": "DeactivateStream", "stream": name })
                        }
                        A::ActivateContract(name) => {
                            serde_json::json!({ "kind": "ActivateContract", "contract": name })
                        }
                        A::DeactivateContract(name) => {
                            serde_json::json!({ "kind": "DeactivateContract", "contract": name })
                        }
                        A::ExerciseOption(name) => {
                            serde_json::json!({ "kind": "ExerciseOption", "option": name })
                        }
                    };
                    actions.push(value);
                }
                if bad {
                    continue;
                }
                let stable = stable_key(&source_stmt.file, &event.name);
                events.push(serde_json::json!({
                    "id": deterministic_id("Event", &stable, id_seed),
                    "name": event.name,
                    "when": { "lang": "cfdl", "src": event.when },
                    "actions": actions,
                    "provenance": {
                        "source_file": source_stmt.file,
                        "source_span": map_span(event.span),
                    },
                }));
            }
            Stmt::Option(option) => {
                let diag = |code: &str, message: String| Diagnostic {
                    code: code.to_string(),
                    severity: "error".to_string(),
                    message,
                    file: Some(source_stmt.file.clone()),
                    span: Some(map_span(option.span)),
                    path: None,
                    hint: None,
                    notes: vec![format!("option '{}'", option.name)],
                };
                let Some(exercise_when) = &option.exercise_when else {
                    diags.push(diag(
                        "E2401_OPTION_MISSING_EXERCISE",
                        "Option requires an 'exercise when' clause.".to_string(),
                    ));
                    continue;
                };
                let Some(payoff) = &option.payoff else {
                    diags.push(diag(
                        "E2402_OPTION_MISSING_PAYOFF",
                        "Option requires a 'payoff' clause.".to_string(),
                    ));
                    continue;
                };
                let mut bad = false;
                for src in [exercise_when, payoff] {
                    if let Err(err) = cfdl_expr::compile_expr(src) {
                        diags.push(diag(&err.code, err.message));
                        bad = true;
                    }
                }
                if bad {
                    continue;
                }
                let stable = stable_key(&source_stmt.file, &option.name);
                let mut obj = serde_json::json!({
                    "id": deterministic_id("Option", &stable, id_seed),
                    "name": option.name,
                    "type": option.type_name,
                    "exercise_when": { "lang": "cfdl", "src": exercise_when },
                    "payoff": { "lang": "cfdl", "src": payoff },
                    "provenance": {
                        "source_file": source_stmt.file,
                        "source_span": map_span(option.span),
                    },
                });
                if let Some(phase) = &option.exercisable_in {
                    obj["exercisable_in_phase"] = serde_json::json!(phase);
                }
                options.push(obj);
            }
            _ => {}
        }
    }
    (events, options, diags)
}

type AssumeMaps = (
    BTreeMap<String, serde_json::Value>,
    BTreeMap<String, serde_json::Value>,
    Vec<Diagnostic>,
);

/// Lower `assume` statements into IR assumptions (constants + random), per
/// docs/schemas/ir.schema.json $defs AssumeConstant / AssumeRandom.
/// Lower `curve` statements into IR curves: dedupe names, sort points by
/// date, reject duplicate point dates.
fn lower_curves(resolve_output: &cfdl_resolver::ResolveOutput) -> (Vec<IrCurve>, Vec<Diagnostic>) {
    let mut curves: Vec<IrCurve> = Vec::new();
    let mut diags = Vec::new();

    for source_stmt in &resolve_output.source_statements {
        let Stmt::Curve(curve) = &source_stmt.statement else {
            continue;
        };
        let make_diag = |message: String| Diagnostic {
            code: "E5008_INVALID_CURVE".to_string(),
            severity: "error".to_string(),
            message,
            file: Some(source_stmt.file.clone()),
            span: Some(map_span(curve.span)),
            path: None,
            hint: None,
            notes: vec![format!("curve '{}'", curve.name)],
        };

        if curves.iter().any(|c| c.name == curve.name) {
            diags.push(make_diag(format!(
                "Curve '{}' is declared more than once.",
                curve.name
            )));
            continue;
        }

        // Sortable key: month-only dates ("2026-01") normalize to day 1.
        let sort_key = |date: &str| -> Option<(i32, u32, u32)> {
            let mut parts = date.split('-');
            let year = parts.next()?.parse().ok()?;
            let month = parts.next()?.parse().ok()?;
            let day = match parts.next() {
                Some(d) => d.parse().ok()?,
                None => 1,
            };
            Some((year, month, day))
        };

        let mut points: Vec<((i32, u32, u32), IrCurvePoint)> = Vec::new();
        let mut bad = false;
        for (date, raw_value) in &curve.points {
            let Some(key) = sort_key(date) else {
                diags.push(make_diag(format!("Curve point date '{date}' is invalid.")));
                bad = true;
                break;
            };
            let Ok(value) = raw_value.parse::<f64>() else {
                diags.push(make_diag(format!(
                    "Curve point value '{raw_value}' is not numeric."
                )));
                bad = true;
                break;
            };
            if points.iter().any(|(k, _)| *k == key) {
                diags.push(make_diag(format!(
                    "Curve '{}' declares more than one point for date '{date}'.",
                    curve.name
                )));
                bad = true;
                break;
            }
            points.push((
                key,
                IrCurvePoint {
                    date: normalize_date(date),
                    value,
                },
            ));
        }
        if bad {
            continue;
        }
        points.sort_by_key(|(key, _)| *key);
        curves.push(IrCurve {
            name: curve.name.clone(),
            interpolation: curve.interpolation.clone(),
            points: points.into_iter().map(|(_, p)| p).collect(),
        });
    }

    curves.sort_by(|a, b| a.name.cmp(&b.name));
    (curves, diags)
}

fn lower_assumptions(resolve_output: &cfdl_resolver::ResolveOutput) -> AssumeMaps {
    let mut constants = BTreeMap::new();
    let mut random = BTreeMap::new();
    let mut diags = Vec::new();

    for source_stmt in &resolve_output.source_statements {
        let Stmt::Assume(assume) = &source_stmt.statement else {
            continue;
        };
        let make_diag = |code: &str, message: String| Diagnostic {
            code: code.to_string(),
            severity: "error".to_string(),
            message,
            file: Some(source_stmt.file.clone()),
            span: Some(map_span(assume.span)),
            path: None,
            hint: None,
            notes: vec![format!("assumption '{}'", assume.name)],
        };

        if constants.contains_key(&assume.name) || random.contains_key(&assume.name) {
            diags.push(make_diag(
                "E1005_DUPLICATE_ASSUME",
                format!("Assumption '{}' is declared more than once.", assume.name),
            ));
            continue;
        }

        if let Some(value_src) = &assume.value {
            if let Err(err) = cfdl_expr::compile_expr(value_src) {
                diags.push(make_diag(&err.code, err.message));
                continue;
            }
            constants.insert(
                assume.name.clone(),
                serde_json::json!({
                    "name": assume.name,
                    "expr": { "lang": "cfdl", "src": value_src },
                    "type": "Decimal",
                }),
            );
        } else if let Some(dist) = &assume.dist {
            let required: &[&[&str]] = match dist.name.as_str() {
                "normal" => &[&["mean"], &["stdev", "stddev"]],
                "lognormal" => &[&["mu"], &["sigma"]],
                "uniform" => &[&["min"], &["max"]],
                "triangular" => &[&["min"], &["mode"], &["max"]],
                other => {
                    diags.push(make_diag(
                        "E2301_ASSUME_UNKNOWN_DIST",
                        format!("Unknown distribution '{other}'."),
                    ));
                    continue;
                }
            };
            let mut params = serde_json::Map::new();
            let mut bad = false;
            for (key, raw) in &dist.args {
                match raw.parse::<f64>() {
                    Ok(v) => {
                        params.insert(key.clone(), serde_json::json!(v));
                    }
                    Err(_) => {
                        diags.push(make_diag(
                            "E2302_ASSUME_INVALID_PARAM",
                            format!("Distribution parameter '{key}' is not a number: {raw}"),
                        ));
                        bad = true;
                    }
                }
            }
            for aliases in required {
                if !aliases.iter().any(|a| params.contains_key(*a)) {
                    diags.push(make_diag(
                        "E2303_ASSUME_MISSING_PARAM",
                        format!(
                            "Distribution '{}' requires parameter '{}'.",
                            dist.name, aliases[0]
                        ),
                    ));
                    bad = true;
                }
            }
            let clip = match &dist.clip {
                Some((lo, hi)) => match (lo.parse::<f64>(), hi.parse::<f64>()) {
                    (Ok(lo), Ok(hi)) if lo <= hi => Some(serde_json::json!([lo, hi])),
                    _ => {
                        diags.push(make_diag(
                            "E2304_ASSUME_INVALID_CLIP",
                            format!("Invalid clip range [{lo}, {hi}]."),
                        ));
                        bad = true;
                        None
                    }
                },
                None => None,
            };
            if bad {
                continue;
            }
            let kind = match dist.name.as_str() {
                "normal" => "Normal",
                "lognormal" => "LogNormal",
                "uniform" => "Uniform",
                _ => "Triangular",
            };
            let mut dist_json = serde_json::json!({ "kind": kind, "params": params });
            if let Some(clip) = clip {
                dist_json["clip"] = clip;
            }
            random.insert(
                assume.name.clone(),
                serde_json::json!({
                    "name": assume.name,
                    "dist": dist_json,
                    "type": "Decimal",
                }),
            );
        }
    }
    (constants, random, diags)
}

fn validate_expressions(resolve_output: &cfdl_resolver::ResolveOutput) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Stream(stream) = &source_stmt.statement else {
            continue;
        };
        for (slot, what) in [
            (stream.amount.as_ref(), "amount"),
            (stream.active_when.as_ref(), "active_when"),
        ] {
            let Some(slot) = slot else { continue };
            if let Err(err) = cfdl_expr::compile_expr(&slot.src) {
                diags.push(Diagnostic {
                    code: err.code.to_string(),
                    severity: "error".to_string(),
                    message: err.message,
                    file: Some(source_stmt.file.clone()),
                    span: Some(expr_error_span(slot, err.span.as_ref())),
                    path: None,
                    hint: None,
                    notes: vec![format!("stream '{}', {} expression", stream.name, what)],
                });
            }
        }
    }
    sort_compile_diagnostics(&mut diags);
    diags
}

/// Map an expression-internal byte-offset span onto file coordinates.
///
/// `slot.src` is the exact source slice covered by `slot.expr_span`, so for a
/// single-line expression the file column is `expr_span.start_col + offset`.
/// Multi-line expressions (rare) fall back to the whole expression span, as do
/// errors without a span.
fn expr_error_span(slot: &cfdl_parser::ExprSlot, err_span: Option<&cfdl_expr::ExprSpan>) -> Span {
    let e = slot.expr_span;
    match err_span {
        Some(s) if e.start_line == e.end_line => {
            let width = slot.src.chars().count() as u32;
            let start = (s.start as u32).min(width.saturating_sub(1));
            // Error spans are byte-exclusive at the end; file cols are inclusive.
            let end = (s.end as u32).clamp(start + 1, width);
            Span {
                start_line: e.start_line,
                start_col: e.start_col + start,
                end_line: e.start_line,
                end_col: e.start_col + end - 1,
            }
        }
        _ => map_span(e),
    }
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
    phase_map: &BTreeMap<String, (String, String)>,
) -> Result<IrSchedule, String> {
    let Some(schedule) = schedule else {
        return Ok(IrSchedule {
            kind: "OnDate".to_string(),
            due: false,
            on: Some(time_start.to_string()),
            every: None,
            from: None,
            to: None,
            on_rule: None,
            phase: None,
            convention: None,
            calendar: None,
            except_dates: Vec::new(),
            also_dates: Vec::new(),
        });
    };

    // The IR's OnRule already declares EndOfMonth; `on eom` now reaches it
    // instead of failing to parse.
    let on_rule = if schedule.end_of_month {
        Some(IrOnRule {
            kind: "EndOfMonth".to_string(),
            day: 0,
        })
    } else {
        schedule.day_of_month.map(|day| IrOnRule {
            kind: "DayOfMonth".to_string(),
            day,
        })
    };
    match &schedule.kind {
        ScheduleKind::OnDate => Ok(IrSchedule {
            kind: "OnDate".to_string(),
            due: false,
            on: Some(normalize_date(
                schedule.from.as_deref().unwrap_or(time_start),
            )),
            every: None,
            from: None,
            to: None,
            on_rule: None,
            phase: None,
            convention: schedule.convention.clone(),
            calendar: schedule.calendar.clone(),
            except_dates: schedule
                .except_dates
                .iter()
                .map(|d| normalize_date(d))
                .collect(),
            also_dates: schedule
                .also_dates
                .iter()
                .map(|d| normalize_date(d))
                .collect(),
        }),
        ScheduleKind::Every => Ok(IrSchedule {
            kind: "Every".to_string(),
            due: schedule.due,
            on: None,
            every: Some(
                schedule
                    .every
                    .as_deref()
                    .map(|i| interval_to_frequency(i).to_string())
                    .unwrap_or_else(|| time_calendar.to_string()),
            ),
            from: Some(normalize_date(
                schedule.from.as_deref().unwrap_or(time_start),
            )),
            to: Some(normalize_date(
                schedule.to.as_deref().unwrap_or(timeline_end),
            )),
            on_rule,
            phase: None,
            convention: schedule.convention.clone(),
            calendar: schedule.calendar.clone(),
            except_dates: schedule
                .except_dates
                .iter()
                .map(|d| normalize_date(d))
                .collect(),
            also_dates: schedule
                .also_dates
                .iter()
                .map(|d| normalize_date(d))
                .collect(),
        }),
        ScheduleKind::PhaseEnter { phase } => {
            let (start, _end) = phase_map.get(phase).ok_or_else(|| {
                format!("Schedule references unknown phase '{phase}'; no matching phase declaration found.")
            })?;
            Ok(IrSchedule {
                kind: "OnDate".to_string(),
                due: false,
                on: Some(start.clone()),
                every: None,
                from: None,
                to: None,
                on_rule: None,
                phase: Some(phase.clone()),
                convention: schedule.convention.clone(),
                calendar: schedule.calendar.clone(),
                except_dates: schedule
                    .except_dates
                    .iter()
                    .map(|d| normalize_date(d))
                    .collect(),
                also_dates: schedule
                    .also_dates
                    .iter()
                    .map(|d| normalize_date(d))
                    .collect(),
            })
        }
        ScheduleKind::EveryPhase { phase } => {
            let (start, end) = phase_map.get(phase).ok_or_else(|| {
                format!("Schedule references unknown phase '{phase}'; no matching phase declaration found.")
            })?;
            Ok(IrSchedule {
                kind: "Every".to_string(),
                due: schedule.due,
                on: None,
                every: Some(
                    schedule
                        .every
                        .as_deref()
                        .map(|i| interval_to_frequency(i).to_string())
                        .unwrap_or_else(|| time_calendar.to_string()),
                ),
                from: Some(start.clone()),
                to: Some(end.clone()),
                on_rule,
                phase: Some(phase.clone()),
                convention: schedule.convention.clone(),
                calendar: schedule.calendar.clone(),
                except_dates: schedule
                    .except_dates
                    .iter()
                    .map(|d| normalize_date(d))
                    .collect(),
                also_dates: schedule
                    .also_dates
                    .iter()
                    .map(|d| normalize_date(d))
                    .collect(),
            })
        }
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

fn find_time(resolve_output: &cfdl_resolver::ResolveOutput) -> Option<(String, String, u32, u32)> {
    resolve_output
        .source_statements
        .iter()
        .find_map(|source_stmt| {
            if let Stmt::Time(time) = &source_stmt.statement {
                Some((
                    cadence_to_frequency(time.cadence).to_string(),
                    normalize_date(&time.from),
                    time.periods,
                    time.projection,
                ))
            } else {
                None
            }
        })
}

/// Source interval (`month`) to the IR's frequency vocabulary (`monthly`).
///
/// The two are deliberately different words: `every month` is an interval, and
/// `time calendar monthly` is a cadence. The IR has always spoken the cadence
/// vocabulary, so normalising here keeps the published schema and every golden
/// unchanged while the source gains the distinction.
fn interval_to_frequency(interval: &str) -> &str {
    match interval {
        "day" => "daily",
        "week" => "weekly",
        "month" => "monthly",
        "quarter" => "quarterly",
        "year" => "annual",
        other => other,
    }
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

fn is_qualified_name(value: &str) -> bool {
    let mut parts = value.split('.');
    let Some(first) = parts.next() else {
        return false;
    };
    if !is_ident_segment(first) {
        return false;
    }
    let mut count = 1usize;
    for part in parts {
        if !is_ident_segment(part) {
            return false;
        }
        count += 1;
    }
    count >= 2
}

fn is_ident_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

// Minimal tokenizer to coerce integer literals to floats in CEL expressions
// e.g. "x + 5" -> "x + 5.0", "10 / 2" -> "10.0 / 2.0" at compilation time.
fn coerce_numeric_literals(expr: &str) -> String {
    let mut out = String::with_capacity(expr.len() + 10);
    let mut current_token = String::new();
    let chars: Vec<char> = expr.chars().collect();
    let len = chars.len();
    let mut i = 0;

    fn flush(out: &mut String, token: &mut String) {
        if token.is_empty() {
            return;
        }
        // Check if pure integer
        if token.chars().all(|c| c.is_ascii_digit()) {
            out.push_str(token);
            out.push_str(".0");
        } else {
            out.push_str(token);
        }
        token.clear();
    }

    while i < len {
        let c = chars[i];

        // String handling: skip content
        if c == '"' || c == '\'' {
            flush(&mut out, &mut current_token);
            out.push(c);
            let quote = c;
            i += 1;
            while i < len {
                let sc = chars[i];
                out.push(sc);
                if sc == quote {
                    // unexpected end of string or escaped?
                    // Simple check: not escaped by backslash
                    if i == 0 || chars[i - 1] != '\\' {
                        break;
                    }
                }
                i += 1;
            }
            i += 1;
            continue;
        }

        if c.is_ascii_digit() {
            current_token.push(c);
        } else if c == '.' || c.is_alphabetic() || c == '_' {
            // Identifier or float or property access
            current_token.push(c);
        } else {
            // Delimiter
            if !current_token.is_empty() {
                // If token contains non-digits, just flush as is.
                // If strictly digits, append .0
                if current_token.chars().all(|tc| tc.is_ascii_digit()) {
                    out.push_str(&current_token);
                    out.push_str(".0");
                } else {
                    out.push_str(&current_token);
                }
                current_token.clear();
            }
            out.push(c);
        }
        i += 1;
    }

    // Final flush
    if !current_token.is_empty() {
        if current_token.chars().all(|tc| tc.is_ascii_digit()) {
            out.push_str(&current_token);
            out.push_str(".0");
        } else {
            out.push_str(&current_token);
        }
    }

    out
}

#[cfg(test)]
mod pack_validation_parity_tests {
    use super::*;
    use std::collections::BTreeMap;

    fn span() -> cfdl_parser::Span {
        cfdl_parser::Span {
            start_line: 1,
            start_col: 1,
            end_line: 1,
            end_col: 2,
        }
    }

    fn contract(name: &str, terms: &[(&str, &str)], term_range: bool) -> cfdl_parser::ContractStmt {
        let mut map = BTreeMap::new();
        for (key, value) in terms {
            map.insert(
                (*key).to_string(),
                cfdl_parser::ContractTerm {
                    value: (*value).to_string(),
                    span: span(),
                },
            );
        }
        cfdl_parser::ContractStmt {
            name: name.to_string(),
            subject_entity: None,
            has_term: term_range,
            has_effects: false,
            term_start: term_range.then(|| "2026-01".to_string()),
            term_end: term_range.then(|| "2026-06".to_string()),
            terms: map,
            span: span(),
        }
    }

    fn source_stmt(contract: &cfdl_parser::ContractStmt) -> cfdl_resolver::SourceStatement {
        cfdl_resolver::SourceStatement {
            file: "model.cfdl".to_string(),
            statement: Stmt::Contract(contract.clone()),
        }
    }

    fn ctx(pack: &str) -> ActivePackContext {
        let registry = cfdl_pack::PackRegistry::load_from_dir(std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../packs"
        )))
        .expect("packs load");
        ActivePackContext {
            name: pack.to_string(),
            version: "0.1.0".to_string(),
            lowering_rules: registry.lowering_rules(pack),
            validations: registry.validations(pack),
        }
    }

    /// Runs a case through the compiler entry point and the evaluator
    /// directly, asserting they agree. Originally the gate that proved the
    /// migration off the hardcoded branches; retained as coverage of every
    /// check kind, including the pairs that must never double-report.
    fn assert_parity(pack: &str, name: &str, terms: &[(&str, &str)], term_range: bool) {
        let pack_ctx = ctx(pack);
        let c = contract(name, terms, term_range);
        let stmt = source_stmt(&c);

        let via_compiler =
            validate_pack_contract(&pack_ctx, &stmt, &c, "monthly", "2026-01", 12, "2026-12");
        let declarative = pack_validation::evaluate(
            &pack_ctx.validations,
            &c,
            valid_contract_term_range(&c, "2026-01", "2026-12"),
            |code, message, severity, span| {
                let mut diag = pack_diag(code, message, &stmt, span);
                diag.severity = severity.as_str().to_string();
                diag
            },
        );

        let codes = |diags: &[Diagnostic]| {
            let mut v: Vec<String> = diags.iter().map(|d| d.code.clone()).collect();
            v.sort();
            v
        };
        assert_eq!(
            codes(&via_compiler),
            codes(&declarative),
            "divergence for {pack}/{name} with terms {terms:?}"
        );
    }

    #[test]
    fn cre_lease_cases() {
        assert_parity("cre", "cre.lease", &[], true);
        assert_parity("cre", "cre.lease", &[("base_rent", "25000")], true);
        assert_parity("cre", "cre.lease", &[("base_rent", "25000")], false);
        // lease_up_months parses as an integer: a decimal must be rejected.
        for value in ["18", "0", "-1", "18.5", "abc"] {
            assert_parity(
                "cre",
                "cre.lease",
                &[("base_rent", "25000"), ("lease_up_months", value)],
                true,
            );
        }
    }

    #[test]
    fn cre_exit_cap_pair_never_double_reports() {
        // absent / unparseable / zero / negative / valid
        for value in [None, Some("abc"), Some("0"), Some("-0.5"), Some("0.06")] {
            let mut terms = vec![("noi_value", "180000")];
            if let Some(v) = value {
                terms.push(("exit_cap", v));
            }
            assert_parity("cre", "cre.exit_cap", &terms, true);
        }
        // noi alternatives
        for noi in ["noi_ref", "noi_value", "noi"] {
            assert_parity(
                "cre",
                "cre.exit_cap",
                &[("exit_cap", "0.06"), (noi, "1")],
                true,
            );
        }
        assert_parity("cre", "cre.exit_cap", &[("exit_cap", "0.06")], true);
    }

    #[test]
    fn cre_ops_cases() {
        for name in ["cre.ops_revenue", "cre.ops_expense"] {
            assert_parity("cre", name, &[], true);
            assert_parity("cre", name, &[("amount", "30000")], true);
            assert_parity("cre", name, &[("amount", "30000")], false);
        }
    }

    #[test]
    fn opco_line_cases() {
        for name in ["opco.revenue_line", "opco.opex_line"] {
            assert_parity("opco", name, &[], true);
            assert_parity("opco", name, &[("amount", "abc")], true);
            assert_parity("opco", name, &[("amount", "1000")], true);
            assert_parity("opco", name, &[("amount", "1000")], false);
            assert_parity(
                "opco",
                name,
                &[("amount", "1000"), ("growth_rate", "x")],
                true,
            );
            assert_parity(
                "opco",
                name,
                &[("amount", "1000"), ("growth_rate", "0.05")],
                true,
            );
        }
    }

    #[test]
    fn opco_exit_and_debt_cases() {
        for value in [None, Some("abc"), Some("0"), Some("-2"), Some("8.5")] {
            let mut terms = vec![("base_value", "1000")];
            if let Some(v) = value {
                terms.push(("exit_multiple", v));
            }
            assert_parity("opco", "opco.exit_multiple", &terms, true);
        }
        assert_parity(
            "opco",
            "opco.exit_multiple",
            &[("exit_multiple", "8.5")],
            true,
        );

        // unwrap_or defaults: absent exit_multiple must still fire E7024
        for value in [None, Some("0"), Some("abc"), Some("8.5")] {
            let terms: Vec<(&str, &str)> = value
                .map(|v| vec![("exit_multiple", v)])
                .unwrap_or_default();
            assert_parity("opco", "opco.exit_ebitda", &terms, true);
        }

        for amort in [None, Some("0"), Some("abc"), Some("84")] {
            for rate in [None, Some("-0.1"), Some("abc"), Some("0"), Some("0.085")] {
                let mut terms: Vec<(&str, &str)> = vec![];
                if let Some(a) = amort {
                    terms.push(("amort_months", a));
                }
                if let Some(r) = rate {
                    terms.push(("rate", r));
                }
                assert_parity("opco", "opco.term_debt", &terms, true);
            }
        }
    }

    #[test]
    fn opco_working_capital_cases() {
        assert_parity("opco", "opco.working_capital", &[], true);
        assert_parity("opco", "opco.working_capital", &[("amount", "abc")], true);
        assert_parity("opco", "opco.working_capital", &[("amount", "500")], true);
        assert_parity("opco", "opco.working_capital", &[("amount", "500")], false);
    }

    #[test]
    fn unknown_contracts_produce_nothing() {
        assert_parity("cre", "cre.not_a_contract", &[], true);
        assert_parity("opco", "opco.not_a_contract", &[], true);
    }
}
