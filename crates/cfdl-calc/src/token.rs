use crate::CalcError;
use rust_decimal::Decimal;
use std::str::FromStr;

/// Byte-offset range into the expression source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn merge(self, other: Span) -> Span {
        Span::new(self.start.min(other.start), self.end.max(other.end))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Number(Decimal),
    Ident(String),
    Str(String),
    True,
    False,
    And,
    Or,
    Not,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    LParen,
    RParen,
    Comma,
    Dot,
    Eq, // == (and `=` alias)
    Ne, // != (and `<>` alias)
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub tok: Tok,
    pub span: Span,
}

pub fn lex(src: &str) -> Result<Vec<Token>, CalcError> {
    let bytes = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        let start = i;
        match c {
            ' ' | '\t' | '\r' | '\n' => {
                i += 1;
            }
            '0'..='9' => {
                let mut j = i + 1;
                let mut seen_dot = false;
                while j < bytes.len() {
                    let d = bytes[j] as char;
                    if d.is_ascii_digit() {
                        j += 1;
                    } else if d == '.'
                        && !seen_dot
                        && j + 1 < bytes.len()
                        && (bytes[j + 1] as char).is_ascii_digit()
                    {
                        seen_dot = true;
                        j += 1;
                    } else if d == '_' {
                        j += 1;
                    } else {
                        break;
                    }
                }
                let raw: String = src[i..j].chars().filter(|&ch| ch != '_').collect();
                let value = Decimal::from_str(&raw).map_err(|e| {
                    CalcError::new(
                        format!("invalid number `{raw}`: {e}"),
                        Some(Span::new(i, j)),
                    )
                })?;
                out.push(Token {
                    tok: Tok::Number(value),
                    span: Span::new(i, j),
                });
                i = j;
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut j = i + 1;
                while j < bytes.len() {
                    let d = bytes[j] as char;
                    if d.is_ascii_alphanumeric() || d == '_' {
                        j += 1;
                    } else {
                        break;
                    }
                }
                let word = &src[i..j];
                let tok = match word {
                    "and" => Tok::And,
                    "or" => Tok::Or,
                    "not" => Tok::Not,
                    "true" => Tok::True,
                    "false" => Tok::False,
                    _ => Tok::Ident(word.to_string()),
                };
                out.push(Token {
                    tok,
                    span: Span::new(i, j),
                });
                i = j;
            }
            '"' => {
                let mut j = i + 1;
                let mut s = String::new();
                let mut closed = false;
                while j < bytes.len() {
                    let d = bytes[j] as char;
                    if d == '\\' && j + 1 < bytes.len() {
                        let e = bytes[j + 1] as char;
                        s.push(match e {
                            'n' => '\n',
                            't' => '\t',
                            other => other,
                        });
                        j += 2;
                    } else if d == '"' {
                        closed = true;
                        j += 1;
                        break;
                    } else {
                        s.push(d);
                        j += 1;
                    }
                }
                if !closed {
                    return Err(CalcError::new(
                        "unterminated string literal",
                        Some(Span::new(i, bytes.len())),
                    ));
                }
                out.push(Token {
                    tok: Tok::Str(s),
                    span: Span::new(i, j),
                });
                i = j;
            }
            '+' => {
                out.push(Token {
                    tok: Tok::Plus,
                    span: Span::new(i, i + 1),
                });
                i += 1;
            }
            '-' => {
                out.push(Token {
                    tok: Tok::Minus,
                    span: Span::new(i, i + 1),
                });
                i += 1;
            }
            '*' => {
                out.push(Token {
                    tok: Tok::Star,
                    span: Span::new(i, i + 1),
                });
                i += 1;
            }
            '/' => {
                out.push(Token {
                    tok: Tok::Slash,
                    span: Span::new(i, i + 1),
                });
                i += 1;
            }
            '%' => {
                out.push(Token {
                    tok: Tok::Percent,
                    span: Span::new(i, i + 1),
                });
                i += 1;
            }
            '^' => {
                out.push(Token {
                    tok: Tok::Caret,
                    span: Span::new(i, i + 1),
                });
                i += 1;
            }
            '(' => {
                out.push(Token {
                    tok: Tok::LParen,
                    span: Span::new(i, i + 1),
                });
                i += 1;
            }
            ')' => {
                out.push(Token {
                    tok: Tok::RParen,
                    span: Span::new(i, i + 1),
                });
                i += 1;
            }
            ',' => {
                out.push(Token {
                    tok: Tok::Comma,
                    span: Span::new(i, i + 1),
                });
                i += 1;
            }
            '.' => {
                out.push(Token {
                    tok: Tok::Dot,
                    span: Span::new(i, i + 1),
                });
                i += 1;
            }
            '=' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    out.push(Token {
                        tok: Tok::Eq,
                        span: Span::new(i, i + 2),
                    });
                    i += 2;
                } else {
                    // Excel-style single `=` equality alias.
                    out.push(Token {
                        tok: Tok::Eq,
                        span: Span::new(i, i + 1),
                    });
                    i += 1;
                }
            }
            '!' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    out.push(Token {
                        tok: Tok::Ne,
                        span: Span::new(i, i + 2),
                    });
                    i += 2;
                } else {
                    return Err(CalcError::new(
                        "unexpected `!` (use `not` or `!=`)",
                        Some(Span::new(i, i + 1)),
                    ));
                }
            }
            '<' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    out.push(Token {
                        tok: Tok::Le,
                        span: Span::new(i, i + 2),
                    });
                    i += 2;
                } else if i + 1 < bytes.len() && bytes[i + 1] == b'>' {
                    // Excel-style `<>` inequality alias.
                    out.push(Token {
                        tok: Tok::Ne,
                        span: Span::new(i, i + 2),
                    });
                    i += 2;
                } else {
                    out.push(Token {
                        tok: Tok::Lt,
                        span: Span::new(i, i + 1),
                    });
                    i += 1;
                }
            }
            '>' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    out.push(Token {
                        tok: Tok::Ge,
                        span: Span::new(i, i + 2),
                    });
                    i += 2;
                } else {
                    out.push(Token {
                        tok: Tok::Gt,
                        span: Span::new(i, i + 1),
                    });
                    i += 1;
                }
            }
            other => {
                return Err(CalcError::new(
                    format!("unexpected character `{other}`"),
                    Some(Span::new(start, start + other.len_utf8())),
                ));
            }
        }
    }
    Ok(out)
}
