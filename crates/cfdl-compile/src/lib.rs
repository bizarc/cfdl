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
    /// Model calendars the pack declares it lowers correctly on; empty means
    /// all. See `cfdl_pack::PackManifest::cadences`.
    cadences: Vec<String>,
    /// The closed vocabulary a stream's `category` must name. See
    /// `cfdl_pack::PackManifest::categories`.
    categories: Vec<String>,
    lowering_rules: Vec<cfdl_pack::LoweringRule>,
    validations: Vec<cfdl_pack::PackValidation>,
}

struct PackLoweringOutput {
    streams: Vec<((String, String), IrStream)>,
    /// States declared by lowering rules, deduplicated by name.
    states: Vec<IrState>,
    diagnostics: Vec<Diagnostic>,
}

struct LoweringContext<'a> {
    id_seed: &'a str,
    model_currency: &'a str,
    time_calendar: &'a str,
    time_start: &'a str,
    time_periods: u32,
    timeline_end: &'a str,
    /// Cash horizon plus the projection tail; the bound a schedule may reach.
    timeline_eval_end: &'a str,
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    states: Vec<IrState>,
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

/// A named value per period, defined by a recurrence. `init` and `next` are
/// both required by validation (E1120/E1121) before lowering runs, so they are
/// plain fields rather than options here.
#[derive(Debug, Serialize)]
struct IrState {
    name: String,
    init: IrExpr,
    next: IrExpr,
    /// When the recurrence steps, and over what window. Absent means every
    /// model period over the whole timeline — the behaviour of every state
    /// written before states had a clock of their own, so omitting it keeps
    /// existing IR byte-identical.
    #[serde(skip_serializing_if = "Option::is_none")]
    schedule: Option<IrSchedule>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    day: Option<i32>,
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
    /// A one-shot flow that settles at the END of its period rather than on
    /// the stated date. Disposals want this; acquisitions do not.
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    at_period_end: bool,
    /// Mid-period convention: cash discounted from halfway through the period
    /// that earned it, rather than from its end. Omitted for the default.
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    mid: bool,
    /// How long after a flow is earned its cash moves. Omitted when cash
    /// lands in the period that earned it.
    #[serde(skip_serializing_if = "Option::is_none")]
    net_days: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    net_months: Option<i64>,
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
    /// What this stream is, economically. Omitted when unclassified, so a
    /// model that classifies nothing produces the IR it always did.
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<String>,
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
    // The model's reporting currency: what `model "x" currency INR` declares,
    // or USD when omitted. Every metric is denominated in it.
    let model_currency = resolve_output
        .source_statements
        .iter()
        .find_map(|stmt| match &stmt.statement {
            Stmt::Model(model) => model.currency.clone(),
            _ => None,
        })
        .unwrap_or_else(|| "USD".to_string());
    let (time_calendar, time_start, time_periods, time_projection) = find_time(resolve_output)
        .unwrap_or_else(|| ("monthly".to_string(), "1970-01-01".to_string(), 1, 0));
    let timeline_end = add_periods_for_timeline_end(&time_start, &time_calendar, time_periods);
    // The furthest period a schedule may legally reach: the cash horizon plus
    // any `project <n>` tail, which the engine also evaluates. Used ONLY for
    // the bounds check — `timeline_end` above stays the cash horizon because it
    // shapes the IR itself.
    let timeline_eval_end = add_periods_for_timeline_end(
        &time_start,
        &time_calendar,
        time_periods.saturating_add(time_projection),
    );
    let compiler_version = env!("CARGO_PKG_VERSION").to_string();
    let pack_seed = active_pack
        .map(|pack| format!("{}@{}", pack.name, pack.version))
        .unwrap_or_default();
    let compiler_hash = hash_hex(&format!("cfdl:{compiler_version}:{pack_seed}"));
    // Object ids identify a thing in a model, so they depend on the model and
    // on the pack that lowered it — not on which compiler build ran. Including
    // the compiler version meant every release rewrote every id: goldens
    // churned wholesale, burying real changes, and any downstream store keyed
    // on an id saw the same entity as a new one after an upgrade. The version
    // still appears in provenance, which is where a build belongs.
    let id_seed = if pack_seed.is_empty() {
        "cfdl".to_string()
    } else {
        format!("cfdl:{pack_seed}")
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

    // The vocabulary a hand-written stream's `category` is checked against.
    // It belongs to the active pack, because the categories are a domain
    // judgement and the folds that consume them are declared there. With no
    // pack there is no vocabulary, so any category is unknown — which is the
    // honest answer rather than a special case: nothing would ever read it.
    let pack_categories: &[String] = active_pack
        .map(|pack| pack.categories.as_slice())
        .unwrap_or(&[]);

    let mut streams: Vec<((String, String), IrStream)> = Vec::new();
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Stream(stream) = &source_stmt.statement else {
            continue;
        };
        if let Some(category) = stream.category.as_deref() {
            if !pack_categories.iter().any(|c| c == category) {
                let known = if pack_categories.is_empty() {
                    "the active pack declares none (or no pack is in use)".to_string()
                } else {
                    pack_categories.join(", ")
                };
                return Err(vec![Diagnostic {
                    code: "E5022_UNKNOWN_STREAM_CATEGORY".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "Stream '{}' declares category '{category}', which is not a category \
                         the active pack defines. Known categories: {known}.",
                        stream.name
                    ),
                    file: Some(source_stmt.file.clone()),
                    span: Some(map_span(stream.span)),
                    path: None,
                    hint: Some(
                        "A category is what a fold aggregates on, so it has to name one the \
                         pack declares — otherwise the stream reports as a line and is counted \
                         in no subtotal."
                            .to_string(),
                    ),
                    notes: vec![],
                }]);
            }
        }
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
            category: stream.category.clone(),
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
            timeline_eval_end: &timeline_eval_end,
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
    let lowered_states = lowered.states;
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

    let ir_states = merge_states(
        lower_states(
            resolve_output,
            &time_calendar,
            &time_start,
            &timeline_end,
            &phase_map,
        ),
        lowered_states,
    );

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
        states: ir_states,
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

    let from_dir = match packs_dir.as_ref() {
        Some(dir) => Some(cfdl_pack::PackRegistry::load_from_dir(dir).map_err(|err| {
            pack_diag(
                err.message,
                None,
                vec![format!("pack root: {}", dir.display())],
            )
        })?),
        None => None,
    };

    // Fall back to the packs built into the binary when the directory holds
    // none — the usual case for a model outside a checkout, where the default
    // `<model_root>/packs` simply does not exist. A directory that *does*
    // contain packs is authoritative: falling back from it would silently
    // hand a pack author the stock pack when they mistyped a path to their
    // own, which is worse than failing.
    let used_embedded = from_dir.as_ref().is_none_or(|reg| reg.list().is_empty());
    let registry = match from_dir {
        Some(reg) if !reg.list().is_empty() => reg,
        _ => load_embedded_registry(&pack_diag)?,
    };
    let where_ = match (used_embedded, packs_dir.as_ref()) {
        (true, _) => "in the packs built into this binary".to_string(),
        (false, Some(dir)) => format!("under '{}'", dir.display()),
        (false, None) => "in the embedded pack registry".to_string(),
    };
    let active = match registry.resolve_pack(&use_pack.name, &use_pack.version) {
        cfdl_pack::PackLookup::Found(active) => active,
        cfdl_pack::PackLookup::Absent => {
            return Err(pack_diag(
                format!("Pack '{}' was not found {where_}.", use_pack.name),
                Some("Add a matching pack manifest or pass --packs <dir>.".to_string()),
                vec![],
            ));
        }
        // The pack is present; only the version differs. Saying "not found"
        // here sends the reader to check their --packs path when the fix is
        // one digit in the model.
        cfdl_pack::PackLookup::VersionMismatch { available } => {
            return Err(pack_diag(
                format!(
                    "Model requires pack '{}' version {}, but the pack found {where_} is version {}.",
                    use_pack.name, use_pack.version, available
                ),
                Some(format!(
                    "Change the model to `use pack \"{}\" version \"{}\"`, or point --packs at a registry holding {}.",
                    use_pack.name, available, use_pack.version
                )),
                vec![],
            ));
        }
    };

    Ok(Some(ActivePackContext {
        name: active.name.clone(),
        version: active.version.clone(),
        cadences: registry.cadences(&active.name),
        categories: registry.categories(&active.name),
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
            states: vec![],
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

    // A deferred term's value is unknown until the run, so its bounds cannot
    // be checked — but a distribution's `clip` states the range it can produce.
    // Where both exist, the clip is checkable now.
    let input_clips: BTreeMap<String, (f64, f64)> = resolve_output
        .source_statements
        .iter()
        .filter_map(|stmt| match &stmt.statement {
            Stmt::Assume(assume) => {
                let dist = assume.dist.as_ref()?;
                let (lo, hi) = dist.clip.as_ref()?;
                Some((assume.name.clone(), (lo.parse().ok()?, hi.parse().ok()?)))
            }
            _ => None,
        })
        .collect();

    // Declared bounds per term, from the pack's own validations.
    let mut term_bounds: BTreeMap<String, (Option<f64>, Option<f64>)> = BTreeMap::new();
    for validation in &pack.validations {
        if let Some(term) = validation.term.as_deref() {
            let entry = term_bounds.entry(term.to_string()).or_insert((None, None));
            let lo = validation.min.or(validation.exclusive_min);
            let hi = validation.max.or(validation.exclusive_max);
            if let Some(v) = lo {
                entry.0 = Some(entry.0.map_or(v, |cur: f64| cur.max(v)));
            }
            if let Some(v) = hi {
                entry.1 = Some(entry.1.map_or(v, |cur: f64| cur.min(v)));
            }
        }
    }

    let mut lowered = Vec::new();
    let mut lowered_states: BTreeMap<String, IrState> = BTreeMap::new();
    let mut diagnostics = Vec::new();
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Contract(contract) = &source_stmt.statement else {
            continue;
        };
        for (key, term) in &contract.terms {
            if let Some(name) = term.input_name() {
                if declared_inputs.contains(name) {
                    // The input exists; if it declares a clip and the pack
                    // declares bounds, the clip must sit inside them.
                    if let (Some((clip_lo, clip_hi)), Some((bound_lo, bound_hi))) =
                        (input_clips.get(name), term_bounds.get(key))
                    {
                        let below = bound_lo.is_some_and(|lo| *clip_lo < lo);
                        let above = bound_hi.is_some_and(|hi| *clip_hi > hi);
                        if below || above {
                            diagnostics.push(lowering_rule_diag(
                                "E5011_TERM_CLIP_OUT_OF_BOUNDS",
                                &format!(
                                    "Contract '{}' term '{}' defers to input '{}', whose clip [{}, {}] can produce values outside the range this term allows ({}). Tighten the clip.",
                                    contract.name,
                                    key,
                                    name,
                                    clip_lo,
                                    clip_hi,
                                    match (bound_lo, bound_hi) {
                                        (Some(lo), Some(hi)) => format!("{lo} to {hi}"),
                                        (Some(lo), None) => format!("at least {lo}"),
                                        (None, Some(hi)) => format!("at most {hi}"),
                                        (None, None) => "unbounded".to_string(),
                                    }
                                ),
                                source_stmt,
                                term.span,
                            ));
                        }
                    }
                } else {
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
            // Terms feed both passes below, and `schedule_every` may defer to
            // one, so resolve plain contract terms first with no cadence
            // awareness. A frequency can only ever come from a literal term
            // (`payment_frequency = "month"`), never from an expression that
            // depends on periods-per-year, so this ordering is well-founded.
            let resolve_plain = |key: &str| -> Option<String> {
                contract
                    .terms
                    .get(key)
                    .map(|term| term.value.clone())
                    .or_else(|| rule.defaults.get(key).cloned())
            };
            let schedule_every =
                cfdl_pack::expand_rule_template(&rule.schedule_every, &resolve_plain)
                    .unwrap_or_else(|_| rule.schedule_every.clone());

            // How many of this rule's periods make a year, and how to count
            // them. Keyed off the rule's own payment interval when it declares
            // one — a monthly-paying loan on a daily book divides by 12, not
            // 365 — falling back to the model's calendar.
            let rule_freq = rule_frequency(&schedule_every, ctx.time_calendar).to_string();
            let ppy = periods_per_year(&rule_freq);
            // Errors raised from inside the resolver, which can only return
            // Option and so cannot emit diagnostics itself.
            let period_errors: std::cell::RefCell<Vec<(String, String)>> =
                std::cell::RefCell::new(Vec::new());

            // Converts a months-denominated term into this rule's periods.
            // `_months` always means calendar months, on every calendar: it
            // describes the contract, not the modeller's grid choice.
            let months_to_periods = |key: &str, whole: bool| -> Option<String> {
                let raw = resolve_plain(key)?;
                if raw.trim_start().starts_with("inputs.") {
                    period_errors.borrow_mut().push((
                        "E5017_PERIOD_TERM_NOT_LITERAL".to_string(),
                        format!(
                            "Pack lowering rule '{}' converts term '{}' from months into periods, so it must be a literal; contract '{}' defers it to {}.",
                            rule.id, key, contract.name, raw.trim()
                        ),
                    ));
                    return None;
                }
                let months: f64 = raw.trim().parse().ok()?;
                let periods = months * f64::from(ppy) / 12.0;
                if whole && (periods.fract().abs() > 1e-9) {
                    period_errors.borrow_mut().push((
                        "E5015_TERM_MONTHS_NOT_DIVISIBLE".to_string(),
                        format!(
                            "Pack lowering rule '{}' uses term '{}' as a count of payment periods, but {} months is {} periods at {} frequency. Use a multiple of {} months, declare a finer payment_frequency, or model on a finer calendar.",
                            rule.id,
                            key,
                            months,
                            periods,
                            rule_freq,
                            12.0 / f64::from(ppy)
                        ),
                    ));
                    return None;
                }
                Some(if whole {
                    format!("{}", periods.round() as i64)
                } else {
                    // Trim to a plain decimal: template output is expression
                    // source, and 0.4166666666666667 reads better than an
                    // exponent form.
                    let text = format!("{periods:.10}");
                    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
                    trimmed.to_string()
                })
            };

            // Template expansion: resolve {{contract.<key>}} placeholders from
            // contract terms (term_start/term_end from the term range), then
            // rule defaults. Missing keys are compile errors.
            let resolve = |key: &str| -> Option<String> {
                // Cadence primitives are claimed before contract terms, so a
                // term could shadow one — E5016 rejects that outright.
                if let Some(term) = key.strip_prefix("periods.") {
                    return months_to_periods(term, false);
                }
                if let Some(term) = key.strip_prefix("whole_periods.") {
                    return months_to_periods(term, true);
                }
                let from_contract = match key {
                    "model.periods_per_year" => Some(ppy.to_string()),
                    // What a NOMINAL annual rate is divided by to get this
                    // period's rate. The default is periods-per-year, which is
                    // the 30/360 reading — every period is 1/ppy of a year —
                    // and expands to exactly the same text as
                    // {{model.periods_per_year}}, so adopting this placeholder
                    // changes no existing model.
                    //
                    // Actual conventions divide by a year length scaled to the
                    // period's real days: rate / (360 / days) is rate * days /
                    // 360. On a monthly grid that correctly pays more in a
                    // 31-day month; on daily it collapses to rate / 360.
                    // What the constant PAYMENT is struck on, as distinct from
                    // what interest accrues on. A commercial Actual/360 loan
                    // fixes its payment on a 30/360 schedule and lets principal
                    // absorb the difference; recomputing both legs from one
                    // varying divisor makes the payment swing with month length,
                    // which no loan document does. Defaults to `day_count`, so a
                    // contract that says nothing keeps a single basis.
                    "model.amortization_divisor" => Some(
                        match resolve_plain("amortization_day_count")
                            .or_else(|| resolve_plain("day_count"))
                            .unwrap_or_default()
                            .trim()
                            .trim_matches('"')
                        {
                            "" | "30/360" | "30e/360" => ppy.to_string(),
                            "act/360" => "(360 / time.days_in_period)".to_string(),
                            "act/365" => "(365 / time.days_in_period)".to_string(),
                            _ => ppy.to_string(),
                        },
                    ),
                    "model.accrual_divisor" => Some(
                        match resolve_plain("day_count")
                            .unwrap_or_default()
                            .trim()
                            .trim_matches('"')
                        {
                            "" | "30/360" | "30e/360" => ppy.to_string(),
                            "act/360" => "(360 / time.days_in_period)".to_string(),
                            "act/365" => "(365 / time.days_in_period)".to_string(),
                            // Unreachable in practice: validate_pack_contract
                            // rejects an unknown value once per contract and
                            // short-circuits lowering, which is where the
                            // diagnostic belongs — emitting it here would give
                            // one copy per matching rule.
                            _ => ppy.to_string(),
                        },
                    ),
                    "model.calendar" => Some(rule_freq.clone()),
                    "time.elapsed_periods" => contract
                        .term_start
                        .as_deref()
                        .map(|start| elapsed_periods_expr(&rule_freq, &normalize_date(start))),
                    "time.elapsed_years" => contract
                        .term_start
                        .as_deref()
                        .map(|start| elapsed_years_expr(&normalize_date(start))),
                    "time.periods_to_term_end" => contract
                        .term_end
                        .as_deref()
                        .map(|end| periods_to_expr(&rule_freq, &normalize_date(end))),
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
                    // The same discriminator as `dot_suffix`, but spelled so it
                    // can sit inside an identifier: `state.<name>` resolves a
                    // single segment, so a dotted state name is unreachable.
                    "suffix_ident" => Some(
                        contract
                            .name
                            .strip_prefix(&rule.contract_name)
                            .unwrap_or_default()
                            .replace('.', "_"),
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
                (
                    &rule.schedule_net_days,
                    &mut expanded_rule.schedule_net_days,
                ),
                (
                    &rule.schedule_net_months,
                    &mut expanded_rule.schedule_net_months,
                ),
                // Templated so a contract can declare its own payment rhythm
                // (`payment_frequency = "month"`), letting one rule serve a
                // monthly, quarterly and daily-book version of the same
                // instrument. Already expanded above to derive ppy; expanding
                // it again here is what puts the result on the rule.
                (&rule.schedule_every, &mut expanded_rule.schedule_every),
                (&rule.state_name, &mut expanded_rule.state_name),
                (&rule.state_init, &mut expanded_rule.state_init),
                (&rule.state_next, &mut expanded_rule.state_next),
                (&rule.state_every, &mut expanded_rule.state_every),
                (&rule.state_from, &mut expanded_rule.state_from),
                (&rule.state_to, &mut expanded_rule.state_to),
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
            // A months-to-periods conversion that could not be done — a
            // non-integral payment count, or a term deferred to an input —
            // would otherwise silently drop the placeholder.
            let period_errors = period_errors.into_inner();
            if !period_errors.is_empty() {
                for (code, message) in &period_errors {
                    diagnostics.push(lowering_rule_diag(
                        code,
                        message,
                        source_stmt,
                        contract.span,
                    ));
                }
                continue;
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

            // A rule may narrow the pack's cadence support, so a pack can
            // carry neutral and month-locked rules side by side while it is
            // being migrated instead of being gated wholesale.
            if !rule.cadences.is_empty()
                && !rule
                    .cadences
                    .iter()
                    .any(|cadence| cadence == ctx.time_calendar)
            {
                diagnostics.push(lowering_rule_diag(
                    "E5014_RULE_CADENCE_UNSUPPORTED",
                    &format!(
                        "Pack lowering rule '{}' lowers correctly on {} calendars; this model declares '{}'. It would produce amounts scaled to the wrong period.",
                        rule.id,
                        rule.cadences.join(", "),
                        ctx.time_calendar
                    ),
                    source_stmt,
                    contract.span,
                ));
                continue;
            }

            // A rule may declare its own interval; it must be a real one and no
            // finer than the grid, or several payments would land in one
            // period and collapse into a single figure.
            if !rule.schedule_every.is_empty() {
                match (
                    interval_grain(&rule.schedule_every),
                    cadence_grain(ctx.time_calendar),
                ) {
                    (None, _) => {
                        diagnostics.push(lowering_rule_diag(
                            "E5012_RULE_INVALID_INTERVAL",
                            &format!(
                                "Pack lowering rule '{}' declares schedule_every = '{}', which is not an interval. Use day, week, month, quarter or year.",
                                rule.id, rule.schedule_every
                            ),
                            source_stmt,
                            contract.span,
                        ));
                        continue;
                    }
                    (Some(i), Some(c)) if i < c => {
                        diagnostics.push(lowering_rule_diag(
                            "E2108_SCHEDULE_FINER_THAN_CALENDAR",
                            &format!(
                                "Pack lowering rule '{}' pays every {} but the model's calendar is {}. Occurrences inside one period share that period's environment and cannot be told apart, so an amount that varies over time would be computed once and multiplied.",
                                rule.id, rule.schedule_every, ctx.time_calendar
                            ),
                            source_stmt,
                            contract.span,
                        ));
                        continue;
                    }
                    _ => {}
                }
            }

            // Mirror the bounds check cfdl-validate applies to hand-written
            // streams. It cannot run there: validation completes and returns
            // before build_ir synthesises these. Without this a pack could
            // schedule cash the engine never evaluates, which a model may not.
            //
            // Only `Every` schedules, matching the native path, which requires
            // both `from` and `to` and so skips `on_date`.
            if expanded_rule.schedule_kind == "every" {
                let from = normalize_date(&expanded_rule.schedule_from);
                let to = normalize_date(&expanded_rule.schedule_to);
                if !from.is_empty()
                    && !to.is_empty()
                    && (from.as_str() < ctx.time_start || to.as_str() > ctx.timeline_eval_end)
                {
                    diagnostics.push(lowering_rule_diag(
                        "E2103_SCHEDULE_OUT_OF_BOUNDS",
                        &format!(
                            "Pack lowering rule '{}' produced a schedule {} to {} for contract '{}', outside the model timeline ({} to {}).",
                            rule.id, from, to, contract.name, ctx.time_start, ctx.timeline_eval_end
                        ),
                        source_stmt,
                        contract.span,
                    ));
                    continue;
                }
            }

            // An empty rule currency defers to the model's, which is what keeps
            // a pack usable outside the United States. A rule that pins one is
            // asserting the instrument is denominated in that currency, so the
            // model must agree — cash flows are summed period by period, and
            // the validate-time check only sees hand-written streams.
            let rule_currency = if rule.currency.is_empty() {
                ctx.model_currency.to_string()
            } else {
                rule.currency.clone()
            };
            if !rule_currency.eq_ignore_ascii_case(ctx.model_currency) {
                diagnostics.push(lowering_rule_diag(
                    "E2107_STREAM_CURRENCY_MISMATCH",
                    &format!(
                        "Pack lowering rule '{}' emits stream '{}' in {} but the model reports in {}. Remove the rule's `currency` so it inherits the model's, or declare the model in {}.",
                        rule.id, rule.stream_name, rule_currency, ctx.model_currency, rule_currency
                    ),
                    source_stmt,
                    contract.span,
                ));
                continue;
            }

            let schedule = lower_pack_rule_schedule(
                rule,
                ctx.time_calendar,
                ctx.time_start,
                ctx.timeline_end,
                contract.payment_net,
            );
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

            if !rule.state_name.is_empty() {
                // Same treatment as the amount: a textual splice can produce an
                // expression the parser rejects, and the engine's fallback for
                // a failed state is zero — which would silently flatten every
                // stream that reads it.
                let mut bad = None;
                for (clause, src) in [("init", &rule.state_init), ("next", &rule.state_next)] {
                    if let Err(err) = cfdl_expr::compile_expr(src) {
                        bad = Some((clause, err, src.clone()));
                        break;
                    }
                }
                if let Some((clause, err, src)) = bad {
                    diagnostics.push(lowering_rule_diag(
                        "E5020_LOWERED_STATE_INVALID",
                        &format!(
                            "Pack lowering rule '{}' produced an invalid state '{}' clause for contract '{}' [{}]: {}. Expanded to: {}",
                            rule.id, clause, contract.name, err.code, err.message, src
                        ),
                        source_stmt,
                        contract.span,
                    ));
                    continue;
                }
                // Two rules naming one state with DIFFERENT recurrences would
                // silently keep whichever lowered first. Identical definitions
                // collapse, which is what several contracts sharing one curve
                // should do.
                match lowered_states.get(&rule.state_name) {
                    Some(existing)
                        if existing.init.src != rule.state_init
                            || existing.next.src != rule.state_next =>
                    {
                        diagnostics.push(lowering_rule_diag(
                            "E5021_DUPLICATE_LOWERED_STATE",
                            &format!(
                                "Contract '{}' lowers to state '{}', which another contract already defines differently. Give the rule's state_name a per-contract discriminator ({{{{contract.suffix_ident}}}}).",
                                contract.name, rule.state_name
                            ),
                            source_stmt,
                            contract.span,
                        ));
                        continue;
                    }
                    Some(_) => {}
                    None => {
                        lowered_states.insert(
                            rule.state_name.clone(),
                            IrState {
                                name: rule.state_name.clone(),
                                schedule: lower_rule_state_schedule(
                                    rule,
                                    ctx.time_calendar,
                                    ctx.time_start,
                                    ctx.timeline_end,
                                ),
                                init: IrExpr {
                                    lang: "cfdl".to_string(),
                                    src: rule.state_init.clone(),
                                },
                                next: IrExpr {
                                    lang: "cfdl".to_string(),
                                    src: rule.state_next.clone(),
                                },
                            },
                        );
                    }
                }
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
                    currency: rule_currency.clone(),
                    // Validated against the pack's vocabulary at load time, so
                    // by here it is either empty or known.
                    category: (!rule.category.is_empty()).then(|| rule.category.clone()),
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
        states: lowered_states.into_values().collect(),
        diagnostics,
    }
}

fn validate_pack_contract(
    pack: &ActivePackContext,
    source_stmt: &cfdl_resolver::SourceStatement,
    contract: &cfdl_parser::ContractStmt,
    timeline_calendar: &str,
    timeline_start: &str,
    _timeline_periods: u32,
    timeline_end: &str,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // A pack whose expressions assume one period is one month produces amounts
    // scaled to the wrong period on any other grid — silently, because the
    // schedule adapts correctly and only the amount is wrong. Refuse to lower
    // rather than emit a plausible number that is out by a factor of twelve.
    if !pack.cadences.is_empty()
        && !pack
            .cadences
            .iter()
            .any(|cadence| cadence == timeline_calendar)
    {
        diagnostics.push(pack_diag(
            "E5013_PACK_CADENCE_UNSUPPORTED",
            &format!(
                "Pack '{}' v{} lowers correctly on {} calendars; this model declares '{}'. Its rules would produce amounts scaled to the wrong period.",
                pack.name,
                pack.version,
                pack.cadences.join(", "),
                timeline_calendar
            ),
            source_stmt,
            contract.span,
        ));
    }

    // The template resolver claims these prefixes before it consults contract
    // terms, so a term named `model.periods_per_year` would be silently
    // unreachable. Contract term keys may legitimately be dotted, so this is
    // reachable by accident rather than only by perversity.
    for (key, term) in &contract.terms {
        if let Some(prefix) = RESERVED_TERM_PREFIXES
            .iter()
            .find(|prefix| key.starts_with(**prefix))
        {
            diagnostics.push(pack_diag(
                "E5016_RESERVED_TERM_PREFIX",
                &format!(
                    "Contract '{}' declares term '{}', but '{}' is reserved for cadence placeholders that lowering rules resolve before contract terms. The term would never be read. Rename it.",
                    contract.name, key, prefix
                ),
                source_stmt,
                term.span,
            ));
        }
    }

    // A misspelled day count must not fall back to a default in silence: the
    // gap between act/360 and act/365 is about 1.4% of interest.
    for key in ["day_count", "amortization_day_count"] {
        let Some(term) = contract.terms.get(key) else {
            continue;
        };
        let value = term.value.trim().trim_matches('"');
        if !matches!(value, "30/360" | "30e/360" | "act/360" | "act/365") {
            diagnostics.push(pack_diag(
                "E5019_UNKNOWN_DAY_COUNT",
                &format!(
                    "Contract '{}' declares {} = '{}'. Supported: 30/360, 30e/360, act/360, act/365.",
                    contract.name, key, value
                ),
                source_stmt,
                term.span,
            ));
        }
    }

    // Elapsed-period counting measures whole calendar steps from the
    // contract's start, so a term that does not begin on a period boundary
    // lands mid-period and counts short. On a monthly grid every `YYYY-MM`
    // term is on-grid and this never fires; it exists so quarterly and annual
    // models cannot silently inherit an off-by-one.
    if let Some(term_start) = contract.term_start.as_deref() {
        if !term_start_on_grid(timeline_start, term_start, timeline_calendar) {
            diagnostics.push(pack_diag(
                "E5018_TERM_START_OFF_GRID",
                &format!(
                    "Contract '{}' starts {} but the model's {} periods begin {} and step from there. A term must start on a period boundary, or elapsed-period counting is off by a partial period.",
                    contract.name,
                    normalize_date(term_start),
                    timeline_calendar,
                    normalize_date(timeline_start)
                ),
                source_stmt,
                contract.span,
            ));
        }
    }

    // Domain constraints are declared by the pack in validations.toml; the
    // compiler supplies only what a pack cannot see — the source span and
    // whether the contract's term sits inside the model timeline.
    diagnostics.extend(pack_validation::evaluate(
        &pack.validations,
        contract,
        valid_contract_term_range(contract, timeline_start, timeline_end),
        |code, message, severity, span| {
            let mut diag = pack_diag(code, message, source_stmt, span);
            diag.severity = severity.as_str().to_string();
            diag
        },
    ));

    diagnostics
}

/// Whether `term_start` falls on one of the model's period boundaries.
///
/// Periods step from `timeline_start` by whole calendar units, so this is a
/// divisibility test on the offset — months for monthly/quarterly/annual, days
/// for daily. An unparseable date is treated as on-grid; the date itself is
/// reported by the parser and by E2104/E2103, and two diagnostics for one typo
/// is noise.
fn term_start_on_grid(timeline_start: &str, term_start: &str, calendar: &str) -> bool {
    let step_months = match calendar {
        "monthly" => 1,
        "quarterly" => 3,
        "annual" => 12,
        // Daily periods step one day at a time, so every date is on-grid.
        _ => return true,
    };
    let (Some((sy, sm, _)), Some((ty, tm, td))) = (
        parse_ymd(&normalize_date(timeline_start)),
        parse_ymd(&normalize_date(term_start)),
    ) else {
        return true;
    };
    // A term is anchored to the first of its month; a mid-month start is not a
    // period boundary on any monthly-or-coarser grid.
    if td != 1 {
        return false;
    }
    let offset = (ty - sy) * 12 + (tm as i32 - sm as i32);
    offset.rem_euclid(step_months) == 0
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

/// Relative coarseness of a schedule interval and a calendar cadence, so the
/// two can be compared. Mirrors `cfdl_validate`'s check for hand-written
/// streams; a pack must not be able to express what a model may not.
fn interval_grain(interval: &str) -> Option<u8> {
    match interval {
        "day" => Some(0),
        "week" => Some(1),
        "month" => Some(2),
        "quarter" => Some(3),
        "year" => Some(4),
        _ => None,
    }
}

fn cadence_grain(cadence: &str) -> Option<u8> {
    match cadence {
        "daily" => Some(0),
        "weekly" => Some(1),
        "monthly" => Some(2),
        "quarterly" => Some(3),
        "annual" => Some(4),
        _ => None,
    }
}

/// Periods per year for a frequency, as a **pack lowering convention**.
///
/// Deliberately a second copy of the engine's table rather than a shared
/// helper. They encode different things — this one converts an annual figure
/// into a per-period one at compile time; the engine's drives discounting and
/// weighted-average life at run time — and a future change to one must not
/// silently move the other. `compile_and_engine_ppy_tables_agree` asserts they
/// match over the four real calendars.
///
/// `weekly` is here because it is a valid rule *interval* (`schedule_every =
/// "week"`) even though it is not a valid model calendar.
fn periods_per_year(frequency: &str) -> u32 {
    match frequency {
        "daily" => 365,
        "weekly" => 52,
        "monthly" => 12,
        "quarterly" => 4,
        "annual" => 1,
        _ => 1,
    }
}

/// The frequency a rule's amounts are denominated in.
///
/// A rule that declares its own interval accrues on that rhythm, not the
/// model's: a monthly-paying loan carried on a daily book still makes twelve
/// payments a year, so its annual figures divide by 12 and not 365. Only the
/// compiler can see this — at run time the expression environment knows the
/// calendar and nothing about the schedule — which is why periods-per-year is
/// resolved here rather than exposed as a runtime binding for packs.
fn rule_frequency<'a>(schedule_every: &'a str, calendar: &'a str) -> &'a str {
    if schedule_every.is_empty() {
        calendar
    } else {
        interval_to_frequency(schedule_every)
    }
}

/// An expression counting whole elapsed periods of `frequency` from `anchor`
/// to `time.date`, for substitution into a lowering rule.
///
/// Date-based rather than `time.t - <start index>` on purpose: the compiler
/// would otherwise have to reimplement the engine's timeline construction,
/// creating a second model of time in a second crate that can drift. This form
/// needs no knowledge of where the model starts and composes with projection
/// tails for free. Its one weakness — a term that does not begin on a period
/// boundary — is closed by `E5018_TERM_START_OFF_GRID`.
fn elapsed_periods_expr(frequency: &str, anchor: &str) -> String {
    match frequency {
        "daily" => format!("days_between(parse_date(\"{anchor}\"), time.date)"),
        "weekly" => format!("round_down(days_between(parse_date(\"{anchor}\"), time.date) / 7, 0)"),
        "monthly" => format!("months_between(parse_date(\"{anchor}\"), time.date)"),
        "quarterly" => {
            format!("round_down(months_between(parse_date(\"{anchor}\"), time.date) / 3, 0)")
        }
        // Annual, and the fallback: whole years is the safest reading of an
        // unknown frequency, and `annual` is the only one that reaches here.
        _ => format!("round_down(months_between(parse_date(\"{anchor}\"), time.date) / 12, 0)"),
    }
}

/// Whole elapsed years since `anchor`, for anniversary stepping.
///
/// Cadence-independent by construction: `months_between` ignores the day and
/// the timeline steps in whole months on every calendar coarser than daily, so
/// this yields correct completed years on all of them. It expands to exactly
/// the text the packs already contain, which is what lets the rename land with
/// an empty gold diff.
fn elapsed_years_expr(anchor: &str) -> String {
    format!("round_down(months_between(parse_date(\"{anchor}\"), time.date) / 12, 0)")
}

/// Whole periods remaining from `time.date` to `anchor` — the mirror of
/// `elapsed_periods_expr`, for "is this the last period" tests.
fn periods_to_expr(frequency: &str, anchor: &str) -> String {
    match frequency {
        "daily" => format!("days_between(time.date, parse_date(\"{anchor}\"))"),
        "weekly" => format!("round_down(days_between(time.date, parse_date(\"{anchor}\")) / 7, 0)"),
        "monthly" => format!("months_between(time.date, parse_date(\"{anchor}\"))"),
        "quarterly" => {
            format!("round_down(months_between(time.date, parse_date(\"{anchor}\")) / 3, 0)")
        }
        _ => format!("round_down(months_between(time.date, parse_date(\"{anchor}\")) / 12, 0)"),
    }
}

/// Template prefixes the resolver claims before contract terms are consulted.
///
/// A contract term with one of these names would be shadowed and silently
/// unreachable, so declaring one is `E5016_RESERVED_TERM_PREFIX`.
const RESERVED_TERM_PREFIXES: [&str; 4] = ["model.", "time.", "periods.", "whole_periods."];

/// The clock a lowering rule gives its state, or `None` for every model period.
///
/// A state is not paid, so this carries cadence and window and nothing else —
/// no `due`, no `mid`, no settlement lag. Those place CASH within a period,
/// and a recurrence has no cash to place.
fn lower_rule_state_schedule(
    rule: &cfdl_pack::LoweringRule,
    time_calendar: &str,
    time_start: &str,
    timeline_end: &str,
) -> Option<IrSchedule> {
    if rule.state_name.is_empty() {
        return None;
    }
    // Absent means every model period — the behaviour of every state written
    // before states had a clock, so an unset field changes nothing.
    if rule.state_every.is_empty() && rule.state_from.is_empty() && rule.state_to.is_empty() {
        return None;
    }
    Some(IrSchedule {
        kind: "Every".to_string(),
        due: false,
        at_period_end: false,
        mid: false,
        net_days: None,
        net_months: None,
        on: None,
        every: Some(if rule.state_every.is_empty() {
            time_calendar.to_string()
        } else {
            interval_to_frequency(&rule.state_every).to_string()
        }),
        from: Some(normalize_date(if rule.state_from.is_empty() {
            time_start
        } else {
            &rule.state_from
        })),
        to: Some(normalize_date(if rule.state_to.is_empty() {
            timeline_end
        } else {
            &rule.state_to
        })),
        on_rule: None,
        phase: None,
        convention: None,
        calendar: None,
        except_dates: Vec::new(),
        also_dates: Vec::new(),
    })
}

fn lower_pack_rule_schedule(
    rule: &cfdl_pack::LoweringRule,
    time_calendar: &str,
    time_start: &str,
    timeline_end: &str,
    contract_net: Option<cfdl_parser::PaymentTerms>,
) -> IrSchedule {
    // A rule may state its own terms; otherwise it inherits the contract's,
    // which is the ordinary case — a contract states its payment terms once
    // and everything billed under it settles that way.
    // A rule may state its own terms, templated so it can defer to a contract
    // term; otherwise it inherits the contract's.
    let rule_days = rule.schedule_net_days.trim().parse::<i64>().ok();
    let rule_months = rule.schedule_net_months.trim().parse::<i64>().ok();
    let (net_days, net_months) = match (rule_days, rule_months) {
        (None, None) => split_payment_terms(contract_net),
        pair => pair,
    };
    if rule.schedule_kind.eq_ignore_ascii_case("on_date") {
        IrSchedule {
            kind: "OnDate".to_string(),
            due: false,
            at_period_end: rule.schedule_at_period_end,
            mid: false,
            net_days: None,
            net_months: None,
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
            at_period_end: false,
            mid: rule.schedule_mid,
            net_days,
            net_months,
            on: None,
            // A rule may pay on its own rhythm — a quarterly coupon on a
            // monthly model. Unset means the calendar cadence, which is what
            // most rules want and what every shipped rule uses.
            every: Some(if rule.schedule_every.is_empty() {
                time_calendar.to_string()
            } else {
                interval_to_frequency(&rule.schedule_every).to_string()
            }),
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
/// Lower `state` statements into IR states, in declaration order.
///
/// Missing clauses and duplicate names are already E1120/E1121/E1122 from
/// validation, so a statement that reaches here without both clauses is
/// skipped rather than re-reported — compilation has already failed and a
/// second diagnostic for one mistake is noise.
/// Model-declared states first, then any a pack rule added.
///
/// A model-declared name wins: a modeller who writes `state x` has said what
/// they mean, and silently substituting the pack's version would be the kind
/// of invisible override a pack should never perform.
fn merge_states(mut declared: Vec<IrState>, lowered: Vec<IrState>) -> Vec<IrState> {
    for state in lowered {
        if !declared.iter().any(|s| s.name == state.name) {
            declared.push(state);
        }
    }
    declared
}

fn lower_states(
    resolve_output: &cfdl_resolver::ResolveOutput,
    time_calendar: &str,
    time_start: &str,
    timeline_end: &str,
    phase_map: &BTreeMap<String, (String, String)>,
) -> Vec<IrState> {
    let mut states: Vec<IrState> = Vec::new();
    for source_stmt in &resolve_output.source_statements {
        let Stmt::State(state) = &source_stmt.statement else {
            continue;
        };
        let (Some(init), Some(next)) = (&state.init, &state.next) else {
            continue;
        };
        if states.iter().any(|s| s.name == state.name) {
            continue;
        }
        // The same lowering a stream's schedule goes through, so a state's
        // cadence cannot drift from a stream's. `None` stays `None`: an absent
        // clause means every period, which `lower_schedule` would instead turn
        // into a one-shot at the model start.
        let schedule = state.schedule.as_ref().and_then(|spec| {
            // Phase resolution is the only failure mode, and it is already
            // reported against the phase statement itself.
            lower_schedule(
                Some(spec),
                time_calendar,
                time_start,
                timeline_end,
                phase_map,
            )
            .ok()
        });
        states.push(IrState {
            name: state.name.clone(),
            schedule,
            init: IrExpr {
                lang: init.lang.clone(),
                src: init.src.clone(),
            },
            next: IrExpr {
                lang: next.lang.clone(),
                src: next.src.clone(),
            },
        });
    }
    states
}

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

/// Split payment terms into the two IR fields.
fn split_payment_terms(terms: Option<cfdl_parser::PaymentTerms>) -> (Option<i64>, Option<i64>) {
    match terms {
        Some(cfdl_parser::PaymentTerms::Days(n)) => (Some(n), None),
        Some(cfdl_parser::PaymentTerms::Months(n)) => (None, Some(n)),
        None => (None, None),
    }
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
            at_period_end: false,
            mid: false,
            net_days: None,
            net_months: None,
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
        // End of month has no day-of-month; the field is omitted rather than
        // written as 0, which the IR schema's 1..31 bound rejects.
        Some(IrOnRule {
            kind: "EndOfMonth".to_string(),
            day: None,
        })
    } else {
        schedule.day_of_month.map(|day| IrOnRule {
            kind: "DayOfMonth".to_string(),
            day: Some(day),
        })
    };
    match &schedule.kind {
        ScheduleKind::OnDate if schedule.net.is_some() => Err(
            "Payment terms do not apply to `schedule on <date>`: a one-shot flow has no accrual period to settle after. State the date the cash moves."
                .to_string(),
        ),
        ScheduleKind::OnDate => Ok(IrSchedule {
            kind: "OnDate".to_string(),
            due: false,
            at_period_end: false,
            mid: schedule.mid,
            net_days: None,
            net_months: None,
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
            at_period_end: false,
            mid: schedule.mid,
            net_days: split_payment_terms(schedule.net).0,
            net_months: split_payment_terms(schedule.net).1,
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
                at_period_end: false,
                mid: false,
                net_days: None,
                net_months: None,
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
                at_period_end: false,
                mid: schedule.mid,
                net_days: split_payment_terms(schedule.net).0,
            net_months: split_payment_terms(schedule.net).1,
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
        // This branch used to ignore `offset` entirely and return the start
        // date, so a daily model's timeline "ended" on day one. It was
        // unreachable in practice — nothing bounds-checked lowered streams and
        // no daily model used a pack — until both changed.
        "daily" => add_days(year, month, day, offset as i32),
        "monthly" => add_months(year, month, day, offset as i32),
        "quarterly" => add_months(year, month, day, (offset as i32) * 3),
        "annual" => add_months(year, month, day, (offset as i32) * 12),
        _ => format!("{year:04}-{month:02}-{day:02}"),
    }
}

/// Civil date plus `days`, via a day count from an epoch. Mirrors the engine's
/// own date arithmetic (cfdl-calc's CalcDate), which is the definition the
/// timeline is actually built from.
fn add_days(year: i32, month: u32, day: u32, days: i32) -> String {
    let mut y = year;
    let mut m = month as i32;
    if m <= 2 {
        y -= 1;
        m += 12;
    }
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (m - 3) + 2) / 5 + day as i32 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let epoch = era as i64 * 146_097 + doe as i64 - 719_468 + days as i64;

    let z = epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
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
            payment_net: None,
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
            cadences: registry.cadences(pack),
            categories: registry.categories(pack),
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

    #[test]
    fn term_start_on_grid_accepts_every_month_on_a_monthly_grid() {
        // The gate must be inert on monthly, where every `YYYY-MM` term is a
        // period boundary — otherwise it would reject the entire existing
        // corpus. This is the assertion that makes S0 a no-op for today.
        for month in 1..=12 {
            let term = format!("2026-{month:02}");
            assert!(
                term_start_on_grid("2026-01", &term, "monthly"),
                "monthly grid rejected {term}"
            );
        }
    }

    #[test]
    fn term_start_on_grid_tracks_the_period_stride() {
        // Quarterly periods begin 2026-01, 2026-04, 2026-07, 2026-10.
        assert!(term_start_on_grid("2026-01", "2026-01", "quarterly"));
        assert!(term_start_on_grid("2026-01", "2026-04", "quarterly"));
        assert!(term_start_on_grid("2026-01", "2027-01", "quarterly"));
        assert!(!term_start_on_grid("2026-01", "2026-02", "quarterly"));
        assert!(!term_start_on_grid("2026-01", "2026-03", "quarterly"));

        // Annual periods begin every January from the model start.
        assert!(term_start_on_grid("2026-01", "2028-01", "annual"));
        assert!(!term_start_on_grid("2026-01", "2026-07", "annual"));

        // A term before the model start is still measured on the same stride,
        // so the check must not assume a non-negative offset.
        assert!(term_start_on_grid("2026-01", "2025-10", "quarterly"));
        assert!(!term_start_on_grid("2026-01", "2025-11", "quarterly"));

        // Daily periods step one day, so every date is a boundary.
        assert!(term_start_on_grid("2026-01-01", "2026-03-17", "daily"));

        // A mid-month start is not a boundary on any monthly-or-coarser grid.
        assert!(!term_start_on_grid("2026-01", "2026-02-15", "monthly"));
    }

    #[test]
    fn shipped_packs_declare_their_cadence_support() {
        // The cadence ratchet, asserted rather than tracked by hand. A pack
        // that still divides by a literal 12 must gate itself to monthly; a
        // converted pack must be unconstrained. Moving a pack between these
        // lists is the deliberate act of declaring it neutral, and it fails
        // here until the conversion actually lands.
        //
        // Terminal state, now reached: no first-party pack is gated. The
        // `cadences` field stays in the schema — a third-party pack may still
        // need it, and it is the honest way to say "these rules assume a
        // period length" — but no shipped pack declares it any more.
        const STILL_MONTHLY: [&str; 0] = [];
        const CADENCE_NEUTRAL: [&str; 5] = ["cre", "credit", "energy", "opco", "testpack"];

        for pack in STILL_MONTHLY {
            assert_eq!(
                ctx(pack).cadences,
                vec!["monthly".to_string()],
                "{pack} still has month-locked rules and must declare it"
            );
        }
        for pack in CADENCE_NEUTRAL {
            assert!(
                ctx(pack).cadences.is_empty(),
                "{pack} is cadence-neutral and must not be gated"
            );
        }
    }

    #[test]
    fn compile_and_engine_ppy_tables_agree() {
        // The lowering convention and the engine's discounting constant are
        // deliberately separate tables (see `periods_per_year`'s comment), so
        // pin the values that must match. The engine side is verified
        // independently: tools/analytic-checks.py asserts closed-form annuity
        // identities on quarterly and annual grids, which only hold if its own
        // periods-per-year is right, and `run.periods_per_year` in
        // gold/results/schedule_quarterly_grid.results.json reads 4.
        for (calendar, expected) in [
            ("daily", 365),
            ("monthly", 12),
            ("quarterly", 4),
            ("annual", 1),
        ] {
            assert_eq!(
                periods_per_year(calendar),
                expected,
                "{calendar} periods per year"
            );
        }
    }

    #[test]
    fn a_rules_own_interval_wins_over_the_calendar() {
        // The case that forces periods-per-year to be resolved at compile
        // time: a monthly-paying instrument carried on a daily book still
        // makes twelve payments a year, and the runtime cannot see that
        // because it only knows the calendar.
        assert_eq!(rule_frequency("month", "daily"), "monthly");
        assert_eq!(periods_per_year(rule_frequency("month", "daily")), 12);
        assert_eq!(periods_per_year(rule_frequency("quarter", "monthly")), 4);
        // No declared interval: the rule accrues on the model's grid.
        assert_eq!(rule_frequency("", "quarterly"), "quarterly");
        assert_eq!(periods_per_year(rule_frequency("", "quarterly")), 4);
    }

    #[test]
    fn elapsed_years_expands_to_the_idiom_the_packs_already_use() {
        // The rename in S2 is only safe — and only provably gold-neutral — if
        // this is byte-identical to what the rules contain today.
        assert_eq!(
            elapsed_years_expr("2026-01-01"),
            "round_down(months_between(parse_date(\"2026-01-01\"), time.date) / 12, 0)"
        );
        // And on a monthly grid, elapsed periods is the bare month count the
        // rules already use as their anchor.
        assert_eq!(
            elapsed_periods_expr("monthly", "2026-01-01"),
            "months_between(parse_date(\"2026-01-01\"), time.date)"
        );
    }

    #[test]
    fn elapsed_periods_counts_the_rules_own_rhythm() {
        assert!(elapsed_periods_expr("quarterly", "2026-01-01").contains("/ 3"));
        assert!(elapsed_periods_expr("annual", "2026-01-01").contains("/ 12"));
        assert!(elapsed_periods_expr("daily", "2026-01-01").starts_with("days_between("));
        // periods_to_* is the same count with the arguments reversed, so a
        // rule can ask "is this the last period" on any grid.
        assert!(periods_to_expr("monthly", "2030-12-01").starts_with("months_between(time.date,"));
    }

    #[test]
    fn daily_timeline_end_advances_by_days() {
        // Regression: this branch ignored the period count and returned the
        // start date, so a daily model's timeline "ended" on day one. Only
        // reachable once lowered streams were bounds-checked.
        assert_eq!(
            add_periods_for_timeline_end("2025-01-01", "daily", 1),
            "2025-01-01"
        );
        assert_eq!(
            add_periods_for_timeline_end("2025-01-01", "daily", 31),
            "2025-01-31"
        );
        assert_eq!(
            add_periods_for_timeline_end("2025-01-01", "daily", 1095),
            "2027-12-31"
        );
        // Across a leap day: 2028-02-29 exists, so +60 from Jan 1 is Feb 29.
        assert_eq!(
            add_periods_for_timeline_end("2028-01-01", "daily", 60),
            "2028-02-29"
        );
        // And the coarser calendars are unaffected.
        assert_eq!(
            add_periods_for_timeline_end("2026-01-01", "monthly", 12),
            "2026-12-01"
        );
        assert_eq!(
            add_periods_for_timeline_end("2026-01-01", "quarterly", 4),
            "2026-10-01"
        );
        assert_eq!(
            add_periods_for_timeline_end("2026-01-01", "annual", 3),
            "2028-01-01"
        );
    }

    #[test]
    fn a_gated_pack_refuses_a_calendar_it_does_not_support() {
        // E5013's fixture used the cre pack on an annual calendar. That now
        // compiles — cre is neutral — so the check moves here rather than
        // being lost. `cadences` remains a supported manifest field: a
        // third-party pack whose rules assume a period length still needs an
        // honest way to say so.
        let mut pack = ctx("testpack");
        pack.cadences = vec!["monthly".to_string()];
        let contract = contract("test.fee_contract", &[("rate", "100")], true);
        let stmt = source_stmt(&contract);

        let on_quarterly = validate_pack_contract(
            &pack,
            &stmt,
            &contract,
            "quarterly",
            "2026-01-01",
            8,
            "2027-10-01",
        );
        assert!(
            on_quarterly
                .iter()
                .any(|d| d.code == "E5013_PACK_CADENCE_UNSUPPORTED"),
            "expected E5013 on an unsupported calendar, got {:?}",
            on_quarterly.iter().map(|d| &d.code).collect::<Vec<_>>()
        );

        let on_monthly = validate_pack_contract(
            &pack,
            &stmt,
            &contract,
            "monthly",
            "2026-01-01",
            12,
            "2026-12-01",
        );
        assert!(
            !on_monthly
                .iter()
                .any(|d| d.code == "E5013_PACK_CADENCE_UNSUPPORTED"),
            "a supported calendar must not fire E5013"
        );

        // And an unconstrained pack is unaffected on every calendar.
        let open = ctx("testpack");
        assert!(open.cadences.is_empty());
        for calendar in ["daily", "monthly", "quarterly", "annual"] {
            let diags = validate_pack_contract(
                &open,
                &stmt,
                &contract,
                calendar,
                "2026-01-01",
                12,
                "2026-12-01",
            );
            assert!(
                !diags
                    .iter()
                    .any(|d| d.code == "E5013_PACK_CADENCE_UNSUPPORTED"),
                "an unconstrained pack must not be gated on {calendar}"
            );
        }
    }
}
