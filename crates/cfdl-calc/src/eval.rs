use crate::funcs;
use crate::parser::{BinOp, Expr, ExprKind, UnOp};
use crate::token::Span;
use crate::value::Value;
use crate::CalcError;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Numeric evaluation mode.
///
/// `Decimal` is the default: exact decimal money math, float64 only for
/// transcendental operations. `ExcelCompat` performs ALL arithmetic in IEEE-754
/// float64 to reproduce Excel's representation artifacts — used by benchmark
/// harnesses to match Excel references and to quantify decimal-vs-float deltas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Decimal,
    ExcelCompat,
}

/// Variable resolution: dotted paths like `time.t` or `contract.term_months`.
pub trait Env {
    fn lookup(&self, path: &str) -> Option<Value>;

    /// Host hook for cross-stream series aggregation (`series_sum` /
    /// `series_avg`). `name` may be an exact stream name or a `prefix.*`
    /// wildcard; the period window [from, to] is inclusive and clamped by the
    /// host. Returns None when the host provides no series (e.g. plain
    /// expression contexts), which surfaces as an evaluation error.
    fn series_aggregate(&self, _name: &str, _from: i64, _to: i64, _mean: bool) -> Option<Decimal> {
        None
    }

    /// Host hook for named curve lookup (`curve_value`). Returns the curve's
    /// value at `date` per the curve's declared interpolation, or None when
    /// the host has no curve by that name (which surfaces as an evaluation
    /// error).
    fn curve_value(&self, _name: &str, _date: crate::CalcDate) -> Option<Decimal> {
        None
    }
}

#[derive(Debug, Default, Clone)]
pub struct MapEnv {
    vars: HashMap<String, Value>,
}

impl MapEnv {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, path: impl Into<String>, value: Value) {
        self.vars.insert(path.into(), value);
    }
}

impl Env for MapEnv {
    fn lookup(&self, path: &str) -> Option<Value> {
        self.vars.get(path).cloned()
    }
}

pub fn eval(expr: &Expr, env: &dyn Env, mode: Mode) -> Result<Value, CalcError> {
    match &expr.kind {
        ExprKind::Number(d) => Ok(Value::Number(*d)),
        ExprKind::Bool(b) => Ok(Value::Bool(*b)),
        ExprKind::Str(s) => Ok(Value::Text(s.clone())),
        ExprKind::Var(path) => env
            .lookup(path)
            .ok_or_else(|| CalcError::new(format!("unknown variable `{path}`"), Some(expr.span))),
        ExprKind::Unary { op, expr: inner } => {
            let v = eval(inner, env, mode)?;
            match (op, v) {
                (UnOp::Neg, Value::Number(d)) => Ok(Value::Number(-d)),
                (UnOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                (op, v) => Err(CalcError::new(
                    format!("cannot apply {op:?} to {}", v.type_name()),
                    Some(expr.span),
                )),
            }
        }
        ExprKind::Binary { op, lhs, rhs } => {
            // Short-circuit logic first: rhs must not be evaluated eagerly.
            if matches!(op, BinOp::And | BinOp::Or) {
                let l = bool_of(eval(lhs, env, mode)?, lhs.span)?;
                return match (op, l) {
                    (BinOp::And, false) => Ok(Value::Bool(false)),
                    (BinOp::Or, true) => Ok(Value::Bool(true)),
                    _ => Ok(Value::Bool(bool_of(eval(rhs, env, mode)?, rhs.span)?)),
                };
            }
            let l = eval(lhs, env, mode)?;
            let r = eval(rhs, env, mode)?;
            binary(*op, l, r, expr.span, mode)
        }
        ExprKind::Call { name, args } => {
            // `if` is a special form: only the taken branch is evaluated.
            if name == "if" {
                if args.len() != 3 {
                    return Err(CalcError::new(
                        format!("if expects 3 arguments, got {}", args.len()),
                        Some(expr.span),
                    ));
                }
                let cond = bool_of(eval(&args[0], env, mode)?, args[0].span)?;
                let taken = if cond { &args[1] } else { &args[2] };
                return eval(taken, env, mode);
            }
            let mut values = Vec::with_capacity(args.len());
            for arg in args {
                values.push((eval(arg, env, mode)?, arg.span));
            }
            if name == "series_sum" || name == "series_avg" {
                return funcs::series_call(name, &values, expr.span, env);
            }
            if name == "curve_value" {
                return funcs::curve_call(name, &values, expr.span, env);
            }
            funcs::call(name, &values, expr.span, mode)
        }
    }
}

fn bool_of(v: Value, span: Span) -> Result<bool, CalcError> {
    match v {
        Value::Bool(b) => Ok(b),
        other => Err(CalcError::new(
            format!("expected bool, got {}", other.type_name()),
            Some(span),
        )),
    }
}

fn binary(op: BinOp, l: Value, r: Value, span: Span, mode: Mode) -> Result<Value, CalcError> {
    use BinOp::*;
    match (op, &l, &r) {
        // Date arithmetic: date - date = days, date +/- number = shifted date.
        (Sub, Value::Date(a), Value::Date(b)) => {
            Ok(Value::Number(Decimal::from(b.days_between(a))))
        }
        (Add, Value::Date(d), Value::Number(n)) | (Add, Value::Number(n), Value::Date(d)) => {
            let days = int_of(*n, span)?;
            Ok(Value::Date(d.add_days(days)))
        }
        (Sub, Value::Date(d), Value::Number(n)) => {
            let days = int_of(*n, span)?;
            Ok(Value::Date(d.add_days(-days)))
        }
        (Add | Sub | Mul | Div | Rem | Pow, Value::Number(a), Value::Number(b)) => {
            Ok(Value::Number(arith(op, *a, *b, span, mode)?))
        }
        (Eq, _, _) => Ok(Value::Bool(values_eq(&l, &r, mode))),
        (Ne, _, _) => Ok(Value::Bool(!values_eq(&l, &r, mode))),
        (Lt | Le | Gt | Ge, _, _) => {
            let ord = compare(&l, &r, span, mode)?;
            Ok(Value::Bool(match op {
                Lt => ord.is_lt(),
                Le => ord.is_le(),
                Gt => ord.is_gt(),
                Ge => ord.is_ge(),
                _ => unreachable!(),
            }))
        }
        _ => Err(CalcError::new(
            format!(
                "cannot apply {:?} to {} and {}",
                op,
                l.type_name(),
                r.type_name()
            ),
            Some(span),
        )),
    }
}

fn values_eq(l: &Value, r: &Value, mode: Mode) -> bool {
    match (l, r) {
        (Value::Number(a), Value::Number(b)) => match mode {
            Mode::Decimal => a == b,
            Mode::ExcelCompat => a.to_f64().unwrap_or(f64::NAN) == b.to_f64().unwrap_or(f64::NAN),
        },
        _ => l == r,
    }
}

fn compare(l: &Value, r: &Value, span: Span, mode: Mode) -> Result<std::cmp::Ordering, CalcError> {
    match (l, r) {
        (Value::Number(a), Value::Number(b)) => match mode {
            Mode::Decimal => Ok(a.cmp(b)),
            Mode::ExcelCompat => {
                let (af, bf) = (
                    a.to_f64().unwrap_or(f64::NAN),
                    b.to_f64().unwrap_or(f64::NAN),
                );
                af.partial_cmp(&bf)
                    .ok_or_else(|| CalcError::new("cannot compare non-finite numbers", Some(span)))
            }
        },
        (Value::Date(a), Value::Date(b)) => Ok(a.cmp(b)),
        (Value::Text(a), Value::Text(b)) => Ok(a.cmp(b)),
        _ => Err(CalcError::new(
            format!("cannot compare {} and {}", l.type_name(), r.type_name()),
            Some(span),
        )),
    }
}

fn int_of(n: Decimal, span: Span) -> Result<i64, CalcError> {
    if n.fract() != Decimal::ZERO {
        return Err(CalcError::new(
            format!("expected a whole number of days, got {n}"),
            Some(span),
        ));
    }
    n.to_i64()
        .ok_or_else(|| CalcError::new(format!("number out of range: {n}"), Some(span)))
}

fn arith(op: BinOp, a: Decimal, b: Decimal, span: Span, mode: Mode) -> Result<Decimal, CalcError> {
    if mode == Mode::ExcelCompat {
        return arith_f64(op, a, b, span);
    }
    match op {
        BinOp::Add => a
            .checked_add(b)
            .ok_or_else(|| CalcError::new("numeric overflow in +", Some(span))),
        BinOp::Sub => a
            .checked_sub(b)
            .ok_or_else(|| CalcError::new("numeric overflow in -", Some(span))),
        BinOp::Mul => a
            .checked_mul(b)
            .ok_or_else(|| CalcError::new("numeric overflow in *", Some(span))),
        BinOp::Div => {
            if b.is_zero() {
                return Err(CalcError::new("division by zero", Some(span)));
            }
            a.checked_div(b)
                .ok_or_else(|| CalcError::new("numeric overflow in /", Some(span)))
        }
        BinOp::Rem => {
            if b.is_zero() {
                return Err(CalcError::new("division by zero in %", Some(span)));
            }
            a.checked_rem(b)
                .ok_or_else(|| CalcError::new("numeric overflow in %", Some(span)))
        }
        BinOp::Pow => pow_decimal(a, b, span),
        _ => unreachable!("non-arithmetic op in arith()"),
    }
}

fn arith_f64(op: BinOp, a: Decimal, b: Decimal, span: Span) -> Result<Decimal, CalcError> {
    let (af, bf) = (to_f64(a, span)?, to_f64(b, span)?);
    let out = match op {
        BinOp::Add => af + bf,
        BinOp::Sub => af - bf,
        BinOp::Mul => af * bf,
        BinOp::Div => {
            if bf == 0.0 {
                return Err(CalcError::new("division by zero", Some(span)));
            }
            af / bf
        }
        BinOp::Rem => {
            if bf == 0.0 {
                return Err(CalcError::new("division by zero in %", Some(span)));
            }
            af % bf
        }
        BinOp::Pow => af.powf(bf),
        _ => unreachable!(),
    };
    // Retain full f64-derived digits: Excel keeps raw IEEE-754 precision through
    // a formula's evaluation, so compat mode must not round between operations
    // (e.g. `0.1 + 0.2 - 0.3` = 5.5511e-17 in Excel, not 0).
    from_f64_retain(out, span)
}

fn from_f64_retain(x: f64, span: Span) -> Result<Decimal, CalcError> {
    if !x.is_finite() {
        return Err(CalcError::new("non-finite result", Some(span)));
    }
    Decimal::from_f64_retain(x)
        .ok_or_else(|| CalcError::new(format!("result out of decimal range: {x}"), Some(span)))
}

/// Power respecting the evaluation mode; shared by the `^` operator and the
/// `pow()` builtin.
pub(crate) fn pow_mode(
    base: Decimal,
    exp: Decimal,
    span: Span,
    mode: Mode,
) -> Result<Decimal, CalcError> {
    match mode {
        Mode::Decimal => pow_decimal(base, exp, span),
        Mode::ExcelCompat => arith_f64(BinOp::Pow, base, exp, span),
    }
}

/// `^` in decimal mode: exact repeated-squaring for integer exponents; the
/// documented float64 escape for fractional exponents.
fn pow_decimal(base: Decimal, exp: Decimal, span: Span) -> Result<Decimal, CalcError> {
    if exp.fract() == Decimal::ZERO {
        if let Some(n) = exp.to_i64() {
            if n.unsigned_abs() <= 10_000 {
                return powi_decimal(base, n, span);
            }
        }
    }
    // f64_escape: fractional or extreme exponent.
    let out = to_f64(base, span)?.powf(to_f64(exp, span)?);
    from_f64(out, span)
}

pub(crate) fn powi_decimal(base: Decimal, n: i64, span: Span) -> Result<Decimal, CalcError> {
    if n < 0 {
        if base.is_zero() {
            return Err(CalcError::new("zero to a negative power", Some(span)));
        }
        let pos = powi_decimal(base, -n, span)?;
        return Decimal::ONE
            .checked_div(pos)
            .ok_or_else(|| CalcError::new("numeric overflow in ^", Some(span)));
    }
    let mut result = Decimal::ONE;
    let mut factor = base;
    let mut k = n;
    while k > 0 {
        if k & 1 == 1 {
            result = result
                .checked_mul(factor)
                .ok_or_else(|| CalcError::new("numeric overflow in ^", Some(span)))?;
        }
        k >>= 1;
        if k > 0 {
            factor = factor
                .checked_mul(factor)
                .ok_or_else(|| CalcError::new("numeric overflow in ^", Some(span)))?;
        }
    }
    Ok(result)
}

pub(crate) fn to_f64(d: Decimal, span: Span) -> Result<f64, CalcError> {
    d.to_f64()
        .ok_or_else(|| CalcError::new(format!("cannot represent {d} as float"), Some(span)))
}

pub(crate) fn from_f64(x: f64, span: Span) -> Result<Decimal, CalcError> {
    if !x.is_finite() {
        return Err(CalcError::new("non-finite result", Some(span)));
    }
    Decimal::from_f64(x)
        .ok_or_else(|| CalcError::new(format!("result out of decimal range: {x}"), Some(span)))
}
