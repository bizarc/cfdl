use crate::funcs;
use crate::parser::{BinOp, Expr, ExprKind, UnOp};
use crate::token::Span;
use crate::value::Value;
use crate::CalcError;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
// Exempt from the workspace's BTreeMap-only rule (see clippy.toml). `MapEnv` is
// a test and embedding helper whose map is only ever inserted into and looked
// up by key — it is never iterated, so no result can depend on its order. A
// HashMap here that DID get iterated would still be caught, because the
// exemption is on this declaration rather than on the module.
#[allow(clippy::disallowed_types)]
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

/// How a series reduction folds the per-period aggregate it is given.
///
/// EVERY REDUCTION FOLDS THE PER-PERIOD AGGREGATE, never the flattened cells.
/// `series_sum("dbt.*", …)` adds the matched streams within a period and then
/// across periods, and because addition is associative the order was invisible.
/// It is NOT invisible for a maximum: the peak of the combined position and the
/// largest single cell are different numbers, and only the first is what "peak
/// outstanding debt" means. So the host aggregates within each period first,
/// and this says what to do with the resulting vector (`docs/13` §7.86).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeriesReduction {
    Sum,
    Mean,
    Max,
    Min,
    Product,
    /// How many periods in the window carry a non-zero aggregate.
    CountNonZero,
}

impl SeriesReduction {
    pub fn of(name: &str) -> Option<Self> {
        match name {
            "series_sum" => Some(Self::Sum),
            "series_avg" => Some(Self::Mean),
            "series_max" => Some(Self::Max),
            "series_min" => Some(Self::Min),
            "series_prod" => Some(Self::Product),
            "series_count" => Some(Self::CountNonZero),
            _ => None,
        }
    }

    /// What an EMPTY SELECTION reduces to, when there is an answer at all.
    ///
    /// A `.*` selector may legitimately match nothing, and each fold has its
    /// own honest answer: nothing sums to 0 and multiplies to 1, and no period
    /// carries a non-zero aggregate. A maximum is different — nothing has no
    /// maximum, and returning 0 would state a value where there is none, which
    /// is the exact failure §7.86 exists to end.
    pub fn empty_selection(self) -> Option<f64> {
        match self {
            Self::Sum | Self::Mean | Self::CountNonZero => Some(0.0),
            Self::Product => Some(1.0),
            Self::Max | Self::Min => None,
        }
    }
}

/// Variable resolution: dotted paths like `time.t` or `contract.term_months`.
pub trait Env {
    fn lookup(&self, path: &str) -> Option<Value>;

    /// Host hook for cross-stream series reduction. `name` may be an exact
    /// stream name or a `prefix.*` wildcard; the period window [from, to] is
    /// inclusive and clamped by the host. Returns None when the host provides
    /// no series (e.g. plain expression contexts), which surfaces as an
    /// evaluation error — and, for `Max`/`Min`, when the selection is empty,
    /// because nothing has no maximum.
    fn series_aggregate(
        &self,
        _name: &str,
        _from: i64,
        _to: i64,
        _reduce: SeriesReduction,
    ) -> Option<Decimal> {
        None
    }

    /// Host hook for named curve lookup (`curve_value`). Returns the curve's
    /// value at `date` per the curve's declared interpolation, or None when
    /// the host has no curve by that name (which surfaces as an evaluation
    /// error).
    fn curve_value(&self, _name: &str, _date: crate::CalcDate) -> Option<Decimal> {
        None
    }

    /// Host hook for `quantile_at`: the declared quantile's value at a
    /// cumulative share. None when the host has no quantile by that name.
    fn quantile_at(&self, _name: &str, _share: Decimal) -> Option<Decimal> {
        None
    }

    /// Host hook for `quantile_mean`: the mean value over a share slice — the
    /// partial expectation the dispersion payoffs need.
    fn quantile_mean(&self, _name: &str, _from: Decimal, _to: Decimal) -> Option<Decimal> {
        None
    }

    /// Host hook for `quantile_of`: the share at or below which a value sits.
    /// The inverse of `quantile_at`.
    fn quantile_of(&self, _name: &str, _value: Decimal) -> Option<Decimal> {
        None
    }

    /// Host hook for a PARTICIPANT'S realised return — `irr("party.lp")` and
    /// `moic("party.lp")`, folded over that party's account: contributions as
    /// negative inflows, receipts as allocations.
    ///
    /// None by default, which is what gates these to the valuation plane: only
    /// the host evaluating a declared metric supplies the flows, so the same
    /// call in a stream amount is an evaluation error rather than a
    /// circularity. `kind` is "irr" or "moic".
    fn participant_return(&self, _party: &str, _kind: &str) -> Option<Decimal> {
        None
    }
}

#[derive(Debug, Default, Clone)]
pub struct MapEnv {
    #[allow(clippy::disallowed_types)] // insert/lookup only; never iterated
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
            // A PARTICIPANT'S RETURN NAMES AN ENTITY, so it takes the
            // reference and not its value. `party.lp` is an entity like any
            // other — the language says `pay … to party.lp` and
            // `owner party.lp` — and a string would drop what a reference
            // carries: the compiler resolves it, checks it is a party, and a
            // typo is a diagnostic rather than a run-time surprise.
            if name == "irr" || name == "moic" {
                let path = match args.first().map(|a| &a.kind) {
                    Some(ExprKind::Var(path)) if args.len() == 1 => path.clone(),
                    _ => {
                        return Err(CalcError::new(
                            format!("{name} takes one argument: a party, as a reference — {name}(party.<name>)."),
                            Some(expr.span),
                        ));
                    }
                };
                return match env.participant_return(&path, name) {
                    Some(value) => Ok(Value::Number(value)),
                    None => Err(CalcError::new(
                        format!(
                            "{name}({path}) is not available here. A participant's return is a fold over the finished projection, so it is computed in a `metric` declaration and nowhere else."
                        ),
                        Some(expr.span),
                    )),
                };
            }
            let mut values = Vec::with_capacity(args.len());
            for arg in args {
                values.push((eval(arg, env, mode)?, arg.span));
            }
            if SeriesReduction::of(name).is_some() {
                return funcs::series_call(name, &values, expr.span, env);
            }
            if name == "curve_value" {
                return funcs::curve_call(name, &values, expr.span, env);
            }
            if name == "quantile_at" || name == "quantile_mean" || name == "quantile_of" {
                return funcs::quantile_call(name, &values, expr.span, env);
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
