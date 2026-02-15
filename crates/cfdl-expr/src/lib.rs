use std::collections::BTreeMap;

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

    fn resolve_path(&self, path: &[String]) -> Option<Value> {
        if path.is_empty() {
            return None;
        }
        let (root, tail) = path.split_first()?;
        let mut current = match root.as_str() {
            "model" => Value::Map(self.model.clone()),
            "time" => Value::Map(self.time.clone()),
            "entity" => Value::Map(self.entity.clone()),
            "cfg" => Value::Map(self.cfg.clone()),
            "obs" => Value::Map(self.obs.clone()),
            _ => return None,
        };
        for segment in tail {
            match current {
                Value::Map(map) => current = map.get(segment)?.clone(),
                _ => return None,
            }
        }
        Some(current)
    }
}

#[derive(Debug, Clone)]
pub struct CompiledExpr {
    ast: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprSpan {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprError {
    pub code: &'static str,
    pub message: String,
    pub span: Option<ExprSpan>,
}

pub fn compile_expr(src: &str) -> Result<CompiledExpr, ExprError> {
    let tokens = lex_expr(src)?;
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_expression()?;
    parser.expect_eof()?;
    validate_roots(&ast)?;
    let _ = infer_type(&ast)?;
    Ok(CompiledExpr { ast })
}

pub fn eval(compiled: &CompiledExpr, env: &ExprEnv) -> Result<Value, ExprError> {
    eval_expr(&compiled.ast, env)
}

#[derive(Debug, Clone)]
enum Expr {
    Bool(bool),
    Int(i64),
    Decimal(f64),
    String(String),
    Path(Vec<String>),
    Unary {
        op: UnOp,
        rhs: Box<Expr>,
    },
    Binary {
        lhs: Box<Expr>,
        op: BinOp,
        rhs: Box<Expr>,
    },
    Conditional {
        cond: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
    Call {
        name: Vec<String>,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone, Copy)]
enum UnOp {
    Not,
    Neg,
}

#[derive(Debug, Clone, Copy)]
enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Ty {
    Unknown,
    Bool,
    Int,
    Decimal,
    String,
    Date,
    Currency,
    Money,
    Optional,
}

fn validate_roots(expr: &Expr) -> Result<(), ExprError> {
    match expr {
        Expr::Path(path) => {
            let root = path.first().cloned().unwrap_or_default();
            if !is_known_root(&root) {
                return Err(unknown_ident(path.join(".")));
            }
            Ok(())
        }
        Expr::Unary { rhs, .. } => validate_roots(rhs),
        Expr::Binary { lhs, rhs, .. } => {
            validate_roots(lhs)?;
            validate_roots(rhs)
        }
        Expr::Conditional {
            cond,
            then_expr,
            else_expr,
        } => {
            validate_roots(cond)?;
            validate_roots(then_expr)?;
            validate_roots(else_expr)
        }
        Expr::Call { name, args } => {
            if let Some(root) = name.first() {
                let plain = name.join(".");
                let known_fn = matches!(
                    plain.as_str(),
                    "money"
                        | "money_amount"
                        | "money_currency"
                        | "add_months"
                        | "add_days"
                        | "days_between"
                        | "yearfrac"
                        | "min"
                        | "max"
                        | "clamp"
                        | "round"
                        | "obs.rate"
                        | "obs.fx"
                        | "obs.index"
                );
                if !(known_fn || is_known_root(root)) {
                    return Err(unknown_ident(plain));
                }
            }
            for arg in args {
                validate_roots(arg)?;
            }
            Ok(())
        }
        Expr::Bool(_) | Expr::Int(_) | Expr::Decimal(_) | Expr::String(_) => Ok(()),
    }
}

fn infer_type(expr: &Expr) -> Result<Ty, ExprError> {
    match expr {
        Expr::Bool(_) => Ok(Ty::Bool),
        Expr::Int(_) => Ok(Ty::Int),
        Expr::Decimal(_) => Ok(Ty::Decimal),
        Expr::String(_) => Ok(Ty::String),
        Expr::Path(_) => Ok(Ty::Unknown),
        Expr::Unary { op, rhs } => {
            let rt = infer_type(rhs)?;
            match op {
                UnOp::Not => {
                    if rt == Ty::Bool || rt == Ty::Unknown {
                        Ok(Ty::Bool)
                    } else {
                        Err(type_error("Unary '!' expects Bool"))
                    }
                }
                UnOp::Neg => {
                    if rt == Ty::Int || rt == Ty::Decimal || rt == Ty::Unknown {
                        Ok(rt)
                    } else {
                        Err(type_error("Unary '-' expects Int or Decimal"))
                    }
                }
            }
        }
        Expr::Binary { lhs, op, rhs } => {
            let lt = infer_type(lhs)?;
            let rt = infer_type(rhs)?;
            match op {
                BinOp::And | BinOp::Or => {
                    if (lt == Ty::Bool || lt == Ty::Unknown)
                        && (rt == Ty::Bool || rt == Ty::Unknown)
                    {
                        Ok(Ty::Bool)
                    } else {
                        Err(type_error("Logical operators require Bool operands"))
                    }
                }
                BinOp::Eq | BinOp::Ne => Ok(Ty::Bool),
                BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                    if compatible_cmp(&lt, &rt) {
                        Ok(Ty::Bool)
                    } else {
                        Err(type_error("Comparison operands are incompatible"))
                    }
                }
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                    infer_arithmetic_type(*op, &lt, &rt)
                }
            }
        }
        Expr::Conditional {
            cond,
            then_expr,
            else_expr,
        } => {
            let ct = infer_type(cond)?;
            if ct != Ty::Bool && ct != Ty::Unknown {
                return Err(type_error("Conditional expects Bool condition"));
            }
            let t1 = infer_type(then_expr)?;
            let t2 = infer_type(else_expr)?;
            if t1 == Ty::Unknown {
                return Ok(t2);
            }
            if t2 == Ty::Unknown || t1 == t2 {
                Ok(t1)
            } else {
                Err(type_error(
                    "Conditional branches must have compatible types",
                ))
            }
        }
        Expr::Call { name, args } => infer_call_type(name, args),
    }
}

fn compatible_cmp(a: &Ty, b: &Ty) -> bool {
    matches!(
        (a, b),
        (Ty::Unknown, _)
            | (_, Ty::Unknown)
            | (Ty::Int, Ty::Int)
            | (Ty::Int, Ty::Decimal)
            | (Ty::Decimal, Ty::Int)
            | (Ty::Decimal, Ty::Decimal)
            | (Ty::String, Ty::String)
            | (Ty::Date, Ty::Date)
            | (Ty::Currency, Ty::Currency)
    )
}

fn infer_arithmetic_type(op: BinOp, lt: &Ty, rt: &Ty) -> Result<Ty, ExprError> {
    if lt == &Ty::Unknown || rt == &Ty::Unknown {
        return Ok(Ty::Unknown);
    }
    match op {
        BinOp::Add => match (lt, rt) {
            (Ty::Int, Ty::Int) => Ok(Ty::Int),
            (Ty::Int, Ty::Decimal) | (Ty::Decimal, Ty::Int) | (Ty::Decimal, Ty::Decimal) => {
                Ok(Ty::Decimal)
            }
            (Ty::String, Ty::String) => Ok(Ty::String),
            (Ty::Money, Ty::Money) => Ok(Ty::Money),
            (Ty::Money, _) | (_, Ty::Money) => {
                Err(illegal_op("Money + X is only allowed for Money + Money"))
            }
            _ => Err(type_error("Arithmetic operands are incompatible")),
        },
        BinOp::Sub => match (lt, rt) {
            (Ty::Int, Ty::Int) => Ok(Ty::Int),
            (Ty::Int, Ty::Decimal) | (Ty::Decimal, Ty::Int) | (Ty::Decimal, Ty::Decimal) => {
                Ok(Ty::Decimal)
            }
            (Ty::Money, Ty::Money) => Ok(Ty::Money),
            (Ty::Money, _) | (_, Ty::Money) => {
                Err(illegal_op("Money - X is only allowed for Money - Money"))
            }
            _ => Err(type_error("Arithmetic operands are incompatible")),
        },
        BinOp::Mul => match (lt, rt) {
            (Ty::Int, Ty::Int) => Ok(Ty::Int),
            (Ty::Int, Ty::Decimal) | (Ty::Decimal, Ty::Int) | (Ty::Decimal, Ty::Decimal) => {
                Ok(Ty::Decimal)
            }
            (Ty::Money, Ty::Int)
            | (Ty::Money, Ty::Decimal)
            | (Ty::Int, Ty::Money)
            | (Ty::Decimal, Ty::Money) => Ok(Ty::Money),
            (Ty::Money, Ty::Money) => Err(illegal_op("Money * Money is not allowed")),
            _ => Err(type_error("Arithmetic operands are incompatible")),
        },
        BinOp::Div => match (lt, rt) {
            (Ty::Int, Ty::Int)
            | (Ty::Int, Ty::Decimal)
            | (Ty::Decimal, Ty::Int)
            | (Ty::Decimal, Ty::Decimal) => Ok(Ty::Decimal),
            (Ty::Money, Ty::Int) | (Ty::Money, Ty::Decimal) => Ok(Ty::Money),
            (Ty::Money, Ty::Money) => Err(illegal_op("Money / Money is not allowed")),
            _ => Err(type_error("Arithmetic operands are incompatible")),
        },
        _ => Ok(Ty::Unknown),
    }
}

fn infer_call_type(name: &[String], args: &[Expr]) -> Result<Ty, ExprError> {
    let n = name.join(".");
    match n.as_str() {
        "money" => {
            if args.len() != 2 {
                return Err(type_error("money(amount, currency) expects 2 arguments"));
            }
            let amount = infer_type(&args[0])?;
            let currency = infer_type(&args[1])?;
            if !matches!(amount, Ty::Int | Ty::Decimal | Ty::Unknown) {
                return Err(type_error("money amount must be numeric"));
            }
            if !matches!(currency, Ty::Currency | Ty::String | Ty::Unknown) {
                return Err(type_error("money currency must be Currency or String"));
            }
            Ok(Ty::Money)
        }
        "money_amount" => Ok(Ty::Decimal),
        "money_currency" => Ok(Ty::Currency),
        "min" | "max" | "clamp" | "round" | "yearfrac" => Ok(Ty::Decimal),
        "add_days" | "add_months" => Ok(Ty::Date),
        "days_between" => Ok(Ty::Int),
        "obs.rate" | "obs.index" | "obs.fx" => Ok(Ty::Optional),
        _ => Ok(Ty::Unknown),
    }
}

fn is_known_root(root: &str) -> bool {
    matches!(root, "model" | "time" | "entity" | "cfg" | "obs")
}

fn eval_expr(expr: &Expr, env: &ExprEnv) -> Result<Value, ExprError> {
    match expr {
        Expr::Bool(v) => Ok(Value::Bool(*v)),
        Expr::Int(v) => Ok(Value::Int(*v)),
        Expr::Decimal(v) => Ok(Value::Decimal(*v)),
        Expr::String(v) => Ok(Value::String(v.clone())),
        Expr::Path(path) => env
            .resolve_path(path)
            .ok_or_else(|| unknown_ident(path.join("."))),
        Expr::Unary { op, rhs } => {
            let v = eval_expr(rhs, env)?;
            match op {
                UnOp::Not => Ok(Value::Bool(expect_bool(v, "Unary '!' expects Bool")?)),
                UnOp::Neg => match v {
                    Value::Int(i) => Ok(Value::Int(-i)),
                    Value::Decimal(d) => Ok(Value::Decimal(-d)),
                    _ => Err(type_error("Unary '-' expects Int or Decimal")),
                },
            }
        }
        Expr::Binary { lhs, op, rhs } => {
            let l = eval_expr(lhs, env)?;
            let r = eval_expr(rhs, env)?;
            eval_binary(*op, l, r)
        }
        Expr::Conditional {
            cond,
            then_expr,
            else_expr,
        } => {
            let c = eval_expr(cond, env)?;
            if expect_bool(c, "Conditional expects Bool condition")? {
                eval_expr(then_expr, env)
            } else {
                eval_expr(else_expr, env)
            }
        }
        Expr::Call { name, args } => eval_call(name, args, env),
    }
}

fn eval_binary(op: BinOp, lhs: Value, rhs: Value) -> Result<Value, ExprError> {
    match op {
        BinOp::And => Ok(Value::Bool(
            expect_bool(lhs, "'&&' expects Bool operands")?
                && expect_bool(rhs, "'&&' expects Bool operands")?,
        )),
        BinOp::Or => Ok(Value::Bool(
            expect_bool(lhs, "'||' expects Bool operands")?
                || expect_bool(rhs, "'||' expects Bool operands")?,
        )),
        BinOp::Eq => Ok(Value::Bool(eq_values(&lhs, &rhs)?)),
        BinOp::Ne => Ok(Value::Bool(!eq_values(&lhs, &rhs)?)),
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => eval_compare(op, lhs, rhs),
        BinOp::Add => eval_add(lhs, rhs),
        BinOp::Sub => eval_sub(lhs, rhs),
        BinOp::Mul => eval_mul(lhs, rhs),
        BinOp::Div => eval_div(lhs, rhs),
    }
}

fn eval_add(lhs: Value, rhs: Value) -> Result<Value, ExprError> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
        (a, b) if is_numeric(&a) && is_numeric(&b) => {
            Ok(Value::Decimal(to_decimal(a)? + to_decimal(b)?))
        }
        (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{a}{b}"))),
        (Value::Money(a), Value::Money(b)) => {
            if a.currency != b.currency {
                return Err(illegal_op("Money + Money requires matching currency"));
            }
            Ok(Value::Money(Money {
                amount: a.amount + b.amount,
                currency: a.currency,
            }))
        }
        _ => Err(illegal_op("Illegal '+' operand types")),
    }
}

fn eval_sub(lhs: Value, rhs: Value) -> Result<Value, ExprError> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
        (a, b) if is_numeric(&a) && is_numeric(&b) => {
            Ok(Value::Decimal(to_decimal(a)? - to_decimal(b)?))
        }
        (Value::Money(a), Value::Money(b)) => {
            if a.currency != b.currency {
                return Err(illegal_op("Money - Money requires matching currency"));
            }
            Ok(Value::Money(Money {
                amount: a.amount - b.amount,
                currency: a.currency,
            }))
        }
        _ => Err(illegal_op("Illegal '-' operand types")),
    }
}

fn eval_mul(lhs: Value, rhs: Value) -> Result<Value, ExprError> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
        (a, b) if is_numeric(&a) && is_numeric(&b) => {
            Ok(Value::Decimal(to_decimal(a)? * to_decimal(b)?))
        }
        (Value::Money(m), b) if is_numeric(&b) => Ok(Value::Money(Money {
            amount: m.amount * to_decimal(b)?,
            currency: m.currency,
        })),
        (a, Value::Money(m)) if is_numeric(&a) => Ok(Value::Money(Money {
            amount: to_decimal(a)? * m.amount,
            currency: m.currency,
        })),
        _ => Err(illegal_op("Illegal '*' operand types")),
    }
}

fn eval_div(lhs: Value, rhs: Value) -> Result<Value, ExprError> {
    let denom = to_decimal(rhs)?;
    if denom == 0.0 {
        return Err(type_error("Division by zero"));
    }
    match lhs {
        Value::Money(m) => Ok(Value::Money(Money {
            amount: m.amount / denom,
            currency: m.currency,
        })),
        Value::Int(i) => Ok(Value::Decimal((i as f64) / denom)),
        Value::Decimal(d) => Ok(Value::Decimal(d / denom)),
        _ => Err(illegal_op("Illegal '/' operand types")),
    }
}

fn eval_compare(op: BinOp, lhs: Value, rhs: Value) -> Result<Value, ExprError> {
    let out = match (lhs, rhs) {
        (a, b) if is_numeric(&a) && is_numeric(&b) => {
            let la = to_decimal(a)?;
            let rb = to_decimal(b)?;
            cmp_f64(op, la, rb)
        }
        (Value::String(a), Value::String(b)) => cmp_ord(op, a.cmp(&b)),
        (Value::Date(a), Value::Date(b)) => cmp_ord(op, a.cmp(&b)),
        _ => {
            return Err(type_error(
                "Comparison operands must have matching comparable types",
            ))
        }
    };
    Ok(Value::Bool(out))
}

fn eval_call(name: &[String], args: &[Expr], env: &ExprEnv) -> Result<Value, ExprError> {
    let n = name.join(".");
    let eval_args = args
        .iter()
        .map(|arg| eval_expr(arg, env))
        .collect::<Result<Vec<_>, _>>()?;
    match n.as_str() {
        "money" => {
            if eval_args.len() != 2 {
                return Err(type_error("money(amount, currency) expects 2 arguments"));
            }
            let amount = to_decimal(eval_args[0].clone())?;
            let currency = to_currency(eval_args[1].clone())?;
            Ok(Value::Money(Money { amount, currency }))
        }
        "money_amount" => {
            if eval_args.len() != 1 {
                return Err(type_error("money_amount(x) expects 1 argument"));
            }
            match &eval_args[0] {
                Value::Money(m) => Ok(Value::Decimal(m.amount)),
                _ => Err(type_error("money_amount(x) expects Money")),
            }
        }
        "money_currency" => {
            if eval_args.len() != 1 {
                return Err(type_error("money_currency(x) expects 1 argument"));
            }
            match &eval_args[0] {
                Value::Money(m) => Ok(Value::Currency(m.currency.clone())),
                _ => Err(type_error("money_currency(x) expects Money")),
            }
        }
        "min" | "max" => {
            if eval_args.len() != 2 {
                return Err(type_error("min/max expect 2 Decimal arguments"));
            }
            let a = to_decimal(eval_args[0].clone())?;
            let b = to_decimal(eval_args[1].clone())?;
            Ok(Value::Decimal(if n == "min" { a.min(b) } else { a.max(b) }))
        }
        "clamp" => {
            if eval_args.len() != 3 {
                return Err(type_error("clamp(x, lo, hi) expects 3 Decimal arguments"));
            }
            let x = to_decimal(eval_args[0].clone())?;
            let lo = to_decimal(eval_args[1].clone())?;
            let hi = to_decimal(eval_args[2].clone())?;
            Ok(Value::Decimal(x.max(lo).min(hi)))
        }
        "round" => {
            if eval_args.len() != 2 {
                return Err(type_error("round(x, dp) expects 2 arguments"));
            }
            let x = to_decimal(eval_args[0].clone())?;
            let dp = to_int(eval_args[1].clone())?;
            let scale = 10_f64.powi(dp as i32);
            Ok(Value::Decimal((x * scale).round() / scale))
        }
        "add_days" => {
            if eval_args.len() != 2 {
                return Err(type_error("add_days(d, n) expects 2 arguments"));
            }
            let d = to_date(eval_args[0].clone())?;
            let n = to_int(eval_args[1].clone())?;
            Ok(Value::Date(add_days(d, n as i32)))
        }
        "add_months" => {
            if eval_args.len() != 2 {
                return Err(type_error("add_months(d, n) expects 2 arguments"));
            }
            let d = to_date(eval_args[0].clone())?;
            let n = to_int(eval_args[1].clone())?;
            Ok(Value::Date(add_months(d, n as i32)))
        }
        "days_between" => {
            if eval_args.len() != 2 {
                return Err(type_error("days_between(a, b) expects 2 arguments"));
            }
            let a = to_date(eval_args[0].clone())?;
            let b = to_date(eval_args[1].clone())?;
            Ok(Value::Int(days_between(&a, &b) as i64))
        }
        "yearfrac" => {
            if eval_args.len() != 2 {
                return Err(type_error("yearfrac(a, b) expects 2 arguments"));
            }
            let a = to_date(eval_args[0].clone())?;
            let b = to_date(eval_args[1].clone())?;
            Ok(Value::Decimal(days_between(&a, &b) as f64 / 365.0))
        }
        "obs.rate" => obs_lookup(env, "rate", &eval_args),
        "obs.index" => obs_lookup(env, "index", &eval_args),
        "obs.fx" => {
            if eval_args.len() != 2 {
                return Err(type_error("obs.fx(from, to) expects 2 arguments"));
            }
            let from = to_currency(eval_args[0].clone())?;
            let to = to_currency(eval_args[1].clone())?;
            let key = format!("fx:{from}:{to}");
            let value = env.obs.get(&key).cloned();
            Ok(Value::Optional(value.map(Box::new)))
        }
        _ => Err(unknown_ident(n)),
    }
}

fn obs_lookup(env: &ExprEnv, kind: &str, args: &[Value]) -> Result<Value, ExprError> {
    if args.len() != 1 {
        return Err(type_error("obs lookup expects one String argument"));
    }
    let name = match &args[0] {
        Value::String(v) => v.clone(),
        _ => return Err(type_error("obs lookup argument must be String")),
    };
    let key = format!("{kind}:{name}");
    let value = env.obs.get(&key).cloned();
    Ok(Value::Optional(value.map(Box::new)))
}

fn eq_values(lhs: &Value, rhs: &Value) -> Result<bool, ExprError> {
    match (lhs, rhs) {
        (Value::Bool(a), Value::Bool(b)) => Ok(a == b),
        (a, b) if is_numeric(a) && is_numeric(b) => {
            Ok((to_decimal(a.clone())? - to_decimal(b.clone())?).abs() < 1e-12)
        }
        (Value::String(a), Value::String(b)) => Ok(a == b),
        (Value::Currency(a), Value::Currency(b)) => Ok(a == b),
        (Value::Date(a), Value::Date(b)) => Ok(a == b),
        (Value::Money(a), Value::Money(b)) => {
            if a.currency != b.currency {
                return Err(illegal_op("Money equality requires matching currency"));
            }
            Ok((a.amount - b.amount).abs() < 1e-12)
        }
        (Value::Optional(a), Value::Optional(b)) => Ok(a == b),
        _ => Err(type_error("Equality operands must have compatible types")),
    }
}

fn is_numeric(value: &Value) -> bool {
    matches!(value, Value::Int(_) | Value::Decimal(_))
}

fn expect_bool(value: Value, message: &str) -> Result<bool, ExprError> {
    match value {
        Value::Bool(v) => Ok(v),
        _ => Err(type_error(message)),
    }
}

fn to_decimal(value: Value) -> Result<f64, ExprError> {
    match value {
        Value::Int(i) => Ok(i as f64),
        Value::Decimal(d) => Ok(d),
        _ => Err(type_error("Expected numeric value")),
    }
}

fn to_int(value: Value) -> Result<i64, ExprError> {
    match value {
        Value::Int(i) => Ok(i),
        _ => Err(type_error("Expected Int value")),
    }
}

fn to_currency(value: Value) -> Result<String, ExprError> {
    match value {
        Value::Currency(c) => Ok(c),
        Value::String(s) => Ok(s),
        _ => Err(type_error("Expected Currency or String value")),
    }
}

fn to_date(value: Value) -> Result<Date, ExprError> {
    match value {
        Value::Date(d) => Ok(d),
        Value::String(s) => Date::parse(&s),
        _ => Err(type_error("Expected Date value")),
    }
}

fn cmp_f64(op: BinOp, lhs: f64, rhs: f64) -> bool {
    match op {
        BinOp::Lt => lhs < rhs,
        BinOp::Le => lhs <= rhs,
        BinOp::Gt => lhs > rhs,
        BinOp::Ge => lhs >= rhs,
        _ => false,
    }
}

fn cmp_ord(op: BinOp, ord: std::cmp::Ordering) -> bool {
    match op {
        BinOp::Lt => ord == std::cmp::Ordering::Less,
        BinOp::Le => ord != std::cmp::Ordering::Greater,
        BinOp::Gt => ord == std::cmp::Ordering::Greater,
        BinOp::Ge => ord != std::cmp::Ordering::Less,
        _ => false,
    }
}

impl Date {
    pub fn parse(raw: &str) -> Result<Self, ExprError> {
        let parts = raw.split('-').collect::<Vec<_>>();
        if parts.len() != 3 {
            return Err(type_error("Expected date in YYYY-MM-DD format"));
        }
        let year = parts[0]
            .parse::<i32>()
            .map_err(|_| type_error("Invalid date year"))?;
        let month = parts[1]
            .parse::<u32>()
            .map_err(|_| type_error("Invalid date month"))?;
        let day = parts[2]
            .parse::<u32>()
            .map_err(|_| type_error("Invalid date day"))?;
        if month == 0 || month > 12 || day == 0 || day > days_in_month(year, month) {
            return Err(type_error("Invalid date value"));
        }
        Ok(Self { year, month, day })
    }
}

impl Ord for Date {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.year, self.month, self.day).cmp(&(other.year, other.month, other.day))
    }
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn add_months(date: Date, months: i32) -> Date {
    let total = date.year * 12 + (date.month as i32 - 1) + months;
    let year = total.div_euclid(12);
    let month = (total.rem_euclid(12) + 1) as u32;
    let day = date.day.min(days_in_month(year, month));
    Date { year, month, day }
}

fn add_days(date: Date, mut days: i32) -> Date {
    let mut out = date;
    while days > 0 {
        let dim = days_in_month(out.year, out.month);
        if out.day < dim {
            out.day += 1;
        } else if out.month == 12 {
            out.year += 1;
            out.month = 1;
            out.day = 1;
        } else {
            out.month += 1;
            out.day = 1;
        }
        days -= 1;
    }
    out
}

fn days_between(a: &Date, b: &Date) -> i32 {
    to_ordinal(b) - to_ordinal(a)
}

fn to_ordinal(date: &Date) -> i32 {
    let mut days = 0_i32;
    for y in 0..date.year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }
    for m in 1..date.month {
        days += days_in_month(date.year, m) as i32;
    }
    days + date.day as i32
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn parse_error(message: impl Into<String>) -> ExprError {
    ExprError {
        code: "E3001_EXPR_PARSE_ERROR",
        message: message.into(),
        span: None,
    }
}

fn unknown_ident(name: String) -> ExprError {
    ExprError {
        code: "E3002_EXPR_UNKNOWN_IDENT",
        message: format!("Unknown identifier '{name}'."),
        span: None,
    }
}

fn type_error(message: impl Into<String>) -> ExprError {
    ExprError {
        code: "E3003_EXPR_TYPE_ERROR",
        message: message.into(),
        span: None,
    }
}

fn illegal_op(message: impl Into<String>) -> ExprError {
    ExprError {
        code: "E3004_EXPR_ILLEGAL_OP",
        message: message.into(),
        span: None,
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Ident(String),
    Int(i64),
    Decimal(f64),
    String(String),
    True,
    False,
    LParen,
    RParen,
    Comma,
    Dot,
    QMark,
    Colon,
    Plus,
    Minus,
    Star,
    Slash,
    Bang,
    EqEq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    AndAnd,
    OrOr,
    Eof,
}

fn lex_expr(src: &str) -> Result<Vec<Tok>, ExprError> {
    let chars = src.chars().collect::<Vec<_>>();
    let mut i = 0usize;
    let mut out = Vec::new();
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let ident = chars[start..i].iter().collect::<String>();
            match ident.as_str() {
                "true" => out.push(Tok::True),
                "false" => out.push(Tok::False),
                _ => out.push(Tok::Ident(ident)),
            }
            continue;
        }
        if c.is_ascii_digit() {
            let start = i;
            i += 1;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            if i < chars.len() && chars[i] == '.' {
                i += 1;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                let raw = chars[start..i].iter().collect::<String>();
                out.push(Tok::Decimal(
                    raw.parse::<f64>()
                        .map_err(|_| parse_error("Invalid decimal literal"))?,
                ));
            } else {
                let raw = chars[start..i].iter().collect::<String>();
                out.push(Tok::Int(
                    raw.parse::<i64>()
                        .map_err(|_| parse_error("Invalid int literal"))?,
                ));
            }
            continue;
        }
        match c {
            '"' => {
                i += 1;
                let start = i;
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\\' {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                if i >= chars.len() {
                    return Err(parse_error("Unterminated string literal"));
                }
                let raw = chars[start..i].iter().collect::<String>();
                let value = raw
                    .replace("\\\"", "\"")
                    .replace("\\n", "\n")
                    .replace("\\t", "\t")
                    .replace("\\\\", "\\");
                out.push(Tok::String(value));
                i += 1;
            }
            '(' => {
                out.push(Tok::LParen);
                i += 1;
            }
            ')' => {
                out.push(Tok::RParen);
                i += 1;
            }
            ',' => {
                out.push(Tok::Comma);
                i += 1;
            }
            '.' => {
                out.push(Tok::Dot);
                i += 1;
            }
            '?' => {
                out.push(Tok::QMark);
                i += 1;
            }
            ':' => {
                out.push(Tok::Colon);
                i += 1;
            }
            '+' => {
                out.push(Tok::Plus);
                i += 1;
            }
            '-' => {
                out.push(Tok::Minus);
                i += 1;
            }
            '*' => {
                out.push(Tok::Star);
                i += 1;
            }
            '/' => {
                out.push(Tok::Slash);
                i += 1;
            }
            '!' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    out.push(Tok::Ne);
                    i += 2;
                } else {
                    out.push(Tok::Bang);
                    i += 1;
                }
            }
            '=' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    out.push(Tok::EqEq);
                    i += 2;
                } else {
                    return Err(parse_error("Unexpected '='; expected '=='"));
                }
            }
            '<' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    out.push(Tok::Le);
                    i += 2;
                } else {
                    out.push(Tok::Lt);
                    i += 1;
                }
            }
            '>' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    out.push(Tok::Ge);
                    i += 2;
                } else {
                    out.push(Tok::Gt);
                    i += 1;
                }
            }
            '&' => {
                if i + 1 < chars.len() && chars[i + 1] == '&' {
                    out.push(Tok::AndAnd);
                    i += 2;
                } else {
                    return Err(parse_error("Unexpected '&'; expected '&&'"));
                }
            }
            '|' => {
                if i + 1 < chars.len() && chars[i + 1] == '|' {
                    out.push(Tok::OrOr);
                    i += 2;
                } else {
                    return Err(parse_error("Unexpected '|'; expected '||'"));
                }
            }
            _ => return Err(parse_error(format!("Unexpected character '{c}'"))),
        }
    }
    out.push(Tok::Eof);
    Ok(out)
}

struct Parser {
    tokens: Vec<Tok>,
    idx: usize,
}

impl Parser {
    fn new(tokens: Vec<Tok>) -> Self {
        Self { tokens, idx: 0 }
    }

    fn parse_expression(&mut self) -> Result<Expr, ExprError> {
        self.parse_conditional()
    }

    fn expect_eof(&self) -> Result<(), ExprError> {
        if matches!(self.peek(), Tok::Eof) {
            Ok(())
        } else {
            Err(parse_error("Unexpected trailing tokens"))
        }
    }

    fn parse_conditional(&mut self) -> Result<Expr, ExprError> {
        let cond = self.parse_or()?;
        if matches!(self.peek(), Tok::QMark) {
            self.bump();
            let then_expr = self.parse_expression()?;
            if !matches!(self.peek(), Tok::Colon) {
                return Err(parse_error("Expected ':' in conditional expression"));
            }
            self.bump();
            let else_expr = self.parse_expression()?;
            Ok(Expr::Conditional {
                cond: Box::new(cond),
                then_expr: Box::new(then_expr),
                else_expr: Box::new(else_expr),
            })
        } else {
            Ok(cond)
        }
    }

    fn parse_or(&mut self) -> Result<Expr, ExprError> {
        self.parse_left_assoc(Self::parse_and, &[Tok::OrOr], &[BinOp::Or])
    }

    fn parse_and(&mut self) -> Result<Expr, ExprError> {
        self.parse_left_assoc(Self::parse_equality, &[Tok::AndAnd], &[BinOp::And])
    }

    fn parse_equality(&mut self) -> Result<Expr, ExprError> {
        self.parse_left_assoc(
            Self::parse_comparison,
            &[Tok::EqEq, Tok::Ne],
            &[BinOp::Eq, BinOp::Ne],
        )
    }

    fn parse_comparison(&mut self) -> Result<Expr, ExprError> {
        self.parse_left_assoc(
            Self::parse_term,
            &[Tok::Lt, Tok::Le, Tok::Gt, Tok::Ge],
            &[BinOp::Lt, BinOp::Le, BinOp::Gt, BinOp::Ge],
        )
    }

    fn parse_term(&mut self) -> Result<Expr, ExprError> {
        self.parse_left_assoc(
            Self::parse_factor,
            &[Tok::Plus, Tok::Minus],
            &[BinOp::Add, BinOp::Sub],
        )
    }

    fn parse_factor(&mut self) -> Result<Expr, ExprError> {
        self.parse_left_assoc(
            Self::parse_unary,
            &[Tok::Star, Tok::Slash],
            &[BinOp::Mul, BinOp::Div],
        )
    }

    fn parse_unary(&mut self) -> Result<Expr, ExprError> {
        match self.peek() {
            Tok::Bang => {
                self.bump();
                Ok(Expr::Unary {
                    op: UnOp::Not,
                    rhs: Box::new(self.parse_unary()?),
                })
            }
            Tok::Minus => {
                self.bump();
                Ok(Expr::Unary {
                    op: UnOp::Neg,
                    rhs: Box::new(self.parse_unary()?),
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, ExprError> {
        match self.bump() {
            Tok::True => Ok(Expr::Bool(true)),
            Tok::False => Ok(Expr::Bool(false)),
            Tok::Int(v) => Ok(Expr::Int(v)),
            Tok::Decimal(v) => Ok(Expr::Decimal(v)),
            Tok::String(v) => Ok(Expr::String(v)),
            Tok::LParen => {
                let expr = self.parse_expression()?;
                if !matches!(self.bump(), Tok::RParen) {
                    return Err(parse_error("Expected ')'"));
                }
                Ok(expr)
            }
            Tok::Ident(first) => {
                let mut path = vec![first];
                while matches!(self.peek(), Tok::Dot) {
                    self.bump();
                    match self.bump() {
                        Tok::Ident(next) => path.push(next),
                        _ => return Err(parse_error("Expected identifier after '.'")),
                    }
                }
                if matches!(self.peek(), Tok::LParen) {
                    self.bump();
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Tok::RParen) {
                        loop {
                            args.push(self.parse_expression()?);
                            if matches!(self.peek(), Tok::Comma) {
                                self.bump();
                                continue;
                            }
                            break;
                        }
                    }
                    if !matches!(self.bump(), Tok::RParen) {
                        return Err(parse_error("Expected ')' after function arguments"));
                    }
                    Ok(Expr::Call { name: path, args })
                } else {
                    Ok(Expr::Path(path))
                }
            }
            _ => Err(parse_error("Expected expression")),
        }
    }

    fn parse_left_assoc(
        &mut self,
        atom: fn(&mut Self) -> Result<Expr, ExprError>,
        toks: &[Tok],
        ops: &[BinOp],
    ) -> Result<Expr, ExprError> {
        let mut out = atom(self)?;
        loop {
            let mut matched = None;
            for (idx, token) in toks.iter().enumerate() {
                if self.peek() == token {
                    matched = Some(ops[idx]);
                    break;
                }
            }
            if let Some(op) = matched {
                self.bump();
                let rhs = atom(self)?;
                out = Expr::Binary {
                    lhs: Box::new(out),
                    op,
                    rhs: Box::new(rhs),
                };
            } else {
                break;
            }
        }
        Ok(out)
    }

    fn peek(&self) -> &Tok {
        self.tokens.get(self.idx).unwrap_or(&Tok::Eof)
    }

    fn bump(&mut self) -> Tok {
        let token = self.peek().clone();
        if !matches!(token, Tok::Eof) {
            self.idx += 1;
        }
        token
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluates_cfg_and_time_expression() {
        let compiled = compile_expr("cfg.base * (1 + cfg.growth * time.t)").expect("compile");
        let mut env = ExprEnv::empty();
        env.cfg.insert("base".to_string(), Value::Decimal(100.0));
        env.cfg.insert("growth".to_string(), Value::Decimal(0.1));
        env.time.insert("t".to_string(), Value::Int(2));
        let value = eval(&compiled, &env).expect("eval");
        match value {
            Value::Decimal(v) => assert!((v - 120.0).abs() < 1e-12),
            other => panic!("expected decimal, got {other:?}"),
        }
    }

    #[test]
    fn reports_unknown_root_identifier() {
        let err = compile_expr("foo.bar + 1").expect_err("expected error");
        assert_eq!(err.code, "E3002_EXPR_UNKNOWN_IDENT");
    }

    #[test]
    fn reports_illegal_money_plus_decimal() {
        let err = compile_expr("money(100, \"USD\") + 2").expect_err("expected error");
        assert_eq!(err.code, "E3004_EXPR_ILLEGAL_OP");
    }
}
