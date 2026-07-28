use crate::date::CalcDate;
use rust_decimal::Decimal;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(Decimal),
    Bool(bool),
    Text(String),
    Date(CalcDate),
    /// Absent value (e.g. entity state not yet set by any event). Equality
    /// comparisons work (`null == null`, `null != anything-else`); ordering
    /// and arithmetic are errors.
    Null,
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "number",
            Value::Bool(_) => "bool",
            Value::Text(_) => "text",
            Value::Date(_) => "date",
            Value::Null => "null",
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(d) => write!(f, "{d}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Text(s) => write!(f, "{s}"),
            Value::Date(d) => write!(f, "{d}"),
            Value::Null => write!(f, "null"),
        }
    }
}
