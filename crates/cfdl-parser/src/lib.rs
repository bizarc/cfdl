//! CFDL parser skeleton for v0.1.
//!
//! Milestone 2 scope:
//! - `version` statement
//! - `model` statement
//! - `time calendar <cadence> from <date> for <int>` statement
//! - parser diagnostics: E0001 + E0004 with file/span

pub use cfdl_lexer::Span;
use cfdl_lexer::{Keyword, Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilationUnit {
    pub statements: Vec<Stmt>,
    pub span: Span,
}

pub type ModelAst = CompilationUnit;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    Version(VersionStmt),
    Model(ModelStmt),
    Import(ImportStmt),
    Time(TimeStmt),
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
pub struct TimeStmt {
    pub cadence: Cadence,
    pub from: String,
    pub periods: u32,
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
            TokenKind::Keyword(Keyword::Import) => self.parse_import_stmt().map(Stmt::Import),
            TokenKind::Keyword(Keyword::Time) => self.parse_time_stmt().map(Stmt::Time),
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

    fn synchronize_to_next_statement(&mut self) {
        while !self.is_eof() {
            match self.peek().kind {
                TokenKind::Keyword(Keyword::Version)
                | TokenKind::Keyword(Keyword::Model)
                | TokenKind::Keyword(Keyword::Import)
                | TokenKind::Keyword(Keyword::Time) => break,
                _ => {
                    let _ = self.bump();
                }
            }
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
        Stmt::Import(s) => s.span,
        Stmt::Time(s) => s.span,
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
        Keyword::Currency => "currency",
        Keyword::Use => "use",
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
        Keyword::Metric => "metric",
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

#[cfg(test)]
mod tests {
    use super::*;
    use cfdl_lexer::lex;

    #[test]
    fn parses_minimal_model_statements() {
        let src = r#"version 0.1
model "demo"
time calendar monthly from 2026-01 for 12
"#;
        let (tokens, lex_diags) = lex(src);
        assert!(lex_diags.is_empty());
        let result = parse("model.cfdl", &tokens);
        assert!(result.diagnostics.is_empty());
        let ast = result.ast.expect("AST expected");
        assert_eq!(ast.statements.len(), 3);
        assert!(matches!(ast.statements[0], Stmt::Version(_)));
        assert!(matches!(ast.statements[1], Stmt::Model(_)));
        assert!(matches!(ast.statements[2], Stmt::Time(_)));
    }

    #[test]
    fn reports_unexpected_token() {
        let src = r#"version 0.1
model "demo"
phase p from 2026-01 to 2026-02
"#;
        let (tokens, lex_diags) = lex(src);
        assert!(lex_diags.is_empty());
        let result = parse("model.cfdl", &tokens);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "E0001_UNEXPECTED_TOKEN");
        assert_eq!(result.diagnostics[0].file, "model.cfdl");
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
}
