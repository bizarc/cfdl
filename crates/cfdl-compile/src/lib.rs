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
    /// Per-period subtotal declarations, in declaration order.
    subtotal_specs: Vec<cfdl_pack::SubtotalSpec>,
    lowering_rules: Vec<cfdl_pack::LoweringRule>,
    validations: Vec<cfdl_pack::PackValidation>,
    /// What a model using this pack may be ABOUT, merged over the language's
    /// own base vocabulary. A pack adds types; it cannot remove the ones every
    /// model has.
    ontology: cfdl_pack::PackOntology,
}

struct PackLoweringOutput {
    streams: Vec<((String, String), IrStream)>,
    /// Fields a lowering rule hangs on the entity it describes, keyed by
    /// (owner symbol, field name). A pack no longer emits model-level state.
    fields: BTreeMap<(String, String), IrFieldRule>,
    /// The field ROLES a rule's field fills, keyed (owner symbol, role) →
    /// field names (docs/40 §3, stage 6). A machine's arrival action names
    /// the role; the entity carries which fields it means.
    field_roles: BTreeMap<(String, String), Vec<String>>,
    /// What each lowered stream consumed. Parallel to `streams`.
    stream_inputs: Vec<IrStreamInputs>,
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
    /// Values indexed by cumulative share. Omitted when a model declares none,
    /// so existing IR stays byte-identical.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    quantiles: Vec<IrQuantile>,
    /// Every quantile call site, resolved. The audit record for a nonlinear
    /// input: which slice each expression asked for, and what it came to.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    quantile_inputs: Vec<IrQuantileCall>,
    /// Declared cash locations whose balances carry across periods. Omitted
    /// when a model declares none, so existing IR stays byte-identical.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    accounts: Vec<IrAccount>,
    /// Every machine an entity binds — pack-declared and model-declared
    /// resolved to the same shape (`docs/28` §6.1). Omitted when no entity
    /// has one, so existing IR stays byte-identical.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    lifecycles: Vec<IrLifecycle>,
    /// Ordered allocations of a pot. Omitted when a model declares none, so
    /// existing IR stays byte-identical.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    waterfalls: Vec<IrWaterfall>,
    /// Per-period subtotals declared by the active pack. Omitted when the pack
    /// declares none, so existing IR is byte-identical.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    subtotals: Vec<IrSubtotal>,
    /// Figures this model solved for, in declaration order (`docs/13` §7.25).
    /// A metric may read the metrics above it, so the order is the meaning.
    /// Omitted when a model declares none, so existing IR is byte-identical.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    metrics: Vec<IrMetric>,
    /// VIEWS — lenses on a completed result, never part of the model.
    ///
    /// A slice filters and a statement organizes; neither produces cash, and
    /// two users who look at identical results differently are running the
    /// same model. So they live under their own key and `model_hash` is taken
    /// over the document WITHOUT it — anything added here is outside the
    /// model's identity by construction, rather than by a rule someone has to
    /// remember to extend.
    ///
    /// A metric is NOT here. It is a figure the model claims, asserted by
    /// every benchmark's `expected_metrics.json`, so it stays in the model.
    #[serde(skip_serializing_if = "IrViews::is_empty")]
    views: IrViews,
    contracts: Vec<IrContract>,
    streams: Vec<IrStream>,
    /// What each pack-lowered stream consumed, keyed by stream name. Omitted
    /// when nothing was lowered from a pack.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    stream_inputs: Vec<IrStreamInputs>,
    events: Vec<serde_json::Value>,
    options: Vec<serde_json::Value>,
    runs: Vec<IrRun>,

    required_observables: Vec<String>,
    required_refs: Vec<String>,
    provenance: IrProvenance,
}

/// A declared metric: a name and the expression that produces it.
#[derive(Debug, Serialize)]
struct IrMetric {
    name: String,
    expr: IrExpr,
    provenance: IrNodeProvenance,
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
struct IrAccount {
    name: String,
    /// The party this account belongs to, when it belongs to one. A general
    /// account has none.
    #[serde(skip_serializing_if = "Option::is_none")]
    owner: Option<String>,
    /// The entity that owns a claim declared in its block (`docs/42` §3.6):
    /// `asset.loan` for `asset.loan.balance`. Absent on a structure or
    /// party account.
    #[serde(skip_serializing_if = "Option::is_none")]
    owner_entity: Option<String>,
    /// `owed` or `due`, from the owner's view: which way a cash stream that
    /// moves this account changes it. Absent when nothing moves it.
    #[serde(skip_serializing_if = "Option::is_none")]
    side: Option<String>,
    /// The balance at the timeline's first period. Absent means zero: a
    /// balance created during the run is raised by the cash that creates it.
    #[serde(skip_serializing_if = "Option::is_none")]
    init: Option<IrExpr>,
    /// What flows in each period. May be negative: an account fed a deal's
    /// whole net cash IS the deal's cumulative position.
    #[serde(skip_serializing_if = "Option::is_none")]
    inflow: Option<IrExpr>,
    /// A RELATION FOLD (`docs/42` §3.4): this container's account of this
    /// name is the sum of its members' accounts of the same name, opening
    /// and closing, through `part of`. Synthesized by the compiler for every
    /// ancestor of an entity that declares a claim; carries no side, init,
    /// inflow or movement of its own.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    fold: bool,
}

#[derive(Debug, Serialize)]
struct IrWaterfall {
    name: String,
    entity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    schedule: Option<IrSchedule>,
    /// The pot this allocates.
    source: IrExpr,
    steps: Vec<IrWaterfallStep>,
}

#[derive(Debug, Serialize)]
struct IrWaterfallStep {
    name: String,
    payee: String,
    /// The payee is an ACCOUNT rather than a party. Omitted when false, so
    /// existing IR stays byte-identical.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    payee_is_account: bool,
    /// The agreement this step pays, and which of its lines (docs/40 §6):
    /// `for contract credit.note.a2 line principal`. Present only when the
    /// model says so; the results attribute the step's series to both.
    #[serde(skip_serializing_if = "Option::is_none")]
    contract: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    line: Option<String>,
    /// What the step is owed. The engine pays `min(max(0, this), remaining)`.
    amount: IrExpr,
}

#[derive(Debug, Serialize)]
struct IrCurvePoint {
    date: String,
    value: f64,
}

/// A named series of values indexed by cumulative share.
///
/// ONE CANONICAL FORM. Points are stored ascending by share whatever the
/// source said: `by exceedance` is reversed here, so the IR carries no
/// orientation and no consumer has to know which way the author wrote it.
#[derive(Debug, Serialize)]
struct IrQuantile {
    name: String,
    /// "step" or "linear" — the same two words a curve uses, and the integral
    /// `quantile_mean` takes is the exact integral of whichever they describe.
    interpolation: String,
    /// The pack reference id this realises, when the declaration named one.
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<String>,
    /// Points sorted ascending by share.
    points: Vec<IrQuantilePoint>,
}

#[derive(Debug, Serialize)]
struct IrQuantilePoint {
    share: f64,
    value: f64,
}

/// One resolved quantile call site.
///
/// Recorded at compile time, the way `stream_inputs` is, so the engine and the
/// per-period evaluation path are untouched by it.
#[derive(Debug, Serialize, Clone)]
struct IrQuantileCall {
    quantile: String,
    function: String,
    args: Vec<f64>,
    /// Absent when an argument is not a literal. The call is still listed: a
    /// silently omitted call site would read as a model that never made one.
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<f64>,
}

#[derive(Debug, Serialize)]
struct IrNodeProvenance {
    source_file: String,
    source_span: Span,
    #[serde(skip_serializing_if = "Option::is_none")]
    generated_by: Option<IrGeneratedBy>,
}

#[derive(Debug, Serialize)]
struct IrSlice {
    name: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    entities: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    types: Vec<String>,
    /// Lines by role, as declared — lineage; expanded into `type_streams`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    lines: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    categories: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    streams: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    except_streams: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    except_categories: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    except_entities: Vec<String>,
    /// The stream names the `type` clauses matched, resolved HERE because
    /// only the compiler holds the ontology: a stream is selected when the
    /// contract type its lowering rule binds to is_a a named type. The
    /// clause stays in `types` as lineage; the engine reads this expansion.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    type_streams: Vec<String>,
    /// A reporting window, inclusive, as declared. Omitted when the slice
    /// spans the whole horizon, so existing IR stays byte-identical.
    #[serde(skip_serializing_if = "Option::is_none")]
    window: Option<IrDateRange>,
    provenance: IrNodeProvenance,
}

/// The lenses a model declares over its own results.
#[derive(Debug, Serialize, Default)]
struct IrViews {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    slices: Vec<IrSlice>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    statements: Vec<IrStatement>,
}

impl IrViews {
    fn is_empty(&self) -> bool {
        self.slices.is_empty() && self.statements.is_empty()
    }
}

/// A declared presentation: which hierarchy, to what level, for what filter.
///
/// It carries no rows. The rows are generated from the structure at run time —
/// an entity hierarchy from `part of`, a category hierarchy from the dotted
/// path — and `depth` decides which of them are shown, so an interior node is
/// a subtotal by virtue of where it sits rather than by declaration.
#[derive(Debug, Serialize)]
struct IrStatement {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    /// Absent for an AUTHORED statement, which states rows instead. Omitted
    /// rather than emitted empty: an empty string is a value that means "no
    /// value", which a consumer has to know to disregard.
    #[serde(skip_serializing_if = "String::is_empty")]
    structure: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    depth: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    grain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    slice: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    metrics: Vec<String>,
    /// Authored rows, for a statement that enumerates rather than generates.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    rows: Vec<IrStatementRow>,
    provenance: IrNodeProvenance,
}

/// One authored row.
#[derive(Debug, Serialize)]
struct IrStatementRow {
    kind: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    label: String,
    #[serde(skip_serializing_if = "is_zero_u32")]
    depth: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    categories: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    streams: Vec<String>,
    /// Ontology types, as declared — lineage. Expanded into `type_streams`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    types: Vec<String>,
    /// Lines by role, as declared — lineage. Expanded into `type_streams`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    lines: Vec<String>,
    /// The streams the `type` and `line` clauses matched — the compiler's
    /// expansion, because only it holds the ontology and the rules. Exact
    /// names; the evaluator claims them beside `streams`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    type_streams: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    slice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    series: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    entity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    numerator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    denominator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    display: Option<String>,
}

#[derive(Debug, Serialize)]
struct IrGeneratedBy {
    pack: IrPackRef,
    rule_id: String,
    /// The LINE the rule emits, by the role its contract's master names
    /// (docs/40 §6) — `interest`, `rent`, `proceeds`. What lets a consumer
    /// fold every debt's interest without knowing any pack's category.
    #[serde(skip_serializing_if = "Option::is_none")]
    line: Option<String>,
    /// The contract the rule lowered this stream FROM — its qualified name.
    /// A rule serves every instance of its type, so the rule alone does not
    /// say which lease a rent stream belongs to; this does, and the results
    /// graph attributes the stream to its contract with it.
    #[serde(skip_serializing_if = "Option::is_none")]
    contract: Option<String>,
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

/// A field's recurrence: its value at the first period, and the rule that
/// carries it to every later one.
#[derive(Debug, Clone, Serialize)]
struct IrFieldRule {
    init: IrExpr,
    next: IrExpr,
    /// A FIELD OF A CONTRACT INHERITS THE CONTRACT'S SCHEDULE.
    ///
    /// Entities are not temporal, so a field a modeller writes has no clock.
    /// But a pack's field comes from a CONTRACT, and a contract has a payment
    /// rhythm — a monthly-paying pool on a daily book must compound twelve
    /// times a year, not 365. Absent means every period, which is what a
    /// modeller's own field means.
    #[serde(skip_serializing_if = "Option::is_none")]
    schedule: Option<IrSchedule>,
}

#[derive(Debug, Serialize)]
struct IrEntity {
    id: String,
    symbol: String,
    r#type: String,
    fields: BTreeMap<String, serde_json::Value>,
    /// Fields that MOVE: an `init`/`next` recurrence owned by this entity.
    /// Separate from `fields` because those are stated facts and these are
    /// rules — the same split the source has between `=` and `init`.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    rules: BTreeMap<String, IrFieldRule>,
    /// The field ROLES this entity's contracts fill, role → the lowered
    /// fields that play it (docs/40 §3, stage 6): `balance` → the survival
    /// factors a pool's streams read. An arrival action naming the role sets
    /// each of them.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    field_roles: BTreeMap<String, Vec<String>>,
    state: BTreeMap<String, serde_json::Value>,
    /// The parent this entity belongs to, when the model groups it. Absent for
    /// an entity that stands alone, which is most of them: hierarchy is always
    /// available and never required.
    #[serde(skip_serializing_if = "Option::is_none")]
    parent: Option<String>,
    /// The lifecycle state this entity starts in. Absent when its type
    /// declares no lifecycle.
    #[serde(skip_serializing_if = "Option::is_none")]
    initial_state: Option<String>,
    /// The machine this entity is governed by — an id into `lifecycles`.
    /// Absent for the many entities that have none.
    #[serde(skip_serializing_if = "Option::is_none")]
    lifecycle: Option<String>,
}

/// A declared finite state machine, as the published IR carries it.
#[derive(Debug, Serialize)]
struct IrLifecycle {
    id: String,
    initial: String,
    states: Vec<String>,
    /// Declared only as used: an absent edge does not exist. Empty means the
    /// machine is unconstrained — `permits()`'s shipped empty-means-open rule.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    edges: Vec<IrLifecycleEdge>,
    /// What is true of a STATE however it was reached. Runs BEFORE the taken
    /// edge's actions — the state's own setup, then the path's refinement.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    entry_actions: Vec<IrStateEntry>,
}

#[derive(Debug, Serialize)]
struct IrLifecycleEdge {
    from: String,
    to: String,
    /// Evaluated each period the entity is in `from`; a guard-less edge is a
    /// permission an event's write may take, never fired by the machine.
    #[serde(skip_serializing_if = "Option::is_none")]
    guard: Option<IrExpr>,
    /// What is true of the PATH taken, on every traversal.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    actions: Vec<IrStateAction>,
}

/// One state's arrival actions.
#[derive(Debug, Serialize)]
struct IrStateEntry {
    state: String,
    actions: Vec<IrStateAction>,
}

/// One arrival action. `author` is emitted always, never inferred: the
/// journal names it, and an `overridden` line that cannot say who wrote the
/// losing value is the one thing the record exists to prevent.
#[derive(Debug, Serialize)]
struct IrStateAction {
    kind: &'static str,
    author: &'static str,
    field: String,
    value: IrExpr,
}

/// An action naming a FIELD ROLE a master declares (docs/40 §3) is emitted
/// as `SetRole`: the engine resolves it to every field on the transitioning
/// entity that plays the role, and to nothing where none does.
fn ir_state_action(action: &ActionDef, declared_roles: &BTreeSet<String>) -> IrStateAction {
    IrStateAction {
        kind: if declared_roles.contains(&action.field) {
            "SetRole"
        } else {
            "SetField"
        },
        author: match action.author {
            cfdl_parser::ActionAuthor::Pack => "pack",
            cfdl_parser::ActionAuthor::Model => "model",
        },
        field: action.field.clone(),
        value: IrExpr {
            lang: "cfdl".to_string(),
            src: coerce_numeric_literals(&action.value),
        },
    }
}

#[derive(Debug, Serialize)]
struct IrAssumptions {
    constants: BTreeMap<String, serde_json::Value>,
    random: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
struct IrExpr {
    lang: String,
    src: String,
}

#[derive(Debug, Serialize)]
struct IrEffects {
    streams: Vec<IrStream>,
}

/// Who a contract is between, by role. The role is the PACK's word as the
/// model bound it; `master_role` is what the master calls it (docs/40 §5), so
/// a consumer reading across packs can find every lender without knowing
/// that a credit pool calls it the holder.
#[derive(Debug, Serialize)]
struct IrPartyBinding {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    master_role: Option<String>,
    entity: IrEntityRef,
}

#[derive(Debug, Serialize)]
struct IrContract {
    id: String,
    name: String,
    /// The ontology type the contract IS — `CRE.Contract.UnitLease` —
    /// resolved once at declaration (docs/40 §8). `core.Contract` only where
    /// no type could be resolved: a contract with no pack active.
    r#type: String,
    /// The pack contract type as the model names it — the rule name,
    /// `cre.lease_unit`.
    #[serde(skip_serializing_if = "Option::is_none")]
    contract_name: Option<String>,
    /// The master at the root of the type's chain — `Contract.Lease`.
    #[serde(skip_serializing_if = "Option::is_none")]
    master: Option<String>,
    /// The instance token where the name carries one — `tenant_a`.
    #[serde(skip_serializing_if = "Option::is_none")]
    instance: Option<String>,
    subject: IrEntityRef,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    parties: Vec<IrPartyBinding>,
    term: IrDateRange,
    currency: String,
    terms: BTreeMap<String, serde_json::Value>,
    effects: IrEffects,
    provenance: IrNodeProvenance,
}

#[derive(Debug, Clone, Serialize)]
struct IrOnRule {
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    day: Option<i32>,
}

/// A pack rule's stated placement, or `None` for the form's default.
/// Validated by the pack loader, so an unknown string cannot reach here.
fn placement_of_rule(stated: Option<&str>) -> Option<Placement> {
    match stated {
        Some("start") => Some(Placement::Start),
        Some("mid") => Some(Placement::Mid),
        Some("end") => Some(Placement::End),
        _ => None,
    }
}

/// The single placement a parsed schedule states, or `None` for the form's
/// default. The parser guarantees at most one is set.
fn placement_of_parsed(due: bool, mid: bool, at_end: bool) -> Option<Placement> {
    match (due, mid, at_end) {
        (true, _, _) => Some(Placement::Start),
        (_, true, _) => Some(Placement::Mid),
        (_, _, true) => Some(Placement::End),
        _ => None,
    }
}

/// Where in its period a flow sits. Serialized as `"start"|"mid"|"end"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Placement {
    Start,
    Mid,
    End,
}

#[derive(Debug, Clone, Serialize)]
struct IrSchedule {
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    on: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    every: Option<String>,
    /// WHERE IN ITS PERIOD THE FLOW SITS — one axis with three positions,
    /// not three booleans, so two placements cannot both be set. Omitted when
    /// the model states none and the form's default applies: a one-shot opens
    /// its period, a recurrence closes it.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    placement: Option<Placement>,
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
    /// `state_enter` anchor (`docs/28` §6.2): the entity and state whose
    /// entries open the windows, and the window length in grid periods.
    /// Present only for kind "StateEnter".
    #[serde(skip_serializing_if = "Option::is_none")]
    anchor_entity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    anchor_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    anchor_periods: Option<i64>,
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
    /// The account this stream's amount moves, resolved to its declared
    /// name (`docs/42` §3.2). Absent on a stream that changes nothing owed.
    #[serde(skip_serializing_if = "Option::is_none")]
    moves: Option<String>,
    schedule: IrSchedule,
    amount: IrExpr,
    active_when: IrExpr,
    provenance: IrNodeProvenance,
}

/// What a pack rule CONSUMED to strike one stream.
///
/// Records the placeholders the rule's templates actually substituted, plus the
/// rule defaults that filled a gap — not the contract's whole term map. A
/// contract lowers to several streams, each reading a different subset of its
/// terms, so "the contract's terms" is not an answer to "what struck this
/// line".
///
/// A side table rather than fields on `IrStream`: the engine passes it through
/// verbatim, so neither `IrStream` nor the per-period evaluation path is
/// widened by it. Pack, rule id and source span are already on the stream's own
/// `provenance` and are not repeated here.
#[derive(Debug, Serialize)]
struct IrStreamInputs {
    stream: String,
    contract: String,
    /// Resolved placeholder values, as the strings the templates substituted.
    /// Not coerced: a term's payload is text plus a span, which is the contract
    /// packs already work against.
    terms: BTreeMap<String, String>,
    /// Keys the contract did not supply, filled from the rule's own defaults.
    /// Separated because "the model said 0" and "the pack assumed 0" are
    /// different facts, and a reader tracing a number needs to tell them apart.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    defaults_applied: Vec<String>,
}

/// A per-period subtotal: a named fold over the ledger, lowered from the
/// active pack's `[[subtotals]]`.
///
/// Lowered into the IR rather than evaluated as a post-pass so that the engine
/// — which is the only thing that has the per-period series — computes it, and
/// so that every host (`cli`, `wasm`, `py`, `server`) gets it with no plumbing:
/// it rides in the IR they already load.
///
/// Array order is dependency order. `parse_subtotal_specs` has already rejected
/// any forward reference, so by here a reference names something earlier.
#[derive(Debug, Serialize)]
struct IrSubtotal {
    id: String,
    kind: String,
    op: String,
    /// Category path prefixes to fold — `operating.*`. The preferred form: it
    /// names what a stream IS rather than what it is called.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    categories: Vec<String>,
    /// Stream-name selectors, for what a category cannot express.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    streams: Vec<String>,
    /// Ids of subtotals declared earlier.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    subtotals: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    numerator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    denominator: Option<String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    formula: String,
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

/// Check every typed entity against the active ontology.
///
/// THE POINT IS THAT A TYPO IS AN ERROR. An entity was a two-part name with no
/// declared type, no declared fields and no declared states, so a misspelled
/// anything was accepted and produced a wrong answer with no signal. Each check
/// below closes one of those.
/// Check that every party binding names a declared party, in a role its
/// contract type recognizes.
///
/// A role belongs to the AGREEMENT, not to the entity — the same party is
/// lessor in one contract and lender in another — so the role list comes from
/// the contract type and the entity only has to be a party.
/// Check that every `exercise option` names an option that exists.
///
/// The other event-action targets are resolved in `cfdl-resolver`, which has
/// the symbol tables. Options are not in them, so this one check lives where
/// the declared options ARE known. Without it a misspelled name matched
/// nothing and the action was silently inert — the option never fired and
/// nothing said so.
/// Lower `active in state a, b` to the comparison it means.
fn state_guard_expr(states: &[cfdl_parser::StateGuard]) -> String {
    states
        .iter()
        // `entity.status`, not `entity.state.status`: an entity's state IS its
        // fields, and `status` is one of them. The second store is gone.
        .map(|guard| format!("entity.status == \"{}\"", guard.state))
        .collect::<Vec<_>>()
        .join(" or ")
}

/// Check every `active in state` names a state its owner's lifecycle declares.
///
/// This is the whole reason the form exists. A string comparison against a
/// status field cannot be checked — a misspelling is simply a comparison that
/// is never true — so the state space has to be declared and the name resolved
/// against it.
/// A stream may not read a field's previous period in the model's FIRST period.
///
/// There is no period before the first, so the read has no answer. Left to the
/// engine it resolves to nothing and the stream evaluates to zero — the same
/// silent wrong number `E1123` and `E1126` exist to refuse, one step along.
///
/// The check is a date comparison rather than a schedule resolution: a stream
/// whose schedule starts on the model's start date runs at period 0. A stream
/// that starts later cannot, whatever its cadence.
/// Every `asset.<name>.<field>` in an expression names a field that exists.
///
/// THE HOLE THIS CLOSES. Field paths resolve through the `entity` root, which
/// is OPEN-WORLD by design: a lifecycle status may not exist until an event
/// writes it, so `entity.status != "refinanced"` has to evaluate before that.
/// Aliasing bare family paths onto that root gave them the same forgiveness —
/// so `asset.tlb.blance` resolved to null, and null in arithmetic becomes zero.
///
/// A DECLARED field is knowable at compile time, so a missing one is a typo
/// rather than an absence. Status keeps the open world; declared fields do not.
/// The complete `time.*` vocabulary the engine binds. Both env builders agree
/// on it, and docs/03 documents the same five.
const TIME_BINDINGS: [&str; 5] = ["t", "date", "days_in_period", "phase", "ppy"];

/// Is an expression decidable without an environment?
///
/// Only literals and operators. Any name, any call, and the answer depends on
/// bindings this stage does not have — the check then says nothing and the
/// engine's run-time warning remains the signal. Conservative on purpose: a
/// false positive here refuses a model that is correct, which is worse than
/// the silence it replaces.
fn is_constant_source(src: &str) -> bool {
    if src.contains('(') {
        return false;
    }
    let mut in_string = false;
    let mut word = String::new();
    let mut words: Vec<String> = Vec::new();
    for c in src.chars() {
        if c == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        if c.is_alphabetic() || c == '_' {
            word.push(c);
        } else {
            if !word.is_empty() {
                words.push(std::mem::take(&mut word));
            }
        }
    }
    if !word.is_empty() {
        words.push(word);
    }
    words
        .iter()
        .all(|w| matches!(w.as_str(), "and" | "or" | "not" | "true" | "false"))
}

/// Constant expressions, checked at compile time.
///
/// `cfdl-expr` has no type inference, so a general type check is a feature and
/// not a missing diagnostic. But an expression built only from literals is
/// decidable by evaluating it, and that covers what an author actually
/// mistypes: `when 42`, `active when 7`, `"text" + 1`, `10 and 3`.
///
/// Each of those ran with `status: ok` — a substituted `false` or `0` and a
/// warning in `deterministic.warnings` that the CLI does not print. `docs/13`
/// §7.71 already settled that shape for series reads: a defect must not hide
/// behind a substituted value and a warning nobody reads. This applies the
/// same rule to the part of the expression language that can be decided.
fn check_constant_expressions(
    resolve_output: &cfdl_resolver::ResolveOutput,
) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    // A constant that fails to evaluate fails for every binding, so the
    // failure is the model's and not the run's.
    let ill_typed =
        |src: &str, what: String, file: &str, span: &Span, out: &mut Vec<Diagnostic>| {
            if !is_constant_source(src) {
                return None;
            }
            let compiled = cfdl_expr::compile_expr(src).ok()?;
            match cfdl_expr::eval(&compiled, &cfdl_expr::ExprEnv::empty()) {
                Ok(value) => Some(value),
                Err(err) if err.code == cfdl_expr::EXPR_UNKNOWN_NAME => None,
                Err(err) => {
                    // Two documented codes split one condition: operands the
                    // operator cannot combine, and an operator applied to the
                    // wrong kind of operand.
                    let code = if err.message.starts_with("cannot apply") {
                        "E3003_EXPR_TYPE_ERROR"
                    } else {
                        "E3004_EXPR_ILLEGAL_OP"
                    };
                    out.push(Diagnostic {
                        code: code.to_string(),
                        severity: "error".to_string(),
                        message: format!("{what} cannot evaluate: {}.", err.message),
                        file: Some(file.to_string()),
                        span: Some(span.clone()),
                        path: None,
                        hint: Some(
                            "Every value in this expression is a literal, so it evaluates the \
                         same way on every period and every run."
                                .to_string(),
                        ),
                        notes: vec![],
                    });
                    None
                }
            }
        };

    for source_stmt in &resolve_output.source_statements {
        let file = source_stmt.file.clone();
        match &source_stmt.statement {
            Stmt::Event(event) => {
                let span = map_span(event.span);
                let what = format!("Event '{}' guard", event.name);
                // A purely scheduled event has no guard to type-check: its
                // occurrences come from the calendar, not from a condition.
                let guard_src = event.when.as_deref().unwrap_or_default();
                if let Some(value) = event
                    .when
                    .as_deref()
                    .and_then(|when| ill_typed(when, what, &file, &span, &mut diagnostics))
                {
                    if !matches!(value, cfdl_expr::Value::Bool(_)) {
                        diagnostics.push(Diagnostic {
                            code: "E2201_EVENT_WHEN_NOT_BOOL".to_string(),
                            severity: "error".to_string(),
                            message: format!(
                                "Event '{}' fires `when {}`, which is not a condition.",
                                event.name,
                                guard_src.trim()
                            ),
                            file: Some(file.clone()),
                            span: Some(span.clone()),
                            path: None,
                            hint: Some(
                                "A guard must be true or false. The engine would take a \
                                 non-boolean as `false`, so the event would never fire."
                                    .to_string(),
                            ),
                            notes: vec![],
                        });
                    }
                }
            }
            Stmt::Stream(stream) => {
                let span = map_span(stream.span);
                if let Some(active) = stream.active_when.as_ref() {
                    let what = format!("Stream '{}' activation", stream.name);
                    if let Some(value) =
                        ill_typed(&active.src, what, &file, &span, &mut diagnostics)
                    {
                        if !matches!(value, cfdl_expr::Value::Bool(_)) {
                            diagnostics.push(Diagnostic {
                                code: "E2202_STREAM_ACTIVE_NOT_BOOL".to_string(),
                                severity: "error".to_string(),
                                message: format!(
                                    "Stream '{}' is `active when {}`, which is not a condition.",
                                    stream.name,
                                    active.src.trim()
                                ),
                                file: Some(file.clone()),
                                span: Some(span.clone()),
                                path: None,
                                hint: Some(
                                    "An activation predicate must be true or false. The engine \
                                     would take a non-boolean as `false`, so the stream would \
                                     never pay."
                                        .to_string(),
                                ),
                                notes: vec![],
                            });
                        }
                    }
                }
                if let Some(amount) = stream.amount.as_ref() {
                    let what = format!("Stream '{}' amount", stream.name);
                    let _ = ill_typed(&amount.src, what, &file, &span, &mut diagnostics);
                }
            }
            _ => {}
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        sort_compile_diagnostics(&mut diagnostics);
        Err(diagnostics)
    }
}

/// `irr`/`moic` belong to a `metric`, and nowhere else.
///
/// Both fold the FINISHED projection. A stream amount that read one would be
/// asking for a return on cash the stream itself has not produced yet — a
/// circularity, not a number. The evaluator refuses it, but a stream amount
/// that fails to evaluate warns and substitutes zero, so without this the
/// model runs `status: ok` on a column of zeroes.
fn check_participant_returns(
    resolve_output: &cfdl_resolver::ResolveOutput,
) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let refuse = |src: &str, what: String, file: &str, span, out: &mut Vec<Diagnostic>| {
        let Ok(compiled) = cfdl_expr::compile_expr(src) else {
            return;
        };
        if !cfdl_expr::uses_participant_return(&compiled) {
            return;
        }
        out.push(Diagnostic {
            code: "E1355_PARTICIPANT_RETURN_OUTSIDE_METRIC".to_string(),
            severity: "error".to_string(),
            message: format!("{what} folds a participant's return."),
            file: Some(file.to_string()),
            span: Some(span),
            path: None,
            hint: Some(
                "`irr` and `moic` are folds over the finished projection, so they belong in a \
                 `metric` declaration. Reading one here would ask for a return on cash this \
                 expression has not produced yet."
                    .to_string(),
            ),
            notes: vec![],
        });
    };

    // WHAT A REFERENCE BUYS OVER A STRING: the party resolves here.
    //
    // A declared entity, of the party family, that owns an account — all three
    // are knowable at compile time, and each was a run-time surprise while the
    // argument was text. `irr(asset.tower)` even reported that an ASSET owned
    // no account.
    let mut parties: BTreeSet<String> = BTreeSet::new();
    let mut entities: BTreeSet<String> = BTreeSet::new();
    for source_stmt in &resolve_output.source_statements {
        if let Stmt::Entity(entity) = &source_stmt.statement {
            let symbol = entity.symbol();
            if symbol.starts_with("party.") {
                parties.insert(symbol.clone());
            }
            entities.insert(symbol);
        }
    }
    let mut owned: BTreeSet<String> = BTreeSet::new();
    for source_stmt in &resolve_output.source_statements {
        if let Stmt::Account(account) = &source_stmt.statement {
            if let Some(owner) = &account.owner {
                let bare = owner.strip_prefix("party.").unwrap_or(owner);
                owned.insert(format!("party.{bare}"));
            }
        }
    }
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Metric(metric) = &source_stmt.statement else {
            continue;
        };
        let Ok(compiled) = cfdl_expr::compile_expr(&metric.expr) else {
            continue;
        };
        if cfdl_expr::participant_return_non_references(&compiled) > 0 {
            diagnostics.push(Diagnostic {
                code: "E1356_PARTICIPANT_RETURN_NOT_A_PARTY".to_string(),
                severity: "error".to_string(),
                message: format!(
                    "Metric '{}' folds a return over something that is not a party reference.",
                    metric.name
                ),
                file: Some(source_stmt.file.clone()),
                span: Some(map_span(metric.span)),
                path: None,
                hint: Some(
                    "Write the party as a reference — `irr(party.lp)` — so the compiler can \
                     resolve it. A party is an entity, like the `owner` of an account and the \
                     payee of a waterfall step."
                        .to_string(),
                ),
                notes: vec![],
            });
        }
        for named in cfdl_expr::participant_return_parties(&compiled) {
            let (code, message, hint) = if !entities.contains(&named) {
                (
                    "E1301_UNRESOLVED_ENTITY_REF",
                    format!("Metric '{}' folds the return of '{named}', which is not a declared entity.", metric.name),
                    "Name a party the model declares.".to_string(),
                )
            } else if !parties.contains(&named) {
                (
                    "E1356_PARTICIPANT_RETURN_NOT_A_PARTY",
                    format!(
                        "Metric '{}' folds the return of '{named}', which is not a party.",
                        metric.name
                    ),
                    "A return belongs to a participant. Name a `party` entity.".to_string(),
                )
            } else if !owned.contains(&named) {
                (
                    "E1356_PARTICIPANT_RETURN_NOT_A_PARTY",
                    format!("Metric '{}' folds the return of '{named}', which owns no account.", metric.name),
                    format!("A participant's return is folded over the party's own account: contributions are negative inflows and receipts are allocations in. Declare `account <name> {{ owner {named} … }}` and pay the waterfall's steps into it."),
                )
            } else {
                continue;
            };
            diagnostics.push(Diagnostic {
                code: code.to_string(),
                severity: "error".to_string(),
                message,
                file: Some(source_stmt.file.clone()),
                span: Some(map_span(metric.span)),
                path: None,
                hint: Some(hint),
                notes: vec![],
            });
        }
    }

    for source_stmt in &resolve_output.source_statements {
        let file = source_stmt.file.clone();
        match &source_stmt.statement {
            Stmt::Stream(stream) => {
                let span = map_span(stream.span);
                if let Some(amount) = stream.amount.as_ref() {
                    let what = format!("Stream '{}' amount", stream.name);
                    refuse(&amount.src, what, &file, span.clone(), &mut diagnostics);
                }
                if let Some(active) = stream.active_when.as_ref() {
                    let what = format!("Stream '{}' activation", stream.name);
                    refuse(&active.src, what, &file, span, &mut diagnostics);
                }
            }
            Stmt::Event(event) => {
                if let Some(when) = event.when.as_deref() {
                    let what = format!("Event '{}' guard", event.name);
                    refuse(when, what, &file, map_span(event.span), &mut diagnostics);
                }
            }
            Stmt::Waterfall(waterfall) => {
                let span = map_span(waterfall.span);
                if let Some(source) = waterfall.source.as_ref() {
                    let what = format!("Waterfall '{}' pot", waterfall.name);
                    refuse(&source.src, what, &file, span.clone(), &mut diagnostics);
                }
                for step in &waterfall.steps {
                    if let Some(amount) = step.amount.as_ref() {
                        let what = format!("Waterfall '{}' step '{}'", waterfall.name, step.name);
                        refuse(&amount.src, what, &file, span.clone(), &mut diagnostics);
                    }
                }
            }
            Stmt::Account(account) => {
                if let Some(inflow) = account.inflow.as_ref() {
                    let what = format!("Account '{}' inflow", account.name);
                    refuse(
                        &inflow.src,
                        what,
                        &file,
                        map_span(account.span),
                        &mut diagnostics,
                    );
                }
            }
            _ => {}
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        sort_compile_diagnostics(&mut diagnostics);
        Err(diagnostics)
    }
}

fn check_field_paths(
    resolve_output: &cfdl_resolver::ResolveOutput,
    // Fields a pack's rules lower onto entities, keyed (owner symbol, field):
    // a structured note's claim is one, and a waterfall step reads it by
    // path exactly as it reads a declared field (docs/40 §4.13).
    lowered_fields: &BTreeMap<(String, String), IrFieldRule>,
) -> Result<(), Vec<Diagnostic>> {
    let mut known: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (owner, field) in lowered_fields.keys() {
        known
            .entry(owner.clone())
            .or_default()
            .insert(field.clone());
    }

    for source_stmt in &resolve_output.source_statements {
        if let Stmt::Entity(entity) = &source_stmt.statement {
            let names = known.entry(entity.symbol()).or_default();
            for f in &entity.literal_fields {
                names.insert(f.name.clone());
            }
            for f in &entity.fields {
                names.insert(f.name.clone());
            }
            // An entity's claim reads as `prev.<entity>.<account>`; the
            // opening-only rule is `check_stream_moves`' business.
            for a in &entity.accounts {
                names.insert(a.name.clone());
            }
            // Lifecycle status stays open: an event may write it later, and a
            // pack's lifecycle declares the states rather than the model.
            names.insert("status".to_string());
            names.insert("state".to_string());
        }
    }
    // A container's folded accounts read the same way (`docs/42` §3.4).
    for full in folded_accounts_of(resolve_output).keys() {
        if let Some((owner, name)) = full.rsplit_once('.') {
            known
                .entry(owner.to_string())
                .or_default()
                .insert(name.to_string());
        }
    }

    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let mut check = |src: &str, span: cfdl_parser::Span, file: &str, ctx: &str| {
        // `time.<field>` is a CLOSED vocabulary the engine binds, so a miss
        // here is a typo — the same silent wrong number E1131 exists to
        // refuse, one root over. Unrejected it evaluates to a warned zero
        // with the run still reporting ok.
        //
        // `inputs.` is deliberately NOT checked here. An input may be
        // declared by an `assume` OR supplied entirely by the run
        // configuration — `run_dists_full` declares no assume at all and
        // takes all five from `monte_carlo.distributions` — and the compiler
        // never sees a run config. Unknown INPUTS are caught by the engine,
        // where both sources are known.
        for name in root_paths(src, "time") {
            if !TIME_BINDINGS.contains(&name.as_str()) {
                diagnostics.push(Diagnostic {
                    code: "E1133_UNKNOWN_TIME_READ".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "{ctx} reads 'time.{name}', which is not a time binding."
                    ),
                    file: Some(file.to_string()),
                    span: Some(map_span(span)),
                    path: None,
                    hint: Some(format!(
                        "The bindings are {}. Unrejected this evaluates to zero and the run still reports ok.",
                        TIME_BINDINGS
                            .iter()
                            .map(|b| format!("`time.{b}`"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )),
                    notes: Vec::new(),
                });
            }
        }
        for (symbol, field) in field_paths(src) {
            let Some(names) = known.get(&symbol) else {
                continue; // an unknown ENTITY is E1301's business, not this one.
            };
            if !names.contains(&field) {
                diagnostics.push(Diagnostic {
                    code: "E1131_UNKNOWN_FIELD_READ".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "{ctx} reads '{symbol}.{field}', which that entity does not declare."
                    ),
                    file: Some(file.to_string()),
                    span: Some(map_span(span)),
                    path: None,
                    hint: Some(
                        "Declare the field on the entity, or correct the name. Unrejected this reads as null and becomes zero in arithmetic."
                            .to_string(),
                    ),
                    notes: Vec::new(),
                });
            }
        }
    };

    for source_stmt in &resolve_output.source_statements {
        let file = source_stmt.file.clone();
        match &source_stmt.statement {
            Stmt::Stream(stream) => {
                for slot in stream.amount.iter().chain(stream.active_when.iter()) {
                    check(
                        &slot.src,
                        slot.span,
                        &file,
                        &format!("Stream '{}'", stream.name),
                    );
                }
            }
            Stmt::Entity(entity) => {
                for f in &entity.fields {
                    let ctx = format!("Field '{}.{}'", entity.symbol(), f.name);
                    check(&f.init.src, f.init.span, &file, &ctx);
                    if let Some(next) = &f.next {
                        check(&next.src, next.span, &file, &ctx);
                    }
                }
            }
            Stmt::Event(event) => {
                if let Some(when) = event.when.as_deref() {
                    check(when, event.span, &file, &format!("Event '{}'", event.name));
                }
            }
            Stmt::Waterfall(w) => {
                if let Some(from) = &w.source {
                    check(
                        &from.src,
                        from.span,
                        &file,
                        &format!("Waterfall '{}'", w.name),
                    );
                }
                for step in &w.steps {
                    if let Some(amount) = &step.amount {
                        check(
                            &amount.src,
                            amount.span,
                            &file,
                            &format!("Waterfall '{}' step '{}'", w.name, step.name),
                        );
                    }
                }
            }
            _ => {}
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

/// `(entity symbol, field)` for every family path in an expression source.
/// Every `<root>.<segment>` in an expression, for a root with a KNOWABLE
/// vocabulary.
///
/// The same shape as `field_paths` and for the same reason. `cfg.` and `obs.`
/// are deliberately absent: those are channels a model opts into by writing
/// the path, with nothing to check against.
fn root_paths(src: &str, root: &str) -> Vec<String> {
    let mut found = Vec::new();
    let needle = format!("{root}.");
    let mut base = 0usize;
    while let Some(idx) = src[base..].find(&needle) {
        let at = base + idx;
        let before_ok = at == 0
            || !src[..at]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.');
        base = at + needle.len();
        if !before_ok {
            continue;
        }
        let rest = &src[base..];
        let seg: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !seg.is_empty() {
            found.push(seg);
        }
    }
    found
}

fn field_paths(src: &str) -> Vec<(String, String)> {
    let mut found = Vec::new();
    for family in cfdl_expr::FIELD_FAMILIES {
        let needle = format!("{family}.");
        // `base` walks forward; the scan is always over `src` itself.
        let mut base = 0usize;
        while let Some(idx) = src[base..].find(&needle) {
            let at = base + idx;
            let before_ok = at == 0
                || !src[..at]
                    .chars()
                    .next_back()
                    .is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.');
            base = at + needle.len();
            if !before_ok {
                continue;
            }
            let tail = &src[base..];
            let end = tail
                .find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '.'))
                .unwrap_or(tail.len());
            let segs: Vec<&str> = tail[..end].split('.').collect();
            if segs.len() >= 2 && !segs[0].is_empty() && !segs[1].is_empty() {
                found.push((format!("{family}.{}", segs[0]), segs[1].to_string()));
            }
        }
    }
    found
}

/// Is `given` close enough to `declared` to be a typo rather than a new field?
///
/// Case alone, or a single edit on a name long enough for that to be unlikely
/// by chance. Short names are excluded: `fee` and `fees` are plausibly both
/// wanted, and calling one a misspelling of the other would be a guess.
fn is_near_miss(declared: &str, given: &str) -> bool {
    if declared.eq_ignore_ascii_case(given) {
        return true;
    }
    if declared.len() < 5 || given.len() < 5 {
        return false;
    }
    edit_distance_at_most_one(declared, given)
}

/// One insertion, deletion or substitution apart.
fn edit_distance_at_most_one(a: &str, b: &str) -> bool {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    if a.len().abs_diff(b.len()) > 1 {
        return false;
    }
    let (mut i, mut j, mut edits) = (0usize, 0usize, 0u8);
    while i < a.len() && j < b.len() {
        if a[i] == b[j] {
            i += 1;
            j += 1;
            continue;
        }
        edits += 1;
        if edits > 1 {
            return false;
        }
        match a.len().cmp(&b.len()) {
            std::cmp::Ordering::Greater => i += 1,
            std::cmp::Ordering::Less => j += 1,
            std::cmp::Ordering::Equal => {
                i += 1;
                j += 1;
            }
        }
    }
    edits + u8::from(i < a.len() || j < b.len()) <= 1
}

fn check_prev_first_period(
    resolve_output: &cfdl_resolver::ResolveOutput,
    model_start: &str,
) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let accounts = declared_accounts_of(resolve_output);
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Stream(stream) = &source_stmt.statement else {
            continue;
        };
        let Some(schedule) = &stream.schedule else {
            continue;
        };
        // Dates are written YYYY-MM or YYYY-MM-DD; compare the month, which is
        // the grain a period boundary sits on.
        let month = |d: &str| d.get(..7).unwrap_or(d).to_string();
        let starts_at_zero = schedule
            .from
            .as_deref()
            .is_some_and(|from| month(from) == month(model_start));
        if !starts_at_zero {
            continue;
        }
        for slot in stream.amount.iter().chain(stream.active_when.iter()) {
            // An account's opening exists in the first period — it is the
            // `init` — so a `prev.<account>` read is not a missing close.
            let without_accounts = strip_prev_accounts(&slot.src, &accounts);
            if reads_prev_field(&without_accounts) {
                diagnostics.push(Diagnostic {
                    code: "E1129_PREV_IN_FIRST_PERIOD".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "Stream '{}' reads a field's previous period but runs from the model's first period, where there is none. Start the stream one period later, or carry the opening value as a field of its own.",
                        stream.name
                    ),
                    file: Some(source_stmt.file.clone()),
                    span: Some(map_span(slot.span)),
                    path: None,
                    hint: Some(
                        "A field's previous period is the close before this one; the first period has no close before it."
                            .to_string(),
                    ),
                    notes: Vec::new(),
                });
            }
        }
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

/// `src` with every `prev.<account>` read blanked, so the field-only checks
/// see only field reads.
fn strip_prev_accounts(src: &str, accounts: &BTreeMap<String, DeclaredAccount>) -> String {
    let mut out = src.to_string();
    for path in prev_paths(src) {
        let bare = path.strip_prefix("entity.").unwrap_or(&path);
        if accounts.contains_key(bare) {
            out = out.replace(&format!("prev.{path}"), "0");
        }
    }
    out
}

/// Does this expression read `prev.<family>.<entity>.<field>`?
/// The same rule as `check_prev_first_period`, on the streams a PACK emitted.
///
/// That check walks source statements, and a lowered stream is not one — it
/// exists only after lowering — so it ran on hand-written streams and on
/// nothing else. A lowered stream reading `prev.<entity>.<field>` at `t = 0`
/// reads a close that does not exist: the previous-value map is empty there, so
/// the engine warns and substitutes zero for that ONE period while every later
/// period is right. One wrong period inside an otherwise correct series is the
/// hardest shape to notice, and the run still reports ok.
///
/// The wording differs from the source-stream case because the remedy does. A
/// model author cannot "start the stream one period later" when the pack owns
/// the schedule, so this names the CONTRACT whose term set it.
/// Event stream targets, checked where every stream that will exist is known.
///
/// `docs/01` §13.2 gives the modeller `deactivate stream <name>`, and §9.1's
/// own example of a stream name — `cre.lease.base_rent` — is a name a CONTRACT
/// produces (`docs/07` §6.4 gives the identical string as its example of a
/// generated name). The specification draws no distinction between a stream a
/// model declared and a stream a contract lowered, and neither does the engine:
/// the action is name-keyed, so a lowered stream stops paying the moment the
/// name resolves.
///
/// The resolver could not resolve it. Its symbol table is built before the pack
/// is even chosen, so at that point a contract's streams do not exist, and the
/// check could not tell "not yet generated" from "misspelled" — it reported
/// both as E1302. A repaid loan therefore kept taking debt service, and the
/// same model expressed it correctly the moment the pack was dropped.
///
/// Here both kinds are in hand. A misspelling still matches nothing, which is
/// what E1302 exists to catch (`docs/08`).
fn check_event_stream_targets(
    resolve_output: &cfdl_resolver::ResolveOutput,
    declared: &[((String, String), IrStream)],
    lowered: &[((String, String), IrStream)],
) -> Result<(), Vec<Diagnostic>> {
    let known: BTreeSet<&str> = declared
        .iter()
        .chain(lowered.iter())
        .map(|((name, _key), _stream)| name.as_str())
        .collect();

    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Event(event) = &source_stmt.statement else {
            continue;
        };
        for action in &event.actions {
            let name = match action {
                cfdl_parser::EventAction::ActivateStream(name)
                | cfdl_parser::EventAction::DeactivateStream(name) => name,
                _ => continue,
            };
            if known.contains(name.as_str()) {
                continue;
            }
            let mut names: Vec<&str> = known.iter().copied().collect();
            names.sort_unstable();
            diagnostics.push(Diagnostic {
                code: "E1302_UNRESOLVED_STREAM_REF".to_string(),
                severity: "error".to_string(),
                message: format!("Event '{}' references unknown stream '{name}'.", event.name),
                file: Some(source_stmt.file.clone()),
                span: Some(map_span(event.span)),
                path: None,
                hint: Some(if names.is_empty() {
                    "The model declares no streams, and no contract lowered any.".to_string()
                } else {
                    format!(
                        "Streams in this model, declared and contract-lowered: {}.",
                        names.join(", ")
                    )
                }),
                notes: vec![],
            });
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        sort_compile_diagnostics(&mut diagnostics);
        Err(diagnostics)
    }
}

fn check_lowered_prev_first_period(
    lowered: &[((String, String), IrStream)],
    stream_inputs: &[IrStreamInputs],
    model_start: &str,
) -> Result<(), Vec<Diagnostic>> {
    let contract_of: BTreeMap<&str, &str> = stream_inputs
        .iter()
        .map(|inputs| (inputs.stream.as_str(), inputs.contract.as_str()))
        .collect();
    // Dates are written YYYY-MM or YYYY-MM-DD; compare the month, which is the
    // grain a period boundary sits on.
    let month = |d: &str| d.get(..7).unwrap_or(d).to_string();
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    for ((name, _key), stream) in lowered {
        let starts_at_zero = stream
            .schedule
            .from
            .as_deref()
            .is_some_and(|from| month(from) == month(model_start));
        if !starts_at_zero {
            continue;
        }
        if !reads_prev_field(&stream.amount.src) && !reads_prev_field(&stream.active_when.src) {
            continue;
        }
        // Name the contract when provenance carries it. A rule that consumed no
        // placeholder records no inputs row, so this is not guaranteed.
        let lowered_from = contract_of
            .get(name.as_str())
            .map(|contract| format!(", lowered from contract '{contract}',"))
            .unwrap_or_else(|| " (pack-lowered)".to_string());
        diagnostics.push(Diagnostic {
            code: "E1129_PREV_IN_FIRST_PERIOD".to_string(),
            severity: "error".to_string(),
            message: format!(
                "Stream '{name}'{lowered_from} reads a field's previous period but runs from the model's first period, where there is none. Start the contract's term one period after the model, or have the rule carry the opening value as a field of its own."
            ),
            file: Some(stream.provenance.source_file.clone()),
            span: Some(stream.provenance.source_span.clone()),
            path: None,
            hint: Some(
                "A field's previous period is the close before this one; the first period has no close before it."
                    .to_string(),
            ),
            notes: Vec::new(),
        });
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

/// `prev.entity.` is here because a field answers to both spellings — and it is
/// the one a PACK LOWERING RULE produces, since `field.<name>` is rewritten to
/// the `entity.` long form. Matching only the bare families let a lowered
/// average-balance stream past this check entirely.
fn reads_prev_field(src: &str) -> bool {
    [
        "prev.asset.",
        "prev.party.",
        "prev.contract.",
        "prev.reference.",
        "prev.entity.",
    ]
    .iter()
    .any(|p| src.contains(p))
}

/// A pack-declared arrival action, stamped `Pack`.
///
/// This is the only place `ActionAuthor::Pack` is produced. Until a pack
/// declared actions the author field had one reachable value, which is why the
/// pack surface and the author stamp belong to the same change.
fn pack_action(action: &cfdl_pack::OntologyAction) -> ActionDef {
    ActionDef {
        field: action.set.clone(),
        value: action.value.clone(),
        author: cfdl_parser::ActionAuthor::Pack,
    }
}

/// One resolved machine, wherever it was declared (`docs/28` §6.1).
///
/// A pack's `types.toml` lifecycle and a model's `lifecycle` block resolve to
/// the same thing: an id, an enumerated state set, and the edges declared —
/// pack edges guard-less (permissions), model edges optionally guarded. The
/// core has the full functionality and packs tailor it, so every consumer
/// looks HERE rather than at the ontology.
#[derive(Debug, Clone)]
struct MachineDef {
    id: String,
    initial: Option<String>,
    states: Vec<String>,
    /// Empty means the machine is unconstrained — `permits()`'s shipped
    /// empty-means-open rule.
    edges: Vec<EdgeDef>,
    /// State name -> its arrival actions, pack's first and the model's after
    /// (`docs/34` D2a). The order IS the resolution rule: the model's write
    /// lands last and wins, and the pack's is what journals `overridden`.
    entry_actions: BTreeMap<String, Vec<ActionDef>>,
    /// Whether a PACK declared this machine. An augmenting model block is
    /// only reachable for one that did — a model's own machine carries its
    /// actions inline and has nothing to augment.
    from_pack: bool,
}

/// One edge and what it does on traversal.
#[derive(Debug, Clone)]
struct EdgeDef {
    from: String,
    to: String,
    guard: Option<String>,
    actions: Vec<ActionDef>,
}

/// One arrival action, carrying WHO WROTE IT.
///
/// The author is stored, never inferred: a pack action whose stamp was
/// forgotten would journal as the model's, and an `overridden` line that
/// cannot name its author is the one thing the record exists to prevent.
#[derive(Debug, Clone)]
struct ActionDef {
    field: String,
    value: String,
    author: cfdl_parser::ActionAuthor,
}

impl MachineDef {
    fn has_state(&self, state: &str) -> bool {
        self.states.iter().any(|s| s == state)
    }
}

/// Every entity's machine, and every machine by id.
///
/// The binding wins over the type only by being checked first elsewhere —
/// an entity with BOTH is `E1350`, refused where the type is at hand.
fn resolve_machines(
    resolve_output: &cfdl_resolver::ResolveOutput,
    ontology: &cfdl_pack::PackOntology,
) -> (BTreeMap<String, MachineDef>, BTreeMap<String, String>) {
    let mut machines: BTreeMap<String, MachineDef> = BTreeMap::new();
    let mut by_entity: BTreeMap<String, String> = BTreeMap::new();

    // THE ORDER OF THESE PASSES IS THE D2a RULE, and it is dependency rather
    // than precedence: a model's actions attach to a pack's machine, so the
    // machine has to exist before they can. The MODEL still wins where the
    // two write the same field — its actions are appended after the pack's
    // and land last (`docs/34` D2a, D5).
    //
    // Reversing these two passes is what the shipped code did, and with
    // `or_insert_with` it meant a model block naming a pack machine REPLACED
    // it. Under D2a that is worse than a collision: an augmenting block
    // carries no `initial` and no `state`, so the replacement was an empty
    // machine and the augmentation deleted what it meant to extend.

    // Pass A — the pack's machines, reached through the entity types that
    // bind them. These are the base layer.
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Entity(entity) = &source_stmt.statement else {
            continue;
        };
        if entity.lifecycle.is_some() {
            continue;
        }
        let from_type = entity
            .type_name
            .as_deref()
            .and_then(|ty| ontology.entity(ty))
            .and_then(|ty| ty.lifecycle.as_deref())
            .and_then(|id| ontology.lifecycle(id));
        if let Some(lifecycle) = from_type {
            machines
                .entry(lifecycle.lifecycle_id.clone())
                .or_insert_with(|| MachineDef {
                    id: lifecycle.lifecycle_id.clone(),
                    initial: Some(lifecycle.initial.clone()),
                    states: lifecycle.states.clone(),
                    edges: lifecycle
                        .transitions
                        .iter()
                        .map(|t| EdgeDef {
                            from: t.from.clone(),
                            to: t.to.clone(),
                            guard: t.guard.clone(),
                            actions: t.actions.iter().map(pack_action).collect(),
                        })
                        .collect(),
                    // The pack's own arrival actions, stamped as its own. A
                    // model's augmentation is appended AFTER these, so the
                    // model's write lands last and wins (`docs/34` D2a).
                    entry_actions: lifecycle
                        .entry_actions
                        .iter()
                        .map(|entry| {
                            (
                                entry.state.clone(),
                                entry.actions.iter().map(pack_action).collect(),
                            )
                        })
                        .collect(),
                    from_pack: true,
                });
        }
    }

    // Pass B — the model's own blocks. A name that already resolves to a
    // pack machine AUGMENTS it; any other name DECLARES one. Nothing is
    // inferred from what the block omits: augmentation is reachable only for
    // a pack machine, because a model's own machine carries its actions
    // inline and has nothing to augment.
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Lifecycle(lc) = &source_stmt.statement else {
            continue;
        };
        let entry_actions = |author| {
            let mut map: BTreeMap<String, Vec<ActionDef>> = BTreeMap::new();
            for entry in &lc.entry_actions {
                map.entry(entry.state.clone())
                    .or_default()
                    .extend(entry.actions.iter().map(|a| ActionDef {
                        field: a.field.clone(),
                        value: a.value.src.clone(),
                        author,
                    }));
            }
            map
        };
        match machines.get_mut(&lc.name) {
            // Augmenting: contribute actions, and nothing else. A block that
            // also states topology is refused in validation (`E1303`); the
            // states and edges are ignored here so a refused model cannot
            // also corrupt the machine it named.
            Some(existing) if existing.from_pack => {
                for (state, actions) in entry_actions(cfdl_parser::ActionAuthor::Model) {
                    existing
                        .entry_actions
                        .entry(state)
                        .or_default()
                        .extend(actions);
                }
                for edge in &lc.edges {
                    if let Some(target) = existing
                        .edges
                        .iter_mut()
                        .find(|e| e.from == edge.from && e.to == edge.to)
                    {
                        target
                            .actions
                            .extend(edge.actions.iter().map(|a| ActionDef {
                                field: a.field.clone(),
                                value: a.value.src.clone(),
                                author: cfdl_parser::ActionAuthor::Model,
                            }));
                    }
                }
            }
            // Declaring: the model's own machine, actions inline.
            _ => {
                machines
                    .entry(lc.name.clone())
                    .or_insert_with(|| MachineDef {
                        id: lc.name.clone(),
                        initial: lc.initial.clone(),
                        states: lc.states.clone(),
                        edges: lc
                            .edges
                            .iter()
                            .map(|e| EdgeDef {
                                from: e.from.clone(),
                                to: e.to.clone(),
                                guard: e.guard.as_ref().map(|g| g.src.clone()),
                                actions: e
                                    .actions
                                    .iter()
                                    .map(|a| ActionDef {
                                        field: a.field.clone(),
                                        value: a.value.src.clone(),
                                        author: cfdl_parser::ActionAuthor::Model,
                                    })
                                    .collect(),
                            })
                            .collect(),
                        entry_actions: entry_actions(cfdl_parser::ActionAuthor::Model),
                        from_pack: false,
                    });
            }
        }
    }

    // Pass C — bindings, once every machine of either origin is resolvable.
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Entity(entity) = &source_stmt.statement else {
            continue;
        };
        if let Some(bound) = &entity.lifecycle {
            if machines.contains_key(bound) {
                by_entity.insert(entity.symbol(), bound.clone());
            }
            // An unknown binding is validate's E1349; nothing to resolve.
            continue;
        }
        let from_type = entity
            .type_name
            .as_deref()
            .and_then(|ty| ontology.entity(ty))
            .and_then(|ty| ty.lifecycle.as_deref())
            .and_then(|id| ontology.lifecycle(id));
        if let Some(lifecycle) = from_type {
            by_entity.insert(entity.symbol(), lifecycle.lifecycle_id.clone());
        }
    }

    (machines, by_entity)
}

/// An arrival action writes a field the ENTITY THAT TRANSITIONED actually has.
///
/// The name is entity-relative, so it resolves against every entity bound to
/// the machine rather than against one named target — and a machine bound by
/// several entities has to satisfy all of them. Checked here rather than in
/// validation because the field set is the union of what the model's block
/// declares and what the ontology type contributes, and only this side has the
/// ontology.
///
/// Without this a misspelled field is a write that lands nowhere: the same
/// silent-substitution shape `docs/13` §7.38 records for a misspelled series.
fn check_arrival_action_fields(
    resolve_output: &cfdl_resolver::ResolveOutput,
    machines: &BTreeMap<String, MachineDef>,
    machines_by_entity: &BTreeMap<String, String>,
    ontology: &cfdl_pack::PackOntology,
    // Field roles the pack's rules fill per entity, (owner, role) → fields
    // (docs/40 §3, stage 6). An action may name a role instead of a field.
    field_roles: &BTreeMap<(String, String), Vec<String>>,
) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    // Every role any master names, so an unfilled role is told apart from a
    // misspelled field.
    let declared_roles: BTreeSet<&str> = ontology
        .contracts
        .iter()
        .flat_map(|c| c.field_roles.iter().map(|r| r.name.as_str()))
        .collect();
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Entity(entity) = &source_stmt.statement else {
            continue;
        };
        let symbol = entity.symbol();
        let Some(machine_id) = machines_by_entity.get(&symbol) else {
            continue;
        };
        let Some(machine) = machines.get(machine_id) else {
            continue;
        };
        let mut known: BTreeSet<&str> = entity.fields.iter().map(|f| f.name.as_str()).collect();
        known.extend(entity.literal_fields.iter().map(|f| f.name.as_str()));
        if let Some(ty) = entity
            .type_name
            .as_deref()
            .and_then(|ty| ontology.entity(ty))
        {
            known.extend(ty.fields.iter().map(|f| f.name.as_str()));
        }
        let filled: BTreeSet<&str> = field_roles
            .keys()
            .filter(|(owner, _)| *owner == symbol)
            .map(|(_, role)| role.as_str())
            .collect();
        let mut report = |action: &ActionDef, where_: String| {
            if known.contains(action.field.as_str()) || filled.contains(action.field.as_str()) {
                return;
            }
            // A ROLE NOTHING ON THIS ENTITY FILLS. A PACK's action is written
            // once for every entity of the type, and an entity of that type
            // carrying no such contract has nothing to extinguish — the action
            // is a no-op there, not an error. A MODEL's action names the
            // entity it means: a closed-form debt has no balance field to
            // extinguish, and saying so is the point.
            if declared_roles.contains(action.field.as_str()) {
                if matches!(action.author, cfdl_parser::ActionAuthor::Pack) {
                    return;
                }
                diagnostics.push(Diagnostic {
                    code: "E1359_ARRIVAL_ACTION_UNKNOWN_FIELD".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "Lifecycle '{machine_id}' {where_} sets role '{}', which no contract on entity '{symbol}' fills.",
                        action.field,
                    ),
                    file: Some(source_stmt.file.clone()),
                    span: Some(map_span(entity.span)),
                    path: None,
                    hint: Some(
                        "A field role is filled by a pack rule that lowers a field (`field_role = \"balance\"`); a contract whose balance is closed-form fills none, and the machine cannot extinguish what it does not carry."
                            .to_string(),
                    ),
                    notes: vec![],
                });
                return;
            }
            let mut names: Vec<&str> = known.iter().copied().collect();
            names.sort_unstable();
            let declared = if names.is_empty() {
                "it declares none".to_string()
            } else {
                format!("declared: {}", names.join(", "))
            };
            diagnostics.push(Diagnostic {
                code: "E1359_ARRIVAL_ACTION_UNKNOWN_FIELD".to_string(),
                severity: "error".to_string(),
                message: format!(
                    "Lifecycle '{machine_id}' {where_} sets '{}', which entity '{symbol}' does not have — {declared}.",
                    action.field,
                ),
                file: Some(source_stmt.file.clone()),
                span: Some(map_span(entity.span)),
                path: None,
                hint: Some(
                    "An arrival action names a field on the entity that transitioned, and one \
                     machine may be bound by several entities — every one of them needs the \
                     field. Declare it on the entity, or correct the name."
                        .to_string(),
                ),
                notes: vec![],
            });
        };
        for (state, actions) in &machine.entry_actions {
            for action in actions {
                report(action, format!("entry into '{state}'"));
            }
        }
        for edge in &machine.edges {
            for action in &edge.actions {
                report(action, format!("edge '{} -> {}'", edge.from, edge.to));
            }
        }
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

/// A model block naming a PACK machine ENHANCES it; it never replaces it.
///
/// D2a's "additively only" has to be refused rather than ignored. Dropping the
/// states an augmenting block wrote would leave a model saying one thing and
/// the machine doing another — the silent-substitution shape `docs/13` §7.38
/// records, one construct over. A model that needs different topology declares
/// its own machine under its own name, which is a declaration and not an
/// augmentation at all.
fn check_lifecycle_augmentations(
    resolve_output: &cfdl_resolver::ResolveOutput,
    machines: &BTreeMap<String, MachineDef>,
) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Lifecycle(lc) = &source_stmt.statement else {
            continue;
        };
        // Only a pack's machine can be augmented: a model's own carries its
        // actions inline and has nothing to augment.
        if !machines.get(&lc.name).is_some_and(|m| m.from_pack) {
            continue;
        }
        let mut stated: Vec<&str> = Vec::new();
        if lc.initial.is_some() {
            stated.push("initial");
        }
        if !lc.states.is_empty() {
            stated.push("state");
        }
        if lc
            .edges
            .iter()
            .any(|e| e.guard.is_some() || e.actions.is_empty())
        {
            stated.push("an edge");
        }
        if stated.is_empty() {
            continue;
        }
        diagnostics.push(Diagnostic {
            code: "E1357_LIFECYCLE_AUGMENT_TOPOLOGY".to_string(),
            severity: "error".to_string(),
            message: format!(
                "Lifecycle '{}' is declared by a pack, and this block states {}.",
                lc.name,
                stated.join(" and "),
            ),
            file: Some(source_stmt.file.clone()),
            span: Some(map_span(lc.span)),
            path: None,
            hint: Some(
                "A model may add arrival actions to a pack's machine — `on enter <state>` and \
                 actions on an existing edge — and nothing else. To change the states or the \
                 edges, declare a separate machine under its own name and bind the entity to \
                 that instead."
                    .to_string(),
            ),
            notes: vec![],
        });
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

/// The compile-statable half of `docs/28` §6.1 rule 3: where an event writes
/// `status` with a string LITERAL, the target is checkable now. An unknown
/// state is `E1316`; a state the declared relation gives no edge INTO can
/// never be legally entered, whatever state the entity is in at run — that
/// certainty is what makes the refusal compile-time. An edge-less machine
/// stays unconstrained, `permits()`'s shipped empty-means-open rule; the
/// run-time half, where the from-state is a fact, lives in the engine.
fn check_status_writes(
    resolve_output: &cfdl_resolver::ResolveOutput,
    machines: &BTreeMap<String, MachineDef>,
    machines_by_entity: &BTreeMap<String, String>,
) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Event(event) = &source_stmt.statement else {
            continue;
        };
        for action in &event.actions {
            let cfdl_parser::EventAction::SetEntityField {
                entity,
                field,
                value,
            } = action
            else {
                continue;
            };
            if field != "status" {
                continue;
            }
            let Some(machine) = machines_by_entity
                .get(entity)
                .and_then(|id| machines.get(id))
            else {
                continue;
            };
            // Statable means a bare string literal — anything computed is the
            // run-time half's problem.
            let trimmed = value.trim();
            let target = match trimmed
                .strip_prefix('"')
                .and_then(|rest| rest.strip_suffix('"'))
            {
                Some(inner) if !inner.contains('"') => inner,
                _ => continue,
            };
            if !machine.has_state(target) {
                diagnostics.push(Diagnostic {
                    code: "E1316_UNKNOWN_LIFECYCLE_STATE".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "Event '{}' sets '{entity}.status' to '{target}', which lifecycle '{}' does not declare.",
                        event.name, machine.id
                    ),
                    file: Some(source_stmt.file.clone()),
                    span: Some(map_span(event.span)),
                    path: None,
                    hint: Some(format!("Declared states: {}.", machine.states.join(", "))),
                    notes: vec![],
                });
                continue;
            }
            if !machine.edges.is_empty() && !machine.edges.iter().any(|e| e.to == *target) {
                diagnostics.push(Diagnostic {
                    code: "E1353_UNREACHABLE_STATE_WRITE".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "Event '{}' sets '{entity}.status' to '{target}', but no edge of lifecycle '{}' enters that state — the write can never be legal.",
                        event.name, machine.id
                    ),
                    file: Some(source_stmt.file.clone()),
                    span: Some(map_span(event.span)),
                    path: None,
                    hint: Some(
                        "Declare the edge — declaring it is what brings the move into existence — or drop the write."
                            .to_string(),
                    ),
                    notes: vec![],
                });
            }
        }
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn check_state_guards(
    resolve_output: &cfdl_resolver::ResolveOutput,
    ontology: &cfdl_pack::PackOntology,
) -> Result<(), Vec<Diagnostic>> {
    let (machines, machines_by_entity) = resolve_machines(resolve_output, ontology);

    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    // A MACHINE GUARD IS LOGIC wherever it was declared: it reads series
    // strictly backward (`docs/28` §4). Model-side guards are checked by the
    // validator against source; a PACK's guards arrive here, so the same
    // rule is applied to every bound machine's edges — one rule, two
    // declaration sites.
    let model_declared: std::collections::BTreeSet<String> = resolve_output
        .source_statements
        .iter()
        .filter_map(|s| match &s.statement {
            Stmt::Lifecycle(lc) => Some(lc.name.clone()),
            _ => None,
        })
        .collect();
    for machine in machines.values() {
        // Model-side guards are the validator's, with source spans; only a
        // PACK's guards are first seen here.
        if model_declared.contains(&machine.id) {
            continue;
        }
        for edge in &machine.edges {
            let (from, to) = (&edge.from, &edge.to);
            let Some(guard) = edge.guard.as_deref() else {
                continue;
            };
            let forward = cfdl_expr::series_windows(guard).iter().any(|w| {
                !(cfdl_expr::window_bound_is_strictly_backward(&w.from_src)
                    && cfdl_expr::window_bound_is_strictly_backward(&w.to_src))
            });
            if forward {
                diagnostics.push(Diagnostic {
                    code: "E1134_SERIES_READ_IN_LOGIC".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "Lifecycle '{}' edge '{from} -> {to}' guard reads a series window that can reach the current period or beyond. Logic reads settled history: at or before the previous period.",
                        machine.id
                    ),
                    file: None,
                    span: None,
                    path: None,
                    hint: Some("Write the window against time.t - 1 or earlier.".to_string()),
                    notes: vec![],
                });
            }
        }
    }

    // A state_enter anchor names an entity with a machine and a state that
    // machine declares — the same finite-set discipline every other state
    // reference has (`docs/28` §6.2).
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Stream(stream) = &source_stmt.statement else {
            continue;
        };
        let Some(schedule) = &stream.schedule else {
            continue;
        };
        let cfdl_parser::ScheduleKind::StateEnter { entity, state, .. } = &schedule.kind else {
            continue;
        };
        let Some(machine) = machines_by_entity
            .get(entity)
            .and_then(|id| machines.get(id))
        else {
            diagnostics.push(Diagnostic {
                code: "E1349_UNRESOLVED_LIFECYCLE_REF".to_string(),
                severity: "error".to_string(),
                message: format!(
                    "Stream '{}' anchors to state_enter({entity}, {state}), but '{entity}' has no lifecycle.",
                    stream.name
                ),
                file: Some(source_stmt.file.clone()),
                span: Some(map_span(schedule.span)),
                path: None,
                hint: Some("Bind a machine to the entity, or anchor to a date or phase.".to_string()),
                notes: vec![],
            });
            continue;
        };
        if !machine.has_state(state) {
            diagnostics.push(Diagnostic {
                code: "E1316_UNKNOWN_LIFECYCLE_STATE".to_string(),
                severity: "error".to_string(),
                message: format!(
                    "Stream '{}' anchors to state_enter({entity}, {state}), but lifecycle '{}' does not declare '{state}'.",
                    stream.name, machine.id
                ),
                file: Some(source_stmt.file.clone()),
                span: Some(map_span(schedule.span)),
                path: None,
                hint: Some(format!("Declared states: {}.", machine.states.join(", "))),
                notes: vec![],
            });
        }
    }
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Stream(stream) = &source_stmt.statement else {
            continue;
        };
        if stream.active_in_states.is_empty() {
            continue;
        }
        if stream.active_when.is_some() {
            diagnostics.push(Diagnostic {
                code: "E1330_CONFLICTING_ACTIVE_CLAUSES".to_string(),
                severity: "error".to_string(),
                message: format!(
                    "Stream '{}' declares both 'active when' and 'active in state'.",
                    stream.name
                ),
                file: Some(source_stmt.file.clone()),
                span: Some(map_span(stream.span)),
                path: None,
                hint: Some(
                    "Use one: 'active in state' for a lifecycle state, 'active when' for anything else."
                        .to_string(),
                ),
                notes: vec![],
            });
            continue;
        }

        let owner = &stream.attached_entity;
        let lifecycle = machines_by_entity
            .get(owner)
            .and_then(|id| machines.get(id));

        let Some(lifecycle) = lifecycle else {
            diagnostics.push(Diagnostic {
                code: "E1331_OWNER_HAS_NO_LIFECYCLE".to_string(),
                severity: "error".to_string(),
                message: format!(
                    "Stream '{}' is active in a lifecycle state, but its owner '{owner}' has no lifecycle.",
                    stream.name
                ),
                file: Some(source_stmt.file.clone()),
                span: Some(map_span(stream.span)),
                path: None,
                hint: Some(
                    "Give the owner a type whose lifecycle declares the states, or use 'active when'."
                        .to_string(),
                ),
                notes: vec![],
            });
            continue;
        };

        for guard in &stream.active_in_states {
            if !lifecycle.has_state(&guard.state) {
                diagnostics.push(Diagnostic {
                    code: "E1332_UNKNOWN_ACTIVE_STATE".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "Stream '{}' is active in state '{}', which lifecycle '{}' does not declare.",
                        stream.name, guard.state, lifecycle.id
                    ),
                    file: Some(source_stmt.file.clone()),
                    span: Some(map_span(guard.span)),
                    path: None,
                    hint: Some(format!("Declared states: {}.", lifecycle.states.join(", "))),
                    notes: vec![],
                });
            }
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

/// Structural checks on a waterfall — the ones that stop cash going missing.
///
/// A waterfall allocates a pot in order. Three things about it are decidable
/// before it ever runs, and each is a wrong answer rather than a crash if it
/// is not caught:
///
///   * a step that names a payee nobody declared pays into the void;
///   * `overflow of <step>` naming a step that is not earlier, or is not
///     capped, pays a shortfall that cannot exist;
///   * a missing or misplaced `remainder` silently LOSES whatever is left in
///     the pot, which is the failure the residual step exists to prevent.
// A declared account as the checks see it: its side (`docs/42` §3.6), and
// whether it is a relation fold rather than a declaration (§3.4).
#[derive(Clone, Debug)]
struct DeclaredAccount {
    side: Option<String>,
    fold: bool,
}

/// Every account a relation folds: for each ancestor of an entity that
/// declares a claim, `<ancestor>.<name>` — the container's balance as the
/// sum of its members' (`docs/42` §3.4). Keyed by full name, valued by the
/// member accounts it sums.
fn folded_accounts_of(
    resolve_output: &cfdl_resolver::ResolveOutput,
) -> BTreeMap<String, Vec<String>> {
    let parent_of: BTreeMap<String, String> = resolve_output
        .source_statements
        .iter()
        .filter_map(|s| match &s.statement {
            Stmt::Entity(e) => e.parent.clone().map(|p| (e.symbol(), p)),
            _ => None,
        })
        .collect();
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Entity(entity) = &source_stmt.statement else {
            continue;
        };
        for acct in &entity.accounts {
            let member = format!("{}.{}", entity.symbol(), acct.name);
            let mut cursor = parent_of.get(&entity.symbol());
            let mut seen: BTreeSet<&str> = BTreeSet::new();
            while let Some(ancestor) = cursor {
                if !seen.insert(ancestor.as_str()) {
                    break;
                }
                out.entry(format!("{ancestor}.{}", acct.name))
                    .or_default()
                    .push(member.clone());
                cursor = parent_of.get(ancestor);
            }
        }
    }
    out
}

/// Every account the model declares, by its full name — a structure or
/// party account by its own name, an entity's claim as `<entity>.<name>`.
fn declared_accounts_of(
    resolve_output: &cfdl_resolver::ResolveOutput,
) -> BTreeMap<String, DeclaredAccount> {
    let mut out: BTreeMap<String, DeclaredAccount> = BTreeMap::new();
    for source_stmt in &resolve_output.source_statements {
        match &source_stmt.statement {
            Stmt::Account(a) => {
                out.insert(
                    a.name.clone(),
                    DeclaredAccount {
                        side: a.side.clone(),
                        fold: false,
                    },
                );
            }
            Stmt::Entity(entity) => {
                for acct in &entity.accounts {
                    out.insert(
                        format!("{}.{}", entity.symbol(), acct.name),
                        DeclaredAccount {
                            side: acct.side.clone(),
                            fold: false,
                        },
                    );
                }
            }
            _ => {}
        }
    }
    // A container's folded account is readable (`prev.container.trust.balance`)
    // but declares nothing: a declaration of the same name is refused, not
    // merged, so the fold stays the sum of its members and nothing else.
    for name in folded_accounts_of(resolve_output).keys() {
        out.entry(name.clone()).or_insert(DeclaredAccount {
            side: None,
            fold: true,
        });
    }
    out
}

/// `moves <name>` on a stream owned by `owner`: a bare name is the owner's
/// own claim first, then a structure account; a qualified name is taken as
/// written.
fn resolve_moved_account(
    owner: &str,
    moves: &str,
    accounts: &BTreeMap<String, DeclaredAccount>,
) -> Option<String> {
    let own = format!("{owner}.{moves}");
    if accounts.contains_key(&own) {
        return Some(own);
    }
    if accounts.contains_key(moves) {
        return Some(moves.to_string());
    }
    None
}

/// The two non-cash directions (`docs/42` §3.2): a claim raised or
/// extinguished with no money moving.
fn is_cash_direction(direction: &str) -> bool {
    !matches!(direction, "accrual" | "writeoff")
}

/// Every `prev.<path>` read in `src`, as the path after `prev.`.
fn prev_paths(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = src.as_bytes();
    let mut i = 0;
    while let Some(idx) = src[i..].find("prev.") {
        let at = i + idx;
        let boundary_ok = at == 0 || {
            let c = bytes[at - 1] as char;
            !c.is_alphanumeric() && c != '_' && c != '.'
        };
        let start = at + 5;
        let mut end = start;
        while end < bytes.len() {
            let c = bytes[end] as char;
            if c.is_alphanumeric() || c == '_' || c == '.' {
                end += 1;
            } else {
                break;
            }
        }
        if boundary_ok && end > start {
            out.push(src[start..end].to_string());
        }
        i = end.max(at + 5);
    }
    out
}

/// A stream or field on `owner` reads its own claim as `prev.balance`; the
/// engine binds the account by its full name, so the read is spelled out
/// here — `prev.asset.loan.balance` — once, at lowering.
fn rewrite_prev_accounts(
    src: &str,
    owner: &str,
    accounts: &BTreeMap<String, DeclaredAccount>,
) -> String {
    let mut out = src.to_string();
    for path in prev_paths(src) {
        if path.contains('.') {
            continue;
        }
        let full = format!("{owner}.{path}");
        if accounts.contains_key(&full) {
            let from = format!("prev.{path}");
            let to = format!("prev.{full}");
            // Whole-path replacement: `prev.balance` must not touch
            // `prev.balance_lag`.
            let mut rebuilt = String::new();
            let mut rest = out.as_str();
            while let Some(idx) = rest.find(&from) {
                let after = rest[idx + from.len()..].chars().next();
                let before = rebuilt.chars().next_back().or_else(|| {
                    if idx == 0 {
                        None
                    } else {
                        rest[..idx].chars().next_back()
                    }
                });
                let before_ok = before.is_none_or(|c| !c.is_alphanumeric() && c != '_' && c != '.');
                let after_ok = after.is_none_or(|c| !c.is_alphanumeric() && c != '_' && c != '.');
                rebuilt.push_str(&rest[..idx]);
                if before_ok && after_ok {
                    rebuilt.push_str(&to);
                } else {
                    rebuilt.push_str(&from);
                }
                rest = &rest[idx + from.len()..];
            }
            rebuilt.push_str(rest);
            out = rebuilt;
        }
    }
    out
}

/// `docs/42` §3: a stream that moves an account names one that exists; a
/// non-cash stream moves something and carries no cash category; a cash
/// stream moves only an account whose side is declared; and a balance is
/// read only as `prev.` — the opening — never as a current value.
fn check_stream_moves(
    resolve_output: &cfdl_resolver::ResolveOutput,
) -> Result<(), Vec<Diagnostic>> {
    let accounts = declared_accounts_of(resolve_output);
    let folded = folded_accounts_of(resolve_output);
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let diag =
        |code: &str, message: String, span: cfdl_parser::Span, file: &str, hint: &str| Diagnostic {
            code: code.to_string(),
            severity: "error".to_string(),
            message,
            file: Some(file.to_string()),
            span: Some(map_span(span)),
            path: None,
            hint: Some(hint.to_string()),
            notes: Vec::new(),
        };
    for source_stmt in &resolve_output.source_statements {
        let file = &source_stmt.file;
        match &source_stmt.statement {
            Stmt::Stream(stream) => {
                let direction = stream.direction.as_deref().unwrap_or("outflow");
                let cash = is_cash_direction(direction);
                if !cash && stream.moves.is_none() {
                    diagnostics.push(diag(
                        "E1378_NONCASH_STREAM_MOVES_NOTHING",
                        format!(
                            "Stream '{}' is `{direction}`, which raises or extinguishes a claim, and names no account to move.",
                            stream.name
                        ),
                        stream.span,
                        file,
                        "A non-cash stream is a movement of a balance and nothing else: add `moves <account>`, or make it an inflow or outflow if money actually moves.",
                    ));
                }
                if !cash && stream.category.is_some() {
                    diagnostics.push(diag(
                        "E1379_NONCASH_STREAM_CATEGORY",
                        format!(
                            "Stream '{}' is `{direction}` and carries a cash flow category.",
                            stream.name
                        ),
                        stream.span,
                        file,
                        "The category roots classify cash. An accrual or a write-off is excluded from every cash fold; drop the category.",
                    ));
                }
                if let Some(moves) = &stream.moves {
                    match resolve_moved_account(&stream.attached_entity, moves, &accounts) {
                        None => diagnostics.push(diag(
                            "E1380_UNKNOWN_ACCOUNT_MOVED",
                            format!(
                                "Stream '{}' moves account '{moves}', which is not declared on '{}' or as a structure account.",
                                stream.name, stream.attached_entity
                            ),
                            stream.span,
                            file,
                            "Declare it — `account <name> owed|due [init <expr>]` in the entity block, or `account <name> owed|due { ... }` at the model level — or correct the name.",
                        )),
                        Some(full) if accounts[&full].fold => diagnostics.push(diag(
                            "E1384_FOLDED_ACCOUNT_MOVED",
                            format!(
                                "Stream '{}' moves account '{full}', which is a relation fold — the sum of its members' balances — and is moved only by moving a member's.",
                                stream.name
                            ),
                            stream.span,
                            file,
                            "Move the member's account (`moves balance` on a stream owned by the member), or declare an account of another name on the container.",
                        )),
                        Some(full) => {
                            if cash && accounts[&full].side.is_none() {
                                diagnostics.push(diag(
                                    "E1381_MOVED_ACCOUNT_HAS_NO_SIDE",
                                    format!(
                                        "Stream '{}' is `{direction}` and moves account '{full}', which declares no side.",
                                        stream.name
                                    ),
                                    stream.span,
                                    file,
                                    "Whether an inflow raises or lowers a balance follows from the account's side: write `owed` (a liability of its owner) or `due` (a receivable) after the account's name.",
                                ));
                            }
                        }
                    }
                }
                for slot in stream.amount.iter().chain(stream.active_when.iter()) {
                    check_current_account_reads(
                        &slot.src,
                        &stream.attached_entity,
                        &accounts,
                        slot.span,
                        file,
                        &format!("Stream '{}'", stream.name),
                        &mut diagnostics,
                    );
                }
            }
            Stmt::Entity(entity) => {
                // A container may not declare a claim a member also declares:
                // its account of that name IS the members' fold (`docs/42`
                // §3.4), and a declaration would either double it or hide it.
                for acct in &entity.accounts {
                    let full = format!("{}.{}", entity.symbol(), acct.name);
                    if let Some(members) = folded.get(&full) {
                        diagnostics.push(diag(
                            "E1383_FOLDED_ACCOUNT_DECLARED",
                            format!(
                                "Entity '{}' declares account '{}', which its members already carry ({}); the container's '{}' is their fold and is not declared.",
                                entity.symbol(),
                                acct.name,
                                members.join(", "),
                                acct.name
                            ),
                            acct.span,
                            file,
                            "Read the fold as `prev.<container>.<name>`; to carry a claim of the container's own, give it a different name.",
                        ));
                    }
                }
                for f in &entity.fields {
                    let ctx = format!("Field '{}.{}'", entity.symbol(), f.name);
                    for slot in std::iter::once(&f.init).chain(f.next.iter()) {
                        check_current_account_reads(
                            &slot.src,
                            &entity.symbol(),
                            &accounts,
                            slot.span,
                            file,
                            &ctx,
                            &mut diagnostics,
                        );
                    }
                }
            }
            _ => {}
        }
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

/// A balance is readable inside a period only as its opening, `prev.<name>`
/// (`docs/42` §3.3). A bare `asset.loan.balance` would name this period's
/// close, which does not exist yet, and would read as zero.
#[allow(clippy::too_many_arguments)]
fn check_current_account_reads(
    src: &str,
    owner: &str,
    accounts: &BTreeMap<String, DeclaredAccount>,
    span: cfdl_parser::Span,
    file: &str,
    ctx: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for full in accounts.keys() {
        // The owner's own claim may be spelled bare; every account may be
        // spelled in full. Either way the read must be through `prev.`.
        let mut spellings = vec![full.clone()];
        if let Some(short) = full.strip_prefix(&format!("{owner}.")) {
            if !short.contains('.') {
                spellings.push(short.to_string());
            }
        }
        for spelling in spellings {
            let mut rest = src;
            while let Some(idx) = rest.find(&spelling) {
                let before = rest[..idx].chars().next_back();
                let after = rest[idx + spelling.len()..].chars().next();
                let before_ok = before.is_none_or(|c| !c.is_alphanumeric() && c != '_' && c != '.');
                let after_ok = after.is_none_or(|c| !c.is_alphanumeric() && c != '_' && c != '.');
                let through_prev = rest[..idx].ends_with("prev.");
                let through_entity = rest[..idx].ends_with("prev.entity.");
                if before_ok && after_ok && !through_prev && !through_entity {
                    diagnostics.push(Diagnostic {
                        code: "E1382_ACCOUNT_READ_WITHOUT_PREV".to_string(),
                        severity: "error".to_string(),
                        message: format!(
                            "{ctx} reads account '{full}' as a current value. A balance is readable inside a period only as its opening: write `prev.{spelling}`."
                        ),
                        file: Some(file.to_string()),
                        span: Some(map_span(span)),
                        path: None,
                        hint: Some(
                            "The opening balance is the prior close — settled state. This period's close is the sum of the streams still being computed."
                                .to_string(),
                        ),
                        notes: Vec::new(),
                    });
                    return;
                }
                rest = &rest[idx + spelling.len()..];
            }
        }
    }
}

fn check_waterfalls(resolve_output: &cfdl_resolver::ResolveOutput) -> Result<(), Vec<Diagnostic>> {
    let entities: BTreeSet<String> = resolve_output
        .source_statements
        .iter()
        .filter_map(|s| match &s.statement {
            Stmt::Entity(entity) => Some(entity.symbol()),
            _ => None,
        })
        .collect();

    // WHICH WATERFALL PUBLISHED A STEP, AND WHEN. A step publishes as
    // `<waterfall>.<step>`, and a waterfall's steps become readable only once
    // that waterfall has finished — composition is declaration order, which is
    // what keeps an ordered allocation from becoming a dependency graph
    // (docs/17 §"Composition"). Naming a step of THIS waterfall, or of a later
    // one, therefore reads a series that exists in the model but cannot be seen
    // from here, and `series_aggregate` answers a plausible zero.
    let step_owner: BTreeMap<String, usize> = resolve_output
        .source_statements
        .iter()
        .filter_map(|s| match &s.statement {
            Stmt::Waterfall(w) => Some(w),
            _ => None,
        })
        .enumerate()
        .flat_map(|(order, w)| {
            w.steps
                .iter()
                .map(move |step| (format!("{}.{}", w.name, step.name), order))
        })
        .collect();

    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    // A STREAM CANNOT READ A WATERFALL STEP. docs/03 §3.2: a step's series is
    // visible to a later waterfall's `from` and to nothing else — steps
    // publish when their waterfall finishes, and every waterfall runs after
    // every stream. The engine's series store never holds a step, so this
    // read aggregated to zero in silence: `check_series_names` counted the
    // step as a known producer and stayed quiet, and no other check looked.
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Stream(stream) = &source_stmt.statement else {
            continue;
        };
        for slot in stream.amount.iter().chain(stream.active_when.iter()) {
            for referenced in cfdl_expr::series_references(&slot.src) {
                // A selector states that matching nothing is intended, and the
                // packs read whole families with them — same allowance E1342
                // makes.
                if referenced.ends_with(".*") {
                    continue;
                }
                if !step_owner.contains_key(&referenced) {
                    continue;
                }
                diagnostics.push(Diagnostic {
                    code: "E1346_STREAM_READS_WATERFALL_STEP".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "Stream '{}' reads series '{referenced}', which is a waterfall \
                         step. Steps publish when their waterfall finishes, and every \
                         waterfall runs after every stream — so this read could only \
                         ever aggregate to zero.",
                        stream.name
                    ),
                    file: Some(source_stmt.file.clone()),
                    span: Some(map_span(slot.span)),
                    path: None,
                    hint: Some(
                        "A step's series is visible to a later waterfall's `from` and to \
                         nothing else. Model the quantity the step pays as a stream or a \
                         field if a stream needs to read it."
                            .to_string(),
                    ),
                    notes: Vec::new(),
                });
            }
        }
    }

    let declared_accounts: std::collections::BTreeSet<String> =
        declared_accounts_of(resolve_output).into_keys().collect();

    let mut waterfall_order = 0usize;
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Waterfall(waterfall) = &source_stmt.statement else {
            continue;
        };
        let order = waterfall_order;
        waterfall_order += 1;
        let file = source_stmt.file.clone();
        let diag = |code: &str, message: String, span, hint: Option<String>| Diagnostic {
            code: code.to_string(),
            severity: "error".to_string(),
            message,
            file: Some(file.clone()),
            span: Some(map_span(span)),
            path: None,
            hint,
            notes: vec![],
        };

        if !entities.contains(&waterfall.attached_entity) {
            diagnostics.push(diag(
                "E1301_UNRESOLVED_ENTITY_REF",
                format!(
                    "Waterfall '{}' is declared on '{}', which is not a declared entity.",
                    waterfall.name, waterfall.attached_entity
                ),
                waterfall.span,
                None,
            ));
        }

        if waterfall.source.is_none() {
            diagnostics.push(diag(
                "E1340_WATERFALL_NO_SOURCE",
                format!(
                    "Waterfall '{}' declares no pot to allocate.",
                    waterfall.name
                ),
                waterfall.span,
                Some("Add `from <expr>` — the cash this waterfall distributes.".to_string()),
            ));
        }

        // WHEN a waterfall distributes is half of what it says. The schedule
        // decides whether cash accumulates and is then split, or is paid out
        // as it arrives — different deals, not different spellings — so there
        // is no default that is right often enough to be silent. The omission
        // used to lower to `on <time.start>`: one distribution, in the first
        // period, of whatever that period happened to produce.
        if waterfall.schedule.is_none() {
            diagnostics.push(diag(
                "E1348_WATERFALL_NO_SCHEDULE",
                format!(
                    "Waterfall '{}' does not say when it distributes.",
                    waterfall.name
                ),
                waterfall.span,
                Some(
                    "Add a `schedule` — `schedule on <date>` for a single distribution \
                     (an exit), `schedule every <period> from <date> to <date>` for a \
                     recurring one. Between its scheduled periods the pot accumulates."
                        .to_string(),
                ),
            ));
        }

        // A `series_sum` naming a step that is not yet published aggregates to
        // zero and says nothing, which is how a preferred return came to be paid
        // in full six times. The two sibling reads already fail loudly —
        // `E1341` for `paid.` naming a later step, the engine's cycle check for
        // a circular series read — so this one does too.
        let series_visibility = |src: &str, where_: String, span| {
            let mut out: Vec<Diagnostic> = Vec::new();
            for referenced in cfdl_expr::series_references(src) {
                // A selector states that matching nothing is intended.
                if referenced.ends_with(".*") {
                    continue;
                }
                let Some(&owner) = step_owner.get(&referenced) else {
                    continue;
                };
                // An EARLIER waterfall has finished and published: the
                // documented composition, and the reason this is an order.
                if owner < order {
                    continue;
                }
                let (why, hint) = if owner == order {
                    (
                        "which this waterfall has not finished paying".to_string(),
                        "A step is a pure function of the pot: accept, allocate, move on. \
                         Read an earlier step's payment this period with `paid.<step>`; \
                         for a running total, carry the quantity as a balance a field \
                         advances and the distribution moves."
                            .to_string(),
                    )
                } else {
                    (
                        "which a later waterfall publishes".to_string(),
                        "Waterfalls compose in declaration order, so a waterfall may read \
                         only ones declared before it. Declare them in the order the cash \
                         moves."
                            .to_string(),
                    )
                };
                out.push(diag(
                    "E1342_WATERFALL_SERIES_NOT_VISIBLE",
                    format!("{where_} reads series '{referenced}', {why}."),
                    span,
                    Some(hint),
                ));
            }
            out
        };

        if let Some(source) = &waterfall.source {
            diagnostics.extend(series_visibility(
                &source.src,
                format!("Waterfall '{}'", waterfall.name),
                waterfall.span,
            ));
        }

        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for (index, step) in waterfall.steps.iter().enumerate() {
            if !seen.insert(step.name.as_str()) {
                diagnostics.push(diag(
                    "E1343_WATERFALL_DUPLICATE_STEP",
                    format!(
                        "Waterfall '{}' declares two steps named '{}'.",
                        waterfall.name, step.name
                    ),
                    step.span,
                    Some(
                        "`paid()` and `owed()` name a step, so step names must be unique."
                            .to_string(),
                    ),
                ));
            }
            // AN ACCOUNT PAYEE RESOLVES AGAINST ACCOUNTS, not entities. An
            // account is not an entity and never was; `to account <name>` says
            // which namespace to look in, which is why the keyword is there.
            if step.to_account {
                if !declared_accounts.contains(&step.payee) {
                    diagnostics.push(diag(
                        "E1347_UNRESOLVED_ACCOUNT_REF",
                        format!(
                            "Waterfall '{}' step '{}' allocates to account '{}', which is not declared.",
                            waterfall.name, step.name, step.payee
                        ),
                        step.span,
                        Some(format!(
                            "Declare it as `account {} {{ }}`, or name a party instead.",
                            step.payee
                        )),
                    ));
                }
            } else if !entities.contains(&step.payee) {
                diagnostics.push(diag(
                    "E1301_UNRESOLVED_ENTITY_REF",
                    format!(
                        "Waterfall '{}' step '{}' pays '{}', which is not a declared entity.",
                        waterfall.name, step.name, step.payee
                    ),
                    step.span,
                    None,
                ));
            }
            let Some(amount) = &step.amount else {
                diagnostics.push(diag(
                    "E1345_WATERFALL_STEP_NO_AMOUNT",
                    format!(
                        "Waterfall '{}' step '{}' says nothing about what it pays.",
                        waterfall.name, step.name
                    ),
                    step.span,
                    Some("Write `= <expr>`. `= remaining` pays everything left.".to_string()),
                ));
                continue;
            };
            // `paid(s)` and `owed(s)` read what an EARLIER step did. Naming a
            // later step would be a forward reference into a value that does
            // not exist yet, which is the one way an ordered allocation could
            // become a dependency graph.
            for referenced in step_references(&amount.src) {
                if !seen.contains(referenced.as_str()) {
                    let earlier: Vec<&str> = seen.iter().copied().collect();
                    diagnostics.push(diag(
                        "E1341_WATERFALL_FORWARD_REF",
                        format!(
                            "Waterfall '{}' step '{}' reads step '{referenced}', which is not an earlier step.",
                            waterfall.name, step.name
                        ),
                        step.span,
                        Some(if earlier.len() <= 1 {
                            "A step may only read steps declared before it.".to_string()
                        } else {
                            format!("Earlier steps: {}.", earlier.join(", "))
                        }),
                    ));
                }
            }
            diagnostics.extend(series_visibility(
                &amount.src,
                format!("Waterfall '{}' step '{}'", waterfall.name, step.name),
                step.span,
            ));
            let _ = index;
        }

        // WHERE DOES WHAT IS LEFT GO? A waterfall that never reads `remaining`
        // allocates a fixed set of amounts and abandons the rest in silence.
        // Publishing the residue would hide it just as well; refusing to
        // compile is what makes the author say.
        let takes_remainder = waterfall
            .steps
            .iter()
            .filter_map(|s| s.amount.as_ref())
            .any(|a| mentions_remaining(&a.src));
        if !takes_remainder && !waterfall.steps.is_empty() {
            diagnostics.push(diag(
                "E1344_WATERFALL_NO_REMAINDER",
                format!(
                    "Waterfall '{}' never says where the remainder goes.",
                    waterfall.name
                ),
                waterfall.span,
                Some(
                    "End with `pay <name> to <payee> = remaining`. Without it, whatever survives the last step is lost silently."
                        .to_string(),
                ),
            ));
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

/// Step names an expression reads through `paid.<step>` or `owed.<step>`.
///
/// Dotted rather than a call, because every other binding in this language is
/// a path — `inputs.x`, `cfg.x`, `entity.state.x`, `prev.x`. A function would
/// have been a second way to say the same thing.
fn step_references(src: &str) -> Vec<String> {
    let mut found = Vec::new();
    for root in ["paid.", "owed."] {
        let mut rest = src;
        while let Some(at) = rest.find(root) {
            let before_ok = at == 0 || !is_name_byte(rest.as_bytes()[at - 1]);
            rest = &rest[at + root.len()..];
            if !before_ok {
                continue;
            }
            let end = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(rest.len());
            if end > 0 {
                found.push(rest[..end].to_string());
            }
        }
    }
    found
}

/// Whether an expression reads `remaining` as a binding rather than as part of
/// a longer name.
fn mentions_remaining(src: &str) -> bool {
    let bytes = src.as_bytes();
    let mut from = 0;
    while let Some(at) = src[from..].find("remaining") {
        let start = from + at;
        let end = start + "remaining".len();
        let before_ok = start == 0 || !is_name_byte(bytes[start - 1]);
        let after_ok = end == bytes.len() || !is_name_byte(bytes[end]);
        if before_ok && after_ok {
            return true;
        }
        from = end;
    }
    false
}

fn is_name_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'.'
}

fn check_exercise_targets(
    resolve_output: &cfdl_resolver::ResolveOutput,
) -> Result<(), Vec<Diagnostic>> {
    let declared: BTreeSet<&str> = resolve_output
        .source_statements
        .iter()
        .filter_map(|s| match &s.statement {
            Stmt::Option(option) => Some(option.name.as_str()),
            _ => None,
        })
        .collect();

    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Event(event) = &source_stmt.statement else {
            continue;
        };
        for action in &event.actions {
            let cfdl_parser::EventAction::ExerciseOption(name) = action else {
                continue;
            };
            if !declared.contains(name.as_str()) {
                let mut known: Vec<&str> = declared.iter().copied().collect();
                known.sort_unstable();
                diagnostics.push(Diagnostic {
                    code: "E1304_UNRESOLVED_OPTION_REF".to_string(),
                    severity: "error".to_string(),
                    message: format!("Event '{}' exercises unknown option '{name}'.", event.name),
                    file: Some(source_stmt.file.clone()),
                    span: Some(map_span(event.span)),
                    path: None,
                    hint: Some(if known.is_empty() {
                        "The model declares no options.".to_string()
                    } else {
                        format!("Declared options: {}.", known.join(", "))
                    }),
                    notes: vec![],
                });
            }
        }
    }
    // A LONGER CYCLE IS THE SAME MISTAKE AS SELF-PARENTING, and it matters more
    // now that a parent aggregates its children: a cycle would be an unbounded
    // walk rather than merely a nonsense.
    let parents: BTreeMap<String, (String, Option<Span>, String)> = resolve_output
        .source_statements
        .iter()
        .filter_map(|s| match &s.statement {
            Stmt::Entity(e) => e.parent.as_ref().map(|p| {
                (
                    e.symbol(),
                    (p.clone(), Some(map_span(e.span)), s.file.clone()),
                )
            }),
            _ => None,
        })
        .collect();
    for start in parents.keys() {
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        let mut cursor = start.as_str();
        let mut chain: Vec<&str> = vec![cursor];
        while let Some((parent, span, file)) = parents.get(cursor) {
            if !seen.insert(cursor) {
                break;
            }
            chain.push(parent.as_str());
            if parent == start {
                // One cycle, one diagnostic. Every member sees the same cycle,
                // so it is reported from its lexicographically first entity
                // rather than three times for one problem.
                if chain.iter().any(|member| *member < start.as_str()) {
                    break;
                }
                diagnostics.push(Diagnostic {
                    code: "E1318_ENTITY_HIERARCHY_CYCLE".to_string(),
                    severity: "error".to_string(),
                    message: format!("Entity hierarchy forms a cycle: {}.", chain.join(" -> ")),
                    file: Some(file.clone()),
                    span: span.clone(),
                    path: None,
                    hint: Some(
                        "An entity aggregates its children, so a cycle has no bottom to sum from."
                            .to_string(),
                    ),
                    notes: vec![],
                });
                break;
            }
            cursor = parent.as_str();
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

/// A contract's type, resolved ONCE from its declaration and carried to every
/// consumer — party checks, term checks, lowering, the IR (docs/40 §8; docs/13
/// §7.58). Before this the type was recovered at each consumer by stripping a
/// rule name off the front of the contract's name.
#[derive(Debug, Clone)]
struct ContractBinding {
    /// The pack contract type as a model names it — the rule name, `cre.lease_unit`.
    contract_name: String,
    /// The ontology type it is: `CRE.Contract.UnitLease`.
    type_id: String,
    /// The master at the root of its chain: `Contract.Lease`.
    master: String,
    /// The instance token, where the name carries one: `tenant_a`.
    instance: Option<String>,
}

/// Resolve a contract's type against the active ontology, or `None` where no
/// pack type claims it. The two-token form STATES the type (`contract
/// cre.lease_unit tenant_a`); the fused form is matched by rule-name prefix
/// on the same boundary lowering uses, so the two can never disagree. A stated
/// type that resolves to nothing is reported by `check_contract_types`, which
/// runs first; here it is simply unbound.
fn resolve_contract_binding(
    contract: &cfdl_parser::ContractStmt,
    ontology: &cfdl_pack::PackOntology,
) -> Option<ContractBinding> {
    let typed = match contract.declared_type.as_deref() {
        Some(stated) => ontology.contract_for_rule(stated)?,
        None => ontology
            .contracts
            .iter()
            .filter(|c| {
                c.contract_name
                    .as_deref()
                    .is_some_and(|rule| cfdl_pack::matches_contract_name(rule, &contract.name))
            })
            // `a.b` and `a.b_c` cannot both match one name (the boundary is
            // a dot), so this picks between a rule and a longer rule that
            // happens to share a prefix segment.
            .max_by_key(|c| c.contract_name.as_ref().map_or(0, |r| r.len()))?,
    };
    let rule_name = typed.contract_name.clone()?;
    let instance = contract.instance.clone().or_else(|| {
        contract
            .name
            .strip_prefix(rule_name.as_str())
            .map(|rest| rest.trim_start_matches('.').to_string())
            .filter(|rest| !rest.is_empty())
    });
    Some(ContractBinding {
        master: ontology
            .master_of(&typed.type_id)
            .unwrap_or_else(|| typed.type_id.clone()),
        type_id: typed.type_id.clone(),
        contract_name: rule_name,
        instance,
    })
}

/// The election types a model may write after `option ... type`: concrete
/// refinements of `Contract.Option`, the base's own included.
fn election_types(ontology: &cfdl_pack::PackOntology) -> Vec<&str> {
    let mut names: Vec<&str> = ontology
        .contracts
        .iter()
        .filter(|c| !c.is_abstract && ontology.is_a(&c.type_id, "Contract.Option"))
        .map(|c| c.type_id.as_str())
        .collect();
    names.sort_unstable();
    names
}

/// The pack contract types a model may declare with `contract`: the rule
/// names of every concrete, lowered type.
fn declarable_contract_names(ontology: &cfdl_pack::PackOntology) -> Vec<&str> {
    let mut names: Vec<&str> = ontology
        .contracts
        .iter()
        .filter(|c| !c.is_abstract)
        .filter_map(|c| c.contract_name.as_deref())
        .collect();
    names.sort_unstable();
    names
}

/// The concrete refinements of a master, as a model would name them.
fn concrete_refinements_of(ontology: &cfdl_pack::PackOntology, master: &str) -> Vec<String> {
    let mut names: Vec<String> = ontology
        .contracts
        .iter()
        .filter(|c| !c.is_abstract && c.type_id != master && ontology.is_a(&c.type_id, master))
        .map(|c| c.contract_name.clone().unwrap_or_else(|| c.type_id.clone()))
        .collect();
    names.sort_unstable();
    names
}

/// Why a contract's stated type does not resolve, in one place, so the
/// validation-phase restatement of `E2002` (the contract no rule lowers) and
/// the compile-phase check of a two-token declaration say the same thing.
/// `stated` is the type as the model wrote it: the two-token form's first
/// token, or the fused name's first two segments.
fn contract_type_refusal(
    contract_name: &str,
    stated: &str,
    instance: Option<&str>,
    ontology: &cfdl_pack::PackOntology,
    pack_name: Option<&str>,
) -> (&'static str, String, Option<String>) {
    let subject = format!("Contract '{contract_name}'");
    match ontology.contract(stated) {
        Some(typed) if typed.is_abstract => (
            "E1374_ABSTRACT_TYPE_INSTANTIATED",
            format!(
                "{subject} declares type '{stated}', which is a master. A master is refined, never declared: a model reaches it through a pack's concrete refinement."
            ),
            Some(format!(
                "Concrete refinements of '{stated}': {}.",
                join_or_none(&concrete_refinements_of(ontology, stated))
            )),
        ),
        Some(typed) if typed.contract_name.is_none() => (
            "E1373_UNKNOWN_CONTRACT_TYPE",
            format!(
                "{subject} declares type '{stated}', which is an election. An election is declared with `option <name> type {stated}`, not `contract`."
            ),
            None,
        ),
        Some(typed) => (
            "E1373_UNKNOWN_CONTRACT_TYPE",
            format!(
                "{subject} declares type '{stated}' by its ontology name. A contract is declared by the pack's name for the type."
            ),
            Some(format!(
                "Write `contract {} {}`.",
                typed.contract_name.as_deref().unwrap_or_default(),
                instance.unwrap_or("<instance>")
            )),
        ),
        None => {
            let known = declarable_contract_names(ontology);
            let near: Vec<String> = known
                .iter()
                .filter(|k| is_near_miss(k, stated))
                .map(|k| k.to_string())
                .collect();
            let hint = if !near.is_empty() {
                format!("Did you mean {}?", near.join(" or "))
            } else {
                match pack_name {
                    Some(pack) => format!(
                        "Contract types of pack '{pack}': {}.",
                        join_or_none(&known.iter().map(|k| k.to_string()).collect::<Vec<_>>())
                    ),
                    None => "No pack is active, so no contract type can be declared; `use pack \"<name>\"` first.".to_string(),
                }
            };
            (
                "E1373_UNKNOWN_CONTRACT_TYPE",
                format!(
                    "{subject} declares type '{stated}', which the active ontology does not define, so no rule lowers it."
                ),
                Some(hint),
            )
        }
    }
}

/// A TYPE NAMED ON A DECLARATION RESOLVES OR IS REFUSED (docs/40 §8).
///
/// `option <name> type <T>` always states its type, and the two-token contract
/// form states one too. Neither was checked before: an option's type resolved
/// against nothing (docs/13 §7.67), so a typo was silent, and a master named
/// where a concrete type belongs would have been accepted with no rule to
/// lower it. An unknown type is `E1373`; a master is `E1374`, with the
/// concrete refinements a model may declare named in the hint.
fn check_contract_types(
    resolve_output: &cfdl_resolver::ResolveOutput,
    ontology: &cfdl_pack::PackOntology,
    pack_name: Option<&str>,
) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let near_misses = |wanted: &str, known: &[&str]| -> Vec<String> {
        known
            .iter()
            .filter(|k| is_near_miss(k, wanted))
            .map(|k| k.to_string())
            .collect()
    };
    let diag =
        |code: &str, message: String, hint: Option<String>, file: &str, span: cfdl_parser::Span| {
            Diagnostic {
                code: code.to_string(),
                severity: "error".to_string(),
                message,
                file: Some(file.to_string()),
                span: Some(map_span(span)),
                path: None,
                hint,
                notes: vec![],
            }
        };
    for source_stmt in &resolve_output.source_statements {
        match &source_stmt.statement {
            Stmt::Contract(contract) => {
                let Some(stated) = contract.declared_type.as_deref() else {
                    continue;
                };
                if ontology.contract_for_rule(stated).is_some() {
                    continue;
                }
                let (code, message, hint) = contract_type_refusal(
                    &contract.name,
                    stated,
                    contract.instance.as_deref(),
                    ontology,
                    pack_name,
                );
                diagnostics.push(diag(
                    code,
                    message,
                    hint,
                    &source_stmt.file,
                    contract.declared_type_span.unwrap_or(contract.span),
                ));
            }
            Stmt::Option(option) => {
                let subject = format!("Option '{}'", option.name);
                let stated = option.type_name.as_str();
                match ontology.contract(stated) {
                    Some(typed) if typed.is_abstract => diagnostics.push(diag(
                        "E1374_ABSTRACT_TYPE_INSTANTIATED",
                        format!(
                            "{subject} declares type '{stated}', which is a master. A master is refined, never declared."
                        ),
                        Some(format!(
                            "Concrete elections: {}.",
                            join_or_none(&election_types(ontology).iter().map(|k| k.to_string()).collect::<Vec<_>>())
                        )),
                        &source_stmt.file,
                        option.span,
                    )),
                    Some(_) if !ontology.is_a(stated, "Contract.Option") => diagnostics.push(diag(
                        "E1373_UNKNOWN_CONTRACT_TYPE",
                        format!(
                            "{subject} declares type '{stated}', which is not an election — it lowers through a pack rule. Declare it with `contract`."
                        ),
                        Some(format!(
                            "Elections: {}.",
                            join_or_none(&election_types(ontology).iter().map(|k| k.to_string()).collect::<Vec<_>>())
                        )),
                        &source_stmt.file,
                        option.span,
                    )),
                    Some(_) => {}
                    None => {
                        let known = election_types(ontology);
                        let near = near_misses(stated, &known);
                        let hint = if near.is_empty() {
                            format!(
                                "Elections: {}.",
                                join_or_none(&known.iter().map(|k| k.to_string()).collect::<Vec<_>>())
                            )
                        } else {
                            format!("Did you mean {}?", near.join(" or "))
                        };
                        diagnostics.push(diag(
                            "E1373_UNKNOWN_CONTRACT_TYPE",
                            format!(
                                "{subject} declares type '{stated}', which the active ontology does not define."
                            ),
                            Some(hint),
                            &source_stmt.file,
                            option.span,
                        ));
                    }
                }
            }
            _ => {}
        }
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn join_or_none(names: &[String]) -> String {
    if names.is_empty() {
        "none".to_string()
    } else {
        names.join(", ")
    }
}

fn check_party_bindings(
    resolve_output: &cfdl_resolver::ResolveOutput,
    ontology: &cfdl_pack::PackOntology,
) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    // Which declared entities are parties. An untyped entity could be anything,
    // so it is accepted — the type is what enables the check.
    let party_types: BTreeSet<&str> = ontology
        .entities
        .iter()
        .filter(|e| e.family == "party")
        .map(|e| e.type_id.as_str())
        .collect();
    let mut entity_is_party: BTreeMap<String, Option<bool>> = BTreeMap::new();
    for source_stmt in &resolve_output.source_statements {
        if let Stmt::Entity(entity) = &source_stmt.statement {
            let known = entity
                .type_name
                .as_deref()
                .map(|t| party_types.contains(t) || t == "Party");
            entity_is_party.insert(entity.symbol(), known);
        }
    }

    let check = |type_name: &str,
                 parties: &[cfdl_parser::PartyBinding],
                 subject: &str,
                 file: &str,
                 diagnostics: &mut Vec<Diagnostic>| {
        // The roles the TYPE carries, its masters' included and the pack's
        // specializations applied (docs/40 §5): a CRE lease binds `landlord`,
        // and that is the master's `lessor`.
        let roles: Vec<cfdl_pack::EffectiveRole> = ontology.effective_roles(type_name);
        for binding in parties {
            match entity_is_party.get(&binding.entity) {
                None => diagnostics.push(Diagnostic {
                    code: "E1320_UNKNOWN_PARTY_ENTITY".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "{subject} binds role '{}' to '{}', which is not a declared entity.",
                        binding.role, binding.entity
                    ),
                    file: Some(file.to_string()),
                    span: Some(map_span(binding.span)),
                    path: None,
                    hint: None,
                    notes: vec![],
                }),
                Some(Some(false)) => diagnostics.push(Diagnostic {
                    code: "E1321_NOT_A_PARTY".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "{subject} binds role '{}' to '{}', which is an asset rather than a party.",
                        binding.role, binding.entity
                    ),
                    file: Some(file.to_string()),
                    span: Some(map_span(binding.span)),
                    path: None,
                    hint: Some("A contract is between parties.".to_string()),
                    notes: vec![],
                }),
                _ => {}
            }
            if roles.is_empty() {
                continue;
            }
            let describe = |r: &cfdl_pack::EffectiveRole| {
                if r.name == r.master {
                    r.name.clone()
                } else {
                    format!("{} (the master's {})", r.name, r.master)
                }
            };
            let bindable: Vec<String> = roles.iter().filter(|r| !r.unbound).map(describe).collect();
            match roles.iter().find(|r| r.name == binding.role) {
                Some(role) if role.unbound => diagnostics.push(Diagnostic {
                    code: "E1322_UNKNOWN_PARTY_ROLE".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "{subject} binds role '{}', which type '{type_name}' leaves unbound: the agreement has no such party in this form.",
                        binding.role
                    ),
                    file: Some(file.to_string()),
                    span: Some(map_span(binding.span)),
                    path: None,
                    hint: Some(format!("Roles a model binds: {}.", join_or_none(&bindable))),
                    notes: vec![],
                }),
                Some(_) => {}
                None => diagnostics.push(Diagnostic {
                    code: "E1322_UNKNOWN_PARTY_ROLE".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "{subject} binds role '{}', which type '{type_name}' does not declare.",
                        binding.role
                    ),
                    file: Some(file.to_string()),
                    span: Some(map_span(binding.span)),
                    path: None,
                    hint: Some(format!("Roles a model binds: {}.", join_or_none(&bindable))),
                    notes: vec![],
                }),
            }
        }
    };

    for source_stmt in &resolve_output.source_statements {
        match &source_stmt.statement {
            Stmt::Option(option) if !option.parties.is_empty() => {
                let subject = format!("Option '{}'", option.name);
                check(
                    &option.type_name,
                    &option.parties,
                    &subject,
                    &source_stmt.file,
                    &mut diagnostics,
                );
            }
            Stmt::Contract(contract) if !contract.parties.is_empty() => {
                let subject = format!("Contract '{}'", contract.name);
                let type_name = resolve_contract_binding(contract, ontology)
                    .map(|b| b.type_id)
                    .unwrap_or_default();
                check(
                    &type_name,
                    &contract.parties,
                    &subject,
                    &source_stmt.file,
                    &mut diagnostics,
                );
            }
            _ => {}
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn check_entity_types(
    resolve_output: &cfdl_resolver::ResolveOutput,
    ontology: &cfdl_pack::PackOntology,
) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let declared: BTreeSet<String> = resolve_output
        .source_statements
        .iter()
        .filter_map(|s| match &s.statement {
            Stmt::Entity(entity) => Some(entity.symbol()),
            _ => None,
        })
        .collect();

    // A STABLE IDENTITY IS A FACT ABOUT ONE THING (docs/13 §7.91). The
    // literal field `id` is engine-opaque and published in the results
    // graph; the one thing the language can check is that two entities do
    // not claim the same one, because a consumer joining on it would merge
    // two things into one.
    let mut seen_ids: BTreeMap<String, String> = BTreeMap::new();
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Entity(entity) = &source_stmt.statement else {
            continue;
        };
        let Some(id_field) = entity.literal_fields.iter().find(|f| f.name == "id") else {
            continue;
        };
        let value = id_field.value.trim().trim_matches('"').to_string();
        if let Some(holder) = seen_ids.get(&value) {
            diagnostics.push(Diagnostic {
                code: "E1360_DUPLICATE_ENTITY_ID".to_string(),
                severity: "error".to_string(),
                message: format!(
                    "Entity '{}' declares id \"{value}\", which '{holder}' already carries.",
                    entity.symbol()
                ),
                file: Some(source_stmt.file.clone()),
                span: Some(map_span(id_field.span)),
                path: None,
                hint: Some(
                    "An id names one thing for the layer above the model; a consumer joining on it would merge the two entities into one."
                        .to_string(),
                ),
                notes: vec![],
            });
        } else {
            seen_ids.insert(value, entity.symbol());
        }
    }

    for source_stmt in &resolve_output.source_statements {
        let Stmt::Entity(entity) = &source_stmt.statement else {
            continue;
        };
        let file = Some(source_stmt.file.clone());
        let span = Some(map_span(entity.span));

        let Some(type_name) = entity.type_name.as_deref() else {
            // Untyped entities stay legal: every model written before types
            // existed still compiles. The type is what unlocks the checks, not
            // a condition of being an entity.
            // A model-declared machine is what makes an untyped entity's
            // `state` checkable — the binding, not the type, declares the
            // set (`docs/28` §6.1). Validate checks the name against it.
            let states_checkable = entity.lifecycle.is_some();
            if entity.parent.is_some() || (entity.initial_state.is_some() && !states_checkable) {
                diagnostics.push(Diagnostic {
                    code: "E1310_ENTITY_BLOCK_WITHOUT_TYPE".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "Entity '{}' uses a block but declares no type. Add ': <Type>' so the block can be checked.",
                        entity.symbol()
                    ),
                    file: file.clone(),
                    span: span.clone(),
                    path: None,
                    hint: Some(
                        "An untyped entity has no declared fields, parent or states to check a block against."
                            .to_string(),
                    ),
                    notes: vec![],
                });
            }
            continue;
        };

        let Some(ty) = ontology.entity(type_name) else {
            let mut known: Vec<&str> = ontology
                .entities
                .iter()
                .map(|e| e.type_id.as_str())
                .collect();
            known.sort_unstable();
            diagnostics.push(Diagnostic {
                code: "E1311_UNKNOWN_ENTITY_TYPE".to_string(),
                severity: "error".to_string(),
                message: format!(
                    "Entity '{}' declares type '{type_name}', which the active ontology does not define.",
                    entity.symbol()
                ),
                file: file.clone(),
                span: span.clone(),
                path: None,
                hint: Some(format!("Known types: {}.", known.join(", "))),
                notes: vec![],
            });
            continue;
        };

        // A required field is required because the type cannot be underwritten
        // without it. EFFECTIVE fields — the masters' included (docs/13
        // §7.92): a field learned from `Asset.Real` holds on everything that
        // is one.
        let effective = ontology.effective_fields(type_name);
        let given: BTreeSet<&str> = entity
            .literal_fields
            .iter()
            .map(|a| a.name.as_str())
            .collect();
        for field in effective.iter().filter(|f| f.required) {
            if !given.contains(field.name.as_str()) {
                diagnostics.push(Diagnostic {
                    code: "E1312_MISSING_REQUIRED_FIELD".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "Entity '{}' of type '{type_name}' is missing required field '{}'.",
                        entity.symbol(),
                        field.name
                    ),
                    file: file.clone(),
                    span: span.clone(),
                    path: None,
                    hint: field.description.clone().or_else(|| {
                        field
                            .unit
                            .as_ref()
                            .map(|unit| format!("Expected {} in {unit}.", field.field_type))
                    }),
                    notes: vec![],
                });
            }
        }
        // A PACK DECLARES A FLOOR, NOT A CEILING. Required fields must be
        // present and declared ones must be spelled right, but a modeller may
        // add fields of their own — that is how a model says something the
        // pack's vocabulary does not cover, and it is already true of fields
        // that carry a rule.
        //
        // What still fails is a NEAR MISS. `senority` next to a declared
        // `seniority` is a typo, and allowing it would make the value a field
        // nobody reads — the quiet kind of wrong this project keeps closing.
        for attr in &entity.literal_fields {
            let declared = effective.iter().any(|f| f.name == attr.name);
            let near_miss =
                !declared && effective.iter().any(|f| is_near_miss(&f.name, &attr.name));
            if near_miss {
                let mut known: Vec<&str> = effective.iter().map(|f| f.name.as_str()).collect();
                known.sort_unstable();
                diagnostics.push(Diagnostic {
                    code: "E1313_UNKNOWN_ENTITY_FIELD".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "Entity '{}' of type '{type_name}' sets '{}', which the type does not declare.",
                        entity.symbol(),
                        attr.name
                    ),
                    file: file.clone(),
                    span: Some(map_span(attr.span)),
                    path: None,
                    hint: Some(if known.is_empty() {
                        format!("'{type_name}' declares no fields.")
                    } else {
                        format!("Declared fields: {}.", known.join(", "))
                    }),
                    notes: vec![],
                });
            }
        }

        // Hierarchy is optional, but a parent that does not exist is a typo,
        // not a choice of grain.
        if let Some(parent) = &entity.parent {
            if !declared.contains(parent) {
                diagnostics.push(Diagnostic {
                    code: "E1314_UNKNOWN_PARENT_ENTITY".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "Entity '{}' is part of '{parent}', which is not declared.",
                        entity.symbol()
                    ),
                    file: file.clone(),
                    span: span.clone(),
                    path: None,
                    hint: Some("Hierarchy is optional; a declared parent is not.".to_string()),
                    notes: vec![],
                });
            } else if parent == &entity.symbol() {
                diagnostics.push(Diagnostic {
                    code: "E1315_ENTITY_PART_OF_ITSELF".to_string(),
                    severity: "error".to_string(),
                    message: format!("Entity '{}' is part of itself.", entity.symbol()),
                    file: file.clone(),
                    span: span.clone(),
                    path: None,
                    hint: None,
                    notes: vec![],
                });
            }
        }

        // ONE MACHINE PER ENTITY. A type that declares a lifecycle and a
        // model binding another would leave two authorities over one status.
        if entity.lifecycle.is_some() {
            if let Some(lifecycle_id) = ty.lifecycle.as_deref() {
                diagnostics.push(Diagnostic {
                    code: "E1350_LIFECYCLE_CONFLICT".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "Entity '{}' binds a model-declared lifecycle, but its type '{type_name}' already declares lifecycle '{lifecycle_id}'. One machine per entity — drop the binding, or use an untyped entity.",
                        entity.symbol()
                    ),
                    file: file.clone(),
                    span: span.clone(),
                    path: None,
                    hint: None,
                    notes: vec![],
                });
            }
        }

        // The state space is declared so that a misspelled status is
        // impossible rather than merely unlikely.
        match (&entity.initial_state, ty.lifecycle.as_deref()) {
            (Some(state), Some(lifecycle_id)) => {
                if let Some(lifecycle) = ontology.lifecycle(lifecycle_id) {
                    if !lifecycle.has_state(state) {
                        diagnostics.push(Diagnostic {
                            code: "E1316_UNKNOWN_LIFECYCLE_STATE".to_string(),
                            severity: "error".to_string(),
                            message: format!(
                                "Entity '{}' starts in state '{state}', which lifecycle '{lifecycle_id}' does not declare.",
                                entity.symbol()
                            ),
                            file: file.clone(),
                            span: span.clone(),
                            path: None,
                            hint: Some(format!("Declared states: {}.", lifecycle.states.join(", "))),
                            notes: vec![],
                        });
                    }
                }
            }
            (Some(state), None) => {
                diagnostics.push(Diagnostic {
                    code: "E1317_TYPE_HAS_NO_LIFECYCLE".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "Entity '{}' starts in state '{state}', but type '{type_name}' declares no lifecycle.",
                        entity.symbol()
                    ),
                    file: file.clone(),
                    span: span.clone(),
                    path: None,
                    hint: None,
                    notes: vec![],
                });
            }
            _ => {}
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
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

    // What the model may be about. With a pack, its vocabulary over the
    // language's; with no pack, the language's alone — because an ontology is a
    // LANGUAGE capability that packs supply defaults for, not one they own.
    // This is the same argument the category vocabulary already makes: refusing
    // it when no pack is active is circular, since nothing reads it only so
    // long as nothing may declare it.
    let ontology = active_pack
        .map(|pack| pack.ontology.clone())
        .unwrap_or_else(cfdl_pack::PackOntology::language_base);
    check_entity_types(resolve_output, &ontology)?;
    check_contract_types(
        resolve_output,
        &ontology,
        active_pack.map(|p| p.name.as_str()),
    )?;
    check_party_bindings(resolve_output, &ontology)?;
    check_exercise_targets(resolve_output)?;
    check_waterfalls(resolve_output)?;
    check_stream_moves(resolve_output)?;
    check_state_guards(resolve_output, &ontology)?;
    check_prev_first_period(resolve_output, &time_start)?;
    check_constant_expressions(resolve_output)?;
    check_participant_returns(resolve_output)?;
    let (machines, machines_by_entity) = resolve_machines(resolve_output, &ontology);
    check_lifecycle_augmentations(resolve_output, &machines)?;
    check_status_writes(resolve_output, &machines, &machines_by_entity)?;

    let mut entities: Vec<((String, String), IrEntity)> = resolve_output
        .source_statements
        .iter()
        .filter_map(|source_stmt| {
            let Stmt::Entity(entity) = &source_stmt.statement else {
                return None;
            };
            let symbol = entity.symbol();
            let stable_key = stable_key(&source_stmt.file, &symbol);
            // An untyped entity keeps `core.Entity`, so every model written
            // before types existed lowers exactly as it did.
            let type_name = entity
                .type_name
                .clone()
                .unwrap_or_else(|| "core.Entity".to_string());
            let fields = entity
                .literal_fields
                .iter()
                .map(|attr| {
                    (
                        attr.name.clone(),
                        serde_json::Value::String(attr.value.clone()),
                    )
                })
                .collect();
            // A field with no `next` HOLDS, which is the recurrence
            // `next prev` — so the absent rule is written out rather than
            // special-cased downstream.
            let declared = declared_accounts_of(resolve_output);
            let rules = entity
                .fields
                .iter()
                .map(|f| {
                    (
                        f.name.clone(),
                        IrFieldRule {
                            init: IrExpr {
                                lang: f.init.lang.clone(),
                                src: rewrite_prev_accounts(&f.init.src, &symbol, &declared),
                            },
                            schedule: None,
                            next: f.next.as_ref().map_or_else(
                                || IrExpr {
                                    lang: "cfdl".to_string(),
                                    src: "prev".to_string(),
                                },
                                |n| IrExpr {
                                    lang: n.lang.clone(),
                                    src: rewrite_prev_accounts(&n.src, &symbol, &declared),
                                },
                            ),
                        },
                    )
                })
                .collect();
            let ir_entity = IrEntity {
                id: deterministic_id("Entity", &stable_key, &id_seed),
                symbol: symbol.clone(),
                r#type: type_name,
                fields,
                rules,
                field_roles: BTreeMap::new(),
                state: BTreeMap::new(),
                parent: entity.parent.clone(),
                initial_state: entity.initial_state.clone(),
                lifecycle: machines_by_entity.get(&symbol).cloned(),
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
            let binding = resolve_contract_binding(contract, &ontology);
            let roles = binding
                .as_ref()
                .map(|b| ontology.effective_roles(&b.type_id))
                .unwrap_or_default();
            let parties = contract
                .parties
                .iter()
                .map(|p| IrPartyBinding {
                    role: p.role.clone(),
                    master_role: roles
                        .iter()
                        .find(|r| r.name == p.role)
                        .map(|r| r.master.clone()),
                    entity: IrEntityRef {
                        symbol: p.entity.clone(),
                    },
                })
                .collect();
            let ir_contract = IrContract {
                id: deterministic_id("Contract", &stable_key, &id_seed),
                name: name.clone(),
                r#type: binding
                    .as_ref()
                    .map(|b| b.type_id.clone())
                    .unwrap_or_else(|| "core.Contract".to_string()),
                contract_name: binding.as_ref().map(|b| b.contract_name.clone()),
                master: binding.as_ref().map(|b| b.master.clone()),
                instance: binding.as_ref().and_then(|b| b.instance.clone()),
                subject: IrEntityRef {
                    symbol: contract
                        .subject_entity
                        .clone()
                        .unwrap_or_else(|| first_entity_symbol.clone()),
                },
                parties,
                term: IrDateRange {
                    start: time_start.clone(),
                    end: timeline_end.clone(),
                },
                currency: model_currency.clone(),
                terms: contract
                    .terms
                    .iter()
                    .map(|(key, term)| (key.clone(), term_value_json(term)))
                    .collect(),
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
    //
    // ONE RULE, PACK OR NO PACK: any well-formed path rooted in `operating`,
    // `investing` or `financing` (`cfdl_pack::CATEGORY_ROOTS`).
    //
    // A pack used to narrow it. Its `categories = [...]` was a closed list that
    // REPLACED the language's rule, so `investing.acquisition.purchase` was valid with no
    // pack and E5022 with one — a language mechanism and a pack mechanism for a
    // single concept. IAS 7 settles the roots and stops there; the IFRS
    // Accounting Taxonomy carries no second level to borrow, and the leaf a
    // given deal needs is not knowable by a pack that shipped before it. A
    // hotel wants `operating.expense.rooms`; no CRE pack list will ever contain
    // every such leaf. See docs/35.
    //
    // The list survives as RECOMMENDED vocabulary rather than a gate: a
    // well-rooted category the pack does not list is valid, and `W5023` names
    // the near match at run time. That keeps the spelling protection the closed
    // list gave without making the pack the authority on what a deal may say.
    let pack_active = active_pack.is_some();

    // A contract may override the category its rule assigns. Same rule as a
    // stream's: rooted in one of the three activities, no empty segment.
    // Checked once per contract rather than once per lowered stream, so a
    // contract emitting six streams reports one diagnostic.
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Contract(contract) = &source_stmt.statement else {
            continue;
        };
        // The bare form flattens, so it is only safe where there is nothing to
        // flatten. A contract lowering several streams has a category per
        // stream — the pack states each one — and one clause that cannot say
        // which it means would set all of them to the same value, silently.
        // That is exactly what it did before `category <stream> = <path>`
        // existed: a permanent mortgage's interest, principal and proceeds all
        // became `financing.debt.interest_paid`, and coverage computed off the result was
        // wrong with nothing to show for it.
        let emitted = active_pack
            .map(|pack| pack.lowering_rules.as_slice())
            .unwrap_or(&[])
            .iter()
            .filter(|rule| rule_matches_contract(&rule.contract_name, &contract.name))
            .filter(|rule| !rule.stream_name.is_empty())
            .count();
        if contract.category.is_some() && emitted > 1 {
            return Err(vec![Diagnostic {
                code: "E5030_AMBIGUOUS_CONTRACT_CATEGORY".to_string(),
                severity: "error".to_string(),
                message: format!(
                    "Contract '{}' states one category, and lowers {emitted} streams. Each \
                     carries its own category, so one clause cannot say which it reclassifies.",
                    contract.name
                ),
                file: Some(source_stmt.file.clone()),
                span: Some(map_span(contract.span)),
                path: None,
                hint: Some(
                    "Name the stream: `category <stream> = <path>`, once per stream you mean \
                     to reclassify. The bare form is for a contract that lowers exactly one."
                        .to_string(),
                ),
                notes: vec![],
            }]);
        }
        let Some(category) = contract.category.as_deref() else {
            continue;
        };
        let root = category.split('.').next().unwrap_or("");
        if !cfdl_pack::CATEGORY_ROOTS.contains(&root)
            || category.split('.').any(|seg| seg.is_empty())
        {
            return Err(vec![Diagnostic {
                code: "E5022_UNKNOWN_STREAM_CATEGORY".to_string(),
                severity: "error".to_string(),
                message: format!(
                    "Contract '{}' declares category '{category}', whose root segment \
                     '{root}' is not one of {}. A category is a path into the cash flow \
                     statement, so it has to say which section it belongs to.",
                    contract.name,
                    cfdl_pack::CATEGORY_ROOTS.join(", ")
                ),
                file: Some(source_stmt.file.clone()),
                span: Some(map_span(contract.span)),
                path: None,
                hint: Some(
                    "A contract's `category` overrides the one its lowering rule assigns, \
                     for the leaf a pack could not have enumerated. It is validated like \
                     any other — for example `category operating.expense.rooms`."
                        .to_string(),
                ),
                notes: vec![],
            }]);
        }
    }

    let mut streams: Vec<((String, String), IrStream)> = Vec::new();
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Stream(stream) = &source_stmt.statement else {
            continue;
        };
        match stream.category.as_deref() {
            Some(category) => {
                let root = category.split('.').next().unwrap_or("");
                let well_formed = cfdl_pack::CATEGORY_ROOTS.contains(&root)
                    && !category.split('.').any(|seg| seg.is_empty());
                if !well_formed {
                    return Err(vec![Diagnostic {
                        code: "E5022_UNKNOWN_STREAM_CATEGORY".to_string(),
                        severity: "error".to_string(),
                        message: format!(
                            "Stream '{}' declares category '{category}', whose root segment \
                             '{root}' is not one of {}. A category is a path into the cash \
                             flow statement, so it has to say which section it belongs to.",
                            stream.name,
                            cfdl_pack::CATEGORY_ROOTS.join(", ")
                        ),
                        file: Some(source_stmt.file.clone()),
                        span: Some(map_span(stream.span)),
                        path: None,
                        hint: Some(
                            "Any dotted path rooted in operating, investing or financing is \
                             valid, with or without a pack — for example \
                             `operating.revenue.rent`. A pack's category list is a \
                             recommendation, not a gate."
                                .to_string(),
                        ),
                        notes: vec![],
                    }]);
                }
            }
            // A stream with no category is invisible to every fold: its cash
            // reaches `model.total` and the entity roll-up, and lands in no
            // subtotal at all. Without a pack nothing folds, so saying nothing
            // is honest. With one, the pack exists precisely to aggregate this
            // cash, and there is always a right answer available — a flow that
            // does not belong in net operating income takes a different root.
            // Silence here is worth money: a coverage ratio computed over a
            // stream that quietly sat outside it is wrong and says so nowhere.
            None if pack_active => {
                return Err(vec![Diagnostic {
                    code: "E5029_STREAM_MISSING_CATEGORY".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "Stream '{}' declares no category, and pack '{}' is active. Its cash \
                         would reach model.total and fold into no subtotal — invisible to \
                         every domain metric, silently.",
                        stream.name,
                        active_pack.map(|p| p.name.as_str()).unwrap_or("")
                    ),
                    file: Some(source_stmt.file.clone()),
                    span: Some(map_span(stream.span)),
                    path: None,
                    hint: Some(
                        "State what the flow IS, as a path into the cash flow statement: \
                         `category operating.revenue.rent`, `category financing.debt.interest_paid`. \
                         A category is only optional when no pack is active, because then \
                         nothing folds."
                            .to_string(),
                    ),
                    notes: vec![],
                }]);
            }
            None => {}
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
        let declared = declared_accounts_of(resolve_output);
        let mut ir_stream = IrStream {
            id: deterministic_id("Stream", &stable_key, &id_seed),
            name: stream.name.clone(),
            owner: IrEntityRef {
                symbol: stream.attached_entity.clone(),
            },
            category: stream.category.clone(),
            moves: stream
                .moves
                .as_deref()
                .and_then(|m| resolve_moved_account(&stream.attached_entity, m, &declared)),
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
                // `active in state a, b` lowers to the comparison it means. The
                // form exists so the state NAME is checked against the owner's
                // lifecycle — a string comparison cannot be, and
                // `entity.state.status != "refinancd"` is true forever.
                src: if !stream.active_in_states.is_empty() {
                    state_guard_expr(&stream.active_in_states)
                } else {
                    stream
                        .active_when
                        .as_ref()
                        .map(|expr| expr.src.clone())
                        .unwrap_or_else(|| "true".to_string())
                },
            },
            provenance: IrNodeProvenance {
                source_file: source_stmt.file.clone(),
                source_span: map_span(stream.span),
                generated_by: None,
            },
        };
        // `prev.balance` on the owning entity is its own claim, spelled out
        // for the engine as `prev.<entity>.balance` (`docs/42` §7).
        ir_stream.amount.src =
            rewrite_prev_accounts(&ir_stream.amount.src, &stream.attached_entity, &declared);
        ir_stream.active_when.src = rewrite_prev_accounts(
            &ir_stream.active_when.src,
            &stream.attached_entity,
            &declared,
        );
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
    // After lowering, so a field a pack rule hangs on an entity is a name a
    // model expression may read, and a role its rules fill is one an arrival
    // action may name.
    check_field_paths(resolve_output, &lowered.fields)?;
    check_arrival_action_fields(
        resolve_output,
        &machines,
        &machines_by_entity,
        &ontology,
        &lowered.field_roles,
    )?;
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
    check_event_stream_targets(resolve_output, &streams, &lowered.streams)?;
    check_lowered_prev_first_period(&lowered.streams, &lowered.stream_inputs, &time_start)?;
    // A LOWERING RULE'S FIELD HANGS ON THE ENTITY IT DESCRIBES.
    //
    // The rule already names its owner — `owner_entity = "${subject}"` — so no
    // new pack vocabulary is needed. Two contracts on different entities may
    // carry the same field name without colliding, which is what made the
    // `{{contract.suffix_ident}}` discriminator necessary while these were
    // global model states.
    let lowered_fields = lowered.fields;
    for ((owner, field), rule) in &lowered_fields {
        if let Some((_, entity)) = entities.iter_mut().find(|(_, e)| &e.symbol == owner) {
            entity.rules.insert(field.clone(), rule.clone());
        }
    }
    for ((owner, role), fields) in &lowered.field_roles {
        if let Some((_, entity)) = entities.iter_mut().find(|(_, e)| &e.symbol == owner) {
            entity.field_roles.insert(role.clone(), fields.clone());
        }
    }
    let stream_inputs = lowered.stream_inputs;
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
    let (ir_quantiles, quantile_diags) = lower_quantiles(resolve_output);
    if !quantile_diags.is_empty() {
        let mut diagnostics = quantile_diags;
        sort_compile_diagnostics(&mut diagnostics);
        return Err(diagnostics);
    }

    let (ir_subtotals, subtotal_diags) = lower_subtotals(active_pack);
    if !subtotal_diags.is_empty() {
        let mut diagnostics = subtotal_diags;
        sort_compile_diagnostics(&mut diagnostics);
        return Err(diagnostics);
    }

    let (ir_events, ir_options, event_diags) = lower_events_options(
        resolve_output,
        &id_seed,
        &time_calendar,
        &time_start,
        &timeline_end,
        &phase_map,
    );
    if !event_diags.is_empty() {
        let mut diagnostics = event_diags;
        sort_compile_diagnostics(&mut diagnostics);
        return Err(diagnostics);
    }

    let mut sources = resolve_output.module_order.clone();
    sources.sort();

    // A pack no longer contributes model-level state: its rules hang fields on
    // the entities they describe, folded into the entity map below.

    let mut ir_accounts: Vec<IrAccount> = Vec::new();
    for source_stmt in &resolve_output.source_statements {
        match &source_stmt.statement {
            Stmt::Account(a) => ir_accounts.push(IrAccount {
                name: a.name.clone(),
                owner: a.owner.clone(),
                owner_entity: a.owner_entity.clone(),
                side: a.side.clone(),
                init: a.init.as_ref().map(|slot| IrExpr {
                    lang: "cfdl".to_string(),
                    src: slot.src.clone(),
                }),
                inflow: a.inflow.as_ref().map(|slot| IrExpr {
                    lang: "cfdl".to_string(),
                    src: slot.src.clone(),
                }),
                fold: false,
            }),
            // A claim declared in an entity block: named by its owner, so
            // two loans' balances never collide (`docs/42` §3.5).
            Stmt::Entity(entity) => {
                for acct in &entity.accounts {
                    ir_accounts.push(IrAccount {
                        name: format!("{}.{}", entity.symbol(), acct.name),
                        owner: None,
                        owner_entity: Some(entity.symbol()),
                        side: acct.side.clone(),
                        init: acct.init.as_ref().map(|slot| IrExpr {
                            lang: "cfdl".to_string(),
                            src: slot.src.clone(),
                        }),
                        inflow: None,
                        fold: false,
                    });
                }
            }
            _ => {}
        }
    }
    // THE RELATION FOLD (`docs/42` §3.4): every ancestor of an entity that
    // declares a claim carries the sum of its members' claims of that name,
    // as a published account with nothing of its own.
    for full in folded_accounts_of(resolve_output).keys() {
        ir_accounts.push(IrAccount {
            name: full.clone(),
            owner: None,
            owner_entity: full.rsplit_once('.').map(|(owner, _)| owner.to_string()),
            side: None,
            init: None,
            inflow: None,
            fold: true,
        });
    }

    // Only machines an entity actually binds are published: an unbound
    // machine governs nothing, and the IR is a record of this model.
    // Every field role any master names, so an arrival action naming one is
    // emitted as a role (docs/40 §3, stage 6).
    let declared_field_roles: BTreeSet<String> = ontology
        .contracts
        .iter()
        .flat_map(|c| c.field_roles.iter().map(|r| r.name.clone()))
        .collect();
    let ir_lifecycles: Vec<IrLifecycle> = {
        let bound: std::collections::BTreeSet<&String> = machines_by_entity.values().collect();
        machines
            .values()
            .filter(|m| bound.contains(&m.id))
            .map(|m| IrLifecycle {
                id: m.id.clone(),
                initial: m.initial.clone().unwrap_or_default(),
                states: m.states.clone(),
                edges: m
                    .edges
                    .iter()
                    .map(|e| IrLifecycleEdge {
                        from: e.from.clone(),
                        to: e.to.clone(),
                        guard: e.guard.as_ref().map(|src| IrExpr {
                            lang: "cfdl".to_string(),
                            src: coerce_numeric_literals(src),
                        }),
                        actions: e
                            .actions
                            .iter()
                            .map(|a| ir_state_action(a, &declared_field_roles))
                            .collect(),
                    })
                    .collect(),
                // BTreeMap iteration is by state name, which makes the IR
                // byte-stable across runs; within a state the vector order is
                // the resolution order — pack's actions, then the model's.
                entry_actions: m
                    .entry_actions
                    .iter()
                    .map(|(state, actions)| IrStateEntry {
                        state: state.clone(),
                        actions: actions
                            .iter()
                            .map(|a| ir_state_action(a, &declared_field_roles))
                            .collect(),
                    })
                    .collect(),
            })
            .collect()
    };

    // DECLARED METRICS, in declaration order (`docs/13` §7.25).
    //
    // A metric may read the metrics above it — the same rule waterfalls
    // already follow, which makes the dependency an order rather than a graph.
    // A forward or circular reference is refused here, so the engine's fold
    // always finds a value already computed.
    let mut ir_metrics: Vec<IrMetric> = Vec::new();
    // Where each metric was written, kept so the check after IR assembly can
    // point at the declaration rather than at the document.
    let mut metric_spans: BTreeMap<String, (String, cfdl_parser::Span)> = BTreeMap::new();
    {
        let mut declared_above: BTreeSet<&str> = BTreeSet::new();
        let mut metric_diagnostics: Vec<Diagnostic> = Vec::new();
        for source_stmt in &resolve_output.source_statements {
            let Stmt::Metric(metric) = &source_stmt.statement else {
                continue;
            };
            for referenced in cfdl_expr::root_references(&metric.expr, "metric") {
                let name = referenced.trim_start_matches("metric.");
                if declared_above.contains(name) {
                    continue;
                }
                let (why, hint) = if name == metric.name {
                    (
                        "itself".to_string(),
                        "A metric is a fold over the finished projection, not a recurrence; \
                         carry a running quantity as a field the walk advances."
                            .to_string(),
                    )
                } else {
                    (
                        format!("'{name}', which is declared below it or not at all"),
                        "Metrics compose in declaration order, so a metric may read the \
                         metrics above it. Move the declaration up, or correct the name."
                            .to_string(),
                    )
                };
                metric_diagnostics.push(Diagnostic {
                    code: "E1354_METRIC_FORWARD_REF".to_string(),
                    severity: "error".to_string(),
                    message: format!("Metric '{}' reads {why}.", metric.name),
                    file: Some(source_stmt.file.clone()),
                    span: Some(map_span(metric.span)),
                    path: None,
                    hint: Some(hint),
                    notes: vec![],
                });
            }
            declared_above.insert(metric.name.as_str());
            metric_spans.insert(metric.name.clone(), (source_stmt.file.clone(), metric.span));
            ir_metrics.push(IrMetric {
                name: metric.name.clone(),
                expr: IrExpr {
                    lang: "cfdl".to_string(),
                    src: coerce_numeric_literals(&metric.expr),
                },
                provenance: IrNodeProvenance {
                    source_file: source_stmt.file.clone(),
                    source_span: map_span(metric.span),
                    generated_by: None,
                },
            });
        }
        if !metric_diagnostics.is_empty() {
            sort_compile_diagnostics(&mut metric_diagnostics);
            return Err(metric_diagnostics);
        }
    }

    // A STEP THAT NAMES AN AGREEMENT NAMES ONE THIS MODEL DECLARES, and a line
    // that agreement's type declares ALLOCATED (docs/40 §6): a step paying a
    // security's `principal` is the mechanism the master describes; a step
    // claiming to pay a line a rule lowers would count the cash twice.
    let declared_contracts: BTreeMap<String, &cfdl_parser::ContractStmt> = resolve_output
        .source_statements
        .iter()
        .filter_map(|s| match &s.statement {
            Stmt::Contract(c) => Some((c.name.clone(), c)),
            _ => None,
        })
        .collect();
    let mut step_diagnostics: Vec<Diagnostic> = Vec::new();
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Waterfall(w) = &source_stmt.statement else {
            continue;
        };
        for step in &w.steps {
            let (Some(contract_name), Some(line)) =
                (step.contract.as_deref(), step.line.as_deref())
            else {
                continue;
            };
            let subject = format!("Waterfall '{}' step '{}'", w.name, step.name);
            let Some(contract) = declared_contracts.get(contract_name) else {
                let mut known: Vec<&str> = declared_contracts.keys().map(|k| k.as_str()).collect();
                known.sort_unstable();
                let near: Vec<&str> = known
                    .iter()
                    .copied()
                    .filter(|k| is_near_miss(k, contract_name))
                    .collect();
                step_diagnostics.push(Diagnostic {
                    code: "E1376_UNKNOWN_REFERENCE".to_string(),
                    severity: "error".to_string(),
                    message: format!("{subject} pays for contract '{contract_name}', which this model does not declare."),
                    file: Some(source_stmt.file.clone()),
                    span: Some(map_span(step.span)),
                    path: None,
                    hint: Some(if near.is_empty() {
                        format!("Declared contracts: {}.", join_or_none(&known.iter().map(|k| k.to_string()).collect::<Vec<_>>()))
                    } else {
                        format!("Did you mean {}?", near.join(" or "))
                    }),
                    notes: vec![],
                });
                continue;
            };
            let Some(binding) = resolve_contract_binding(contract, &ontology) else {
                continue;
            };
            let lines = ontology.effective_lines(&binding.type_id);
            let allocated: Vec<&str> = lines
                .iter()
                .filter(|l| l.allocated)
                .map(|l| l.name.as_str())
                .collect();
            let ok = lines.iter().any(|l| l.name == line && l.allocated);
            if !ok {
                step_diagnostics.push(Diagnostic {
                    code: "E1377_STEP_LINE_NOT_ALLOCATED".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "{subject} pays line '{line}' of contract '{contract_name}', which type '{}' does not declare as allocated. A step pays what the structure allocates; a line a rule lowers is paid by the rule.",
                        binding.type_id
                    ),
                    file: Some(source_stmt.file.clone()),
                    span: Some(map_span(step.span)),
                    path: None,
                    hint: Some(format!("Allocated lines of '{}': {}.", binding.type_id, join_or_none(&allocated.iter().map(|l| l.to_string()).collect::<Vec<_>>()))),
                    notes: vec![],
                });
            }
        }
    }
    if !step_diagnostics.is_empty() {
        sort_compile_diagnostics(&mut step_diagnostics);
        return Err(step_diagnostics);
    }

    let ir_waterfalls: Vec<IrWaterfall> = resolve_output
        .source_statements
        .iter()
        .filter_map(|source_stmt| match &source_stmt.statement {
            Stmt::Waterfall(w) => Some(IrWaterfall {
                name: w.name.clone(),
                entity: w.attached_entity.clone(),
                schedule: lower_schedule(
                    w.schedule.as_ref(),
                    &time_calendar,
                    &time_start,
                    &timeline_end,
                    &phase_map,
                )
                .ok(),
                source: IrExpr {
                    lang: "cfdl".to_string(),
                    src: w
                        .source
                        .as_ref()
                        .map(|e| coerce_numeric_literals(&e.src))
                        .unwrap_or_else(|| "0".to_string()),
                },
                steps: w
                    .steps
                    .iter()
                    .map(|step| IrWaterfallStep {
                        name: step.name.clone(),
                        payee: step.payee.clone(),
                        payee_is_account: step.to_account,
                        contract: step.contract.clone(),
                        line: step.line.clone(),
                        amount: IrExpr {
                            lang: "cfdl".to_string(),
                            src: step
                                .amount
                                .as_ref()
                                .map(|e| coerce_numeric_literals(&e.src))
                                .unwrap_or_else(|| "0".to_string()),
                        },
                    })
                    .collect(),
            }),
            _ => None,
        })
        .collect();

    // SLICES (docs/13 §7.90): a named, deliberately partial selection.
    // Clause kinds intersect, values within a kind union, `except` subtracts.
    // Validated here — a reference is what the compiler can resolve — and the
    // `type` clauses are EXPANDED here, because only the compiler holds the
    // ontology the transitive match walks.
    // DECLARED PRESENTATIONS (`docs/13` §7.55). Carried to the IR as declared;
    // the rows are generated after the run, because a hierarchy's shape is a
    // fact about the results rather than about the source.
    // The vocabulary `type` and `line` clauses resolve against, shared by
    // statement rows and slices below; and one diagnostics list for both.
    let onto = active_pack
        .map(|p| p.ontology.clone())
        .unwrap_or_else(cfdl_pack::PackOntology::language_base);
    // rule_id -> the contract type its rule family binds to.
    let rule_type: BTreeMap<&str, &str> = active_pack
        .map(|p| {
            p.lowering_rules
                .iter()
                .filter_map(|r| {
                    onto.contract_for_rule(&r.contract_name)
                        .map(|c| (r.id.as_str(), c.type_id.as_str()))
                })
                .collect()
        })
        .unwrap_or_default();
    // rule_id -> the line the rule emits, by role (docs/40 §6).
    let rule_line: BTreeMap<&str, &str> = active_pack
        .map(|p| {
            p.lowering_rules
                .iter()
                .filter_map(|r| r.line.as_deref().map(|l| (r.id.as_str(), l)))
                .collect()
        })
        .unwrap_or_default();
    // Steps that pay for a contract, with the contract's type and the line.
    let attributed_steps: Vec<(String, String, String)> = resolve_output
        .source_statements
        .iter()
        .filter_map(|s| match &s.statement {
            Stmt::Waterfall(w) => Some(w),
            _ => None,
        })
        .flat_map(|w| {
            let declared_contracts = &declared_contracts;
            let ontology = &ontology;
            w.steps.iter().filter_map(move |step| {
                let (Some(name), Some(line)) = (step.contract.as_deref(), step.line.as_deref())
                else {
                    return None;
                };
                let type_id = declared_contracts
                    .get(name)
                    .and_then(|c| resolve_contract_binding(c, ontology))
                    .map(|b| b.type_id)?;
                Some((
                    format!("{}.{}", w.name, step.name),
                    type_id,
                    line.to_string(),
                ))
            })
        })
        .collect();
    let mut view_diagnostics: Vec<Diagnostic> = Vec::new();
    let mut statement_spans: BTreeMap<String, (String, cfdl_parser::Span)> = BTreeMap::new();
    for source_stmt in &resolve_output.source_statements {
        if let Stmt::Statement(st) = &source_stmt.statement {
            statement_spans.insert(st.name.clone(), (source_stmt.file.clone(), st.span));
        }
    }
    let ir_statements: Vec<IrStatement> = resolve_output
        .source_statements
        .iter()
        .filter_map(|source_stmt| match &source_stmt.statement {
            Stmt::Statement(statement_stmt) => Some(IrStatement {
                name: statement_stmt.name.clone(),
                label: statement_stmt.label.clone(),
                structure: statement_stmt.structure.clone(),
                depth: statement_stmt.depth,
                grain: statement_stmt.grain.clone(),
                slice: statement_stmt.slice.clone(),
                metrics: statement_stmt.metrics.clone(),
                rows: statement_stmt
                    .rows
                    .iter()
                    .map(|r| IrStatementRow {
                        kind: r.kind.clone(),
                        label: r.label.clone(),
                        depth: r.depth,
                        categories: r.categories.clone(),
                        streams: r.streams.clone(),
                        types: r.types.clone(),
                        lines: r.lines.clone(),
                        type_streams: expand_type_and_line_clauses(
                            &format!("Statement '{}' row \"{}\"", statement_stmt.name, r.label),
                            &r.types,
                            &r.lines,
                            &streams,
                            &entities,
                            &onto,
                            &rule_type,
                            &rule_line,
                            &attributed_steps,
                            &source_stmt.file,
                            statement_stmt.span,
                            &mut view_diagnostics,
                        ),
                        slice: r.slice.clone(),
                        series: r.series.clone(),
                        entity: r.entity.clone(),
                        numerator: r.ratio_of.as_ref().map(|(n, _)| n.clone()),
                        denominator: r.ratio_of.as_ref().map(|(_, d)| d.clone()),
                        display: r.display.clone(),
                    })
                    .collect(),
                provenance: IrNodeProvenance {
                    source_file: source_stmt.file.clone(),
                    source_span: map_span(statement_stmt.span),
                    generated_by: None,
                },
            }),
            _ => None,
        })
        .collect();

    let ir_slices: Vec<IrSlice> = {
        let declared_entities: BTreeSet<String> = resolve_output
            .source_statements
            .iter()
            .filter_map(|s| match &s.statement {
                Stmt::Entity(e) => Some(e.symbol()),
                _ => None,
            })
            .collect();
        let mut slice_diagnostics: Vec<Diagnostic> = view_diagnostics;
        let mut out: Vec<IrSlice> = Vec::new();
        for source_stmt in &resolve_output.source_statements {
            let Stmt::Slice(slice_stmt) = &source_stmt.statement else {
                continue;
            };
            for entity_ref in slice_stmt
                .entities
                .iter()
                .chain(slice_stmt.except_entities.iter())
            {
                if !declared_entities.contains(entity_ref) {
                    slice_diagnostics.push(Diagnostic {
                        code: "E1362_SLICE_UNKNOWN_ENTITY".to_string(),
                        severity: "error".to_string(),
                        message: format!(
                            "Slice '{}' selects entity '{entity_ref}', which is not declared.",
                            slice_stmt.name
                        ),
                        file: Some(source_stmt.file.clone()),
                        span: Some(map_span(slice_stmt.span)),
                        path: None,
                        hint: Some("A slice selects by reference, and a reference is what the compiler can check — correct the name or declare the entity.".to_string()),
                        notes: vec![],
                    });
                }
            }
            for category in slice_stmt
                .categories
                .iter()
                .chain(slice_stmt.except_categories.iter())
            {
                let root = category.split('.').next().unwrap_or("");
                if !cfdl_pack::CATEGORY_ROOTS.contains(&root) {
                    slice_diagnostics.push(Diagnostic {
                        code: "E1364_SLICE_CATEGORY_ROOT".to_string(),
                        severity: "error".to_string(),
                        message: format!(
                            "Slice '{}' selects category '{category}', whose root '{root}' is not one of {}.",
                            slice_stmt.name,
                            cfdl_pack::CATEGORY_ROOTS.join(", ")
                        ),
                        file: Some(source_stmt.file.clone()),
                        span: Some(map_span(slice_stmt.span)),
                        path: None,
                        hint: Some("A category is a path into the cash flow statement; a selector that could never match anything is a typo, not a choice.".to_string()),
                        notes: vec![],
                    });
                }
            }
            let type_streams = expand_type_and_line_clauses(
                &format!("Slice '{}'", slice_stmt.name),
                &slice_stmt.types,
                &slice_stmt.lines,
                &streams,
                &entities,
                &onto,
                &rule_type,
                &rule_line,
                &attributed_steps,
                &source_stmt.file,
                slice_stmt.span,
                &mut slice_diagnostics,
            );
            out.push(IrSlice {
                name: slice_stmt.name.clone(),
                entities: slice_stmt.entities.clone(),
                types: slice_stmt.types.clone(),
                lines: slice_stmt.lines.clone(),
                categories: slice_stmt.categories.clone(),
                streams: slice_stmt.streams.clone(),
                except_streams: slice_stmt.except_streams.clone(),
                except_categories: slice_stmt.except_categories.clone(),
                except_entities: slice_stmt.except_entities.clone(),
                type_streams,
                // Normalised the way a phase's range is, so a month-only
                // bound means the first of that month and the engine's
                // `Date::parse` — which takes YYYY-MM-DD only — reads it.
                window: slice_stmt.window.as_ref().map(|(from, to)| IrDateRange {
                    start: normalize_date(from),
                    end: normalize_date(to),
                }),
                provenance: IrNodeProvenance {
                    source_file: source_stmt.file.clone(),
                    source_span: map_span(slice_stmt.span),
                    generated_by: None,
                },
            });
        }
        if !slice_diagnostics.is_empty() {
            sort_compile_diagnostics(&mut slice_diagnostics);
            return Err(slice_diagnostics);
        }
        out
    };

    let mut ir = Ir {
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
        quantiles: ir_quantiles,
        // Filled below, once the document exists to be walked.
        quantile_inputs: Vec::new(),
        accounts: ir_accounts,
        lifecycles: ir_lifecycles,
        waterfalls: ir_waterfalls,
        metrics: ir_metrics,
        views: IrViews {
            slices: ir_slices,
            statements: ir_statements,
        },
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
        stream_inputs,
        subtotals: ir_subtotals,
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
    };

    // Provenance, computed once the rest of the document exists.
    //
    // Both walk the ASSEMBLED IR rather than the statement list, so an
    // expression reaches these no matter which construct carries it — a
    // stream amount, a field's `next`, an event guard, a waterfall step —
    // and a construct added later is covered without touching this.
    // A METRIC NAMES A SERIES THIS MODEL DOES NOT PUBLISH (`docs/13` §7.85).
    //
    // Walked here rather than in the metric block above, because the
    // vocabulary is the WHOLE assembled document — lowered streams, waterfall
    // steps, entity rollups, accounts, fields, pack subtotals — and half of it
    // does not exist yet where metrics are read.
    // A STATEMENT NAMING SOMETHING THAT IS NOT THERE (`docs/13` §7.55).
    // Refused here rather than reported inside the rendered statement: a
    // presentation that silently shows nothing is the failure mode the whole
    // entry exists to end.
    let statement_diagnostics = check_statements(&ir, &statement_spans);
    if !statement_diagnostics.is_empty() {
        let mut diagnostics = statement_diagnostics;
        sort_compile_diagnostics(&mut diagnostics);
        return Err(diagnostics);
    }

    let metric_series_diagnostics = check_metric_series_names(&ir, &metric_spans);
    if !metric_series_diagnostics.is_empty() {
        let mut diagnostics = metric_series_diagnostics;
        sort_compile_diagnostics(&mut diagnostics);
        return Err(diagnostics);
    }

    let quantile_defs = ir_quantile_defs_for_provenance(&ir.quantiles);
    ir.quantile_inputs = collect_quantile_inputs(&ir, &quantile_defs);
    ir.required_refs = ir
        .quantiles
        .iter()
        .filter_map(|q| q.reference.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    Ok(ir)
}

/// Every series name a metric may fold over: the vocabulary the valuation
/// plane actually publishes, in both dialects.
///
/// `docs/13` §7.85. `series_sum` returns 0.0 for a selector that matches
/// nothing, which is right for a `.*` selector and wrong for a name spelled
/// out in full — and in a metric it is worse than wrong, because a fold
/// publishes ONE number under a name the author chose, with no series beside
/// it to show the zero. A typo, a published aggregate the metric environment
/// did not bind, and a correct answer were indistinguishable.
///
/// The vocabulary here MUST match what the engine binds into a metric's
/// environment. Both are derived from the same published-series rules; the
/// fixtures pin the pairing from both ends.
fn metric_series_vocabulary(ir: &Ir) -> BTreeSet<String> {
    let mut known = BTreeSet::new();
    // A stream, in the expression dialect and under its published key.
    for stream in &ir.streams {
        known.insert(stream.name.clone());
        known.insert(format!("stream.{}", stream.name));
    }
    // A waterfall step is a stream, and publishes under the same prefix.
    for waterfall in &ir.waterfalls {
        for step in &waterfall.steps {
            let name = format!("{}.{}", waterfall.name, step.name);
            known.insert(format!("stream.{name}"));
            known.insert(name);
        }
    }
    // The aggregate over an entity and everything `part of` it — the figure
    // §7.43 published and a metric could not reach.
    for entity in &ir.entities {
        known.insert(format!("entity.{}.net_cash_flow", entity.symbol));
        // A field publishes as a series under the thing that owns it.
        for field in entity.rules.keys() {
            known.insert(format!("{}.{field}", entity.symbol));
        }
    }
    for account in &ir.accounts {
        known.insert(format!("account.{}", account.name));
    }
    // A MONEY SUBTOTAL ONLY. A ratio has periods that are genuinely undefined
    // — a coverage ratio in a period with no debt service — which is why it
    // publishes `null` rather than zero, and a fold over it needs a decision
    // (skip the undefined periods, or refuse the fold) that belongs with the
    // reductions of §7.86. Naming one is refused here, with a hint that says
    // so, rather than folded as though `null` were nothing.
    for subtotal in &ir.subtotals {
        if subtotal.kind == "money" {
            known.insert(subtotal.id.clone());
        }
    }
    // A SLICE'S NET (docs/13 §7.90; docs/40 stage 5): a named selection by
    // type and line is what lets a metric fold every debt's interest without
    // naming a stream or a pack's category.
    for slice in &ir.views.slices {
        known.insert(format!("slice.{}", slice.name));
    }
    known.insert("model.net_cash_flow".to_string());
    known
}

/// EXPAND `type` AND `line` CLAUSES TO THE STREAMS THEY SELECT (docs/40 §6,
/// stage 5). A stream is matched by TYPE when the contract type its lowering
/// rule binds to is_a the wanted type — the transitive walk the recorded
/// refinement makes answerable — or when its owner's entity type is; by LINE
/// when its rule emits that line by role. Kinds intersect: `type
/// Contract.Debt line interest` is the interest of every debt, whichever
/// pack lowered it and whatever category that pack spelled. Only the
/// compiler can do this, because only it holds the ontology and the rules;
/// the evaluator receives exact names. Shared by slices and statement rows
/// so the two can never disagree about what a type selects.
#[allow(clippy::too_many_arguments)]
fn expand_type_and_line_clauses(
    subject: &str,
    types: &[String],
    lines: &[String],
    streams: &[((String, String), IrStream)],
    entities: &[((String, String), IrEntity)],
    onto: &cfdl_pack::PackOntology,
    rule_type: &BTreeMap<&str, &str>,
    rule_line: &BTreeMap<&str, &str>,
    // A waterfall step that pays for a contract: (`<waterfall>.<step>`,
    // the contract's type, the line). Allocated lines are paid by steps,
    // so a selection by type and line reaches them here.
    steps: &[(String, String, String)],
    file: &str,
    span: cfdl_parser::Span,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<String> {
    if types.is_empty() && lines.is_empty() {
        return Vec::new();
    }
    let diag = |code: &str, message: String, hint: String| Diagnostic {
        code: code.to_string(),
        severity: "error".to_string(),
        message,
        file: Some(file.to_string()),
        span: Some(map_span(span)),
        path: None,
        hint: Some(hint),
        notes: vec![],
    };
    fn rule_of(stream: &IrStream) -> Option<&str> {
        stream
            .provenance
            .generated_by
            .as_ref()
            .map(|g| g.rule_id.as_str())
    }
    let mut by_type: BTreeSet<String> = BTreeSet::new();
    for wanted in types {
        let known = onto.entities.iter().any(|e| e.type_id == *wanted)
            || onto.contracts.iter().any(|c| c.type_id == *wanted);
        if !known {
            let mut names: Vec<&str> = onto.contracts.iter().map(|c| c.type_id.as_str()).collect();
            names.sort_unstable();
            diagnostics.push(diag(
                "E1363_SLICE_UNKNOWN_TYPE",
                format!(
                    "{subject} selects type '{wanted}', which the active ontology does not define."
                ),
                format!("Known contract types: {}.", names.join(", ")),
            ));
            continue;
        }
        for (_, stream) in streams {
            let by_contract = rule_of(stream)
                .and_then(|id| rule_type.get(id))
                .is_some_and(|t| onto.is_a(t, wanted));
            let by_owner = entities
                .iter()
                .find(|(_, e)| e.symbol == stream.owner.symbol)
                .map(|(_, e)| e.r#type.as_str())
                .is_some_and(|t| !t.is_empty() && onto.is_a(t, wanted));
            if by_contract || by_owner {
                by_type.insert(stream.name.clone());
            }
        }
        for (step, type_id, _) in steps {
            if onto.is_a(type_id, wanted) {
                by_type.insert(step.clone());
            }
        }
    }
    let mut by_line: BTreeSet<String> = BTreeSet::new();
    if !lines.is_empty() {
        // The lines the active vocabulary can produce: every master's and
        // every pack type's, plus what the rules actually emit.
        let mut known_lines: BTreeSet<&str> = rule_line.values().copied().collect();
        for contract in &onto.contracts {
            for line in &contract.lines {
                known_lines.insert(line.name.as_str());
            }
        }
        for wanted in lines {
            if !known_lines.contains(wanted.as_str()) {
                let near: Vec<&str> = known_lines
                    .iter()
                    .copied()
                    .filter(|k| is_near_miss(k, wanted))
                    .collect();
                let hint = if near.is_empty() {
                    format!(
                        "Lines by role: {}.",
                        known_lines.iter().copied().collect::<Vec<_>>().join(", ")
                    )
                } else {
                    format!("Did you mean {}?", near.join(" or "))
                };
                diagnostics.push(diag(
                    "E1375_UNKNOWN_LINE_ROLE",
                    format!("{subject} selects line '{wanted}', which no contract type in the active ontology produces."),
                    hint,
                ));
                continue;
            }
            for (_, stream) in streams {
                if rule_of(stream)
                    .and_then(|id| rule_line.get(id))
                    .is_some_and(|l| *l == wanted.as_str())
                {
                    by_line.insert(stream.name.clone());
                }
            }
            for (step, _, line) in steps {
                if line == wanted {
                    by_line.insert(step.clone());
                }
            }
        }
    }
    let matched: BTreeSet<String> = match (types.is_empty(), lines.is_empty()) {
        (false, false) => by_type.intersection(&by_line).cloned().collect(),
        (false, true) => by_type,
        (true, false) => by_line,
        (true, true) => BTreeSet::new(),
    };
    matched.into_iter().collect()
}

/// Known structures a statement may present.
///
/// Both read a hierarchy the results already carry: `entity` walks the `part
/// of` tree `graph` publishes, `category` walks the dotted category path. A
/// third would be added here and in `cfdl-statement::generate` together.
const STATEMENT_STRUCTURES: &[&str] = &["entity", "category"];

fn check_statements(
    ir: &Ir,
    spans: &BTreeMap<String, (String, cfdl_parser::Span)>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let slice_names: BTreeSet<&str> = ir.views.slices.iter().map(|s| s.name.as_str()).collect();
    let metric_names: BTreeSet<&str> = ir.metrics.iter().map(|m| m.name.as_str()).collect();
    let categorised = ir.streams.iter().any(|s| s.category.is_some());
    for statement in &ir.views.statements {
        let (file, span) = spans
            .get(&statement.name)
            .map(|(f, s)| (Some(f.clone()), Some(map_span(*s))))
            .unwrap_or((None, None));
        let mut found: Vec<(String, String, String)> = Vec::new();
        let mut push = |code: &str, message: String, hint: String| {
            found.push((code.to_string(), message, hint));
        };
        // AUTHORED OR GENERATED, NEVER BOTH. A generated statement partitions
        // the cash by construction; an authored one partitions it by the
        // author's care. Mixed, neither holds: an authored `line` beside
        // generated rows claims streams the generated rows already claimed, so
        // the bottom line double-counts and the reconciliation that makes a
        // statement trustworthy becomes noise.
        if !statement.rows.is_empty() && !statement.structure.is_empty() {
            push(
                "E1369_STATEMENT_AUTHORED_AND_GENERATED",
                format!(
                    "Statement '{}' states both a structure and its own rows.",
                    statement.name
                ),
                "A statement either names a `structure` and lets the rows follow from the tree, or states its rows. Remove one."
                    .to_string(),
            );
        } else if statement.rows.is_empty() && statement.structure.is_empty() {
            push(
                "E1369_STATEMENT_AUTHORED_AND_GENERATED",
                format!(
                    "Statement '{}' states neither a structure nor any rows, so it would render nothing.",
                    statement.name
                ),
                "Name a `structure` to generate rows from a hierarchy, or state rows.".to_string(),
            );
        }
        if !statement.rows.is_empty() {
            // An authored row draws from a declared slice; a ratio divides two.
            for row in &statement.rows {
                // A `series` ROW DRAWS A FOLD, NOT CASH. Its figure is
                // presentation of something already computed, so it claims no
                // streams and stays out of the bottom line — which makes any
                // claim clause beside it a contradiction: the row cannot both
                // present a fold and claim cash. Refused rather than resolved
                // by precedence, because a precedence a reader cannot see is a
                // silently ignored clause (`docs/13` §7.55).
                if row.series.is_some()
                    && (!row.categories.is_empty()
                        || !row.streams.is_empty()
                        || !row.types.is_empty()
                        || !row.lines.is_empty()
                        || row.slice.is_some()
                        || row.entity.is_some()
                        || row.numerator.is_some()
                        || row.denominator.is_some())
                {
                    push(
                        "E1370_STATEMENT_SERIES_ROW_CLAIMS",
                        format!(
                            "Statement '{}' has a row drawing series '{}' beside a claim clause.",
                            statement.name,
                            row.series.as_deref().unwrap_or_default()
                        ),
                        "A row draws a published series or claims cash, never both. Remove the `series`, or the other draw clauses.".to_string(),
                    );
                }
                for referenced in [&row.slice, &row.numerator, &row.denominator]
                    .into_iter()
                    .flatten()
                {
                    if !slice_names.contains(referenced.as_str()) {
                        push(
                            "E1368_STATEMENT_UNKNOWN_REFERENCE",
                            format!(
                                "Statement '{}' has a row drawing slice '{referenced}', which this model does not declare.",
                                statement.name
                            ),
                            "A row draws a declared slice; check the spelling.".to_string(),
                        );
                    }
                }
            }
            // A generated statement's structure is checked below; an authored
            // one has none to check.
            for (code, message, hint) in found {
                diagnostics.push(Diagnostic {
                    code,
                    severity: "error".to_string(),
                    message,
                    file: file.clone(),
                    span: span.clone(),
                    path: None,
                    hint: Some(hint),
                    notes: vec![],
                });
            }
            continue;
        }
        if !STATEMENT_STRUCTURES.contains(&statement.structure.as_str()) {
            push(
                "E1367_STATEMENT_UNKNOWN_STRUCTURE",
                format!(
                    "Statement '{}' presents structure '{}', which is not one this engine builds.",
                    statement.name, statement.structure
                ),
                format!("Known structures: {}.", STATEMENT_STRUCTURES.join(", ")),
            );
        } else if statement.structure == "category" && !categorised {
            // A CATEGORY STATEMENT OVER UNCATEGORISED STREAMS would render as
            // one residual row and nothing else — technically complete, and
            // useless. Refused with the reason rather than shipped empty.
            push(
                "E1367_STATEMENT_UNKNOWN_STRUCTURE",
                format!(
                    "Statement '{}' presents a category hierarchy, and no stream in this model declares a category.",
                    statement.name
                ),
                "Give the streams a `category`, or present `structure entity`.".to_string(),
            );
        }
        if let Some(slice) = &statement.slice {
            if !slice_names.contains(slice.as_str()) {
                push(
                    "E1368_STATEMENT_UNKNOWN_REFERENCE",
                    format!(
                        "Statement '{}' filters by slice '{slice}', which this model does not declare.",
                        statement.name
                    ),
                    "A statement filters by a declared slice; check the spelling.".to_string(),
                );
            }
        }
        for metric in &statement.metrics {
            if !metric_names.contains(metric.as_str()) {
                push(
                    "E1368_STATEMENT_UNKNOWN_REFERENCE",
                    format!(
                        "Statement '{}' shows metric '{metric}', which this model does not declare.",
                        statement.name
                    ),
                    "A statement shows declared metrics; check the spelling.".to_string(),
                );
            }
        }
        for (code, message, hint) in found {
            diagnostics.push(Diagnostic {
                code,
                severity: "error".to_string(),
                message,
                file: file.clone(),
                span: span.clone(),
                path: None,
                hint: Some(hint),
                notes: vec![],
            });
        }
    }
    diagnostics
}

fn check_metric_series_names(
    ir: &Ir,
    spans: &BTreeMap<String, (String, cfdl_parser::Span)>,
) -> Vec<Diagnostic> {
    if ir.metrics.is_empty() {
        return Vec::new();
    }
    let known = metric_series_vocabulary(ir);
    let ratio_subtotals: BTreeSet<&str> = ir
        .subtotals
        .iter()
        .filter(|s| s.kind != "money")
        .map(|s| s.id.as_str())
        .collect();
    let mut diagnostics = Vec::new();
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    for metric in &ir.metrics {
        for referenced in cfdl_expr::series_references(&metric.expr.src) {
            // A `.*` SELECTOR MAY MATCH NOTHING, and says so at the call site.
            if referenced.ends_with(".*") || known.contains(&referenced) {
                continue;
            }
            if !seen.insert((metric.name.clone(), referenced.clone())) {
                continue;
            }
            let hint = if ratio_subtotals.contains(referenced.as_str()) {
                "'{}' is a RATIO subtotal. Its undefined periods publish as null rather than \
                 zero, so a sum or a mean over it would have to decide what null means; that \
                 decision has not been made. Fold the money subtotals it is built from."
                    .replace("{}", &referenced)
            } else {
                "Check the spelling. A metric may fold any series this model publishes: a \
                 stream by its own name or as `stream.<name>`, a waterfall step, \
                 `entity.<symbol>.net_cash_flow`, `account.<name>`, an entity field, a money \
                 subtotal, a slice's net as `slice.<name>`, or `model.net_cash_flow`. A \
                 selector ending in `.*` states that matching nothing is intended."
                    .to_string()
            };
            let (file, span) = spans
                .get(&metric.name)
                .map(|(f, s)| (Some(f.clone()), Some(map_span(*s))))
                .unwrap_or((None, None));
            diagnostics.push(Diagnostic {
                code: "E1365_METRIC_UNKNOWN_SERIES".to_string(),
                severity: "error".to_string(),
                message: format!(
                    "Metric '{}' folds series '{}', which this model does not publish.",
                    metric.name, referenced
                ),
                file,
                span,
                path: None,
                hint: Some(hint),
                notes: vec![],
            });
        }
    }
    diagnostics
}

/// The quantile definitions in the shape `cfdl_expr` resolves against, so the
/// slice published here is computed by the SAME code the engine runs rather
/// than by a second implementation that could drift from it.
fn ir_quantile_defs_for_provenance(
    quantiles: &[IrQuantile],
) -> std::collections::BTreeMap<String, cfdl_expr::QuantileDef> {
    quantiles
        .iter()
        .map(|q| {
            (
                q.name.clone(),
                cfdl_expr::QuantileDef {
                    interpolation: q.interpolation.clone(),
                    points: q.points.iter().map(|p| (p.share, p.value)).collect(),
                },
            )
        })
        .collect()
}

/// Every quantile call site in the document, resolved and deduplicated.
///
/// Deduplicated because the same slice appears in every period's expression
/// once and in several streams often; the audit record wants the distinct
/// questions asked, not one row per occurrence. Sorted so the IR is canonical.
fn collect_quantile_inputs(
    ir: &Ir,
    defs: &std::collections::BTreeMap<String, cfdl_expr::QuantileDef>,
) -> Vec<IrQuantileCall> {
    if defs.is_empty() {
        return Vec::new();
    }
    let Ok(doc) = serde_json::to_value(ir) else {
        return Vec::new();
    };
    let mut srcs: Vec<String> = Vec::new();
    collect_expr_sources(&doc, &mut srcs);
    let mut seen = std::collections::BTreeMap::new();
    for src in srcs {
        for call in cfdl_expr::quantile_calls(&src, defs) {
            let key = (
                call.quantile.clone(),
                call.function.clone(),
                call.args.iter().map(|a| a.to_bits()).collect::<Vec<_>>(),
            );
            seen.entry(key).or_insert(IrQuantileCall {
                quantile: call.quantile,
                function: call.function,
                args: call.args,
                // Rounded to the engine's single global policy for published
                // numbers (1e-6). Two reasons, and the second is the load-
                // bearing one: it matches what the ledger publishes, and this
                // value enters the IR and therefore `model_hash`. An
                // unrounded integral would carry the last bits of f64
                // arithmetic into a hash compared across three platforms —
                // 425.99999999999994 for what is exactly 426.
                value: call.value.map(round_published),
            });
        }
    }
    seen.into_values().collect()
}

/// The engine's rounding policy for published numbers, applied here so a
/// compile-time figure and the ledger figure it explains agree exactly.
fn round_published(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

/// Pull every `{ "lang": _, "src": _ }` out of a serialized IR — the shape
/// every expression takes, wherever it sits.
fn collect_expr_sources(value: &serde_json::Value, out: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(map) => {
            if let (Some(_), Some(serde_json::Value::String(src))) =
                (map.get("lang"), map.get("src"))
            {
                out.push(src.clone());
            }
            for v in map.values() {
                collect_expr_sources(v, out);
            }
        }
        serde_json::Value::Array(items) => {
            for v in items {
                collect_expr_sources(v, out);
            }
        }
        _ => {}
    }
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
        subtotal_specs: registry.subtotal_specs(&active.name),
        lowering_rules: registry.lowering_rules(&active.name),
        validations: registry.validations(&active.name),
        ontology: registry
            .ontology(&active.name)
            .map(|o| o.merged_with_base())
            .unwrap_or_else(cfdl_pack::PackOntology::language_base),
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

    // A `lifecycle <pack machine>` block ENHANCES that machine rather than
    // declaring one (`docs/34` D2a), so the requirements on a DECLARATION do
    // not apply to it: it states no `initial` and no `state` because the pack
    // already did, and an edge it names for its actions is the pack's edge.
    //
    // Validation is pack-unaware by construction, which is the same position
    // `E2002_CONTRACT_MISSING_EFFECTS` is in above — a contract with no
    // `effects` is an error until you know a pack rule lowers it. The answer
    // is the same too: validate reports what it can see, and the caller, which
    // HAS the pack, drops what the pack accounts for. Nothing here teaches
    // validation about packs, and no diagnostic is waived on a name that does
    // not actually resolve to a machine the pack declares.
    let augmenting_blocks: Vec<(String, cfdl_parser::Span)> = resolve_output
        .source_statements
        .iter()
        .filter_map(|source_stmt| {
            let Stmt::Lifecycle(lc) = &source_stmt.statement else {
                return None;
            };
            pack.ontology
                .lifecycle(&lc.name)
                .map(|_| (source_stmt.file.clone(), lc.span))
        })
        .collect();
    let inside_augmenting_block = |diag: &cfdl_validate::ValidationDiagnostic| {
        augmenting_blocks.iter().any(|(file, span)| {
            *file == diag.file
                && diag.span.start_line >= span.start_line
                && diag.span.start_line <= span.end_line
        })
    };

    // A CONTRACT NO RULE LOWERS IS A TYPE THE PACK DOES NOT DECLARE. Validation
    // sees it as a contract with no `effects` (`E2002`), which is true and
    // useless: the modeller wrote `cre.leas_unit` and the answer is the type
    // they meant, not a block the pack would have supplied. Where the pack
    // declares contract types at all, the diagnostic is restated as `E1373`
    // with the near miss — the same code the two-token form gets from
    // `check_contract_types`, so both spellings of the mistake read alike.
    let pack_declares_types = pack
        .ontology
        .contracts
        .iter()
        .any(|c| c.contract_name.is_some());
    let unresolved_contract = |diag: &cfdl_validate::ValidationDiagnostic| {
        resolve_output
            .source_statements
            .iter()
            .find_map(|source_stmt| {
                let Stmt::Contract(contract) = &source_stmt.statement else {
                    return None;
                };
                (source_stmt.file == diag.file
                    && contract.span.start_line == diag.span.start_line
                    && contract.span.start_col == diag.span.start_col)
                    .then_some(contract)
            })
    };

    diagnostics
        .into_iter()
        .filter(|diag| {
            match diag.code {
                "E2002_CONTRACT_MISSING_EFFECTS" => {
                    !lowered_contract_anchors.iter().any(|(file, span)| {
                        *file == diag.file
                            && span.start_line == diag.span.start_line
                            && span.start_col == diag.span.start_col
                    })
                }
                // The pack declared the opening state and the state set; an
                // enhancing block restates neither.
                "E1351_LIFECYCLE_NO_INITIAL" | "E1316_UNKNOWN_LIFECYCLE_STATE" => {
                    !inside_augmenting_block(diag)
                }
                _ => true,
            }
        })
        .map(|mut diag| {
            if diag.code != "E2002_CONTRACT_MISSING_EFFECTS" || !pack_declares_types {
                return diag;
            }
            let Some(contract) = unresolved_contract(&diag) else {
                return diag;
            };
            // The two-token form states its type; a fused name states it as
            // its first two segments.
            let stated: String = contract.declared_type.clone().unwrap_or_else(|| {
                contract
                    .name
                    .split('.')
                    .take(2)
                    .collect::<Vec<_>>()
                    .join(".")
            });
            let (code, message, hint) = contract_type_refusal(
                &contract.name,
                &stated,
                contract.instance.as_deref(),
                &pack.ontology,
                Some(pack.name.as_str()),
            );
            diag.code = code;
            diag.message = match hint {
                Some(hint) => format!("{message} {hint}"),
                None => message,
            };
            diag
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
            fields: BTreeMap::new(),
            field_roles: BTreeMap::new(),
            stream_inputs: vec![],
            diagnostics: vec![],
        };
    };
    let mut rules = pack.lowering_rules.clone();
    rules.sort_by(|a, b| a.id.cmp(&b.id));

    // What a reference term may name (docs/40 §3).
    let declared_contract_names: Vec<String> = resolve_output
        .source_statements
        .iter()
        .filter_map(|s| match &s.statement {
            Stmt::Contract(c) => Some(c.name.clone()),
            _ => None,
        })
        .collect();
    let declared_account_names: Vec<String> = resolve_output
        .source_statements
        .iter()
        .filter_map(|s| match &s.statement {
            Stmt::Account(a) => Some(a.name.clone()),
            _ => None,
        })
        .collect();

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
    // A PACK EMITS A FIELD OF THE THING, not a model-level state. Keyed by
    // owner and name: two contracts on DIFFERENT entities may carry the same
    // field name without colliding, which is what made the suffix necessary
    // while these were global.
    let mut lowered_fields: BTreeMap<(String, String), IrFieldRule> = BTreeMap::new();
    let mut lowered_field_roles: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    let mut stream_inputs: Vec<IrStreamInputs> = Vec::new();
    let mut diagnostics = Vec::new();
    for source_stmt in &resolve_output.source_statements {
        let Stmt::Contract(contract) = &source_stmt.statement else {
            continue;
        };
        // A UNIT ANNOTATION IS AN ASSERTION, and the rule is the truth.
        //
        // The energy pack's own comments spend a paragraph warning that
        // 0.1 c/kWh is $1.00/MWh and that getting it wrong rounds to a
        // hundredth of a cent — indistinguishable from not rounding at all.
        // That warning is now checkable: a model may state the unit it
        // believes it is writing, and a disagreement is an error.
        //
        // The mismatch is NOT converted, and there is deliberately no
        // conversion table. Rescaling would mean the number in the model text
        // is not the number the engine used, so reading the model would require
        // knowing a conversion had happened — and a wrong entry in a conversion
        // table would be a new silent-wrong-answer path, which is the whole
        // class of failure this work has been closing. The model restates the
        // value in the unit the rule expects, and stays literal.
        let before = diagnostics.len();
        // TERMS AGAINST THE EFFECTIVE ROSTER (docs/40 §8). A term the type
        // does not declare is refused (`E1371`) — before this it was quietly
        // ignored, and a misspelled `escalation` was a lease that never
        // escalated. A required term the contract omits, or a group of
        // alternatives it states none of, is `E1372`.
        if let Some(binding) = resolve_contract_binding(contract, &pack.ontology) {
            let fields = pack.ontology.effective_fields(&binding.type_id);
            let roster = || {
                fields
                    .iter()
                    .map(|f| f.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            for (key, term) in &contract.terms {
                if fields.iter().any(|f| f.name == *key) {
                    continue;
                }
                let near: Vec<&str> = fields
                    .iter()
                    .filter(|f| is_near_miss(&f.name, key))
                    .map(|f| f.name.as_str())
                    .collect();
                let mut diag = lowering_rule_diag(
                    "E1371_UNKNOWN_CONTRACT_TERM",
                    &format!(
                        "Contract '{}' states term '{key}', which type '{}' does not declare. The term would never be read.",
                        contract.name, binding.type_id
                    ),
                    source_stmt,
                    term.span,
                );
                diag.hint = Some(if near.is_empty() {
                    format!("Terms of '{}': {}.", binding.type_id, roster())
                } else {
                    format!("Did you mean {}?", near.join(" or "))
                });
                diagnostics.push(diag);
            }
            // A REFERENCE TERM NAMES SOMETHING THIS MODEL DECLARES: a
            // guarantee's `covered` names a contract, a note's
            // `principal_account` names an account (docs/40 §3). Checked here
            // so a misspelled account is refused rather than read as zero.
            for field in fields
                .iter()
                .filter(|f| f.field_type == "contract" || f.field_type == "account")
            {
                let Some(term) = contract.terms.get(&field.name) else {
                    continue;
                };
                let named = term.value.trim().trim_matches('"');
                let (kind, known): (&str, Vec<String>) = if field.field_type == "contract" {
                    ("contract", declared_contract_names.clone())
                } else {
                    ("account", declared_account_names.clone())
                };
                if known.iter().any(|k| k == named) {
                    continue;
                }
                let near: Vec<&str> = known
                    .iter()
                    .map(|k| k.as_str())
                    .filter(|k| is_near_miss(k, named))
                    .collect();
                let mut diag = lowering_rule_diag(
                    "E1376_UNKNOWN_REFERENCE",
                    &format!(
                        "Contract '{}' term '{}' names {kind} '{named}', which this model does not declare.",
                        contract.name, field.name
                    ),
                    source_stmt,
                    term.span,
                );
                diag.hint = Some(if near.is_empty() {
                    format!("Declared {kind}s: {}.", join_or_none(&known))
                } else {
                    format!("Did you mean {}?", near.join(" or "))
                });
                diagnostics.push(diag);
            }
            for field in fields.iter().filter(|f| f.required) {
                if !contract.terms.contains_key(&field.name) {
                    diagnostics.push(lowering_rule_diag(
                        "E1372_MISSING_CONTRACT_TERM",
                        &format!(
                            "Contract '{}' omits term '{}', which type '{}' requires.",
                            contract.name, field.name, binding.type_id
                        ),
                        source_stmt,
                        contract.span,
                    ));
                }
            }
            let mut groups: Vec<&str> = fields.iter().filter_map(|f| f.one_of.as_deref()).collect();
            groups.sort_unstable();
            groups.dedup();
            for group in groups {
                let members: Vec<&cfdl_pack::OntologyField> = fields
                    .iter()
                    .filter(|f| f.one_of.as_deref() == Some(group))
                    .collect();
                // A required member already carries the obligation and was
                // reported above if missing.
                if members.iter().any(|m| m.required) {
                    continue;
                }
                if !members.iter().any(|m| contract.terms.contains_key(&m.name)) {
                    diagnostics.push(lowering_rule_diag(
                        "E1372_MISSING_CONTRACT_TERM",
                        &format!(
                            "Contract '{}' states none of {}; type '{}' requires one of them.",
                            contract.name,
                            members
                                .iter()
                                .map(|m| m.name.as_str())
                                .collect::<Vec<_>>()
                                .join(", "),
                            binding.type_id
                        ),
                        source_stmt,
                        contract.span,
                    ));
                }
            }
        }
        for (key, term) in &contract.terms {
            // An expression term is compiled HERE, at the term's own span,
            // rather than left for E5009 to reject after substitution — by
            // then the text is a spliced template and the error points at a
            // rule the modeller did not write.
            if term.kind == cfdl_parser::TermValueKind::Expr {
                if let Err(err) = cfdl_expr::compile_expr(&term.value) {
                    diagnostics.push(lowering_rule_diag(
                        "E5025_TERM_EXPR_INVALID",
                        &format!(
                            "Contract '{}' term '{}' is an expression that does not compile [{}]: {}",
                            contract.name, key, err.code, err.message
                        ),
                        source_stmt,
                        term.span,
                    ));
                    continue;
                }
            }
            let matched = pack
                .lowering_rules
                .iter()
                .find(|rule| rule_matches_contract(&rule.contract_name, &contract.name));
            if let (Some(stated), Some(rule)) = (term.unit.as_deref(), matched) {
                if let Some(declared) = rule.units.get(key.as_str()) {
                    if !units_equal(stated, declared) {
                        let mut diag = lowering_rule_diag(
                            "E5024_TERM_UNIT_MISMATCH",
                            &format!(
                                "Contract '{}' term '{}' is stated in {stated}, but the rule expresses it in {declared}.",
                                contract.name, key
                            ),
                            source_stmt,
                            term.span,
                        );
                        diag.hint = Some(format!(
                            "Restate the value in {declared}. Units are not converted: the number \
                             in the model is the number the engine uses."
                        ));
                        diagnostics.push(diag);
                    }
                }
            }
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
        diagnostics.extend(validate_pack_contract(
            pack,
            source_stmt,
            contract,
            ctx.time_calendar,
            ctx.time_start,
            ctx.time_periods,
            // The furthest period a term may legally reach: the cash horizon
            // plus any `project` tail — the SAME boundary the lowering's own
            // bounds check uses. A lease running through the valuation tail
            // is what a derived forward exit requires ("runs through the
            // projection tail so exit valuation sees a full forward year"),
            // and the pack validation must not refuse what the lowering
            // demands.
            ctx.timeline_eval_end,
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
            // A field-only rule (an empty stream name) lowers a claim and no
            // cash; the loader admitted it because it names a field.
            let field_only = rule.stream_name.is_empty();
            if !field_only
                && !rule.stream_name.contains("{{")
                && !is_qualified_name(&rule.stream_name)
            {
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
            // Errors raised from inside the resolvers, which can only
            // return Option and so cannot emit diagnostics themselves.
            let resolver_errors: std::cell::RefCell<Vec<(String, String)>> =
                std::cell::RefCell::new(Vec::new());
            let resolve_plain = |key: &str| -> Option<String> {
                match contract.terms.get(key) {
                    Some(term) => {
                        if term.kind == cfdl_parser::TermValueKind::Expr {
                            resolver_errors.borrow_mut().push((
                                "E5026_TERM_EXPR_IN_LITERAL_SLOT".to_string(),
                                format!(
                                    "Pack lowering rule '{}' uses term '{}' as a literal (a frequency or day count), so it cannot hold an expression; contract '{}' supplies `{}`.",
                                    rule.id, key, contract.name, term.value
                                ),
                            ));
                            return None;
                        }
                        Some(term.value.clone())
                    }
                    None => rule.defaults.get(key).cloned(),
                }
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
            // Converts a months-denominated term into this rule's periods.
            // `_months` always means calendar months, on every calendar: it
            // describes the contract, not the modeller's grid choice.
            let months_to_periods = |key: &str, whole: bool| -> Option<String> {
                // Classified by kind, not by sniffing the text — an
                // expression term is as non-literal as an input ref, and both
                // are rejected the same way.
                if let Some(term) = contract.terms.get(key) {
                    if term.kind != cfdl_parser::TermValueKind::Literal {
                        resolver_errors.borrow_mut().push((
                            "E5017_PERIOD_TERM_NOT_LITERAL".to_string(),
                            format!(
                                "Pack lowering rule '{}' converts term '{}' from months into periods, so it must be a literal; contract '{}' supplies `{}`.",
                                rule.id, key, contract.name, term.value.trim()
                            ),
                        ));
                        return None;
                    }
                }
                let raw = match contract.terms.get(key) {
                    Some(term) => term.value.clone(),
                    None => rule.defaults.get(key).cloned()?,
                };
                let months: f64 = match raw.trim().parse() {
                    Ok(months) => months,
                    Err(_) => {
                        // `.ok()?` here used to surface as E5006 "missing
                        // term" — present but non-numeric is a different
                        // fact, and the message should say which.
                        resolver_errors.borrow_mut().push((
                            "E5017_PERIOD_TERM_NOT_LITERAL".to_string(),
                            format!(
                                "Pack lowering rule '{}' converts term '{}' from months into periods, but `{}` is not a number.",
                                rule.id, key, raw.trim()
                            ),
                        ));
                        return None;
                    }
                };
                let periods = months * f64::from(ppy) / 12.0;
                if whole && (periods.fract().abs() > 1e-9) {
                    resolver_errors.borrow_mut().push((
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
            let resolve_with = |key: &str, expr_slot: bool| -> Option<String> {
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
                    _ => match contract.terms.get(key) {
                        Some(term) => match term.kind {
                            // A compound value is PARENTHESISED on
                            // substitution. Expansion is a textual splice, so
                            // `a + b` into `{{x}} * {{y}}` would otherwise
                            // silently associate as `a + (b * y)` — an error
                            // worth real money and invisible in the output.
                            // Atomic values splice verbatim, byte-identically
                            // to every model written before expressions.
                            cfdl_parser::TermValueKind::Expr if expr_slot => {
                                Some(format!("({})", term.value))
                            }
                            // A name, a date, a frequency, a net-days count:
                            // these slots are never parsed as expressions, so
                            // an expression here is not late — it is wrong.
                            cfdl_parser::TermValueKind::Expr => {
                                resolver_errors.borrow_mut().push((
                                    "E5026_TERM_EXPR_IN_LITERAL_SLOT".to_string(),
                                    format!(
                                        "Pack lowering rule '{}' uses term '{}' in a slot that is not an expression (a name, date, frequency, or count), so it cannot hold an expression; contract '{}' supplies `{}`.",
                                        rule.id, key, contract.name, term.value
                                    ),
                                ));
                                None
                            }
                            _ => Some(term.value.clone()),
                        },
                        None => None,
                    },
                };
                from_contract.or_else(|| rule.defaults.get(key).cloned())
            };
            let resolve_expr = |key: &str| resolve_with(key, true);
            let resolve_literal = |key: &str| resolve_with(key, false);
            let mut expanded_rule = rule.clone();
            let mut missing_keys: Vec<String> = Vec::new();
            // A slot is an EXPRESSION slot when its expansion is compiled by
            // cfdl-expr and evaluated per period; everything else — names,
            // dates, frequencies, counts — takes a term literally.
            for (slot, target, expr_slot) in [
                (&rule.amount_expr, &mut expanded_rule.amount_expr, true),
                (&rule.schedule_from, &mut expanded_rule.schedule_from, false),
                (&rule.schedule_to, &mut expanded_rule.schedule_to, false),
                (&rule.stream_name, &mut expanded_rule.stream_name, false),
                (
                    &rule.schedule_net_days,
                    &mut expanded_rule.schedule_net_days,
                    false,
                ),
                (
                    &rule.schedule_net_months,
                    &mut expanded_rule.schedule_net_months,
                    false,
                ),
                // Templated so a contract can declare its own payment rhythm
                // (`payment_frequency = "month"`), letting one rule serve a
                // monthly, quarterly and daily-book version of the same
                // instrument. Already expanded above to derive ppy; expanding
                // it again here is what puts the result on the rule.
                (
                    &rule.schedule_every,
                    &mut expanded_rule.schedule_every,
                    false,
                ),
                (&rule.field_name, &mut expanded_rule.field_name, false),
                (&rule.field_init, &mut expanded_rule.field_init, true),
                (&rule.field_next, &mut expanded_rule.field_next, true),
                (&rule.field_every, &mut expanded_rule.field_every, false),
                (&rule.field_from, &mut expanded_rule.field_from, false),
                (&rule.field_to, &mut expanded_rule.field_to, false),
            ] {
                let resolver: &dyn Fn(&str) -> Option<String> = if expr_slot {
                    &resolve_expr
                } else {
                    &resolve_literal
                };
                match cfdl_pack::expand_rule_template(slot, resolver) {
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
            let resolver_errors = resolver_errors.into_inner();
            if !resolver_errors.is_empty() {
                for (code, message) in &resolver_errors {
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
            if !field_only && !is_qualified_name(&expanded_rule.stream_name) {
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
            // The UNEXPANDED rule, kept before `rule` is rebound below. Its
            // templates still carry their `{{contract.<key>}}` placeholders,
            // which is the only place the set of terms a rule consumes can be
            // read — after expansion the keys are gone and only their values
            // remain, indistinguishable from literals.
            let source_rule = rule;
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

            // The schedule reads `net_days`/`net_months` with `.parse().ok()`,
            // silently falling back to the contract's payment terms when the
            // expansion is not an integer. A garbage expansion must be an
            // error, not a different schedule.
            let mut bad_net = false;
            for (label, expanded) in [
                ("schedule_net_days", rule.schedule_net_days.trim()),
                ("schedule_net_months", rule.schedule_net_months.trim()),
            ] {
                if !expanded.is_empty() && expanded.parse::<i64>().is_err() {
                    diagnostics.push(lowering_rule_diag(
                        "E5004_INVALID_LOWERING_RULE",
                        &format!(
                            "Pack lowering rule '{}' expanded {label} to `{}` for contract '{}', which is not a whole number of {}.",
                            rule.id,
                            expanded,
                            contract.name,
                            if label == "schedule_net_days" { "days" } else { "months" }
                        ),
                        source_stmt,
                        contract.span,
                    ));
                    bad_net = true;
                }
            }
            if bad_net {
                continue;
            }

            let schedule = lower_pack_rule_schedule(
                rule,
                ctx.time_calendar,
                ctx.time_start,
                ctx.timeline_end,
                contract.payment_net,
            );
            let mut amount_src = rule.amount_expr.clone();
            // Pack terms are applied declaratively via rule templates; the
            // legacy hardcoded paths (CRE, then OpCo) were removed with the
            // v1 rule migrations.

            // Template expansion is a textual splice, so a term can produce an
            // expression the parser rejects. Catch it here: the engine's
            // fallback is to evaluate a failed expression as zero and carry on
            // with a warning, which turns a malformed model into a silently
            // empty stream. A field-only rule has no amount to compile.
            if let Err(err) = cfdl_expr::compile_expr(&amount_src)
                .map(|_| ())
                .or_else(|e| if field_only { Ok(()) } else { Err(e) })
            {
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

            if !rule.field_name.is_empty() {
                // Same treatment as the amount: a textual splice can produce an
                // expression the parser rejects, and the engine's fallback for
                // a failed state is zero — which would silently flatten every
                // stream that reads it.
                let mut bad = None;
                for (clause, src) in [("init", &rule.field_init), ("next", &rule.field_next)] {
                    if let Err(err) = cfdl_expr::compile_expr(src) {
                        bad = Some((clause, err, src.clone()));
                        break;
                    }
                }
                if let Some((clause, err, src)) = bad {
                    diagnostics.push(lowering_rule_diag(
                        "E5020_LOWERED_FIELD_INVALID",
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
                // THE SUBJECT, not this rule's owner. One contract's rules may
                // have different owners — collections on the pool, the purchase
                // on the buyer — and they share one factor. Keying it to each
                // rule's own owner put the field where some rules could not see
                // it, and the read resolved to nothing.
                let field_owner = contract
                    .subject_entity
                    .clone()
                    .unwrap_or_else(|| owner_symbol.clone());
                let field_key = (field_owner.clone(), rule.field_name.clone());
                if let Some(role) = source_rule.field_role.as_deref() {
                    let filled = lowered_field_roles
                        .entry((field_owner.clone(), role.to_string()))
                        .or_default();
                    if !filled.contains(&rule.field_name) {
                        filled.push(rule.field_name.clone());
                    }
                }
                match lowered_fields.get(&field_key) {
                    Some(existing)
                        if existing.init.src != rule.field_init
                            || existing.next.src != rule.field_next =>
                    {
                        diagnostics.push(lowering_rule_diag(
                            "E5021_DUPLICATE_LOWERED_FIELD",
                            &format!(
                                "Contract '{}' lowers to field '{}' on an entity where another contract already defines it differently. Two contracts on ONE entity need distinct field names; two contracts on different entities do not collide.",
                                contract.name, rule.field_name
                            ),
                            source_stmt,
                            contract.span,
                        ));
                        continue;
                    }
                    Some(_) => {}
                    None => {
                        lowered_fields.insert(
                            field_key,
                            IrFieldRule {
                                init: IrExpr {
                                    lang: "cfdl".to_string(),
                                    src: rule.field_init.clone(),
                                },
                                next: IrExpr {
                                    lang: "cfdl".to_string(),
                                    src: rule.field_next.clone(),
                                },
                                schedule: lower_rule_state_schedule(
                                    rule,
                                    ctx.time_calendar,
                                    ctx.time_start,
                                    ctx.timeline_end,
                                ),
                            },
                        );
                    }
                }
            }

            // A FIELD-ONLY RULE IS DONE HERE: its claim is lowered, and there is
            // no stream to emit — the priority of payments pays it.
            if field_only {
                continue;
            }
            // AND THE EXPRESSIONS READ THE FIELD. A rule writes `field.<name>`
            // because it cannot know which entity it will be attached to; here
            // that placeholder becomes the path the value actually lives at.
            // EVERY FIELD THE CONTRACT LOWERS, not only this rule's own: a
            // field-only rule declares the state once (a pool's balance) and
            // the stream rules read it without restating it. Longest name
            // first so `field.x_lag_a` is never clipped by `field.x_a`.
            {
                let field_owner = contract
                    .subject_entity
                    .clone()
                    .unwrap_or_else(|| owner_symbol.clone());
                let mut names: Vec<String> = lowered_fields
                    .keys()
                    .filter(|(owner, _)| *owner == field_owner)
                    .map(|(_, name)| name.clone())
                    .collect();
                if !rule.field_name.is_empty() && !names.contains(&rule.field_name) {
                    names.push(rule.field_name.clone());
                }
                names.sort_by(|a, b| b.len().cmp(&a.len()).then(a.cmp(b)));
                for name in names {
                    let from = format!("field.{name}");
                    // `entity.<owner>.<field>`, the long form. The bare alias
                    // covers the four declared families only, and a lowering
                    // rule may sit on any entity — so the spelling that always
                    // resolves is the one that goes through the entity root.
                    let to = format!("entity.{field_owner}.{name}");
                    amount_src = amount_src.replace(&from, &to);
                }
            }

            // What this rule actually read to strike this stream. Derived from
            // the rule's own templates rather than from the contract, because a
            // contract lowers to several streams and each reads a different
            // subset — the debt-service rule and the interest rule of one loan
            // do not consume the same terms.
            {
                let mut consumed: BTreeMap<String, String> = BTreeMap::new();
                let mut defaults_applied: Vec<String> = Vec::new();
                for template in [
                    &source_rule.amount_expr,
                    &source_rule.schedule_from,
                    &source_rule.schedule_to,
                    &source_rule.stream_name,
                    &source_rule.schedule_net_days,
                    &source_rule.schedule_net_months,
                    &source_rule.schedule_every,
                    &source_rule.field_init,
                    &source_rule.field_next,
                ] {
                    for key in cfdl_pack::template_placeholders(template) {
                        // Cadence primitives and name derivations are computed,
                        // not supplied — they say nothing about what the model
                        // stated, so they are not provenance.
                        if key.starts_with("periods.")
                            || key.starts_with("whole_periods.")
                            || key.starts_with("model.")
                            || key.starts_with("time.")
                            || matches!(
                                key.as_str(),
                                "name" | "suffix" | "dot_suffix" | "suffix_ident"
                            )
                        {
                            continue;
                        }
                        if let Some(term) = contract.terms.get(&key) {
                            consumed.insert(key.clone(), term.value.clone());
                        } else if let Some(default) = source_rule.defaults.get(&key) {
                            consumed.insert(key.clone(), default.clone());
                            if !defaults_applied.contains(&key) {
                                defaults_applied.push(key.clone());
                            }
                        }
                    }
                }
                if !consumed.is_empty() {
                    defaults_applied.sort();
                    stream_inputs.push(IrStreamInputs {
                        stream: rule.stream_name.clone(),
                        contract: contract.name.clone(),
                        terms: consumed,
                        defaults_applied,
                    });
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
                    moves: None,
                    direction: if rule.direction.is_empty() {
                        "outflow".to_string()
                    } else {
                        rule.direction.clone()
                    },
                    currency: rule_currency.clone(),
                    // The instance wins, then the rule.
                    //
                    // A pack states what its own contracts are and is usually
                    // right. It cannot be right about a leaf it never
                    // enumerated — a departmental operating expense, or an
                    // entity whose main business activity puts interest
                    // somewhere its default does not (docs/35 §2.5). The
                    // instance's category is validated against the three roots
                    // before lowering; the rule's was validated at pack load.
                    // Per-stream override, then the bare form, then the rule.
                    //
                    // A contract lowers one or more streams and its pack states
                    // a category for each. `category <stream> = <path>` names
                    // which one is being reclassified; the bare `category
                    // <path>` is sugar for a contract that lowers exactly one,
                    // and is refused where it would flatten several onto one
                    // category (E5030).
                    category: contract
                        .stream_categories
                        .get(&expanded_rule.stream_name)
                        .cloned()
                        .or_else(|| contract.category.clone())
                        .or_else(|| (!rule.category.is_empty()).then(|| rule.category.clone())),
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
                            line: source_rule.line.clone(),
                            contract: Some(contract.name.clone()),
                        }),
                    },
                },
            ));
        }
    }
    PackLoweringOutput {
        streams: lowered,
        fields: lowered_fields,
        field_roles: lowered_field_roles,
        stream_inputs,
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

    // A LEVEL PAYMENT CANNOT BE STRUCK FROM A VARYING DIVISOR.
    //
    // `{{model.amortization_divisor}}` expands an Actual convention to
    // `(360 / time.days_in_period)`, which is a per-period value, and the
    // annuity it feeds — `pmt(rate / divisor, n - p, 1)` — applies it to ALL
    // `n - p` remaining periods. January therefore strikes a payment as if
    // every remaining month had 31 days and February as if every one had 28,
    // so the payment moves with month length. No loan document does that: a
    // commercial Actual/360 loan fixes its payment on a 30/360 schedule and
    // lets principal absorb the difference.
    //
    // Measured on a single 1,200,000 loan at 6% with no prepayment and no
    // defaults, `amortization_day_count = "act/360"` swings the payment by
    // 460.68 over twelve months (7,349.63 in a 31-day month, 6,888.95 in
    // February). This is NOT a pool effect — it is the closed form applying a
    // period-local divisor to a whole remaining term, and it is wrong for a
    // single loan too. The sibling failure was measured once already, on the
    // ACCRUAL divisor, at 697k-754k in `benchmarks/credit/mbs_pool_conventions`;
    // splitting the two divisors fixed that spelling and left this one.
    //
    // Accrual is unaffected: `day_count = "act/360"` is a per-period accrual
    // and a per-period divisor is exactly right for it.
    if let Some(term) = contract.terms.get("amortization_day_count") {
        let value = term.value.trim().trim_matches('"');
        if matches!(value, "act/360" | "act/365") {
            diagnostics.push(pack_diag(
                "E5027_ACTUAL_AMORTIZATION_BASIS",
                &format!(
                    "Contract '{}' declares amortization_day_count = '{}'. A level payment \
                     is struck once and held; an Actual basis makes it move with month \
                     length, because the divisor is period-local and the annuity applies \
                     it to every remaining period. Strike the payment on '30/360' or \
                     '30e/360' and accrue interest on the Actual basis with `day_count`, \
                     which is what an Actual/360 loan document says.",
                    contract.name, value
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
/// Compare two unit strings.
///
/// Case- and space-insensitive, because `USD/MWh` and `usd / mwh` are the same
/// dimension written by two people. Deliberately NOT a dimensional algebra: it
/// does not know that `c/kWh` and `USD/MWh` are related, and that is the point
/// — a pack states one spelling and a model has to match it, which is a
/// comparison that cannot itself be wrong.
fn units_equal(a: &str, b: &str) -> bool {
    let norm = |s: &str| {
        s.chars()
            .filter(|c| !c.is_whitespace())
            .flat_map(|c| c.to_lowercase())
            .collect::<String>()
    };
    norm(a) == norm(b)
}

fn rule_matches_contract(rule_contract: &str, contract_name: &str) -> bool {
    // One predicate, shared with pack validations. It was duplicated here and
    // in `cfdl-pack` — the same six lines twice, which is a drift waiting to
    // happen between "which rule lowers this contract" and "which rule
    // validates it". Those two must never disagree.
    cfdl_pack::matches_contract_name(rule_contract, contract_name)
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
    if rule.field_name.is_empty() {
        return None;
    }
    // Absent means every model period — the behavior of every state written
    // before states had a clock, so an unset field changes nothing.
    if rule.field_every.is_empty() && rule.field_from.is_empty() && rule.field_to.is_empty() {
        return None;
    }
    Some(IrSchedule {
        kind: "Every".to_string(),
        placement: None,
        net_days: None,
        net_months: None,
        on: None,
        every: Some(if rule.field_every.is_empty() {
            time_calendar.to_string()
        } else {
            interval_to_frequency(&rule.field_every).to_string()
        }),
        from: Some(normalize_date(if rule.field_from.is_empty() {
            time_start
        } else {
            &rule.field_from
        })),
        to: Some(normalize_date(if rule.field_to.is_empty() {
            timeline_end
        } else {
            &rule.field_to
        })),
        on_rule: None,
        phase: None,
        convention: None,
        calendar: None,
        except_dates: Vec::new(),
        also_dates: Vec::new(),
        anchor_entity: None,
        anchor_state: None,
        anchor_periods: None,
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
            placement: placement_of_rule(rule.schedule_placement.as_deref()),
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
            anchor_entity: None,
            anchor_state: None,
            anchor_periods: None,
        }
    } else {
        IrSchedule {
            kind: "Every".to_string(),
            placement: placement_of_rule(rule.schedule_placement.as_deref()),
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
            anchor_entity: None,
            anchor_state: None,
            anchor_periods: None,
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
    time_calendar: &str,
    time_start: &str,
    timeline_end: &str,
    phase_map: &BTreeMap<String, (String, String)>,
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
                // A purely scheduled event has no condition to compile.
                if let Some(when) = event.when.as_deref() {
                    if let Err(err) = cfdl_expr::compile_expr(when) {
                        diags.push(diag(&err.code, err.message));
                        continue;
                    }
                }
                // The event's occurrences, lowered through the SAME path a
                // stream's schedule takes — one schedule sub-language, one
                // lowering, so the two cannot drift.
                let event_schedule = match event.schedule.as_ref() {
                    Some(spec) => match lower_schedule(
                        Some(spec),
                        time_calendar,
                        time_start,
                        timeline_end,
                        phase_map,
                    ) {
                        Ok(sched) => Some(sched),
                        Err(msg) => {
                            diags.push(diag("E5005_PHASE_NOT_FOUND", msg));
                            continue;
                        }
                    },
                    None => None,
                };
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
                let mut node = serde_json::json!({
                    "id": deterministic_id("Event", &stable, id_seed),
                    "name": event.name,
                    "actions": actions,
                    "provenance": {
                        "source_file": source_stmt.file,
                        "source_span": map_span(event.span),
                    },
                });
                // Both clauses are optional and at least one is present, so
                // each is emitted only when the model wrote it — an absent
                // `when` means every scheduled occurrence fires, and an absent
                // schedule means the condition's own rising edges are the
                // occurrences. Emitting a placeholder for either would make
                // the IR claim the model said something it did not.
                let obj = node.as_object_mut().expect("event node is an object");
                if let Some(sched) = event_schedule {
                    obj.insert(
                        "schedule".to_string(),
                        serde_json::to_value(sched).expect("schedule serializes"),
                    );
                }
                if let Some(when) = event.when.as_deref() {
                    obj.insert(
                        "when".to_string(),
                        serde_json::json!({ "lang": "cfdl", "src": when }),
                    );
                }
                events.push(node);
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
                // An option is a contract with an election, so it carries the
                // same two things every contract does: what it is written on,
                // and who it is between. Without an owner its payoff belonged
                // to no entity and fell out of every per-entity total.
                if let Some(subject) = &option.subject_entity {
                    obj["owner"] = serde_json::json!({ "symbol": subject });
                }
                if !option.parties.is_empty() {
                    obj["parties"] = serde_json::json!(option
                        .parties
                        .iter()
                        .map(|p| serde_json::json!({
                            "role": p.role,
                            "entity": { "symbol": p.entity },
                        }))
                        .collect::<Vec<_>>());
                }
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

/// Lower `curve` statements into IR curves: dedupe names, sort points by
/// date, reject duplicate point dates.
/// Lower the active pack's `[[subtotals]]` into IR nodes.
///
/// The pack loader has already checked shape and rejected forward references,
/// so the remaining job is the one thing only the compiler can see: whether a
/// category a subtotal folds is actually in the pack's declared vocabulary. A
/// subtotal over `operating.revenu.*` — one letter out — would otherwise fold
/// nothing, publish a series of zeros, and say so nowhere.
fn lower_subtotals(active_pack: Option<&ActivePackContext>) -> (Vec<IrSubtotal>, Vec<Diagnostic>) {
    let Some(pack) = active_pack else {
        return (vec![], vec![]);
    };
    let mut out = Vec::new();
    let mut diags = Vec::new();
    for spec in &pack.subtotal_specs {
        for selector in &spec.categories {
            let reaches_any = pack
                .categories
                .iter()
                .any(|declared| cfdl_expr::selector_matches(selector, declared));
            if !reaches_any {
                diags.push(Diagnostic {
                    code: "E5023_SUBTOTAL_UNKNOWN_CATEGORY".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "Subtotal '{}' folds category '{selector}', which matches none of the \
                         categories pack '{}' declares. It would sum nothing and publish zeros.",
                        spec.id, pack.name
                    ),
                    file: None,
                    span: None,
                    path: None,
                    hint: Some(format!(
                        "Declared categories: {}.",
                        pack.categories.join(", ")
                    )),
                    notes: vec![],
                });
            }
        }
        out.push(IrSubtotal {
            id: spec.id.clone(),
            kind: spec.kind.clone(),
            op: spec.op.clone(),
            categories: spec.categories.clone(),
            streams: spec.streams.clone(),
            subtotals: spec.subtotals.clone(),
            numerator: spec.numerator.clone(),
            denominator: spec.denominator.clone(),
            formula: spec.formula.clone(),
        });
    }
    (out, diags)
}

/// Lower `quantile` statements into IR: dedupe names, reject a malformed
/// shape, and normalise every declaration into the one canonical ascending
/// form.
///
/// The checks exist because a quantile has an axis a curve does not, and every
/// way of getting that axis wrong produces a plausible number rather than an
/// obvious failure. A share outside [0, 1] is not a share. A repeated share is
/// two answers to one question. And values must be non-decreasing in share, or
/// `quantile_of` — the inverse — has no single answer, which would make a
/// threshold lookup silently pick one.
fn lower_quantiles(
    resolve_output: &cfdl_resolver::ResolveOutput,
) -> (Vec<IrQuantile>, Vec<Diagnostic>) {
    let mut quantiles: Vec<IrQuantile> = Vec::new();
    let mut diags = Vec::new();

    for source_stmt in &resolve_output.source_statements {
        let Stmt::Quantile(q) = &source_stmt.statement else {
            continue;
        };
        let make_diag = |message: String| Diagnostic {
            code: "E5028_INVALID_QUANTILE".to_string(),
            severity: "error".to_string(),
            message,
            file: Some(source_stmt.file.clone()),
            span: Some(map_span(q.span)),
            path: None,
            hint: None,
            notes: vec![format!("quantile '{}'", q.name)],
        };

        if quantiles.iter().any(|existing| existing.name == q.name) {
            diags.push(make_diag(format!(
                "Quantile '{}' is declared more than once.",
                q.name
            )));
            continue;
        }

        let mut points: Vec<IrQuantilePoint> = Vec::with_capacity(q.points.len());
        let mut bad = false;
        for (share_lit, value_lit) in &q.points {
            let (Ok(share), Ok(value)) = (share_lit.parse::<f64>(), value_lit.parse::<f64>())
            else {
                diags.push(make_diag(format!(
                    "Quantile '{}' has a malformed point '{share_lit}: {value_lit}'.",
                    q.name
                )));
                bad = true;
                break;
            };
            if !(0.0..=1.0).contains(&share) {
                diags.push(make_diag(format!(
                    "Quantile '{}' has share {share}, which is outside 0..1. A share is a \
                     fraction of the measure, and the measure itself belongs to the contract \
                     that reads it.",
                    q.name
                )));
                bad = true;
                break;
            }
            points.push(IrQuantilePoint { share, value });
        }
        if bad {
            continue;
        }

        // `by exceedance` is written worst-first. Reversing here is what keeps
        // the IR to one orientation.
        if q.order == "exceedance" {
            points.reverse();
        }

        if let Some(w) = points.windows(2).find(|w| w[0].share >= w[1].share) {
            diags.push(make_diag(format!(
                "Quantile '{}' has shares {} and {} out of order or repeated. Points must be \
                 strictly increasing in share once read in the declared order.",
                q.name, w[0].share, w[1].share
            )));
            continue;
        }
        if let Some(w) = points.windows(2).find(|w| w[0].value > w[1].value) {
            diags.push(make_diag(format!(
                "Quantile '{}' falls from {} to {} as share increases. A quantile function is \
                 non-decreasing; without that, `quantile_of` has no single answer and a \
                 threshold lookup would silently pick one of several.",
                q.name, w[0].value, w[1].value
            )));
            continue;
        }

        quantiles.push(IrQuantile {
            name: q.name.clone(),
            interpolation: q.interpolation.clone(),
            reference: q.reference.clone(),
            points,
        });
    }

    quantiles.sort_by(|a, b| a.name.cmp(&b.name));
    (quantiles, diags)
}

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
            placement: None,
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
            anchor_entity: None,
            anchor_state: None,
            anchor_periods: None,
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
            placement: placement_of_parsed(false, schedule.mid, schedule.at_period_end),
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
            anchor_entity: None,
            anchor_state: None,
            anchor_periods: None,
        }),
        ScheduleKind::StateEnter {
            entity,
            state,
            periods,
        } => Ok(IrSchedule {
            kind: "StateEnter".to_string(),
            placement: placement_of_parsed(schedule.due, schedule.mid, schedule.at_period_end),
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
            // No dates: the windows open where the WALK finds the entries,
            // and a re-entered state re-anchors (`docs/28` §6.2).
            from: None,
            to: None,
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
            anchor_entity: Some(entity.clone()),
            anchor_state: Some(state.clone()),
            anchor_periods: Some(*periods),
        }),
        ScheduleKind::Every => Ok(IrSchedule {
            kind: "Every".to_string(),
            placement: placement_of_parsed(schedule.due, schedule.mid, schedule.at_period_end),
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
            anchor_entity: None,
            anchor_state: None,
            anchor_periods: None,
        }),
        ScheduleKind::PhaseEnter { phase } => {
            let (start, _end) = phase_map.get(phase).ok_or_else(|| {
                format!("Schedule references unknown phase '{phase}'; no matching phase declaration found.")
            })?;
            Ok(IrSchedule {
                kind: "OnDate".to_string(),
                placement: None,
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
                anchor_entity: None,
                anchor_state: None,
                anchor_periods: None,
            })
        }
        ScheduleKind::EveryPhase { phase } => {
            let (start, end) = phase_map.get(phase).ok_or_else(|| {
                format!("Schedule references unknown phase '{phase}'; no matching phase declaration found.")
            })?;
            Ok(IrSchedule {
                kind: "Every".to_string(),
                placement: placement_of_parsed(schedule.due, schedule.mid, schedule.at_period_end),
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
                anchor_entity: None,
                anchor_state: None,
                anchor_periods: None,
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

/// A contract term as the IR records it — a typed value, not a spliced
/// string. A literal keeps its kind (a number is a number, a quoted string is
/// a string, a date or a bare name is a string); an input reference or an
/// expression is carried as CFDL source, the way every other expression in
/// the IR is.
fn term_value_json(term: &cfdl_parser::ContractTerm) -> serde_json::Value {
    match term.kind {
        cfdl_parser::TermValueKind::Literal => {
            let raw = term.value.trim();
            if let Ok(int) = raw.parse::<i64>() {
                return serde_json::Value::from(int);
            }
            if let Ok(float) = raw.parse::<f64>() {
                if let Some(number) = serde_json::Number::from_f64(float) {
                    return serde_json::Value::Number(number);
                }
            }
            match raw {
                "true" => serde_json::Value::Bool(true),
                "false" => serde_json::Value::Bool(false),
                quoted if quoted.len() >= 2 && quoted.starts_with('"') && quoted.ends_with('"') => {
                    serde_json::Value::String(quoted[1..quoted.len() - 1].to_string())
                }
                other => serde_json::Value::String(other.to_string()),
            }
        }
        cfdl_parser::TermValueKind::InputRef | cfdl_parser::TermValueKind::Expr => {
            serde_json::json!({ "lang": "cfdl", "src": term.value })
        }
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
                    kind: cfdl_parser::ContractTerm::classify_atomic(value),
                    unit: None,
                    span: span(),
                },
            );
        }
        cfdl_parser::ContractStmt {
            payment_net: None,
            name: name.to_string(),
            declared_type: None,
            declared_type_span: None,
            instance: None,
            subject_entity: None,
            has_term: term_range,
            has_effects: false,
            term_start: term_range.then(|| "2026-01".to_string()),
            term_end: term_range.then(|| "2026-06".to_string()),
            terms: map,
            category: None,
            stream_categories: Default::default(),
            parties: vec![],
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
            subtotal_specs: registry.subtotal_specs(pack),
            lowering_rules: registry.lowering_rules(pack),
            validations: registry.validations(pack),
            ontology: registry
                .ontology(pack)
                .map(|o| o.merged_with_base())
                .unwrap_or_else(cfdl_pack::PackOntology::language_base),
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
            let mut terms = vec![("income", "180000")];
            if let Some(v) = value {
                terms.push(("cap_rate", v));
            }
            assert_parity("cre", "cre.exit_cap", &terms, true);
        }
        assert_parity(
            "cre",
            "cre.exit_cap",
            &[("cap_rate", "0.06"), ("income", "1")],
            true,
        );
        assert_parity("cre", "cre.exit_cap", &[("cap_rate", "0.06")], true);
    }

    #[test]
    fn cre_ops_cases() {
        for name in ["cre.ops_revenue", "cre.opex_line"] {
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
