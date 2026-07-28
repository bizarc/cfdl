//! CFDL parser skeleton for v0.1.
//!
//! Milestone 2 scope:
//! - `version` statement
//! - `model` statement
//! - `time calendar <cadence> from <date> for <int>` statement
//! - parser diagnostics: E0001 + E0004 with file/span

pub use cfdl_lexer::Span;
use cfdl_lexer::{Keyword, Punct, Token, TokenKind};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CompilationUnit {
    pub statements: Vec<Stmt>,
    pub span: Span,
}

pub type ModelAst = CompilationUnit;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[allow(clippy::large_enum_variant)]
pub enum Stmt {
    Version(VersionStmt),
    Model(ModelStmt),
    UsePack(UsePackStmt),
    Import(ImportStmt),
    Time(TimeStmt),
    Phase(PhaseStmt),
    Entity(EntityStmt),
    Assume(AssumeStmt),
    Curve(CurveStmt),
    Contract(ContractStmt),
    Stream(StreamStmt),
    Event(EventStmt),
    Option(OptionStmt),
    Run(RunStmt),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RunStmt {
    /// "deterministic" | "monte_carlo"
    pub kind: String,
    pub trials: Option<u64>,
    pub seed: Option<u64>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct VersionStmt {
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ModelStmt {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ImportStmt {
    pub path: String,
    pub alias: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct UsePackStmt {
    pub name: String,
    pub version: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TimeStmt {
    pub cadence: Cadence,
    pub from: String,
    pub periods: u32,
    /// Valuation projection tail (`project <n>`): extra periods computed for
    /// series lookups (e.g. forward NOI) but excluded from cash results.
    pub projection: u32,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct EntityStmt {
    pub namespace: String,
    pub name: String,
    pub span: Span,
}

impl EntityStmt {
    pub fn symbol(&self) -> String {
        format!("{}.{}", self.namespace, self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct StreamStmt {
    pub name: String,
    pub attached_entity: String,
    /// Optional: "inflow" or "outflow". Default when lowering is "outflow".
    pub direction: Option<String>,
    /// Optional: currency code (e.g. "USD"). Default when lowering is model currency.
    pub currency: Option<String>,
    pub schedule: Option<ScheduleSpec>,
    pub amount: Option<ExprSlot>,
    pub active_when: Option<ExprSlot>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ExprSlot {
    pub lang: String,
    pub src: String,
    /// Span of the whole slot (statement keyword through expression end).
    pub span: Span,
    /// Span of the expression text itself — `src` is the source slice this
    /// covers, so expression-internal byte offsets map into it.
    pub expr_span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PhaseStmt {
    pub name: String,
    pub from: String,
    pub to: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AssumeStmt {
    pub name: String,
    /// Deterministic form: `assume x = <expr>` (raw expression source).
    pub value: Option<String>,
    /// Stochastic form: `assume x ~ Dist(...)`.
    pub dist: Option<AssumeDist>,
    pub span: Span,
}

/// `curve <name> [step|linear] { <date>: <number>, ... }` — a named
/// date-indexed value curve (e.g. a forward rate curve), looked up in
/// expressions with `curve_value("<name>", <date>)`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CurveStmt {
    pub name: String,
    /// "step" (flat-forward, default) or "linear".
    pub interpolation: String,
    /// (date literal, numeric literal) pairs in source order.
    pub points: Vec<(String, String)>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AssumeDist {
    /// "normal" | "lognormal" | "uniform" | "triangular"
    pub name: String,
    /// Named numeric args in source order, e.g. [("mean", "0.03"), ("stdev", "0.01")].
    pub args: Vec<(String, String)>,
    /// Optional `clip=[lo, hi]`.
    pub clip: Option<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ContractStmt {
    pub name: String,
    pub subject_entity: Option<String>,
    pub has_term: bool,
    pub has_effects: bool,
    pub term_start: Option<String>,
    pub term_end: Option<String>,
    pub terms: BTreeMap<String, ContractTerm>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ContractTerm {
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ScheduleSpec {
    pub kind: ScheduleKind,
    /// Recurrence interval as written (`day`/`week`/`month`/`quarter`/`year`).
    /// Distinct from the model's calendar cadence: a stream may pay quarterly
    /// on a monthly grid. `None` for non-recurring kinds.
    pub every: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub day_of_month: Option<i32>,
    /// `on eom` — place the occurrence on the last day of its period.
    pub end_of_month: bool,
    /// Business-day roll convention: none/following/modified_following/
    /// preceding/modified_preceding.
    pub convention: Option<String>,
    /// Holiday calendar name (e.g. "us", "target", "uk", "weekend").
    pub calendar: Option<String>,
    /// Dates removed from the schedule (`except [d1, d2]`).
    pub except_dates: Vec<String>,
    /// Dates added to the schedule (`also [d1, d2]`).
    pub also_dates: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum ScheduleKind {
    OnDate,
    Every,
    PhaseEnter { phase: String },
    EveryPhase { phase: String },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct EventStmt {
    pub name: String,
    /// Boolean trigger expression (raw source).
    pub when: String,
    pub actions: Vec<EventAction>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum EventAction {
    /// `set entity <ns.name>.<field> = <expr>`
    SetEntityField {
        entity: String,
        field: String,
        value: String,
    },
    ActivateStream(String),
    DeactivateStream(String),
    ActivateContract(String),
    DeactivateContract(String),
    ExerciseOption(String),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct OptionStmt {
    pub name: String,
    pub type_name: String,
    pub exercisable_in: Option<String>,
    /// Boolean trigger expression (raw source).
    pub exercise_when: Option<String>,
    /// Payoff amount expression (raw source).
    pub payoff: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Cadence {
    Daily,
    Monthly,
    Quarterly,
    Annual,
}

/// Expression stop classes for `consume_expr_until`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokStopKind {
    /// Stop at `{` (event trigger before the action block).
    LBrace,
    /// Stop at action keywords (set/activate/deactivate/exercise).
    Action,
    /// Stop at `payoff`.
    Payoff,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDiagnostic {
    pub code: &'static str,
    pub message: String,
    pub file: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseResult {
    pub ast: Option<ModelAst>,
    pub diagnostics: Vec<ParseDiagnostic>,
}

pub fn parse(file: &str, source: &str, tokens: &[Token]) -> ParseResult {
    let mut parser = Parser::new(file, source, tokens);
    let ast = parser.parse_compilation_unit();
    let has_errors = !parser.diagnostics.is_empty();
    ParseResult {
        ast: if has_errors { None } else { Some(ast) },
        diagnostics: parser.diagnostics,
    }
}

struct Parser<'a> {
    file: String,
    tokens: &'a [Token],
    idx: usize,
    diagnostics: Vec<ParseDiagnostic>,
    /// Source lines, used to slice raw expression text by token spans.
    lines: Vec<Vec<char>>,
}

impl<'a> Parser<'a> {
    fn new(file: &str, source: &str, tokens: &'a [Token]) -> Self {
        Self {
            file: file.to_string(),
            tokens,
            idx: 0,
            diagnostics: Vec::new(),
            lines: source
                .lines()
                .map(|l| l.trim_end_matches('\r').chars().collect())
                .collect(),
        }
    }

    /// Slice the raw source text covered by `span` (lines/cols are 1-based,
    /// end_col inclusive — the lexer's span convention).
    fn slice_source(&self, span: Span) -> String {
        let mut out = String::new();
        for line_no in span.start_line..=span.end_line {
            let Some(line) = self.lines.get(line_no as usize - 1) else {
                continue;
            };
            let from = if line_no == span.start_line {
                span.start_col as usize - 1
            } else {
                0
            };
            let to = if line_no == span.end_line {
                (span.end_col as usize).min(line.len())
            } else {
                line.len()
            };
            if line_no != span.start_line {
                out.push(' ');
            }
            if from < to {
                out.extend(&line[from..to]);
            }
        }
        out.trim().to_string()
    }

    fn parse_compilation_unit(&mut self) -> CompilationUnit {
        let mut statements = Vec::new();
        while !self.is_eof() {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            } else {
                self.synchronize_to_next_statement();
            }
        }

        let span = if statements.is_empty() {
            self.current_span()
        } else {
            let start = statement_span(&statements[0]);
            let end = statement_span(statements.last().expect("non-empty statements"));
            Span {
                start_line: start.start_line,
                start_col: start.start_col,
                end_line: end.end_line,
                end_col: end.end_col,
            }
        };

        CompilationUnit { statements, span }
    }

    fn parse_statement(&mut self) -> Option<Stmt> {
        match self.peek().kind {
            TokenKind::Keyword(Keyword::Version) => self.parse_version_stmt().map(Stmt::Version),
            TokenKind::Keyword(Keyword::Model) => self.parse_model_stmt().map(Stmt::Model),
            TokenKind::Keyword(Keyword::Use) => self.parse_use_pack_stmt().map(Stmt::UsePack),
            TokenKind::Keyword(Keyword::Import) => self.parse_import_stmt().map(Stmt::Import),
            TokenKind::Keyword(Keyword::Time) => self.parse_time_stmt().map(Stmt::Time),
            TokenKind::Keyword(Keyword::Phase) => self.parse_phase_stmt().map(Stmt::Phase),
            TokenKind::Keyword(Keyword::Entity) => self.parse_entity_stmt().map(Stmt::Entity),
            TokenKind::Keyword(Keyword::Assume) => self.parse_assume_stmt().map(Stmt::Assume),
            TokenKind::Keyword(Keyword::Curve) => self.parse_curve_stmt().map(Stmt::Curve),
            TokenKind::Keyword(Keyword::Event) => self.parse_event_stmt().map(Stmt::Event),
            TokenKind::Keyword(Keyword::Option) => self.parse_option_stmt().map(Stmt::Option),
            TokenKind::Keyword(Keyword::Run) => self.parse_run_stmt().map(Stmt::Run),
            TokenKind::Keyword(Keyword::Contract) => self.parse_contract_stmt().map(Stmt::Contract),
            TokenKind::Keyword(Keyword::Stream) => self.parse_stream_stmt().map(Stmt::Stream),
            TokenKind::Eof => None,
            _ => {
                let found = token_label(self.peek());
                self.push_unexpected(self.current_span(), format!("Unexpected token {found}."));
                let _ = self.bump();
                None
            }
        }
    }

    fn parse_version_stmt(&mut self) -> Option<VersionStmt> {
        let start = self.expect_keyword(Keyword::Version, "'version'")?;
        let value_tok = self.bump();
        match value_tok.kind {
            TokenKind::Number(ref n) => Some(VersionStmt {
                value: n.clone(),
                span: merge_spans(start.span, value_tok.span),
            }),
            _ => {
                self.push_expected(
                    value_tok.span,
                    "Expected token <number> after 'version'.".to_string(),
                );
                None
            }
        }
    }

    fn parse_model_stmt(&mut self) -> Option<ModelStmt> {
        let start = self.expect_keyword(Keyword::Model, "'model'")?;
        let name_tok = self.bump();
        match name_tok.kind {
            TokenKind::String(ref s) => Some(ModelStmt {
                name: s.clone(),
                span: merge_spans(start.span, name_tok.span),
            }),
            _ => {
                self.push_expected(
                    name_tok.span,
                    "Expected token <string> after 'model'.".to_string(),
                );
                None
            }
        }
    }

    fn parse_time_stmt(&mut self) -> Option<TimeStmt> {
        let start = self.expect_keyword(Keyword::Time, "'time'")?;
        let _calendar_kw = self.expect_keyword(Keyword::Calendar, "'calendar'")?;
        let cadence_tok = self.bump();
        let cadence = match cadence_tok.kind {
            TokenKind::Keyword(Keyword::Daily) => Cadence::Daily,
            TokenKind::Keyword(Keyword::Monthly) => Cadence::Monthly,
            TokenKind::Keyword(Keyword::Quarterly) => Cadence::Quarterly,
            TokenKind::Keyword(Keyword::Annual) => Cadence::Annual,
            _ => {
                self.push_expected(
                    cadence_tok.span,
                    "Expected token <frequency> after 'time calendar'.".to_string(),
                );
                return None;
            }
        };
        let _from_kw = self.expect_keyword(Keyword::From, "'from'")?;
        let from_tok = self.bump();
        let from = match from_tok.kind {
            TokenKind::Date(ref d) => d.clone(),
            _ => {
                self.push_expected(
                    from_tok.span,
                    "Expected token <date> after 'from'.".to_string(),
                );
                return None;
            }
        };
        let _for_kw = self.expect_keyword(Keyword::For, "'for'")?;
        let periods_tok = self.bump();
        let periods = match periods_tok.kind {
            TokenKind::Number(ref n) => match n.parse::<u32>() {
                Ok(value) => value,
                Err(_) => {
                    self.push_expected(
                        periods_tok.span,
                        "Expected token <int> after 'for'.".to_string(),
                    );
                    return None;
                }
            },
            _ => {
                self.push_expected(
                    periods_tok.span,
                    "Expected token <int> after 'for'.".to_string(),
                );
                return None;
            }
        };
        let mut projection = 0_u32;
        let mut end_span = periods_tok.span;
        // Contextual keyword: `project` stays a plain identifier elsewhere
        // (e.g. `entity project microgrid`).
        if matches!(self.peek().kind, TokenKind::Ident(ref word) if word == "project") {
            let _ = self.bump();
            let proj_tok = self.bump();
            match proj_tok.kind {
                TokenKind::Number(ref n) => match n.parse::<u32>() {
                    Ok(value) => {
                        projection = value;
                        end_span = proj_tok.span;
                    }
                    Err(_) => {
                        self.push_expected(
                            proj_tok.span,
                            "Expected token <int> after 'project'.".to_string(),
                        );
                        return None;
                    }
                },
                _ => {
                    self.push_expected(
                        proj_tok.span,
                        "Expected token <int> after 'project'.".to_string(),
                    );
                    return None;
                }
            }
        }
        Some(TimeStmt {
            cadence,
            from,
            periods,
            projection,
            span: merge_spans(start.span, end_span),
        })
    }

    fn parse_use_pack_stmt(&mut self) -> Option<UsePackStmt> {
        let start = self.expect_keyword(Keyword::Use, "'use'")?;
        let _pack_kw = self.expect_keyword(Keyword::Pack, "'pack'")?;
        let pack_name_tok = self.bump();
        let pack_name = match pack_name_tok.kind {
            TokenKind::String(ref s) => s.clone(),
            _ => {
                self.push_expected(
                    pack_name_tok.span,
                    "Expected token <string> after 'use pack'.".to_string(),
                );
                return None;
            }
        };
        let _version_kw = self.expect_keyword(Keyword::Version, "'version'")?;
        let version_tok = self.bump();
        let version = match version_tok.kind {
            TokenKind::String(ref s) => s.clone(),
            _ => {
                self.push_expected(
                    version_tok.span,
                    "Expected token <string> after 'version'.".to_string(),
                );
                return None;
            }
        };
        Some(UsePackStmt {
            name: pack_name,
            version,
            span: merge_spans(start.span, version_tok.span),
        })
    }

    fn parse_import_stmt(&mut self) -> Option<ImportStmt> {
        let start = self.expect_keyword(Keyword::Import, "'import'")?;
        let path_tok = self.bump();
        let path = match path_tok.kind {
            TokenKind::String(ref s) => s.clone(),
            _ => {
                self.push_expected(
                    path_tok.span,
                    "Expected token <string> after 'import'.".to_string(),
                );
                return None;
            }
        };

        let mut alias = None;
        let mut end_span = path_tok.span;
        if matches!(self.peek().kind, TokenKind::Keyword(Keyword::As)) {
            let _as_kw = self.bump();
            let alias_tok = self.bump();
            match alias_tok.kind {
                TokenKind::Ident(ref ident) => {
                    alias = Some(ident.clone());
                    end_span = alias_tok.span;
                }
                _ => {
                    self.push_expected(
                        alias_tok.span,
                        "Expected token <identifier> after 'as'.".to_string(),
                    );
                    return None;
                }
            }
        }

        Some(ImportStmt {
            path,
            alias,
            span: merge_spans(start.span, end_span),
        })
    }

    fn parse_entity_stmt(&mut self) -> Option<EntityStmt> {
        let start = self.expect_keyword(Keyword::Entity, "'entity'")?;
        let namespace_tok = self.bump();
        let namespace = match namespace_tok.kind {
            TokenKind::Ident(ref ident) => ident.clone(),
            _ => {
                self.push_expected(
                    namespace_tok.span,
                    "Expected token <identifier> after 'entity'.".to_string(),
                );
                return None;
            }
        };

        let name_tok = self.bump();
        let name = match name_tok.kind {
            TokenKind::Ident(ref ident) => ident.clone(),
            _ => {
                self.push_expected(
                    name_tok.span,
                    "Expected token <identifier> for entity name.".to_string(),
                );
                return None;
            }
        };

        Some(EntityStmt {
            namespace,
            name,
            span: merge_spans(start.span, name_tok.span),
        })
    }

    fn parse_phase_stmt(&mut self) -> Option<PhaseStmt> {
        let start = self.expect_keyword(Keyword::Phase, "'phase'")?;
        let name_tok = self.bump();
        let name = match name_tok.kind {
            TokenKind::Ident(ref ident) => ident.clone(),
            _ => {
                self.push_expected(
                    name_tok.span,
                    "Expected token <identifier> after 'phase'.".to_string(),
                );
                return None;
            }
        };
        let _from_kw = self.expect_keyword(Keyword::From, "'from'")?;
        let from_tok = self.bump();
        let from = match from_tok.kind {
            TokenKind::Date(ref d) => d.clone(),
            _ => {
                self.push_expected(
                    from_tok.span,
                    "Expected token <date> after 'from'.".to_string(),
                );
                return None;
            }
        };
        let _to_kw = self.expect_keyword(Keyword::To, "'to'")?;
        let to_tok = self.bump();
        let to = match to_tok.kind {
            TokenKind::Date(ref d) => d.clone(),
            _ => {
                self.push_expected(to_tok.span, "Expected token <date> after 'to'.".to_string());
                return None;
            }
        };

        Some(PhaseStmt {
            name,
            from,
            to,
            span: merge_spans(start.span, to_tok.span),
        })
    }

    fn parse_contract_stmt(&mut self) -> Option<ContractStmt> {
        let start = self.expect_keyword(Keyword::Contract, "'contract'")?;
        let mut name: Option<String> = None;
        let mut name_span: Option<Span> = None;
        let mut subject_entity: Option<String> = None;
        let mut has_term = false;
        let mut has_effects = false;
        let mut term_start = None;
        let mut term_end = None;
        let mut terms = BTreeMap::new();
        let mut end_span = start.span;
        let mut depth = 0usize;

        // Parse leading contract signature:
        // - Legacy form: contract <name> { ... }
        // - Typed form:  contract <type_id> <name> ...
        if let Some(first_head) = self.parse_name_like_token() {
            if let Some(second_head) = self.parse_name_like_token() {
                name = Some(second_head.0);
                name_span = Some(second_head.1);
                end_span = second_head.1;
                let _ = first_head;
            } else {
                name = Some(first_head.0);
                name_span = Some(first_head.1);
                end_span = first_head.1;
            }
        }

        while !self.is_eof() {
            if depth == 0 && is_statement_start(self.peek()) {
                break;
            }

            let tok = self.bump();
            end_span = tok.span;
            match tok.kind {
                TokenKind::Keyword(Keyword::Term) => {
                    has_term = true;
                    if let Some((from, to, span)) = self.parse_contract_term_range() {
                        term_start = Some(from);
                        term_end = Some(to);
                        end_span = span;
                    }
                }
                TokenKind::Keyword(Keyword::Terms) => {
                    if let Some((parsed_terms, span)) = self.parse_contract_terms_block() {
                        for (key, value) in parsed_terms {
                            terms.insert(key, value);
                        }
                        end_span = span;
                    }
                }
                TokenKind::Keyword(Keyword::Effects) => has_effects = true,
                TokenKind::Keyword(Keyword::On) if depth == 0 => {
                    let entity_kw = self.bump();
                    if !matches!(entity_kw.kind, TokenKind::Keyword(Keyword::Entity)) {
                        self.push_expected(
                            entity_kw.span,
                            "Expected token 'entity' after 'on'.".to_string(),
                        );
                        continue;
                    }
                    let entity_ref_tok = self.bump();
                    if let Some(entity_ref) = self.parse_entity_ref_token(&entity_ref_tok) {
                        subject_entity = Some(entity_ref);
                        end_span = entity_ref_tok.span;
                    }
                }
                TokenKind::Punct(Punct::LBrace) => depth += 1,
                TokenKind::Punct(Punct::RBrace) => depth = depth.saturating_sub(1),
                _ => {}
            }
        }

        let final_name = name.unwrap_or_else(|| "contract".to_string());
        if let Some(span) = name_span {
            if final_name != "contract" && !is_valid_entity_ref(&final_name) {
                self.push_expected(
                    span,
                    "Contract name must be a dotted qualified name with at least two segments (e.g. cre.lease.primary).".to_string(),
                );
            }
        }

        Some(ContractStmt {
            name: final_name,
            subject_entity,
            has_term,
            has_effects,
            term_start,
            term_end,
            terms,
            span: merge_spans(start.span, end_span),
        })
    }

    fn parse_contract_term_range(&mut self) -> Option<(String, String, Span)> {
        let from_tok = self.peek().clone();
        let from = match from_tok.kind {
            TokenKind::Date(ref d) => d.clone(),
            _ => return None,
        };
        let _ = self.bump();
        if !matches!(self.peek().kind, TokenKind::Punct(Punct::DotDot)) {
            return None;
        }
        let _ = self.bump();
        let to_tok = self.peek().clone();
        let to = match to_tok.kind {
            TokenKind::Date(ref d) => d.clone(),
            _ => return None,
        };
        let _ = self.bump();
        Some((from, to, merge_spans(from_tok.span, to_tok.span)))
    }

    fn parse_contract_terms_block(&mut self) -> Option<(BTreeMap<String, ContractTerm>, Span)> {
        if !matches!(self.peek().kind, TokenKind::Punct(Punct::LBrace)) {
            return None;
        }
        let lbrace = self.bump();
        let mut depth = 1usize;
        let mut terms = BTreeMap::new();
        let mut end_span = lbrace.span;

        while !self.is_eof() && depth > 0 {
            let tok = self.bump();
            end_span = tok.span;
            match tok.kind {
                TokenKind::Punct(Punct::LBrace) => depth += 1,
                TokenKind::Punct(Punct::RBrace) => depth = depth.saturating_sub(1),
                TokenKind::Ident(ref key) | TokenKind::Qname(ref key) if depth == 1 => {
                    if !matches!(self.peek().kind, TokenKind::Punct(Punct::Equal)) {
                        continue;
                    }
                    let _ = self.bump();
                    // A signed number lexes as a sign punct followed by the
                    // number. Without this, `escalation = -0.02` silently
                    // dropped the whole term and the pack default applied —
                    // the model said one thing and the engine did another.
                    let sign = match self.peek().kind {
                        TokenKind::Punct(Punct::Minus) => {
                            let _ = self.bump();
                            "-"
                        }
                        TokenKind::Punct(Punct::Plus) => {
                            let _ = self.bump();
                            ""
                        }
                        _ => "",
                    };
                    let value_tok = self.bump();
                    let value = match value_tok.kind {
                        TokenKind::String(ref s) => s.clone(),
                        TokenKind::Number(ref n) => format!("{sign}{n}"),
                        TokenKind::Date(ref d) => d.clone(),
                        TokenKind::Ident(ref ident) => ident.clone(),
                        TokenKind::Qname(ref qname) => qname.clone(),
                        TokenKind::Keyword(Keyword::True) => "true".to_string(),
                        TokenKind::Keyword(Keyword::False) => "false".to_string(),
                        _ => continue,
                    };
                    end_span = value_tok.span;
                    terms.insert(
                        key.clone(),
                        ContractTerm {
                            value,
                            span: merge_spans(tok.span, value_tok.span),
                        },
                    );
                }
                _ => {}
            }
        }

        Some((terms, end_span))
    }

    fn parse_stream_stmt(&mut self) -> Option<StreamStmt> {
        let start = self.expect_keyword(Keyword::Stream, "'stream'")?;
        let name_tok = self.bump();
        let name = match &name_tok.kind {
            TokenKind::Qname(qname) => {
                if !is_valid_entity_ref(qname) {
                    self.push_expected(
                        name_tok.span,
                        "Stream name must be a dotted qualified name with at least two segments (e.g. cre.lease.rent).".to_string(),
                    );
                    return None;
                }
                qname.clone()
            }
            TokenKind::Ident(_) => {
                self.push_expected(
                    name_tok.span,
                    "Stream name must be a dotted qualified name (e.g. cre.lease.rent), not a single identifier.".to_string(),
                );
                return None;
            }
            _ => {
                self.push_expected(
                    name_tok.span,
                    "Expected token <qname> after 'stream'; stream name must be a dotted qualified name.".to_string(),
                );
                return None;
            }
        };

        let _on_kw = self.expect_keyword(Keyword::On, "'on'")?;
        let _entity_kw = self.expect_keyword(Keyword::Entity, "'entity'")?;
        let entity_ref_tok = self.bump();
        let attached_entity = self.parse_entity_ref_token(&entity_ref_tok)?;

        let mut direction = None;
        if matches!(self.peek().kind, TokenKind::Keyword(Keyword::Inflow)) {
            let _ = self.bump();
            direction = Some("inflow".to_string());
        } else if matches!(self.peek().kind, TokenKind::Keyword(Keyword::Outflow)) {
            let _ = self.bump();
            direction = Some("outflow".to_string());
        }

        let mut currency = None;
        if matches!(self.peek().kind, TokenKind::Keyword(Keyword::Currency)) {
            let _ = self.bump();
            let curr_tok = self.bump();
            if let TokenKind::Ident(ref c) = curr_tok.kind {
                currency = Some(c.clone());
            }
        }

        let mut schedule = None;
        let mut amount = None;
        let mut active_when = None;
        let mut end_span = entity_ref_tok.span;

        if matches!(self.peek().kind, TokenKind::Punct(Punct::LBrace)) {
            let (parsed_schedule, parsed_amount, parsed_active_when, parsed_end_span) =
                self.parse_stream_block();
            schedule = parsed_schedule;
            amount = parsed_amount;
            active_when = parsed_active_when;
            end_span = parsed_end_span;
        }

        Some(StreamStmt {
            name,
            attached_entity,
            direction,
            currency,
            schedule,
            amount,
            active_when,
            span: merge_spans(start.span, end_span),
        })
    }

    fn parse_stream_block(
        &mut self,
    ) -> (
        Option<ScheduleSpec>,
        Option<ExprSlot>,
        Option<ExprSlot>,
        Span,
    ) {
        let lbrace = self.bump();
        let mut schedule = None;
        let mut amount = None;
        let mut active_when = None;
        let mut end_span = lbrace.span;

        while !self.is_eof() {
            let tok = self.peek().clone();
            match tok.kind {
                TokenKind::Punct(Punct::RBrace) => {
                    end_span = self.bump().span;
                    break;
                }
                TokenKind::Keyword(Keyword::Schedule) => {
                    let _ = self.bump();
                    let parsed = self.parse_schedule_expr();
                    if let Some(spec) = parsed {
                        end_span = spec.span;
                        schedule = Some(spec);
                    }
                }
                TokenKind::Ident(ref ident) if ident == "amount" => {
                    if let Some(expr) = self.parse_amount_stmt() {
                        end_span = expr.span;
                        amount = Some(expr);
                    } else {
                        end_span = self.consume_stream_item();
                    }
                }
                TokenKind::Keyword(Keyword::Active) => {
                    if let Some(expr) = self.parse_active_stmt() {
                        end_span = expr.span;
                        active_when = Some(expr);
                    } else {
                        end_span = self.consume_stream_item();
                    }
                }
                _ => {
                    end_span = self.bump().span;
                }
            }
        }

        (schedule, amount, active_when, end_span)
    }

    fn parse_amount_stmt(&mut self) -> Option<ExprSlot> {
        let amount_tok = self.bump();
        match amount_tok.kind {
            TokenKind::Ident(ref ident) if ident == "amount" => {
                // Canonical form: `amount = <expression>`
                if !matches!(self.peek().kind, TokenKind::Punct(Punct::Equal)) {
                    self.push_expected(
                        self.current_span(),
                        "Expected '=' after 'amount'.".to_string(),
                    );
                    return None;
                }
                let _ = self.bump();
                self.parse_expr_slot(amount_tok.span)
            }
            _ => None,
        }
    }

    fn parse_active_stmt(&mut self) -> Option<ExprSlot> {
        let active_tok = self.bump();
        match active_tok.kind {
            TokenKind::Keyword(Keyword::Active) => {
                let when_tok = self.bump();
                match when_tok.kind {
                    TokenKind::Keyword(Keyword::When) => self.parse_expr_slot(active_tok.span),
                    _ => {
                        self.push_expected(
                            when_tok.span,
                            "Expected token 'when' after 'active'.".to_string(),
                        );
                        None
                    }
                }
            }
            _ => None,
        }
    }

    /// Parse a bare native expression: consume tokens until a statement
    /// delimiter (`schedule`, `active`, `amount`, `}`, EOF) and slice the raw
    /// source text they cover. The expression itself is validated by the
    /// cfdl-calc parser at compile time (EXPR_PARSE diagnostics with spans).
    fn parse_expr_slot(&mut self, start_span: Span) -> Option<ExprSlot> {
        let mut first: Option<Span> = None;
        let mut last: Option<Span> = None;
        loop {
            match self.peek().kind {
                TokenKind::Eof
                | TokenKind::Punct(Punct::RBrace)
                | TokenKind::Keyword(Keyword::Schedule)
                | TokenKind::Keyword(Keyword::Active) => break,
                TokenKind::Ident(ref ident) if ident == "amount" => break,
                _ => {
                    let tok = self.bump();
                    if first.is_none() {
                        first = Some(tok.span);
                    }
                    last = Some(tok.span);
                }
            }
        }
        let (Some(first), Some(last)) = (first, last) else {
            self.push_expected(self.current_span(), "Expected expression.".to_string());
            return None;
        };
        let expr_span = merge_spans(first, last);
        Some(ExprSlot {
            lang: "cfdl".to_string(),
            src: self.slice_source(expr_span),
            span: merge_spans(start_span, expr_span),
            expr_span,
        })
    }

    fn consume_stream_item(&mut self) -> Span {
        let mut end_span = self.bump().span;
        while !self.is_eof() {
            if matches!(self.peek().kind, TokenKind::Punct(Punct::RBrace))
                || matches!(self.peek().kind, TokenKind::Keyword(Keyword::Schedule))
                || matches!(self.peek().kind, TokenKind::Keyword(Keyword::Active))
                || matches!(self.peek().kind, TokenKind::Ident(ref ident) if ident == "amount")
            {
                break;
            }
            end_span = self.bump().span;
        }
        end_span
    }

    /// The interval after `every`, e.g. the `month` in `every month`.
    ///
    /// Required by the grammar. It used to be optional in practice — the token
    /// was bumped and discarded, so `every from … to …` parsed and the
    /// compiler substituted the model calendar. Requiring it here is what
    /// makes the declared interval trustworthy downstream.
    fn parse_schedule_interval(&mut self) -> Option<String> {
        let tok = self.peek().clone();
        let interval = match tok.kind {
            TokenKind::Keyword(Keyword::Day) => "day",
            TokenKind::Keyword(Keyword::Week) => "week",
            TokenKind::Keyword(Keyword::Month) => "month",
            TokenKind::Keyword(Keyword::Quarter) => "quarter",
            TokenKind::Keyword(Keyword::Year) => "year",
            _ => {
                self.push_expected(
                    tok.span,
                    "Expected an interval after 'every': day, week, month, quarter or year."
                        .to_string(),
                );
                return None;
            }
        };
        let _ = self.bump();
        Some(interval.to_string())
    }

    fn parse_schedule_expr(&mut self) -> Option<ScheduleSpec> {
        let start = self.current_span();
        match self.peek().kind {
            TokenKind::Keyword(Keyword::On) => {
                let _ = self.bump();
                if matches!(self.peek().kind, TokenKind::Keyword(Keyword::PhaseEnter)) {
                    let _ = self.bump();
                    let _ = self.expect_punct(Punct::LParen, "'('")?;
                    let phase_tok = self.bump();
                    let phase = match phase_tok.kind {
                        TokenKind::String(ref s) => s.clone(),
                        _ => {
                            self.push_expected(
                                phase_tok.span,
                                "Expected token <string> for phase name.".to_string(),
                            );
                            return None;
                        }
                    };
                    let end_tok = self.expect_punct(Punct::RParen, "')'")?;
                    let mut spec = ScheduleSpec {
                        kind: ScheduleKind::PhaseEnter { phase },
                        every: None,
                        end_of_month: false,
                        from: None,
                        to: None,
                        day_of_month: None,
                        convention: None,
                        calendar: None,
                        except_dates: Vec::new(),
                        also_dates: Vec::new(),
                        span: merge_spans(start, end_tok.span),
                    };
                    self.parse_schedule_opts(&mut spec);
                    return Some(spec);
                }

                let date_tok = self.bump();
                let date = match date_tok.kind {
                    TokenKind::Date(ref d) => d.clone(),
                    _ => {
                        self.push_expected(
                            date_tok.span,
                            "Expected token <date> after 'schedule on'.".to_string(),
                        );
                        return None;
                    }
                };
                let mut spec = ScheduleSpec {
                    kind: ScheduleKind::OnDate,
                    every: None,
                    end_of_month: false,
                    from: Some(date.clone()),
                    to: Some(date),
                    day_of_month: None,
                    convention: None,
                    calendar: None,
                    except_dates: Vec::new(),
                    also_dates: Vec::new(),
                    span: merge_spans(start, date_tok.span),
                };
                self.parse_schedule_opts(&mut spec);
                Some(spec)
            }
            TokenKind::Keyword(Keyword::Every) => {
                let _ = self.bump();
                let every = self.parse_schedule_interval()?;

                let mut day_of_month = None;
                let mut end_of_month = false;
                if matches!(self.peek().kind, TokenKind::Keyword(Keyword::On)) {
                    let _ = self.bump();
                    if matches!(self.peek().kind, TokenKind::Keyword(Keyword::Eom)) {
                        let _ = self.bump();
                        end_of_month = true;
                    } else if matches!(self.peek().kind, TokenKind::Keyword(Keyword::Day)) {
                        let _ = self.bump();
                        let day_tok = self.bump();
                        match day_tok.kind {
                            // Range is judged by the validator, which owns
                            // E2105_SCHEDULE_INVALID_DAY_OF_MONTH and reports it
                            // against the whole schedule. Parsing only insists
                            // the token is a representable integer — previously
                            // an unrepresentable one was silently dropped.
                            TokenKind::Number(ref n) => match n.parse::<i32>() {
                                Ok(value) => day_of_month = Some(value),
                                Err(_) => {
                                    self.push_expected(
                                        day_tok.span,
                                        "Day of month must be a whole number.".to_string(),
                                    );
                                    return None;
                                }
                            },
                            _ => {
                                self.push_expected(
                                    day_tok.span,
                                    "Expected token <int> after 'on day'.".to_string(),
                                );
                                return None;
                            }
                        }
                    } else {
                        let tok = self.peek().clone();
                        self.push_expected(
                            tok.span,
                            "Expected 'day <n>' or 'eom' after 'on'.".to_string(),
                        );
                        return None;
                    }
                }

                let _from_kw = self.expect_keyword(Keyword::From, "'from'")?;
                if matches!(self.peek().kind, TokenKind::Keyword(Keyword::PhaseStart)) {
                    let _ = self.bump();
                    let _ = self.expect_punct(Punct::LParen, "'('")?;
                    let phase_tok = self.bump();
                    let phase = match phase_tok.kind {
                        TokenKind::String(ref s) => s.clone(),
                        _ => {
                            self.push_expected(
                                phase_tok.span,
                                "Expected token <string> for phase name.".to_string(),
                            );
                            return None;
                        }
                    };
                    let _ = self.expect_punct(Punct::RParen, "')'")?;
                    let _to_kw = self.expect_keyword(Keyword::To, "'to'")?;
                    let _phase_end = self.expect_keyword(Keyword::PhaseEnd, "'phase_end'")?;
                    let _ = self.expect_punct(Punct::LParen, "'('")?;
                    let phase_end_tok = self.bump();
                    match phase_end_tok.kind {
                        TokenKind::String(_) => {}
                        _ => {
                            self.push_expected(
                                phase_end_tok.span,
                                "Expected token <string> for phase name.".to_string(),
                            );
                            return None;
                        }
                    }
                    let end_tok = self.expect_punct(Punct::RParen, "')'")?;
                    let mut spec = ScheduleSpec {
                        kind: ScheduleKind::EveryPhase { phase },
                        every: Some(every.clone()),
                        end_of_month,
                        from: None,
                        to: None,
                        day_of_month,
                        convention: None,
                        calendar: None,
                        except_dates: Vec::new(),
                        also_dates: Vec::new(),
                        span: merge_spans(start, end_tok.span),
                    };
                    self.parse_schedule_opts(&mut spec);
                    return Some(spec);
                }

                let from_tok = self.bump();
                let from = match from_tok.kind {
                    TokenKind::Date(ref d) => d.clone(),
                    _ => {
                        self.push_expected(
                            from_tok.span,
                            "Expected token <date> after 'from'.".to_string(),
                        );
                        return None;
                    }
                };
                let _to_kw = self.expect_keyword(Keyword::To, "'to'")?;
                let to_tok = self.bump();
                let to = match to_tok.kind {
                    TokenKind::Date(ref d) => d.clone(),
                    _ => {
                        self.push_expected(
                            to_tok.span,
                            "Expected token <date> after 'to'.".to_string(),
                        );
                        return None;
                    }
                };
                let mut spec = ScheduleSpec {
                    kind: ScheduleKind::Every,
                    every: Some(every),
                    end_of_month,
                    from: Some(from),
                    to: Some(to),
                    day_of_month,
                    convention: None,
                    calendar: None,
                    except_dates: Vec::new(),
                    also_dates: Vec::new(),
                    span: merge_spans(start, to_tok.span),
                };
                self.parse_schedule_opts(&mut spec);
                Some(spec)
            }
            _ => None,
        }
    }

    /// `run deterministic` | `run monte_carlo trials <int> seed <int>`
    fn parse_run_stmt(&mut self) -> Option<RunStmt> {
        let start = self.expect_keyword(Keyword::Run, "'run'")?;
        match self.peek().kind {
            TokenKind::Keyword(Keyword::Deterministic) => {
                let tok = self.bump();
                Some(RunStmt {
                    kind: "deterministic".to_string(),
                    trials: None,
                    seed: None,
                    span: merge_spans(start.span, tok.span),
                })
            }
            TokenKind::Keyword(Keyword::MonteCarlo) => {
                let _ = self.bump();
                let _ = self.expect_keyword(Keyword::Trials, "'trials'")?;
                let trials_tok = self.bump();
                let trials = match trials_tok.kind {
                    TokenKind::Number(ref n) => n.parse::<u64>().ok(),
                    _ => None,
                };
                if trials.is_none() {
                    self.push_expected(
                        trials_tok.span,
                        "Expected positive integer after 'trials'.".to_string(),
                    );
                    return None;
                }
                let _ = self.expect_keyword(Keyword::Seed, "'seed'")?;
                let seed_tok = self.bump();
                let seed = match seed_tok.kind {
                    TokenKind::Number(ref n) => n.parse::<u64>().ok(),
                    _ => None,
                };
                if seed.is_none() {
                    self.push_expected(
                        seed_tok.span,
                        "Expected non-negative integer after 'seed'.".to_string(),
                    );
                    return None;
                }
                Some(RunStmt {
                    kind: "monte_carlo".to_string(),
                    trials,
                    seed,
                    span: merge_spans(start.span, seed_tok.span),
                })
            }
            _ => {
                self.push_expected(
                    self.current_span(),
                    "Expected 'deterministic' or 'monte_carlo' after 'run'.".to_string(),
                );
                None
            }
        }
    }

    /// Consume expression tokens until any of `stops` (or a statement
    /// boundary), returning the raw source slice.
    fn consume_expr_until(&mut self, stops: &[TokStopKind]) -> Option<String> {
        let mut first: Option<Span> = None;
        let mut last: Option<Span> = None;
        loop {
            let stop = match &self.peek().kind {
                TokenKind::Eof => true,
                TokenKind::Punct(Punct::LBrace) => stops.contains(&TokStopKind::LBrace),
                TokenKind::Punct(Punct::RBrace) => true,
                TokenKind::Keyword(Keyword::Set)
                | TokenKind::Keyword(Keyword::Activate)
                | TokenKind::Keyword(Keyword::Deactivate)
                | TokenKind::Keyword(Keyword::Exercise) => stops.contains(&TokStopKind::Action),
                TokenKind::Keyword(Keyword::Payoff) => stops.contains(&TokStopKind::Payoff),
                _ => false,
            };
            if stop {
                break;
            }
            let tok = self.bump();
            if first.is_none() {
                first = Some(tok.span);
            }
            last = Some(tok.span);
        }
        let (first, last) = (first?, last?);
        Some(self.slice_source(merge_spans(first, last)))
    }

    /// `event <qname> when <expr> { action* }`
    fn parse_event_stmt(&mut self) -> Option<EventStmt> {
        let start = self.expect_keyword(Keyword::Event, "'event'")?;
        let name_tok = self.bump();
        let name = match name_tok.kind {
            TokenKind::Ident(ref s) | TokenKind::Qname(ref s) => s.clone(),
            _ => {
                self.push_expected(name_tok.span, "Expected event name.".to_string());
                return None;
            }
        };
        let _ = self.expect_keyword(Keyword::When, "'when'")?;
        let Some(when) = self.consume_expr_until(&[TokStopKind::LBrace]) else {
            self.push_expected(
                self.current_span(),
                "Expected trigger expression after 'when'.".to_string(),
            );
            return None;
        };
        let _ = self.expect_punct(Punct::LBrace, "'{'")?;
        let mut actions = Vec::new();
        loop {
            match self.peek().kind {
                TokenKind::Punct(Punct::RBrace) | TokenKind::Eof => break,
                TokenKind::Keyword(Keyword::Set) => {
                    let _ = self.bump();
                    let _ = self.expect_keyword(Keyword::Entity, "'entity'")?;
                    let target_tok = self.bump();
                    let target = match target_tok.kind {
                        TokenKind::Qname(ref s) => s.clone(),
                        _ => {
                            self.push_expected(
                                target_tok.span,
                                "Expected qualified entity field (ns.name.field) after 'set entity'.".to_string(),
                            );
                            return None;
                        }
                    };
                    let segments: Vec<&str> = target.split('.').collect();
                    if segments.len() < 3 {
                        self.push_expected(
                            target_tok.span,
                            "Expected entity field reference of the form ns.name.field."
                                .to_string(),
                        );
                        return None;
                    }
                    let field = segments.last().expect("segments non-empty").to_string();
                    let entity = segments[..segments.len() - 1].join(".");
                    let _ = self.expect_punct(Punct::Equal, "'='")?;
                    let Some(value) = self.consume_expr_until(&[TokStopKind::Action]) else {
                        self.push_expected(
                            self.current_span(),
                            "Expected value expression after '='.".to_string(),
                        );
                        return None;
                    };
                    actions.push(EventAction::SetEntityField {
                        entity,
                        field,
                        value,
                    });
                }
                TokenKind::Keyword(Keyword::Activate) | TokenKind::Keyword(Keyword::Deactivate) => {
                    let activate =
                        matches!(self.peek().kind, TokenKind::Keyword(Keyword::Activate));
                    let _ = self.bump();
                    let kind_tok = self.bump();
                    let is_stream = match kind_tok.kind {
                        TokenKind::Keyword(Keyword::Stream) => true,
                        TokenKind::Keyword(Keyword::Contract) => false,
                        _ => {
                            self.push_expected(
                                kind_tok.span,
                                "Expected 'stream' or 'contract' after activate/deactivate."
                                    .to_string(),
                            );
                            return None;
                        }
                    };
                    let target_tok = self.bump();
                    let target = match target_tok.kind {
                        TokenKind::Ident(ref s) | TokenKind::Qname(ref s) => s.clone(),
                        _ => {
                            self.push_expected(target_tok.span, "Expected name.".to_string());
                            return None;
                        }
                    };
                    actions.push(match (activate, is_stream) {
                        (true, true) => EventAction::ActivateStream(target),
                        (false, true) => EventAction::DeactivateStream(target),
                        (true, false) => EventAction::ActivateContract(target),
                        (false, false) => EventAction::DeactivateContract(target),
                    });
                }
                TokenKind::Keyword(Keyword::Exercise) => {
                    let _ = self.bump();
                    let _ = self.expect_keyword(Keyword::Option, "'option'")?;
                    let target_tok = self.bump();
                    let target = match target_tok.kind {
                        TokenKind::Ident(ref s) | TokenKind::Qname(ref s) => s.clone(),
                        _ => {
                            self.push_expected(
                                target_tok.span,
                                "Expected option name.".to_string(),
                            );
                            return None;
                        }
                    };
                    actions.push(EventAction::ExerciseOption(target));
                }
                _ => {
                    self.push_expected(
                        self.current_span(),
                        "Expected action (set, activate, deactivate, exercise) or '}'.".to_string(),
                    );
                    return None;
                }
            }
        }
        let end = self.expect_punct(Punct::RBrace, "'}'")?;
        Some(EventStmt {
            name,
            when,
            actions,
            span: merge_spans(start.span, end.span),
        })
    }

    /// `option <qname> type <qname> [exercisable in <ident>] { exercise when <expr> payoff <expr> }`
    fn parse_option_stmt(&mut self) -> Option<OptionStmt> {
        let start = self.expect_keyword(Keyword::Option, "'option'")?;
        let name_tok = self.bump();
        let name = match name_tok.kind {
            TokenKind::Ident(ref s) | TokenKind::Qname(ref s) => s.clone(),
            _ => {
                self.push_expected(name_tok.span, "Expected option name.".to_string());
                return None;
            }
        };
        let _ = self.expect_keyword(Keyword::Type, "'type'")?;
        let type_tok = self.bump();
        let type_name = match type_tok.kind {
            TokenKind::Ident(ref s) | TokenKind::Qname(ref s) => s.clone(),
            _ => {
                self.push_expected(type_tok.span, "Expected option type.".to_string());
                return None;
            }
        };
        let mut exercisable_in = None;
        if matches!(self.peek().kind, TokenKind::Keyword(Keyword::Exercisable)) {
            let _ = self.bump();
            let _ = self.expect_keyword(Keyword::In, "'in'")?;
            let phase_tok = self.bump();
            match phase_tok.kind {
                TokenKind::Ident(ref s) | TokenKind::Qname(ref s) => {
                    exercisable_in = Some(s.clone());
                }
                _ => {
                    self.push_expected(phase_tok.span, "Expected phase name.".to_string());
                    return None;
                }
            }
        }
        let _ = self.expect_punct(Punct::LBrace, "'{'")?;
        let mut exercise_when = None;
        let mut payoff = None;
        loop {
            match self.peek().kind {
                TokenKind::Punct(Punct::RBrace) | TokenKind::Eof => break,
                TokenKind::Keyword(Keyword::Exercise) => {
                    let _ = self.bump();
                    let _ = self.expect_keyword(Keyword::When, "'when'")?;
                    exercise_when =
                        self.consume_expr_until(&[TokStopKind::Payoff, TokStopKind::Action]);
                    if exercise_when.is_none() {
                        self.push_expected(
                            self.current_span(),
                            "Expected expression after 'exercise when'.".to_string(),
                        );
                        return None;
                    }
                }
                TokenKind::Keyword(Keyword::Payoff) => {
                    let _ = self.bump();
                    payoff = self.consume_expr_until(&[TokStopKind::Action]);
                    if payoff.is_none() {
                        self.push_expected(
                            self.current_span(),
                            "Expected expression after 'payoff'.".to_string(),
                        );
                        return None;
                    }
                }
                _ => {
                    self.push_expected(
                        self.current_span(),
                        "Expected 'exercise when', 'payoff', or '}'.".to_string(),
                    );
                    return None;
                }
            }
        }
        let end = self.expect_punct(Punct::RBrace, "'}'")?;
        Some(OptionStmt {
            name,
            type_name,
            exercisable_in,
            exercise_when,
            payoff,
            span: merge_spans(start.span, end.span),
        })
    }

    /// `assume <ident> = <expr>` or `assume <ident> ~ Dist(name=num, ..., clip=[lo, hi])`
    fn parse_assume_stmt(&mut self) -> Option<AssumeStmt> {
        let start = self.expect_keyword(Keyword::Assume, "'assume'")?;
        let name_tok = self.bump();
        let name = match name_tok.kind {
            TokenKind::Ident(ref s) => s.clone(),
            _ => {
                self.push_expected(
                    name_tok.span,
                    "Expected identifier after 'assume'.".to_string(),
                );
                return None;
            }
        };
        match self.peek().kind {
            TokenKind::Punct(Punct::Equal) => {
                let _ = self.bump();
                // Consume expression tokens until the next top-level statement.
                let mut first: Option<Span> = None;
                let mut last: Option<Span> = None;
                loop {
                    match self.peek().kind {
                        TokenKind::Eof
                        | TokenKind::Keyword(Keyword::Version)
                        | TokenKind::Keyword(Keyword::Model)
                        | TokenKind::Keyword(Keyword::Use)
                        | TokenKind::Keyword(Keyword::Import)
                        | TokenKind::Keyword(Keyword::Time)
                        | TokenKind::Keyword(Keyword::Phase)
                        | TokenKind::Keyword(Keyword::Entity)
                        | TokenKind::Keyword(Keyword::Assume)
                        | TokenKind::Keyword(Keyword::Curve)
                        | TokenKind::Keyword(Keyword::Contract)
                        | TokenKind::Keyword(Keyword::Stream)
                        | TokenKind::Keyword(Keyword::Event)
                        | TokenKind::Keyword(Keyword::Option)
                        | TokenKind::Keyword(Keyword::Run) => break,
                        _ => {
                            let tok = self.bump();
                            if first.is_none() {
                                first = Some(tok.span);
                            }
                            last = Some(tok.span);
                        }
                    }
                }
                let (Some(first), Some(last)) = (first, last) else {
                    self.push_expected(
                        self.current_span(),
                        "Expected expression after '='.".to_string(),
                    );
                    return None;
                };
                let expr_span = merge_spans(first, last);
                Some(AssumeStmt {
                    name,
                    value: Some(self.slice_source(expr_span)),
                    dist: None,
                    span: merge_spans(start.span, expr_span),
                })
            }
            TokenKind::Punct(Punct::Tilde) => {
                let _ = self.bump();
                let dist_tok = self.bump();
                let dist_name = match dist_tok.kind {
                    TokenKind::Keyword(Keyword::Normal) => "normal",
                    TokenKind::Keyword(Keyword::LogNormal) => "lognormal",
                    TokenKind::Keyword(Keyword::Uniform) => "uniform",
                    TokenKind::Keyword(Keyword::Triangular) => "triangular",
                    _ => {
                        self.push_expected(
                            dist_tok.span,
                            "Expected distribution (Normal, LogNormal, Uniform, Triangular) after '~'.".to_string(),
                        );
                        return None;
                    }
                };
                let _ = self.expect_punct(Punct::LParen, "'('")?;
                let mut args: Vec<(String, String)> = Vec::new();
                let mut clip: Option<(String, String)> = None;
                loop {
                    if matches!(self.peek().kind, TokenKind::Punct(Punct::RParen)) {
                        break;
                    }
                    let key_tok = self.bump();
                    let is_clip = matches!(key_tok.kind, TokenKind::Keyword(Keyword::Clip));
                    let key = match key_tok.kind {
                        TokenKind::Ident(ref s) => s.clone(),
                        TokenKind::Keyword(Keyword::Clip) => "clip".to_string(),
                        _ => {
                            self.push_expected(
                                key_tok.span,
                                "Expected argument name in distribution.".to_string(),
                            );
                            return None;
                        }
                    };
                    let _ = self.expect_punct(Punct::Equal, "'='")?;
                    if is_clip {
                        let _ = self.expect_punct(Punct::LBracket, "'['")?;
                        let lo = self.parse_signed_number()?;
                        let _ = self.expect_punct(Punct::Comma, "','")?;
                        let hi = self.parse_signed_number()?;
                        let _ = self.expect_punct(Punct::RBracket, "']'")?;
                        clip = Some((lo, hi));
                    } else {
                        let value = self.parse_signed_number()?;
                        args.push((key, value));
                    }
                    if matches!(self.peek().kind, TokenKind::Punct(Punct::Comma)) {
                        let _ = self.bump();
                    } else {
                        break;
                    }
                }
                let end = self.expect_punct(Punct::RParen, "')'")?;
                Some(AssumeStmt {
                    name,
                    value: None,
                    dist: Some(AssumeDist {
                        name: dist_name.to_string(),
                        args,
                        clip,
                    }),
                    span: merge_spans(start.span, end.span),
                })
            }
            _ => {
                self.push_expected(
                    self.current_span(),
                    "Expected '=' or '~' after assumption name.".to_string(),
                );
                None
            }
        }
    }

    fn parse_signed_number(&mut self) -> Option<String> {
        let mut negative = false;
        if matches!(self.peek().kind, TokenKind::Punct(Punct::Minus)) {
            let _ = self.bump();
            negative = true;
        }
        let tok = self.bump();
        match tok.kind {
            TokenKind::Number(ref n) => Some(if negative { format!("-{n}") } else { n.clone() }),
            _ => {
                self.push_expected(tok.span, "Expected number.".to_string());
                None
            }
        }
    }

    /// Parse trailing schedule options: `convention <roll>`, `calendar <str>`,
    /// `except [dates]`, `also [dates]`. Order-insensitive, each at most once.
    fn parse_schedule_opts(&mut self, spec: &mut ScheduleSpec) {
        loop {
            match self.peek().kind {
                TokenKind::Keyword(Keyword::Convention) => {
                    let _ = self.bump();
                    let tok = self.bump();
                    let value = match tok.kind {
                        TokenKind::Keyword(Keyword::None) => "none",
                        TokenKind::Keyword(Keyword::Following) => "following",
                        TokenKind::Keyword(Keyword::ModifiedFollowing) => "modified_following",
                        TokenKind::Keyword(Keyword::Preceding) => "preceding",
                        TokenKind::Keyword(Keyword::ModifiedPreceding) => "modified_preceding",
                        _ => {
                            self.push_expected(
                                tok.span,
                                "Expected roll convention after 'convention' (none, following, modified_following, preceding, modified_preceding).".to_string(),
                            );
                            return;
                        }
                    };
                    spec.convention = Some(value.to_string());
                    spec.span = merge_spans(spec.span, tok.span);
                }
                TokenKind::Keyword(Keyword::Calendar) => {
                    let _ = self.bump();
                    let tok = self.bump();
                    match tok.kind {
                        TokenKind::String(ref s) => {
                            spec.calendar = Some(s.clone());
                            spec.span = merge_spans(spec.span, tok.span);
                        }
                        _ => {
                            self.push_expected(
                                tok.span,
                                "Expected token <string> after 'calendar'.".to_string(),
                            );
                            return;
                        }
                    }
                }
                TokenKind::Keyword(Keyword::Except) => {
                    let _ = self.bump();
                    match self.parse_date_list() {
                        Some((dates, end)) => {
                            spec.except_dates = dates;
                            spec.span = merge_spans(spec.span, end);
                        }
                        None => return,
                    }
                }
                TokenKind::Keyword(Keyword::Also) => {
                    let _ = self.bump();
                    match self.parse_date_list() {
                        Some((dates, end)) => {
                            spec.also_dates = dates;
                            spec.span = merge_spans(spec.span, end);
                        }
                        None => return,
                    }
                }
                _ => return,
            }
        }
    }

    /// `[ date, date, ... ]`
    fn parse_date_list(&mut self) -> Option<(Vec<String>, Span)> {
        let _ = self.expect_punct(Punct::LBracket, "'['")?;
        let mut dates = Vec::new();
        loop {
            let tok = self.bump();
            match tok.kind {
                TokenKind::Date(ref d) => dates.push(d.clone()),
                _ => {
                    self.push_expected(tok.span, "Expected token <date> in date list.".to_string());
                    return None;
                }
            }
            match self.peek().kind {
                TokenKind::Punct(Punct::Comma) => {
                    let _ = self.bump();
                }
                _ => break,
            }
        }
        let end = self.expect_punct(Punct::RBracket, "']'")?;
        Some((dates, end.span))
    }

    /// `curve <ident> [step|linear] { <date>: <number>[,] ... }`
    fn parse_curve_stmt(&mut self) -> Option<CurveStmt> {
        let start = self.expect_keyword(Keyword::Curve, "'curve'")?;
        let name_tok = self.bump();
        let name = match name_tok.kind {
            TokenKind::Ident(ref s) => s.clone(),
            _ => {
                self.push_expected(
                    name_tok.span,
                    "Expected identifier after 'curve'.".to_string(),
                );
                return None;
            }
        };
        let mut interpolation = "step".to_string();
        if let TokenKind::Ident(ref s) = self.peek().kind {
            match s.as_str() {
                "step" | "linear" => {
                    interpolation = s.clone();
                    let _ = self.bump();
                }
                other => {
                    self.push_expected(
                        self.current_span(),
                        format!("Unknown curve interpolation '{other}' (use 'step' or 'linear')."),
                    );
                    return None;
                }
            }
        }
        let _ = self.expect_punct(Punct::LBrace, "'{'")?;
        let mut points: Vec<(String, String)> = Vec::new();
        let end;
        loop {
            if matches!(self.peek().kind, TokenKind::Punct(Punct::RBrace)) {
                end = self.bump();
                break;
            }
            let date_tok = self.bump();
            let date = match date_tok.kind {
                TokenKind::Date(ref d) => d.clone(),
                TokenKind::Eof => {
                    self.push_expected(
                        date_tok.span,
                        "Expected <date> or '}' in curve block.".to_string(),
                    );
                    return None;
                }
                _ => {
                    self.push_expected(
                        date_tok.span,
                        "Expected <date> point in curve block (e.g. 2026-01: 0.043).".to_string(),
                    );
                    return None;
                }
            };
            let _ = self.expect_punct(Punct::Colon, "':'")?;
            let mut negative = false;
            if matches!(self.peek().kind, TokenKind::Punct(Punct::Minus)) {
                negative = true;
                let _ = self.bump();
            }
            let value_tok = self.bump();
            let value = match value_tok.kind {
                TokenKind::Number(ref n) => {
                    if negative {
                        format!("-{n}")
                    } else {
                        n.clone()
                    }
                }
                _ => {
                    self.push_expected(
                        value_tok.span,
                        "Expected <number> after ':' in curve point.".to_string(),
                    );
                    return None;
                }
            };
            points.push((date, value));
            if matches!(self.peek().kind, TokenKind::Punct(Punct::Comma)) {
                let _ = self.bump();
            }
        }
        if points.is_empty() {
            self.push_expected(
                end.span,
                format!("Curve '{name}' must declare at least one point."),
            );
            return None;
        }
        Some(CurveStmt {
            name,
            interpolation,
            points,
            span: merge_spans(start.span, end.span),
        })
    }

    fn synchronize_to_next_statement(&mut self) {
        while !self.is_eof() {
            match self.peek().kind {
                TokenKind::Keyword(Keyword::Version)
                | TokenKind::Keyword(Keyword::Model)
                | TokenKind::Keyword(Keyword::Use)
                | TokenKind::Keyword(Keyword::Import)
                | TokenKind::Keyword(Keyword::Time)
                | TokenKind::Keyword(Keyword::Phase)
                | TokenKind::Keyword(Keyword::Entity)
                | TokenKind::Keyword(Keyword::Assume)
                | TokenKind::Keyword(Keyword::Curve)
                | TokenKind::Keyword(Keyword::Contract)
                | TokenKind::Keyword(Keyword::Event)
                | TokenKind::Keyword(Keyword::Option)
                | TokenKind::Keyword(Keyword::Run)
                | TokenKind::Keyword(Keyword::Stream) => break,
                _ => {
                    let _ = self.bump();
                }
            }
        }
    }

    fn parse_entity_ref_token(&mut self, token: &Token) -> Option<String> {
        let qname = match &token.kind {
            TokenKind::Qname(value) => value,
            TokenKind::Ident(_) => {
                self.push_expected(
                    token.span,
                    "Expected token <entity-ref> after 'on entity'; entity refs must be qualified (e.g. legal.borrower).".to_string(),
                );
                return None;
            }
            _ => {
                self.push_expected(
                    token.span,
                    "Expected token <entity-ref> after 'on entity'.".to_string(),
                );
                return None;
            }
        };
        if !is_valid_entity_ref(qname) {
            self.push_expected(
                token.span,
                "Expected token <entity-ref> after 'on entity'; entity refs must contain at least two identifier segments.".to_string(),
            );
            return None;
        }
        Some(qname.clone())
    }

    fn parse_name_like_token(&mut self) -> Option<(String, Span)> {
        let tok = self.peek().clone();
        match tok.kind {
            TokenKind::Ident(ident) => {
                let _ = self.bump();
                Some((ident, tok.span))
            }
            TokenKind::Qname(qname) => {
                let _ = self.bump();
                Some((qname, tok.span))
            }
            _ => None,
        }
    }

    fn expect_keyword(&mut self, expected: Keyword, expected_label: &str) -> Option<Token> {
        let tok = self.bump();
        match tok.kind {
            TokenKind::Keyword(k) if k == expected => Some(tok),
            _ => {
                self.push_expected(
                    tok.span,
                    format!(
                        "Expected token {expected_label}, found {}.",
                        token_label(&tok)
                    ),
                );
                None
            }
        }
    }

    fn expect_punct(&mut self, expected: Punct, expected_label: &str) -> Option<Token> {
        let tok = self.bump();
        match tok.kind {
            TokenKind::Punct(p) if p == expected => Some(tok),
            _ => {
                self.push_expected(
                    tok.span,
                    format!(
                        "Expected token {expected_label}, found {}.",
                        token_label(&tok)
                    ),
                );
                None
            }
        }
    }

    fn push_unexpected(&mut self, span: Span, message: String) {
        self.diagnostics.push(ParseDiagnostic {
            code: "E0001_UNEXPECTED_TOKEN",
            message,
            file: self.file.clone(),
            span,
        });
    }

    fn push_expected(&mut self, span: Span, message: String) {
        self.diagnostics.push(ParseDiagnostic {
            code: "E0004_EXPECTED_TOKEN",
            message,
            file: self.file.clone(),
            span,
        });
    }

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.idx)
            .unwrap_or_else(|| self.tokens.last().expect("token stream has EOF"))
    }

    fn bump(&mut self) -> Token {
        let tok = self.peek().clone();
        if !matches!(tok.kind, TokenKind::Eof) {
            self.idx += 1;
        }
        tok
    }

    fn is_eof(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    fn current_span(&self) -> Span {
        self.peek().span
    }
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
        Stmt::Curve(s) => s.span,
        Stmt::Run(s) => s.span,
        Stmt::Contract(s) => s.span,
        Stmt::Stream(s) => s.span,
        Stmt::Event(s) => s.span,
        Stmt::Option(s) => s.span,
    }
}

fn merge_spans(start: Span, end: Span) -> Span {
    Span {
        start_line: start.start_line,
        start_col: start.start_col,
        end_line: end.end_line,
        end_col: end.end_col,
    }
}

fn token_label(token: &Token) -> String {
    match &token.kind {
        TokenKind::Keyword(k) => format!("keyword '{}'", keyword_text(*k)),
        TokenKind::Ident(_) => "<identifier>".to_string(),
        TokenKind::Qname(_) => "<qname>".to_string(),
        TokenKind::String(_) => "<string>".to_string(),
        TokenKind::Number(_) => "<number>".to_string(),
        TokenKind::Date(_) => "<date>".to_string(),
        TokenKind::Punct(_) => "<punctuation>".to_string(),
        TokenKind::Eof => "<eof>".to_string(),
    }
}

fn keyword_text(keyword: Keyword) -> &'static str {
    match keyword {
        Keyword::Version => "version",
        Keyword::Model => "model",
        Keyword::Use => "use",
        Keyword::Currency => "currency",
        Keyword::Pack => "pack",
        Keyword::Import => "import",
        Keyword::As => "as",
        Keyword::Time => "time",
        Keyword::Calendar => "calendar",
        Keyword::From => "from",
        Keyword::For => "for",
        Keyword::Daily => "daily",
        Keyword::Monthly => "monthly",
        Keyword::Quarterly => "quarterly",
        Keyword::Annual => "annual",
        Keyword::Week => "week",
        Keyword::Month => "month",
        Keyword::Quarter => "quarter",
        Keyword::Year => "year",
        Keyword::Phase => "phase",
        Keyword::To => "to",
        Keyword::Entity => "entity",
        Keyword::Assume => "assume",
        Keyword::Contract => "contract",
        Keyword::On => "on",
        Keyword::Term => "term",
        Keyword::Terms => "terms",
        Keyword::Effects => "effects",
        Keyword::Parties => "parties",
        Keyword::Tags => "tags",
        Keyword::Stream => "stream",
        Keyword::Owner => "owner",
        Keyword::Direction => "direction",
        Keyword::Inflow => "inflow",
        Keyword::Outflow => "outflow",
        Keyword::Schedule => "schedule",
        Keyword::Every => "every",
        Keyword::PhaseEnter => "phase_enter",
        Keyword::PhaseStart => "phase_start",
        Keyword::PhaseEnd => "phase_end",
        Keyword::Day => "day",
        Keyword::Eom => "eom",
        Keyword::Mon => "Mon",
        Keyword::Tue => "Tue",
        Keyword::Wed => "Wed",
        Keyword::Thu => "Thu",
        Keyword::Fri => "Fri",
        Keyword::Sat => "Sat",
        Keyword::Sun => "Sun",
        Keyword::Convention => "convention",
        Keyword::Stub => "stub",
        Keyword::Except => "except",
        Keyword::Also => "also",
        Keyword::None => "none",
        Keyword::Following => "following",
        Keyword::ModifiedFollowing => "modified_following",
        Keyword::Preceding => "preceding",
        Keyword::ModifiedPreceding => "modified_preceding",
        Keyword::ShortFront => "short_front",
        Keyword::ShortBack => "short_back",
        Keyword::LongFront => "long_front",
        Keyword::LongBack => "long_back",
        Keyword::Event => "event",
        Keyword::When => "when",
        Keyword::Set => "set",
        Keyword::Activate => "activate",
        Keyword::Deactivate => "deactivate",
        Keyword::Exercise => "exercise",
        Keyword::Option => "option",
        Keyword::Type => "type",
        Keyword::Exercisable => "exercisable",
        Keyword::In => "in",
        Keyword::Payoff => "payoff",
        Keyword::Run => "run",
        Keyword::Deterministic => "deterministic",
        Keyword::MonteCarlo => "monte_carlo",
        Keyword::Trials => "trials",
        Keyword::Seed => "seed",
        Keyword::Curve => "curve",

        Keyword::True => "true",
        Keyword::False => "false",
        Keyword::Normal => "Normal",
        Keyword::LogNormal => "LogNormal",
        Keyword::Uniform => "Uniform",
        Keyword::Triangular => "Triangular",
        Keyword::Clip => "clip",
        Keyword::Active => "active",
    }
}

fn is_statement_start(token: &Token) -> bool {
    matches!(
        token.kind,
        TokenKind::Keyword(Keyword::Version)
            | TokenKind::Keyword(Keyword::Model)
            | TokenKind::Keyword(Keyword::Use)
            | TokenKind::Keyword(Keyword::Import)
            | TokenKind::Keyword(Keyword::Time)
            | TokenKind::Keyword(Keyword::Phase)
            | TokenKind::Keyword(Keyword::Entity)
            | TokenKind::Keyword(Keyword::Contract)
            | TokenKind::Keyword(Keyword::Stream)
    )
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
    use cfdl_lexer::lex;

    #[test]
    fn parses_minimal_model_statements() {
        let src = r#"version 0.1
model "demo"
time calendar monthly from 2026-01 for 12
entity legal borrower
stream legal.principal on entity legal.borrower
"#;
        let (tokens, lex_diags) = lex(src);
        assert!(lex_diags.is_empty());
        let result = parse("model.cfdl", src, &tokens);
        assert!(result.diagnostics.is_empty());
        let ast = result.ast.expect("AST expected");
        assert_eq!(ast.statements.len(), 5);
        assert!(matches!(ast.statements[0], Stmt::Version(_)));
        assert!(matches!(ast.statements[1], Stmt::Model(_)));
        assert!(matches!(ast.statements[2], Stmt::Time(_)));
        assert!(matches!(ast.statements[3], Stmt::Entity(_)));
        assert!(matches!(ast.statements[4], Stmt::Stream(_)));
    }

    #[test]
    fn parses_phase_statement() {
        let src = r#"version 0.1
model "demo"
phase p from 2026-01 to 2026-02
"#;
        let (tokens, lex_diags) = lex(src);
        assert!(lex_diags.is_empty());
        let result = parse("model.cfdl", src, &tokens);
        assert!(result.diagnostics.is_empty());
        let ast = result.ast.expect("AST expected");
        assert_eq!(ast.statements.len(), 3);
        assert!(matches!(ast.statements[2], Stmt::Phase(_)));
    }

    #[test]
    fn parses_curve_statement() {
        let src = r#"version 0.1
model "demo"
curve sofr {
  2026-01: 0.045
  2026-07: 0.042, 2027-01-15: 0.040
}
curve ramp linear { 2026-01: 0.01 }
"#;
        let (tokens, lex_diags) = lex(src);
        assert!(lex_diags.is_empty());
        let result = parse("model.cfdl", src, &tokens);
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        let ast = result.ast.expect("AST expected");
        let Stmt::Curve(curve) = &ast.statements[2] else {
            panic!("expected curve statement");
        };
        assert_eq!(curve.name, "sofr");
        assert_eq!(curve.interpolation, "step");
        assert_eq!(
            curve.points,
            vec![
                ("2026-01".to_string(), "0.045".to_string()),
                ("2026-07".to_string(), "0.042".to_string()),
                ("2027-01-15".to_string(), "0.040".to_string()),
            ]
        );
        let Stmt::Curve(ramp) = &ast.statements[3] else {
            panic!("expected curve statement");
        };
        assert_eq!(ramp.interpolation, "linear");
    }

    #[test]
    fn curve_statement_errors() {
        for (src, needle) in [
            ("curve sofr { }", "at least one point"),
            (
                "curve sofr cubic { 2026-01: 0.01 }",
                "Unknown curve interpolation",
            ),
            ("curve sofr { 42: 0.01 }", "Expected <date> point"),
            ("curve sofr { 2026-01: x }", "Expected <number>"),
        ] {
            let (tokens, _) = lex(src);
            let result = parse("model.cfdl", src, &tokens);
            assert!(
                result
                    .diagnostics
                    .iter()
                    .any(|d| d.message.contains(needle)),
                "{src}: {:?}",
                result.diagnostics
            );
        }
    }

    #[test]
    fn reports_expected_token() {
        let src = r#"version 0.1
model "demo"
time monthly from 2026-01 for 12
"#;
        let (tokens, lex_diags) = lex(src);
        assert!(lex_diags.is_empty());
        let result = parse("model.cfdl", src, &tokens);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "E0004_EXPECTED_TOKEN");
        assert_eq!(result.diagnostics[0].file, "model.cfdl");
    }

    #[test]
    fn parses_import_statement() {
        let src = r#"import "sub/module.cfdl" as sub"#;
        let (tokens, lex_diags) = lex(src);
        assert!(lex_diags.is_empty());
        let result = parse("model.cfdl", src, &tokens);
        assert!(result.diagnostics.is_empty());
        let ast = result.ast.expect("AST expected");
        assert_eq!(ast.statements.len(), 1);
        match &ast.statements[0] {
            Stmt::Import(stmt) => {
                assert_eq!(stmt.path, "sub/module.cfdl");
                assert_eq!(stmt.alias.as_deref(), Some("sub"));
            }
            other => panic!("expected import stmt, got {other:?}"),
        }
    }

    #[test]
    fn parses_use_pack_statement() {
        let src = r#"use pack "testpack" version "0.1.0""#;
        let (tokens, lex_diags) = lex(src);
        assert!(lex_diags.is_empty());
        let result = parse("model.cfdl", src, &tokens);
        assert!(result.diagnostics.is_empty());
        let ast = result.ast.expect("AST expected");
        assert_eq!(ast.statements.len(), 1);
        match &ast.statements[0] {
            Stmt::UsePack(stmt) => {
                assert_eq!(stmt.name, "testpack");
                assert_eq!(stmt.version, "0.1.0");
            }
            other => panic!("expected use-pack stmt, got {other:?}"),
        }
    }

    #[test]
    fn parses_stream_amount_expression_slot() {
        let src = r#"version 0.1
model "demo"
time calendar monthly from 2026-01 for 2
entity legal borrower
stream legal.rent on entity legal.borrower {
  schedule every month from 2026-01 to 2026-02
  amount = 1000
}
"#;
        let (tokens, lex_diags) = lex(src);
        assert!(lex_diags.is_empty());
        let result = parse("model.cfdl", src, &tokens);
        assert!(result.diagnostics.is_empty());
        let ast = result.ast.expect("AST expected");
        let stream = ast
            .statements
            .iter()
            .find_map(|stmt| match stmt {
                Stmt::Stream(stream) => Some(stream),
                _ => None,
            })
            .expect("stream statement");
        assert_eq!(
            stream
                .amount
                .as_ref()
                .expect("amount expression expected")
                .src,
            "1000"
        );
    }

    #[test]
    fn parses_contract_subject_entity_when_present() {
        let src = r#"version 0.1
model "demo"
time calendar monthly from 2026-01 for 2
entity legal borrower
contract cre.lease_one on entity legal.borrower {
  term 2026-01..2026-02
}
"#;
        let (tokens, lex_diags) = lex(src);
        assert!(lex_diags.is_empty());
        let result = parse("model.cfdl", src, &tokens);
        assert!(result.diagnostics.is_empty());
        let ast = result.ast.expect("AST expected");
        let contract = ast
            .statements
            .iter()
            .find_map(|stmt| match stmt {
                Stmt::Contract(contract) => Some(contract),
                _ => None,
            })
            .expect("contract statement");
        assert_eq!(contract.subject_entity.as_deref(), Some("legal.borrower"));
    }

    #[test]
    fn keeps_contract_subject_entity_optional_for_compatibility() {
        let src = r#"version 0.1
model "demo"
time calendar monthly from 2026-01 for 2
entity legal borrower
contract cre.lease_one {
  term 2026-01..2026-02
}
"#;
        let (tokens, lex_diags) = lex(src);
        assert!(lex_diags.is_empty());
        let result = parse("model.cfdl", src, &tokens);
        assert!(result.diagnostics.is_empty());
        let ast = result.ast.expect("AST expected");
        let contract = ast
            .statements
            .iter()
            .find_map(|stmt| match stmt {
                Stmt::Contract(contract) => Some(contract),
                _ => None,
            })
            .expect("contract statement");
        assert_eq!(contract.subject_entity, None);
    }

    #[test]
    fn parses_dotted_stream_and_contract_names() {
        let src = r#"version 0.1
model "demo"
time calendar monthly from 2026-01 for 2
entity legal borrower
contract lease.core.primary on entity legal.borrower {
  term 2026-01..2026-02
}
stream cre.lease.base_rent on entity legal.borrower {
  schedule every month from 2026-01 to 2026-02
  amount = 1000
}
"#;
        let (tokens, lex_diags) = lex(src);
        assert!(lex_diags.is_empty());
        let result = parse("model.cfdl", src, &tokens);
        assert!(result.diagnostics.is_empty());
        let ast = result.ast.expect("AST expected");
        let contract = ast
            .statements
            .iter()
            .find_map(|stmt| match stmt {
                Stmt::Contract(contract) => Some(contract),
                _ => None,
            })
            .expect("contract statement");
        assert_eq!(contract.name, "lease.core.primary");
        let stream = ast
            .statements
            .iter()
            .find_map(|stmt| match stmt {
                Stmt::Stream(stream) => Some(stream),
                _ => None,
            })
            .expect("stream statement");
        assert_eq!(stream.name, "cre.lease.base_rent");
    }

    #[test]
    fn rejects_unqualified_entity_ref() {
        let src = r#"version 0.1
model "demo"
time calendar monthly from 2026-01 for 2
entity legal borrower
stream legal.rent on entity borrower {
  schedule every month from 2026-01 to 2026-02
  amount = 1000
}
"#;
        let (tokens, lex_diags) = lex(src);
        assert!(lex_diags.is_empty());
        let result = parse("model.cfdl", src, &tokens);
        assert!(!result.diagnostics.is_empty());
        assert!(result
            .diagnostics
            .iter()
            .any(|diag| diag.message.contains("entity refs must be qualified")));
    }
}

#[cfg(test)]
mod fuzz_tests {
    use super::*;
    use cfdl_lexer::lex;

    /// Deterministic PRNG (splitmix64) — no external deps, reproducible.
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            z ^ (z >> 31)
        }
    }

    fn lex_parse_no_panic(src: &str) {
        let (tokens, _diags) = lex(src);
        let _ = parse("fuzz.cfdl", src, &tokens);
    }

    #[test]
    fn random_ascii_soup_never_panics() {
        let mut rng = Rng(0xC0FFEE);
        for _ in 0..2_000 {
            let len = (rng.next() % 200) as usize;
            let src: String = (0..len)
                .map(|_| (0x20 + (rng.next() % 0x5f) as u8) as char)
                .collect();
            lex_parse_no_panic(&src);
        }
    }

    #[test]
    fn mutated_valid_source_never_panics() {
        let base = "version 0.1\nmodel \"m\"\ntime calendar monthly from 2026-01 for 12\nphase p from 2026-01 to 2026-06\nentity legal borrower\nassume growth ~ Normal(mean=0.03, stdev=0.01, clip=[0.0, 0.08])\ncontract cre.lease on entity legal.borrower {\n  term 2026-01..2026-12\n  terms { base_rent = 25000 }\n}\nstream legal.rent on entity legal.borrower inflow currency USD {\n  schedule every month from 2026-01 to 2026-12 convention following calendar \"us\" except [2026-03-01]\n  amount = 1000 * pow(1.03, time.t / 12.0)\n  active when entity.status != \"gone\"\n}\nevent stop when time.t >= 6 {\n  deactivate stream legal.rent\n}\noption o1 type Option.X {\n  exercise when time.t == 3\n  payoff 100 - 1\n}\nrun monte_carlo trials 10 seed 42\n";
        lex_parse_no_panic(base);
        let bytes: Vec<char> = base.chars().collect();
        let mut rng = Rng(0xDEADBEEF);
        for _ in 0..2_000 {
            let mut mutated = bytes.clone();
            for _ in 0..(1 + rng.next() % 6) {
                let idx = (rng.next() as usize) % mutated.len();
                match rng.next() % 3 {
                    0 => {
                        mutated[idx] = (0x20 + (rng.next() % 0x5f) as u8) as char;
                    }
                    1 => {
                        mutated.remove(idx);
                    }
                    _ => {
                        mutated.insert(idx, (0x20 + (rng.next() % 0x5f) as u8) as char);
                    }
                }
                if mutated.is_empty() {
                    break;
                }
            }
            let src: String = mutated.into_iter().collect();
            lex_parse_no_panic(&src);
        }
    }

    #[test]
    fn truncations_never_panic() {
        let base = "stream legal.rent on entity legal.borrower {\n  schedule every month from 2026-01 to 2026-12\n  amount = 1000 * (1 + 0.03) ^ (time.t / 12)\n}\n";
        for cut in 0..base.len() {
            if base.is_char_boundary(cut) {
                lex_parse_no_panic(&base[..cut]);
            }
        }
    }
}
