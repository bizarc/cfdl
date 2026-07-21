pub use cel_interpreter::{Context, Program, Value as CelValue};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;

// --- Domain Types (Restored for Compatibility) ---

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
}

impl ExprEnv {
    pub fn empty() -> Self {
        Self {
            model: BTreeMap::new(),
            time: BTreeMap::new(),
            entity: BTreeMap::new(),
            cfg: BTreeMap::new(),
            obs: BTreeMap::new(),
        }
    }
}

// --- Error Types ---

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

// --- Compilation & Evaluation ---

#[derive(Debug, Clone)]
pub struct CompiledExpr {
    program: Arc<Program>,
}

pub fn compile_expr(src: &str) -> Result<CompiledExpr, ExprError> {
    Program::compile(src)
        .map(|program| CompiledExpr {
            program: Arc::new(program),
        })
        .map_err(|e| ExprError {
            code: "CEL_COMPILE".to_string(),
            message: e.to_string(),
            span: None,
        })
}

pub fn eval(compiled: &CompiledExpr, env: &ExprEnv) -> Result<Value, ExprError> {
    let mut ctx = Context::default();

    // Register root variables
    ctx.add_variable("model", env_map_to_cel(&env.model))
        .map_err(|e| ExprError {
            code: "CTX".into(),
            message: e.to_string(),
            span: None,
        })?;
    ctx.add_variable("time", env_map_to_cel(&env.time))
        .map_err(|e| ExprError {
            code: "CTX".into(),
            message: e.to_string(),
            span: None,
        })?;
    ctx.add_variable("entity", env_map_to_cel(&env.entity))
        .map_err(|e| ExprError {
            code: "CTX".into(),
            message: e.to_string(),
            span: None,
        })?;
    ctx.add_variable("cfg", env_map_to_cel(&env.cfg))
        .map_err(|e| ExprError {
            code: "CTX".into(),
            message: e.to_string(),
            span: None,
        })?;
    ctx.add_variable("obs", env_map_to_cel(&env.obs))
        .map_err(|e| ExprError {
            code: "CTX".into(),
            message: e.to_string(),
            span: None,
        })?;

    // Register standard library functions missing in CEL-Rust or custom to CFDL
    ctx.add_function("clamp", |val: f64, min: f64, max: f64| -> f64 {
        val.clamp(min, max)
    });
    ctx.add_function("pow", |base: f64, exp: f64| -> f64 { base.powf(exp) });

    // Execute
    let cel_result = compiled.program.execute(&ctx).map_err(|e| ExprError {
        code: "CEL_EXEC".to_string(),
        message: e.to_string(),
        span: None,
    })?;

    // Convert back to Domain Value
    Ok(cel_value_to_domain(cel_result))
}

// --- Converters ---

use cel_interpreter::objects::{Key, Map as CelMap};

fn env_map_to_cel(map: &BTreeMap<String, Value>) -> CelValue {
    let mut hm = HashMap::new();
    for (k, v) in map {
        hm.insert(k.clone().into(), domain_value_to_cel(v));
    }
    CelValue::Map(CelMap { map: Arc::new(hm) })
}

fn domain_value_to_cel(v: &Value) -> CelValue {
    match v {
        Value::Bool(b) => CelValue::Bool(*b),
        // Coerce Int to Float to allow mixed arithmetic (e.g. 1 / 0.5) which CEL-Rust doesn't support
        Value::Int(i) => CelValue::Float(*i as f64),
        Value::Decimal(f) => CelValue::Float(*f),
        Value::String(s) => CelValue::String(Arc::new(s.clone())),
        Value::Date(d) => {
            // Convert Date to Map {year, month, day}
            let mut m = HashMap::new();
            m.insert("year".into(), CelValue::Int(d.year as i64));
            m.insert("month".into(), CelValue::UInt(d.month as u64));
            m.insert("day".into(), CelValue::UInt(d.day as u64));
            m.insert(
                "_type".into(),
                CelValue::String(Arc::new("Date".to_string())),
            );
            CelValue::Map(CelMap { map: Arc::new(m) })
        }
        Value::Currency(c) => CelValue::String(Arc::new(c.clone())), // Treat currency as string
        Value::Money(m) => {
            // Convert Money to Map {amount, currency}
            let mut map = HashMap::new();
            map.insert("amount".into(), CelValue::Float(m.amount));
            map.insert(
                "currency".into(),
                CelValue::String(Arc::new(m.currency.clone())),
            );
            map.insert(
                "_type".into(),
                CelValue::String(Arc::new("Money".to_string())),
            );
            CelValue::Map(CelMap { map: Arc::new(map) })
        }
        Value::Optional(opt) => match opt {
            Some(inner) => domain_value_to_cel(inner),
            None => CelValue::Null,
        },
        Value::Map(m) => env_map_to_cel(m),
    }
}

fn cel_value_to_domain(v: CelValue) -> Value {
    match v {
        CelValue::Bool(b) => Value::Bool(b),
        CelValue::Int(i) => Value::Int(i),
        CelValue::UInt(u) => Value::Int(u as i64),
        CelValue::Float(f) => Value::Decimal(f),
        CelValue::String(s) => Value::String(s.to_string()),
        CelValue::Bytes(_) => Value::String("<bytes>".to_string()), // Unsupported
        CelValue::List(_l) => {
            // Unsupported list type in legacy Value
            Value::String("<list>".to_string())
        }
        CelValue::Map(m) => {
            // Check for Duck Types
            // Key::from needs specific types usually, but let's try strict matching
            let amount_key = Key::from("amount");
            let currency_key = Key::from("currency");
            let type_key = Key::from("_type");

            if let (Some(a), Some(c), Some(t)) = (
                m.map.get(&amount_key),
                m.map.get(&currency_key),
                m.map.get(&type_key),
            ) {
                if let (
                    CelValue::Float(amount),
                    CelValue::String(curr),
                    CelValue::String(type_str),
                ) = (a, c, t)
                {
                    if type_str.as_str() == "Money" {
                        return Value::Money(Money {
                            amount: *amount,
                            currency: curr.to_string(),
                        });
                    }
                }
            }

            let year_key = Key::from("year");
            let month_key = Key::from("month");
            let day_key = Key::from("day");

            if let (Some(y), Some(mo), Some(d), Some(CelValue::String(type_str))) = (
                m.map.get(&year_key),
                m.map.get(&month_key),
                m.map.get(&day_key),
                m.map.get(&type_key),
            ) {
                if type_str.as_str() == "Date" {
                    let year = match y {
                        CelValue::Int(i) => *i as i32,
                        _ => 0,
                    };
                    let month = match mo {
                        CelValue::UInt(u) => *u as u32,
                        CelValue::Int(i) => *i as u32,
                        _ => 0,
                    };
                    let day = match d {
                        CelValue::UInt(u) => *u as u32,
                        CelValue::Int(i) => *i as u32,
                        _ => 0,
                    };
                    return Value::Date(Date { year, month, day });
                }
            }

            // Standard Map
            let mut bm = BTreeMap::new();
            for (k, v) in m.map.iter() {
                // Key to string conversion might need helpers if Key isn't effectively string
                // cel_interpreter Key is usually String/Int/Bool.
                let k_str = match k {
                    Key::String(s) => s.to_string(),
                    Key::Bool(b) => b.to_string(),
                    Key::Int(i) => i.to_string(),
                    Key::Uint(u) => u.to_string(),
                };
                bm.insert(k_str, cel_value_to_domain(v.clone()));
            }
            Value::Map(bm)
        }
        CelValue::Null => Value::Optional(None),
        _ => Value::String(format!("{:?}", v)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_eval() {
        // Source should use floats as cfdl-compile coerces them.
        let src = "model.base + 10.0";
        let compiled = compile_expr(src).expect("compile");

        let mut env = ExprEnv::empty();
        env.model.insert("base".to_string(), Value::Int(5)); // Converted to 5.0

        let result = eval(&compiled, &env).expect("eval");
        match result {
            Value::Decimal(f) => assert_eq!(f, 15.0),
            _ => panic!("Expected Decimal(15.0), got {:?}", result),
        }
    }

    #[test]
    fn test_duck_typing_money() {
        // CEL: {'amount': 100.0, 'currency': 'USD', '_type': 'Money'}
        let src = "{'amount': 100.0, 'currency': 'USD', '_type': 'Money'}";
        let compiled = compile_expr(src).expect("compile");
        let env = ExprEnv::empty();
        let result = eval(&compiled, &env).expect("eval");
        match result {
            Value::Money(m) => {
                assert_eq!(m.amount, 100.0);
                assert_eq!(m.currency, "USD");
            }
            _ => panic!("Expected Money, got {:?}", result),
        }
    }
}
