//! CFDL lexer for v0.1 grammar.
//!
//! The lexer is deterministic and produces spans for every token and diagnostic.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct Span {
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Position {
    line: u32,
    col: u32,
}

impl Position {
    fn to_span(self, end: Self) -> Span {
        Span {
            start_line: self.line,
            start_col: self.col,
            end_line: end.line,
            end_col: end.col,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Keyword(Keyword),
    Ident(String),
    Qname(String),
    String(String),
    Number(String),
    Date(String),
    Punct(Punct),
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Punct {
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Colon,
    Comma,
    Dot,
    DotDot,
    Equal,
    Tilde,
    // Expression operators (bare native expressions, Workstream B)
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    Lt,
    Le,
    Gt,
    Ge,
    EqEq,
    NotEq,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Version,
    Model,
    Currency,
    Use,
    Pack,
    Import,
    As,
    Time,
    Calendar,
    From,
    For,
    Daily,
    Monthly,
    Quarterly,
    Annual,
    Phase,
    To,
    Entity,
    Assume,
    Contract,
    On,
    Term,
    Terms,
    Effects,
    Parties,
    Tags,
    Stream,
    Owner,
    Direction,
    Inflow,
    Outflow,
    Schedule,
    Every,
    PhaseEnter,
    PhaseStart,
    PhaseEnd,
    Day,
    Eom,
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
    Convention,
    Stub,
    Except,
    Also,
    None,
    Following,
    ModifiedFollowing,
    Preceding,
    ModifiedPreceding,
    ShortFront,
    ShortBack,
    LongFront,
    LongBack,
    Event,
    When,
    Set,
    Activate,
    Deactivate,
    Exercise,
    Option,
    Type,
    Exercisable,
    In,
    Payoff,
    Run,
    Deterministic,
    MonteCarlo,
    Trials,
    Seed,
    Curve,

    True,
    False,
    Normal,
    LogNormal,
    Uniform,
    Triangular,
    Clip,
    Active,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexDiagnostic {
    pub code: &'static str,
    pub message: String,
    pub span: Span,
}

impl fmt::Display for LexDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

pub fn lex(source: &str) -> (Vec<Token>, Vec<LexDiagnostic>) {
    let mut lexer = Lexer::new(source);
    lexer.lex_all();
    (lexer.tokens, lexer.diagnostics)
}

struct Lexer<'a> {
    chars: Vec<char>,
    idx: usize,
    line: u32,
    col: u32,
    _source: &'a str,
    tokens: Vec<Token>,
    diagnostics: Vec<LexDiagnostic>,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().collect(),
            idx: 0,
            line: 1,
            col: 1,
            _source: source,
            tokens: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn lex_all(&mut self) {
        while self.peek().is_some() {
            self.skip_ws_and_comments();
            if self.peek().is_none() {
                break;
            }

            let Some(ch) = self.peek() else {
                break;
            };
            if is_ident_start(ch) {
                self.lex_ident_or_qname_or_keyword();
                continue;
            }
            if ch.is_ascii_digit() {
                self.lex_date_or_number();
                continue;
            }
            if ch == '"' {
                self.lex_string();
                continue;
            }
            self.lex_punctuation_or_skip_unknown();
        }

        let eof_pos = self.current_position();
        self.tokens.push(Token {
            kind: TokenKind::Eof,
            span: eof_pos.to_span(eof_pos),
        });
    }

    fn skip_ws_and_comments(&mut self) {
        loop {
            let before = self.idx;
            while matches!(self.peek(), Some(c) if c.is_whitespace()) {
                let _ = self.bump();
            }

            if self.peek() == Some('/') && self.peek_n(1) == Some('/') {
                self.bump();
                self.bump();
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    let _ = self.bump();
                }
            } else if self.peek() == Some('/') && self.peek_n(1) == Some('*') {
                let start = self.current_position();
                let _ = self.bump();
                let mut last = start;
                if let Some((_, star_pos)) = self.bump() {
                    last = star_pos;
                }
                let mut terminated = false;
                while self.peek().is_some() {
                    let (c, pos) = self.bump().expect("checked peek is some");
                    last = pos;
                    if c == '*' && self.peek() == Some('/') {
                        let (_, slash_pos) = self.bump().expect("slash exists");
                        last = slash_pos;
                        terminated = true;
                        break;
                    }
                }
                if !terminated {
                    self.diagnostics.push(LexDiagnostic {
                        code: "E0003_UNTERMINATED_BLOCK_COMMENT",
                        message: "Unterminated block comment.".to_string(),
                        span: start.to_span(last),
                    });
                    break;
                }
            }

            if self.idx == before {
                break;
            }
        }
    }

    fn lex_ident_or_qname_or_keyword(&mut self) {
        let start = self.current_position();
        let mut raw = String::new();

        let (first, pos) = self.bump().expect("identifier start exists");
        raw.push(first);
        let mut last = pos;

        while matches!(self.peek(), Some(c) if is_ident_continue(c)) {
            let (c, pos) = self.bump().expect("identifier continuation exists");
            raw.push(c);
            last = pos;
        }

        let mut is_qname = false;
        while self.peek() == Some('.') && matches!(self.peek_n(1), Some(c) if is_ident_start(c)) {
            is_qname = true;
            let (dot, dot_pos) = self.bump().expect("dot exists");
            raw.push(dot);
            last = dot_pos;
            while matches!(self.peek(), Some(c) if is_ident_continue(c)) {
                let (c, pos) = self.bump().expect("identifier continuation exists");
                raw.push(c);
                last = pos;
            }
        }

        let kind = if is_qname {
            TokenKind::Qname(raw)
        } else if let Some(kw) = keyword_from(&raw) {
            TokenKind::Keyword(kw)
        } else {
            TokenKind::Ident(raw)
        };
        self.tokens.push(Token {
            kind,
            span: start.to_span(last),
        });
    }

    fn lex_date_or_number(&mut self) {
        if let Some((text, end)) = self.try_lex_date() {
            let start = self.current_position();
            for _ in 0..text.len() {
                let _ = self.bump();
            }
            self.tokens.push(Token {
                kind: TokenKind::Date(text),
                span: start.to_span(end),
            });
            return;
        }

        let start = self.current_position();
        let mut text = String::new();
        let mut last = start;

        while matches!(self.peek(), Some(c) if c.is_ascii_digit() || c == '_') {
            let (c, pos) = self.bump().expect("number character exists");
            text.push(c);
            last = pos;
        }

        if self.peek() == Some('.') && matches!(self.peek_n(1), Some(c) if c.is_ascii_digit()) {
            let (dot, dot_pos) = self.bump().expect("dot exists");
            text.push(dot);
            last = dot_pos;
            while matches!(self.peek(), Some(c) if c.is_ascii_digit() || c == '_') {
                let (c, pos) = self.bump().expect("decimal character exists");
                text.push(c);
                last = pos;
            }
        }

        self.tokens.push(Token {
            kind: TokenKind::Number(text),
            span: start.to_span(last),
        });
    }

    fn try_lex_date(&self) -> Option<(String, Position)> {
        let remaining = &self.chars[self.idx..];
        if remaining.len() < 7 {
            return None;
        }

        let try_len = if remaining.len() >= 10
            && remaining[0].is_ascii_digit()
            && remaining[1].is_ascii_digit()
            && remaining[2].is_ascii_digit()
            && remaining[3].is_ascii_digit()
            && remaining[4] == '-'
            && remaining[5].is_ascii_digit()
            && remaining[6].is_ascii_digit()
            && remaining[7] == '-'
            && remaining[8].is_ascii_digit()
            && remaining[9].is_ascii_digit()
        {
            Some(10)
        } else if remaining[0].is_ascii_digit()
            && remaining[1].is_ascii_digit()
            && remaining[2].is_ascii_digit()
            && remaining[3].is_ascii_digit()
            && remaining[4] == '-'
            && remaining[5].is_ascii_digit()
            && remaining[6].is_ascii_digit()
        {
            Some(7)
        } else {
            None
        }?;

        if matches!(
            remaining.get(try_len),
            Some(c) if c.is_ascii_alphanumeric() || *c == '_' || *c == '-'
        ) {
            return None;
        }

        let mut line = self.line;
        let mut col = self.col;
        let mut end = Position { line, col };
        let mut text = String::with_capacity(try_len);
        for c in remaining.iter().take(try_len) {
            text.push(*c);
            end = Position { line, col };
            if *c == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        Some((text, end))
    }

    fn lex_string(&mut self) {
        let start = self.current_position();
        let mut last = start;
        let mut value = String::new();
        let _ = self.bump();

        while let Some((c, pos)) = self.bump() {
            last = pos;
            if c == '"' {
                self.tokens.push(Token {
                    kind: TokenKind::String(value),
                    span: start.to_span(last),
                });
                return;
            }
            if c == '\\' {
                if let Some((escaped, escaped_pos)) = self.bump() {
                    last = escaped_pos;
                    value.push(match escaped {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        '"' => '"',
                        '\\' => '\\',
                        other => other,
                    });
                    continue;
                }
                self.diagnostics.push(LexDiagnostic {
                    code: "E0002_UNTERMINATED_STRING",
                    message: "Unterminated string literal.".to_string(),
                    span: start.to_span(last),
                });
                return;
            }
            value.push(c);
        }

        self.diagnostics.push(LexDiagnostic {
            code: "E0002_UNTERMINATED_STRING",
            message: "Unterminated string literal.".to_string(),
            span: start.to_span(last),
        });
    }

    fn lex_punctuation_or_skip_unknown(&mut self) {
        let start = self.current_position();
        let Some((c, pos)) = self.bump() else {
            return;
        };
        let kind = match c {
            '{' => Some(Punct::LBrace),
            '}' => Some(Punct::RBrace),
            '(' => Some(Punct::LParen),
            ')' => Some(Punct::RParen),
            '[' => Some(Punct::LBracket),
            ']' => Some(Punct::RBracket),
            ':' => Some(Punct::Colon),
            ',' => Some(Punct::Comma),
            '~' => Some(Punct::Tilde),
            '+' => Some(Punct::Plus),
            '-' => Some(Punct::Minus),
            '*' => Some(Punct::Star),
            '/' => Some(Punct::Slash),
            '%' => Some(Punct::Percent),
            '^' => Some(Punct::Caret),
            '=' => {
                if self.peek() == Some('=') {
                    let _ = self.bump();
                    Some(Punct::EqEq)
                } else {
                    Some(Punct::Equal)
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    let _ = self.bump();
                    Some(Punct::NotEq)
                } else {
                    None
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    let _ = self.bump();
                    Some(Punct::Le)
                } else {
                    Some(Punct::Lt)
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    let _ = self.bump();
                    Some(Punct::Ge)
                } else {
                    Some(Punct::Gt)
                }
            }
            '.' => {
                if self.peek() == Some('.') {
                    let _ = self.bump();
                    Some(Punct::DotDot)
                } else {
                    Some(Punct::Dot)
                }
            }
            _ => None,
        };

        if let Some(punct) = kind {
            let end = if matches!(
                punct,
                Punct::DotDot | Punct::EqEq | Punct::NotEq | Punct::Le | Punct::Ge
            ) {
                Position {
                    line: self.line,
                    col: self.col.saturating_sub(1),
                }
            } else {
                pos
            };
            self.tokens.push(Token {
                kind: TokenKind::Punct(punct),
                span: start.to_span(end),
            });
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.idx).copied()
    }

    fn peek_n(&self, n: usize) -> Option<char> {
        self.chars.get(self.idx + n).copied()
    }

    fn current_position(&self) -> Position {
        Position {
            line: self.line,
            col: self.col,
        }
    }

    fn bump(&mut self) -> Option<(char, Position)> {
        let ch = self.peek()?;
        let pos = self.current_position();
        self.idx += 1;
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some((ch, pos))
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn keyword_from(s: &str) -> Option<Keyword> {
    Some(match s {
        "version" => Keyword::Version,
        "model" => Keyword::Model,
        "currency" => Keyword::Currency,
        "use" => Keyword::Use,
        "pack" => Keyword::Pack,
        "import" => Keyword::Import,
        "as" => Keyword::As,
        "time" => Keyword::Time,
        "calendar" => Keyword::Calendar,
        "from" => Keyword::From,
        "for" => Keyword::For,
        "daily" => Keyword::Daily,
        "monthly" => Keyword::Monthly,
        "quarterly" => Keyword::Quarterly,
        "annual" => Keyword::Annual,
        "phase" => Keyword::Phase,
        "to" => Keyword::To,
        "entity" => Keyword::Entity,
        "assume" => Keyword::Assume,
        "contract" => Keyword::Contract,
        "on" => Keyword::On,
        "term" => Keyword::Term,
        "terms" => Keyword::Terms,
        "effects" => Keyword::Effects,
        "parties" => Keyword::Parties,
        "tags" => Keyword::Tags,
        "stream" => Keyword::Stream,
        "owner" => Keyword::Owner,
        "direction" => Keyword::Direction,
        "inflow" => Keyword::Inflow,
        "outflow" => Keyword::Outflow,
        "schedule" => Keyword::Schedule,
        "every" => Keyword::Every,
        "phase_enter" => Keyword::PhaseEnter,
        "phase_start" => Keyword::PhaseStart,
        "phase_end" => Keyword::PhaseEnd,
        "day" => Keyword::Day,
        "eom" => Keyword::Eom,
        "Mon" => Keyword::Mon,
        "Tue" => Keyword::Tue,
        "Wed" => Keyword::Wed,
        "Thu" => Keyword::Thu,
        "Fri" => Keyword::Fri,
        "Sat" => Keyword::Sat,
        "Sun" => Keyword::Sun,
        "convention" => Keyword::Convention,
        "stub" => Keyword::Stub,
        "except" => Keyword::Except,
        "also" => Keyword::Also,
        "none" => Keyword::None,
        "following" => Keyword::Following,
        "modified_following" => Keyword::ModifiedFollowing,
        "preceding" => Keyword::Preceding,
        "modified_preceding" => Keyword::ModifiedPreceding,
        "short_front" => Keyword::ShortFront,
        "short_back" => Keyword::ShortBack,
        "long_front" => Keyword::LongFront,
        "long_back" => Keyword::LongBack,
        "event" => Keyword::Event,
        "when" => Keyword::When,
        "set" => Keyword::Set,
        "activate" => Keyword::Activate,
        "deactivate" => Keyword::Deactivate,
        "exercise" => Keyword::Exercise,
        "option" => Keyword::Option,
        "type" => Keyword::Type,
        "exercisable" => Keyword::Exercisable,
        "in" => Keyword::In,
        "payoff" => Keyword::Payoff,
        "run" => Keyword::Run,
        "curve" => Keyword::Curve,
        "deterministic" => Keyword::Deterministic,
        "monte_carlo" => Keyword::MonteCarlo,
        "trials" => Keyword::Trials,
        "seed" => Keyword::Seed,

        "true" => Keyword::True,
        "false" => Keyword::False,
        "Normal" => Keyword::Normal,
        "LogNormal" => Keyword::LogNormal,
        "Uniform" => Keyword::Uniform,
        "Triangular" => Keyword::Triangular,
        "clip" => Keyword::Clip,
        "active" => Keyword::Active,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexes_keywords_literals_and_comments() {
        let src = r#"version 0.1
// line comment
time calendar monthly from 2026-01 for 12
entity ns Name: pack.Type { k "v" }
/* block comment */
"#;
        let (tokens, diags) = lex(src);
        assert!(diags.is_empty());
        assert!(tokens
            .iter()
            .any(|t| matches!(t.kind, TokenKind::Keyword(Keyword::Version))));
        assert!(tokens
            .iter()
            .any(|t| matches!(t.kind, TokenKind::Number(ref s) if s == "0.1")));
        assert!(tokens
            .iter()
            .any(|t| matches!(t.kind, TokenKind::Date(ref s) if s == "2026-01")));
        assert!(tokens
            .iter()
            .any(|t| matches!(t.kind, TokenKind::Qname(ref s) if s == "pack.Type")));
    }

    #[test]
    fn reports_unterminated_string() {
        let src = "model \"oops";
        let (_tokens, diags) = lex(src);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "E0002_UNTERMINATED_STRING");
        assert_eq!(diags[0].span.start_line, 1);
        assert_eq!(diags[0].span.start_col, 7);
    }

    #[test]
    fn reports_unterminated_block_comment() {
        let src = "/* oops";
        let (_tokens, diags) = lex(src);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "E0003_UNTERMINATED_BLOCK_COMMENT");
        assert_eq!(diags[0].span.start_line, 1);
        assert_eq!(diags[0].span.start_col, 1);
    }
}
