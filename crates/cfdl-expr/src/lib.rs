//! cfdl-expr — the expression evaluation facade used by the compiler and engine.
//!
//! This is backed by `cfdl-calc` (the CFDL-native expression
//! engine: decimal-first numerics, spanned diagnostics, snake_case builtins).
//! The public API — `compile_expr`, `eval`, `ExprEnv`, `Value` — is unchanged
//! from the CEL era so `cfdl-compile` and `cfdl-engine` did not have to change.
//!
//! Boundary semantics during the migration:
//! - Env values arrive as f64-based `Value`s (the engine is not yet
//!   decimal-native); they are bridged with `Decimal::from_f64` (nearest
//!   decimal), evaluated exactly, and numeric results are returned as
//!   `Value::Decimal(f64)`.
//! - Evaluation runs in `cfdl_calc::Mode::Decimal`. The `excel_compat` mode is
//!   plumbed for the benchmark harness via `eval_with_mode`.

pub use cfdl_calc::Mode;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use std::collections::BTreeMap;
use std::sync::Arc;

// --- Domain types (stable API surface) ---

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Decimal(f64),
    String(String),
    Date(Date),
    Currency(String),
    Money(Money),
    Optional(Option<Box<Value>>),
    Map(BTreeMap<String, Value>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Money {
    pub amount: f64,
    pub currency: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprEnv {
    pub model: BTreeMap<String, Value>,
    pub time: BTreeMap<String, Value>,
    pub entity: BTreeMap<String, Value>,
    pub cfg: BTreeMap<String, Value>,
    pub obs: BTreeMap<String, Value>,
    /// Assumption values (`assume` statements), referenced as `inputs.<name>`.
    pub inputs: BTreeMap<String, Value>,
    /// Per-period stream series (signed amounts) available to `series_sum` /
    /// `series_avg`. Populated by the engine for phase-2 stream evaluation;
    /// empty elsewhere.
    ///
    /// Shared rather than owned: an env is built per accrual, and copying every
    /// stream's full series into each one made this the hot spot of a run.
    /// `Arc` makes handing it over a refcount bump. Nothing mutates it.
    pub series: Arc<BTreeMap<String, Vec<f64>>>,
    /// Named date-indexed value curves (`curve` statements) available to
    /// `curve_value(name, date)`. Populated by the engine from IR; empty
    /// elsewhere.
    pub curves: BTreeMap<String, CurveDef>,
    /// Declared states at the CURRENT period, read as `state.<name>`.
    /// Populated when evaluating a stream; empty when evaluating a state's
    /// own `next`, which is what makes a same-period read unreachable rather
    /// than merely rejected. See docs/14_state_and_recurrence.md.
    pub states: BTreeMap<String, Value>,
    /// Declared states at the PREVIOUS period, read as `prev.<name>`.
    /// The mirror of the above: populated for `next`, empty for a stream.
    pub prev_states: BTreeMap<String, Value>,
    /// The state being evaluated, at the previous period — bare `prev`.
    pub prev_self: Option<Value>,
}

/// A named curve: date/value points plus interpolation policy.
#[derive(Debug, Clone, PartialEq)]
pub struct CurveDef {
    /// "step" (flat-forward: last point at or before the query date; the
    /// first value before the first point) or "linear" (linear in calendar
    /// days between bracketing points, clamped flat outside the range).
    pub interpolation: String,
    /// Points sorted ascending by date.
    pub points: Vec<(Date, f64)>,
}

impl ExprEnv {
    pub fn empty() -> Self {
        Self {
            model: BTreeMap::new(),
            time: BTreeMap::new(),
            entity: BTreeMap::new(),
            cfg: BTreeMap::new(),
            obs: BTreeMap::new(),
            inputs: BTreeMap::new(),
            states: BTreeMap::new(),
            prev_states: BTreeMap::new(),
            prev_self: None,
            series: Arc::default(),
            curves: BTreeMap::new(),
        }
    }
}

// --- Error types ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprSpan {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprError {
    pub code: String,
    pub message: String,
    pub span: Option<ExprSpan>,
}

impl std::fmt::Display for ExprError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for ExprError {}

fn parse_error(e: cfdl_calc::CalcError) -> ExprError {
    ExprError {
        code: "EXPR_PARSE".to_string(),
        message: e.message,
        span: e.span.map(|s| ExprSpan {
            start: s.start,
            end: s.end,
        }),
    }
}

fn eval_error(e: cfdl_calc::CalcError) -> ExprError {
    ExprError {
        code: "EXPR_EVAL".to_string(),
        message: e.message,
        span: e.span.map(|s| ExprSpan {
            start: s.start,
            end: s.end,
        }),
    }
}

// --- Compilation & evaluation ---

#[derive(Debug, Clone)]
pub struct CompiledExpr {
    expr: Arc<cfdl_calc::Expr>,
}

pub fn compile_expr(src: &str) -> Result<CompiledExpr, ExprError> {
    cfdl_calc::parse(src)
        .map(|expr| CompiledExpr {
            expr: Arc::new(expr),
        })
        .map_err(parse_error)
}

/// Does this expression call `series_sum` / `series_avg`? The engine uses
/// this to schedule the stream into the second evaluation phase.
pub fn uses_series(compiled: &CompiledExpr) -> bool {
    cfdl_calc::expr_calls_any(&compiled.expr, &["series_sum", "series_avg"])
}

pub fn eval(compiled: &CompiledExpr, env: &ExprEnv) -> Result<Value, ExprError> {
    eval_with_mode(compiled, env, Mode::Decimal)
}

pub fn eval_with_mode(
    compiled: &CompiledExpr,
    env: &ExprEnv,
    mode: Mode,
) -> Result<Value, ExprError> {
    let adapter = EnvAdapter { env };
    let result = cfdl_calc::eval(&compiled.expr, &adapter, mode).map_err(eval_error)?;
    Ok(calc_to_domain(result))
}

// --- Env bridging ---

struct EnvAdapter<'a> {
    env: &'a ExprEnv,
}

impl cfdl_calc::Env for EnvAdapter<'_> {
    fn lookup(&self, path: &str) -> Option<cfdl_calc::Value> {
        let mut parts = path.split('.');
        let root = parts.next()?;
        // `state.<name>` is the current period; `prev` and `prev.<name>` are the
        // previous one. Each pair of maps is populated in exactly one context —
        // states for a stream, prev for a `next` — so the same-period edge a
        // recurrence could otherwise create is absent rather than rejected.
        if root == "state" {
            let name = parts.next()?;
            return self.env.states.get(name).and_then(domain_to_calc);
        }
        if root == "prev" {
            return match parts.next() {
                None => self.env.prev_self.as_ref().and_then(domain_to_calc),
                Some(name) => self.env.prev_states.get(name).and_then(domain_to_calc),
            };
        }
        let map = match root {
            "model" => &self.env.model,
            "time" => &self.env.time,
            "entity" => &self.env.entity,
            "cfg" => &self.env.cfg,
            "obs" => &self.env.obs,
            "inputs" => &self.env.inputs,
            _ => return None,
        };
        // The `entity` root is open-world: entity state fields may not exist
        // until an event sets them, and expressions like
        // `entity.status != \"refinanced\"` must evaluate (to null) before
        // that. Other roots stay strict so typos are hard errors.
        let open_world = root == "entity";
        let first = parts.next()?;
        let Some(mut current) = map.get(first) else {
            return open_world.then_some(cfdl_calc::Value::Null);
        };
        for segment in parts {
            let unwrapped = match unwrap_optional(current) {
                Some(v) => v,
                None => return Some(cfdl_calc::Value::Null),
            };
            match unwrapped {
                Value::Map(m) => match m.get(segment) {
                    Some(next) => current = next,
                    // Missing key inside a map value resolves to null.
                    None => return Some(cfdl_calc::Value::Null),
                },
                _ => return None,
            }
        }
        match unwrap_optional(current) {
            Some(v) => domain_to_calc(v),
            None => Some(cfdl_calc::Value::Null),
        }
    }

    fn series_aggregate(&self, name: &str, from: i64, to: i64, mean: bool) -> Option<Decimal> {
        if self.env.series.is_empty() {
            return None;
        }
        let matched = self.matching_series(name);
        if matched.is_empty() {
            // Unknown series name: aggregate over nothing = 0 (streams that
            // never lowered contribute nothing, mirroring metric sums).
            return Decimal::from_f64(0.0);
        }
        // Inclusive window, clamped to available periods (projection tail
        // included). The divisor for series_avg is the REQUESTED window
        // length, so a window that extends past the data averages the
        // available amounts over the full window.
        let mut total = 0.0_f64;
        for series in matched {
            let lo = from.max(0) as usize;
            let hi = to.min(series.len() as i64 - 1);
            if hi < lo as i64 {
                continue;
            }
            total += series[lo..=hi as usize].iter().sum::<f64>();
        }
        if mean {
            let window = (to - from + 1).max(1);
            total /= window as f64;
        }
        Decimal::from_f64(total)
    }

    fn curve_value(&self, name: &str, date: cfdl_calc::CalcDate) -> Option<Decimal> {
        self.curve_lookup(name, date).and_then(Decimal::from_f64)
    }
}

impl EnvAdapter<'_> {
    fn curve_lookup(&self, name: &str, date: cfdl_calc::CalcDate) -> Option<f64> {
        let curve = self.env.curves.get(name)?;
        let epoch =
            |d: &Date| cfdl_calc::CalcDate::new(d.year, d.month, d.day).map(|c| c.to_epoch_days());
        let query = date.to_epoch_days();
        let mut points: Vec<(i64, f64)> = Vec::with_capacity(curve.points.len());
        for (d, v) in &curve.points {
            points.push((epoch(d)?, *v));
        }
        if points.is_empty() {
            return None;
        }
        let first = points[0];
        if query <= first.0 {
            return Some(first.1);
        }
        let last = points[points.len() - 1];
        if query >= last.0 {
            return Some(last.1);
        }
        // points bracketing the query: prev.0 <= query < next.0
        let idx = points.partition_point(|(d, _)| *d <= query);
        let prev = points[idx - 1];
        if curve.interpolation == "linear" {
            let next = points[idx];
            let frac = (query - prev.0) as f64 / (next.0 - prev.0) as f64;
            Some(prev.1 + (next.1 - prev.1) * frac)
        } else {
            // step (flat-forward)
            Some(prev.1)
        }
    }

    fn matching_series(&self, name: &str) -> Vec<&Vec<f64>> {
        if let Some(prefix) = name.strip_suffix(".*") {
            let dot_prefix = format!("{prefix}.");
            self.env
                .series
                .iter()
                .filter(|(key, _)| key.as_str() == prefix || key.starts_with(&dot_prefix))
                .map(|(_, v)| v)
                .collect()
        } else {
            self.env.series.get(name).into_iter().collect()
        }
    }
}

fn unwrap_optional(v: &Value) -> Option<&Value> {
    match v {
        Value::Optional(Some(inner)) => unwrap_optional(inner),
        Value::Optional(None) => None,
        other => Some(other),
    }
}

fn domain_to_calc(v: &Value) -> Option<cfdl_calc::Value> {
    match v {
        Value::Bool(b) => Some(cfdl_calc::Value::Bool(*b)),
        Value::Int(i) => Some(cfdl_calc::Value::Number(Decimal::from(*i))),
        // Bridge from the f64 world: nearest decimal (documented boundary).
        Value::Decimal(f) => Decimal::from_f64(*f).map(cfdl_calc::Value::Number),
        Value::String(s) => Some(cfdl_calc::Value::Text(s.clone())),
        Value::Currency(c) => Some(cfdl_calc::Value::Text(c.clone())),
        Value::Date(d) => {
            cfdl_calc::CalcDate::new(d.year, d.month, d.day).map(cfdl_calc::Value::Date)
        }
        Value::Money(m) => Decimal::from_f64(m.amount).map(cfdl_calc::Value::Number),
        Value::Optional(_) => unwrap_optional(v).and_then(domain_to_calc),
        // Maps are traversed by dotted path in `lookup`; a map is not itself a value.
        Value::Map(_) => None,
    }
}

fn calc_to_domain(v: cfdl_calc::Value) -> Value {
    match v {
        cfdl_calc::Value::Number(d) => Value::Decimal(d.to_f64().unwrap_or(f64::NAN)),
        cfdl_calc::Value::Bool(b) => Value::Bool(b),
        cfdl_calc::Value::Text(s) => Value::String(s),
        cfdl_calc::Value::Date(d) => Value::Date(Date {
            year: d.year(),
            month: d.month(),
            day: d.day(),
        }),
        cfdl_calc::Value::Null => Value::Optional(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_eval_with_env() {
        let compiled = compile_expr("model.base + 10.0").expect("compile");
        let mut env = ExprEnv::empty();
        env.model.insert("base".to_string(), Value::Int(5));
        let result = eval(&compiled, &env).expect("eval");
        assert_eq!(result, Value::Decimal(15.0));
    }

    #[test]
    fn legacy_cel_corpus_parses_and_evaluates() {
        // Representative expressions from the pre-migration fixture corpus.
        let mut env = ExprEnv::empty();
        env.time.insert("t".to_string(), Value::Int(24));
        env.cfg.insert("base".to_string(), Value::Decimal(100.0));
        env.obs.insert("rate".to_string(), Value::Decimal(0.06));

        let cases: &[(&str, Value)] = &[
            // 120000 * 1.03^23, decimal-exact = 236830.3813351925455536160439
            (
                "120000 * pow(1.03, time.t - 1)",
                Value::Decimal(236830.38133519256),
            ),
            (
                "35000 * clamp((time.t - 36.0 + 1.0) / 6.0, 0.0, 1.0)",
                Value::Decimal(0.0),
            ),
            ("2000 + time.t * 50", Value::Decimal(3200.0)),
            ("cfg.base + time.t * 10", Value::Decimal(340.0)),
            ("obs.rate", Value::Decimal(0.06)),
            ("time.t >= 12", Value::Bool(true)),
            ("time.t < 3", Value::Bool(false)),
            ("180000 / 0.06", Value::Decimal(3000000.0)),
        ];
        for (src, expected) in cases {
            let compiled = compile_expr(src).expect(src);
            let got = eval(&compiled, &env).expect(src);
            match (&got, expected) {
                (Value::Decimal(g), Value::Decimal(e)) => {
                    assert!((g - e).abs() < 1e-6, "{src}: got {g}, expected {e}")
                }
                _ => assert_eq!(&got, expected, "{src}"),
            }
        }
    }

    #[test]
    fn parse_error_has_code_and_span() {
        let err = compile_expr("1 + ").unwrap_err();
        assert_eq!(err.code, "EXPR_PARSE");
        assert!(err.span.is_some());
    }

    #[test]
    fn unknown_variable_is_eval_error() {
        let compiled = compile_expr("nope.missing + 1").expect("compile");
        let err = eval(&compiled, &ExprEnv::empty()).unwrap_err();
        assert_eq!(err.code, "EXPR_EVAL");
        assert!(err.message.contains("nope.missing"));
    }

    #[test]
    fn curve_value_step_and_linear() {
        let mut env = ExprEnv::empty();
        env.curves.insert(
            "sofr".to_string(),
            CurveDef {
                interpolation: "step".to_string(),
                points: vec![
                    (
                        Date {
                            year: 2026,
                            month: 1,
                            day: 1,
                        },
                        0.045,
                    ),
                    (
                        Date {
                            year: 2026,
                            month: 7,
                            day: 1,
                        },
                        0.042,
                    ),
                ],
            },
        );
        env.curves.insert(
            "ramp".to_string(),
            CurveDef {
                interpolation: "linear".to_string(),
                // 2026-01-01 -> 2026-01-11: 10 days, 0.0 -> 1.0
                points: vec![
                    (
                        Date {
                            year: 2026,
                            month: 1,
                            day: 1,
                        },
                        0.0,
                    ),
                    (
                        Date {
                            year: 2026,
                            month: 1,
                            day: 11,
                        },
                        1.0,
                    ),
                ],
            },
        );
        let cases: &[(&str, f64)] = &[
            // step: before first -> first; between -> last at-or-before; after last -> last
            ("curve_value(\"sofr\", date(2025, 6, 1))", 0.045),
            ("curve_value(\"sofr\", date(2026, 1, 1))", 0.045),
            ("curve_value(\"sofr\", date(2026, 6, 30))", 0.045),
            ("curve_value(\"sofr\", date(2026, 7, 1))", 0.042),
            ("curve_value(\"sofr\", date(2027, 1, 1))", 0.042),
            // linear: interpolate by calendar days, clamp outside
            ("curve_value(\"ramp\", date(2026, 1, 1))", 0.0),
            ("curve_value(\"ramp\", date(2026, 1, 4))", 0.3),
            ("curve_value(\"ramp\", date(2026, 1, 11))", 1.0),
            ("curve_value(\"ramp\", date(2026, 2, 1))", 1.0),
        ];
        for (src, expected) in cases {
            let compiled = compile_expr(src).expect(src);
            let Value::Decimal(got) = eval(&compiled, &env).expect(src) else {
                panic!("{src}: non-numeric result");
            };
            assert!((got - expected).abs() < 1e-12, "{src}: got {got}");
        }
    }

    #[test]
    fn curve_value_unknown_curve_is_eval_error() {
        let compiled = compile_expr("curve_value(\"missing\", date(2026, 1, 1))").expect("compile");
        let err = eval(&compiled, &ExprEnv::empty()).unwrap_err();
        assert!(err.message.contains("missing"), "{err}");
    }

    #[test]
    fn nested_map_paths_resolve() {
        let mut env = ExprEnv::empty();
        let mut terms = BTreeMap::new();
        terms.insert("term_months".to_string(), Value::Int(120));
        env.entity.insert("contract".to_string(), Value::Map(terms));
        let compiled = compile_expr("entity.contract.term_months / 12").expect("compile");
        assert_eq!(eval(&compiled, &env).unwrap(), Value::Decimal(10.0));
    }
}
