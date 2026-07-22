use crate::token::{lex, Span, Tok, Token};
use crate::CalcError;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Number(Decimal),
    Bool(bool),
    Str(String),
    /// Dotted variable path, e.g. `time.t` or `escalation`.
    Var(String),
    Unary {
        op: UnOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

pub fn parse(source: &str) -> Result<Expr, CalcError> {
    let tokens = lex(source)?;
    let mut p = Parser {
        tokens,
        pos: 0,
        src_len: source.len(),
    };
    let expr = p.parse_or()?;
    if p.pos < p.tokens.len() {
        let t = &p.tokens[p.pos];
        return Err(CalcError::new(
            format!("unexpected token after expression: {:?}", t.tok),
            Some(t.span),
        ));
    }
    Ok(expr)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    src_len: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.tokens.get(self.pos).map(|t| &t.tok)
    }

    fn peek_span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map(|t| t.span)
            .unwrap_or(Span::new(self.src_len, self.src_len))
    }

    fn bump(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, tok: Tok, what: &str) -> Result<Span, CalcError> {
        match self.tokens.get(self.pos) {
            Some(t) if t.tok == tok => {
                let span = t.span;
                self.pos += 1;
                Ok(span)
            }
            Some(t) => Err(CalcError::new(
                format!("expected {what}, found {:?}", t.tok),
                Some(t.span),
            )),
            None => Err(CalcError::new(
                format!("expected {what}, found end of expression"),
                Some(Span::new(self.src_len, self.src_len)),
            )),
        }
    }

    fn parse_or(&mut self) -> Result<Expr, CalcError> {
        let mut lhs = self.parse_and()?;
        while matches!(self.peek(), Some(Tok::Or)) {
            self.bump();
            let rhs = self.parse_and()?;
            let span = lhs.span.merge(rhs.span);
            lhs = Expr {
                kind: ExprKind::Binary {
                    op: BinOp::Or,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span,
            };
        }
        Ok(lhs)
    }

    fn parse_and(&mut self) -> Result<Expr, CalcError> {
        let mut lhs = self.parse_not()?;
        while matches!(self.peek(), Some(Tok::And)) {
            self.bump();
            let rhs = self.parse_not()?;
            let span = lhs.span.merge(rhs.span);
            lhs = Expr {
                kind: ExprKind::Binary {
                    op: BinOp::And,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span,
            };
        }
        Ok(lhs)
    }

    fn parse_not(&mut self) -> Result<Expr, CalcError> {
        if matches!(self.peek(), Some(Tok::Not)) {
            let start = self.peek_span();
            self.bump();
            let expr = self.parse_not()?;
            let span = start.merge(expr.span);
            return Ok(Expr {
                kind: ExprKind::Unary {
                    op: UnOp::Not,
                    expr: Box::new(expr),
                },
                span,
            });
        }
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Expr, CalcError> {
        let lhs = self.parse_additive()?;
        let op = match self.peek() {
            Some(Tok::Eq) => Some(BinOp::Eq),
            Some(Tok::Ne) => Some(BinOp::Ne),
            Some(Tok::Lt) => Some(BinOp::Lt),
            Some(Tok::Le) => Some(BinOp::Le),
            Some(Tok::Gt) => Some(BinOp::Gt),
            Some(Tok::Ge) => Some(BinOp::Ge),
            _ => None,
        };
        if let Some(op) = op {
            self.bump();
            let rhs = self.parse_additive()?;
            let span = lhs.span.merge(rhs.span);
            return Ok(Expr {
                kind: ExprKind::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span,
            });
        }
        Ok(lhs)
    }

    fn parse_additive(&mut self) -> Result<Expr, CalcError> {
        let mut lhs = self.parse_multiplicative()?;
        loop {
            let op = match self.peek() {
                Some(Tok::Plus) => BinOp::Add,
                Some(Tok::Minus) => BinOp::Sub,
                _ => break,
            };
            self.bump();
            let rhs = self.parse_multiplicative()?;
            let span = lhs.span.merge(rhs.span);
            lhs = Expr {
                kind: ExprKind::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span,
            };
        }
        Ok(lhs)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, CalcError> {
        let mut lhs = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Some(Tok::Star) => BinOp::Mul,
                Some(Tok::Slash) => BinOp::Div,
                Some(Tok::Percent) => BinOp::Rem,
                _ => break,
            };
            self.bump();
            let rhs = self.parse_unary()?;
            let span = lhs.span.merge(rhs.span);
            lhs = Expr {
                kind: ExprKind::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span,
            };
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Expr, CalcError> {
        if matches!(self.peek(), Some(Tok::Minus)) {
            let start = self.peek_span();
            self.bump();
            let expr = self.parse_unary()?;
            let span = start.merge(expr.span);
            return Ok(Expr {
                kind: ExprKind::Unary {
                    op: UnOp::Neg,
                    expr: Box::new(expr),
                },
                span,
            });
        }
        self.parse_power()
    }

    fn parse_power(&mut self) -> Result<Expr, CalcError> {
        let base = self.parse_primary()?;
        if matches!(self.peek(), Some(Tok::Caret)) {
            self.bump();
            // Right-associative; exponent may itself be unary (`2 ^ -1`).
            let exp = self.parse_unary_for_power()?;
            let span = base.span.merge(exp.span);
            return Ok(Expr {
                kind: ExprKind::Binary {
                    op: BinOp::Pow,
                    lhs: Box::new(base),
                    rhs: Box::new(exp),
                },
                span,
            });
        }
        Ok(base)
    }

    fn parse_unary_for_power(&mut self) -> Result<Expr, CalcError> {
        if matches!(self.peek(), Some(Tok::Minus)) {
            let start = self.peek_span();
            self.bump();
            let expr = self.parse_unary_for_power()?;
            let span = start.merge(expr.span);
            return Ok(Expr {
                kind: ExprKind::Unary {
                    op: UnOp::Neg,
                    expr: Box::new(expr),
                },
                span,
            });
        }
        self.parse_power()
    }

    fn parse_primary(&mut self) -> Result<Expr, CalcError> {
        let Some(token) = self.bump() else {
            return Err(CalcError::new(
                "unexpected end of expression",
                Some(Span::new(self.src_len, self.src_len)),
            ));
        };
        match token.tok {
            Tok::Number(value) => Ok(Expr {
                kind: ExprKind::Number(value),
                span: token.span,
            }),
            Tok::True => Ok(Expr {
                kind: ExprKind::Bool(true),
                span: token.span,
            }),
            Tok::False => Ok(Expr {
                kind: ExprKind::Bool(false),
                span: token.span,
            }),
            Tok::Str(s) => Ok(Expr {
                kind: ExprKind::Str(s),
                span: token.span,
            }),
            Tok::LParen => {
                let inner = self.parse_or()?;
                let close = self.expect(Tok::RParen, "`)`")?;
                Ok(Expr {
                    kind: inner.kind,
                    span: token.span.merge(close),
                })
            }
            Tok::Ident(first) => {
                // Function call: bare identifier followed by `(`.
                if matches!(self.peek(), Some(Tok::LParen)) {
                    self.bump();
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Some(Tok::RParen)) {
                        loop {
                            args.push(self.parse_or()?);
                            if matches!(self.peek(), Some(Tok::Comma)) {
                                self.bump();
                            } else {
                                break;
                            }
                        }
                    }
                    let close = self.expect(Tok::RParen, "`)`")?;
                    return Ok(Expr {
                        kind: ExprKind::Call { name: first, args },
                        span: token.span.merge(close),
                    });
                }
                // Dotted variable path: ident (`.` ident)*
                let mut path = first;
                let mut span = token.span;
                while matches!(self.peek(), Some(Tok::Dot)) {
                    self.bump();
                    match self.bump() {
                        Some(Token {
                            tok: Tok::Ident(seg),
                            span: seg_span,
                        }) => {
                            path.push('.');
                            path.push_str(&seg);
                            span = span.merge(seg_span);
                        }
                        Some(t) => {
                            return Err(CalcError::new(
                                format!("expected identifier after `.`, found {:?}", t.tok),
                                Some(t.span),
                            ));
                        }
                        None => {
                            return Err(CalcError::new(
                                "expected identifier after `.`",
                                Some(Span::new(self.src_len, self.src_len)),
                            ));
                        }
                    }
                }
                Ok(Expr {
                    kind: ExprKind::Var(path),
                    span,
                })
            }
            other => Err(CalcError::new(
                format!("unexpected token {other:?}"),
                Some(token.span),
            )),
        }
    }
}
