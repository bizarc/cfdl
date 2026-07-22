//! cfdl-expr — the expression evaluation facade used by the compiler and engine.
//!
//! As of Workstream B this is backed by `cfdl-calc` (the CFDL-native expression
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
    fn nested_map_paths_resolve() {
        let mut env = ExprEnv::empty();
        let mut terms = BTreeMap::new();
        terms.insert("term_months".to_string(), Value::Int(120));
        env.entity.insert("contract".to_string(), Value::Map(terms));
        let compiled = compile_expr("entity.contract.term_months / 12").expect("compile");
        assert_eq!(eval(&compiled, &env).unwrap(), Value::Decimal(10.0));
    }
}
