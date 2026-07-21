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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilationUnit {
    pub statements: Vec<Stmt>,
    pub span: Span,
}

pub type ModelAst = CompilationUnit;

#[derive(Debug, Clone, PartialEq, Eq)]
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
    Contract(ContractStmt),
    Stream(StreamStmt),
    Event(EventStmt),
    Option(OptionStmt),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionStmt {
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelStmt {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportStmt {
    pub path: String,
    pub alias: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsePackStmt {
    pub name: String,
    pub version: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeStmt {
    pub cadence: Cadence,
    pub from: String,
    pub periods: u32,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprSlot {
    pub lang: String,
    pub src: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseStmt {
    pub name: String,
    pub from: String,
    pub to: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssumeStmt {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractTerm {
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleSpec {
    pub kind: ScheduleKind,
    pub from: Option<String>,
    pub to: Option<String>,
    pub day_of_month: Option<i32>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleKind {
    OnDate,
    Every,
    PhaseEnter { phase: String },
    EveryPhase { phase: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventStmt {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionStmt {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cadence {
    Daily,
    Monthly,
    Quarterly,
    Annual,
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

pub fn parse(file: &str, tokens: &[Token]) -> ParseResult {
    let mut parser = Parser::new(file, tokens);
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
}

impl<'a> Parser<'a> {
    fn new(file: &str, tokens: &'a [Token]) -> Self {
        Self {
            file: file.to_string(),
            tokens,
            idx: 0,
            diagnostics: Vec::new(),
        }
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
        Some(TimeStmt {
            cadence,
            from,
            periods,
            span: merge_spans(start.span, periods_tok.span),
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
                    let value_tok = self.bump();
                    let value = match value_tok.kind {
                        TokenKind::String(ref s) => s.clone(),
                        TokenKind::Number(ref n) => n.clone(),
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

    fn parse_expr_slot(&mut self, start_span: Span) -> Option<ExprSlot> {
        let lang_tok = self.bump();
        match lang_tok.kind {
            TokenKind::Keyword(Keyword::Cel) => {}
            _ => {
                self.push_expected(
                    lang_tok.span,
                    "Expected token 'cel' for expression language.".to_string(),
                );
                return None;
            }
        }

        let src_tok = self.bump();
        match src_tok.kind {
            TokenKind::String(ref src) => Some(ExprSlot {
                lang: "cel".to_string(),
                src: src.clone(),
                span: merge_spans(start_span, src_tok.span),
            }),
            _ => {
                self.push_expected(
                    src_tok.span,
                    "Expected token <string> after 'cel'.".to_string(),
                );
                None
            }
        }
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
                    return Some(ScheduleSpec {
                        kind: ScheduleKind::PhaseEnter { phase },
                        from: None,
                        to: None,
                        day_of_month: None,
                        span: merge_spans(start, end_tok.span),
                    });
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
                Some(ScheduleSpec {
                    kind: ScheduleKind::OnDate,
                    from: Some(date.clone()),
                    to: Some(date),
                    day_of_month: None,
                    span: merge_spans(start, date_tok.span),
                })
            }
            TokenKind::Keyword(Keyword::Every) => {
                let _ = self.bump();
                if matches!(
                    self.peek().kind,
                    TokenKind::Keyword(Keyword::Daily)
                        | TokenKind::Keyword(Keyword::Monthly)
                        | TokenKind::Keyword(Keyword::Quarterly)
                        | TokenKind::Keyword(Keyword::Annual)
                ) {
                    let _ = self.bump();
                }

                let mut day_of_month = None;
                if matches!(self.peek().kind, TokenKind::Keyword(Keyword::On)) {
                    let _ = self.bump();
                    if matches!(self.peek().kind, TokenKind::Keyword(Keyword::Day)) {
                        let _ = self.bump();
                        let day_tok = self.bump();
                        match day_tok.kind {
                            TokenKind::Number(ref n) => {
                                if let Ok(value) = n.parse::<i32>() {
                                    day_of_month = Some(value);
                                }
                            }
                            _ => {
                                self.push_expected(
                                    day_tok.span,
                                    "Expected token <int> after 'on day'.".to_string(),
                                );
                                return None;
                            }
                        }
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
                    return Some(ScheduleSpec {
                        kind: ScheduleKind::EveryPhase { phase },
                        from: None,
                        to: None,
                        day_of_month,
                        span: merge_spans(start, end_tok.span),
                    });
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
                Some(ScheduleSpec {
                    kind: ScheduleKind::Every,
                    from: Some(from),
                    to: Some(to),
                    day_of_month,
                    span: merge_spans(start, to_tok.span),
                })
            }
            _ => None,
        }
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
                | TokenKind::Keyword(Keyword::Contract)
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

        Keyword::Cel => "cel",
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
        let result = parse("model.cfdl", &tokens);
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
        let result = parse("model.cfdl", &tokens);
        assert!(result.diagnostics.is_empty());
        let ast = result.ast.expect("AST expected");
        assert_eq!(ast.statements.len(), 3);
        assert!(matches!(ast.statements[2], Stmt::Phase(_)));
    }

    #[test]
    fn reports_expected_token() {
        let src = r#"version 0.1
model "demo"
time monthly from 2026-01 for 12
"#;
        let (tokens, lex_diags) = lex(src);
        assert!(lex_diags.is_empty());
        let result = parse("model.cfdl", &tokens);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "E0004_EXPECTED_TOKEN");
        assert_eq!(result.diagnostics[0].file, "model.cfdl");
    }

    #[test]
    fn parses_import_statement() {
        let src = r#"import "sub/module.cfdl" as sub"#;
        let (tokens, lex_diags) = lex(src);
        assert!(lex_diags.is_empty());
        let result = parse("model.cfdl", &tokens);
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
        let result = parse("model.cfdl", &tokens);
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
  schedule every monthly from 2026-01 to 2026-02
  amount cel "1000"
}
"#;
        let (tokens, lex_diags) = lex(src);
        assert!(lex_diags.is_empty());
        let result = parse("model.cfdl", &tokens);
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
        let result = parse("model.cfdl", &tokens);
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
        let result = parse("model.cfdl", &tokens);
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
  schedule every monthly from 2026-01 to 2026-02
  amount cel "1000"
}
"#;
        let (tokens, lex_diags) = lex(src);
        assert!(lex_diags.is_empty());
        let result = parse("model.cfdl", &tokens);
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
  schedule every monthly from 2026-01 to 2026-02
  amount cel "1000"
}
"#;
        let (tokens, lex_diags) = lex(src);
        assert!(lex_diags.is_empty());
        let result = parse("model.cfdl", &tokens);
        assert!(!result.diagnostics.is_empty());
        assert!(result
            .diagnostics
            .iter()
            .any(|diag| diag.message.contains("entity refs must be qualified")));
    }
}
