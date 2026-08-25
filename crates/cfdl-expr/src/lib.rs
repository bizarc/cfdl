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
    /// Which arithmetic the expressions in this run evaluate under.
    ///
    /// Decimal is the default and is what every published number uses. A run
    /// may select `ExcelCompat` to reproduce a spreadsheet's float artifacts
    /// when reconciling against one; it belongs to the RUN rather than the
    /// model, because it is a property of the comparison being made and not of
    /// the deal. Carried on the environment so selecting it does not thread a
    /// parameter through every evaluation site.
    pub mode: Mode,
    /// Named date-indexed value curves (`curve` statements) available to
    /// `curve_value(name, date)`. Populated by the engine from IR; empty
    /// elsewhere.
    pub curves: BTreeMap<String, CurveDef>,
    /// Named quantile declarations (`quantile` statements) available to
    /// `quantile_at`, `quantile_mean` and `quantile_of`. A quantile is indexed
    /// by cumulative share rather than by date, which is the whole difference
    /// from a curve. Populated by the engine from IR; empty elsewhere.
    pub quantiles: BTreeMap<String, QuantileDef>,
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
    /// What is still in the pot at this waterfall step — bare `remaining`.
    ///
    /// Bare, the way `prev` is, because a waterfall step is written about the
    /// pot and reads better for saying so: `= remaining` is the residual.
    /// `None` everywhere outside a waterfall, which makes reading it there an
    /// unresolved name rather than a silent zero.
    pub remaining: Option<Value>,
    /// The cash the waterfall's entity produced this period — bare `available`.
    ///
    /// Streams only, netted, with the entity's children rolled up by the
    /// `part of` relation: the quantity `docs/17` §4 names as the pot. Bound
    /// by the engine before the waterfall runs, so no model declares a field
    /// for it. `None` everywhere outside a waterfall, so reading it there is
    /// an unresolved name rather than a silent zero.
    pub available: Option<Value>,
    /// What each earlier step in this waterfall actually paid — `paid.<step>`.
    pub paid: BTreeMap<String, Value>,
    /// What each earlier step would have paid unbounded — `owed.<step>`. The
    /// difference between the two IS the shortfall, so a step that pays an
    /// earlier step's overflow writes `owed.x - paid.x`.
    pub owed: BTreeMap<String, Value>,
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

/// A named quantile: (share, value) points plus interpolation policy.
///
/// CANONICAL FORM IS ASCENDING. `by exceedance` is an authoring convenience
/// that the compiler reverses, so every consumer sees shares rising from 0 and
/// no consumer carries an orientation. See docs/27_quantiles.md.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantileDef {
    /// "step" (the value of the last point at or below the query share) or
    /// "linear" (linear between bracketing points). The same two words a curve
    /// uses, with the same meanings — and the integral is defined as the exact
    /// integral of whichever function they describe, so quadrature is derived
    /// rather than declared.
    pub interpolation: String,
    /// Points sorted ascending by share. Shares lie in [0, 1] and values are
    /// non-decreasing, both enforced at compile time.
    pub points: Vec<(f64, f64)>,
}

impl ExprEnv {
    pub fn empty() -> Self {
        Self {
            mode: Mode::default(),
            model: BTreeMap::new(),
            time: BTreeMap::new(),
            entity: BTreeMap::new(),
            cfg: BTreeMap::new(),
            obs: BTreeMap::new(),
            inputs: BTreeMap::new(),
            states: BTreeMap::new(),
            prev_states: BTreeMap::new(),
            prev_self: None,
            remaining: None,
            available: None,
            paid: BTreeMap::new(),
            owed: BTreeMap::new(),
            series: Arc::default(),
            curves: BTreeMap::new(),
            quantiles: BTreeMap::new(),
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

/// An unresolvable NAME, as distinct from arithmetic that failed.
///
/// The distinction is what lets the engine refuse the first and tolerate the
/// second. docs/03 is explicit — "Unknown variables are hard errors
/// (EXPR_EVAL), not nulls" — and every layer honoured it except the engine,
/// which caught the error and substituted zero.
pub const EXPR_UNKNOWN_NAME: &str = "EXPR_UNKNOWN_NAME";

fn eval_error(e: cfdl_calc::CalcError) -> ExprError {
    let code = if e.message.starts_with("unknown variable") {
        EXPR_UNKNOWN_NAME
    } else {
        "EXPR_EVAL"
    };
    ExprError {
        code: code.to_string(),
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
/// this to decide whether the stream reads other streams at all.
pub fn uses_series(compiled: &CompiledExpr) -> bool {
    cfdl_calc::expr_calls_any(&compiled.expr, &["series_sum", "series_avg"])
}

/// Does any series call compute its name at runtime instead of naming it as a
/// literal? `series_references` cannot extract an edge for such a read, so the
/// engine schedules the stream after every literally-named one — and refuses
/// to let anything read it, since its own position cannot be reasoned about.
pub fn has_computed_series_name(compiled: &CompiledExpr) -> bool {
    cfdl_calc::has_computed_call_name(&compiled.expr, &["series_sum", "series_avg"])
}

/// Every `<root>.<segment>` an expression names, as written.
///
/// A source scan, for the same reason `series_references` is one: the callers
/// hold the source, and one scanner means the compiler and the engine cannot
/// drift into disagreeing about what a model says. Guards against matching
/// mid-identifier, so `my_inputs.x` is not an `inputs.` read.
pub fn root_references(src: &str, root: &str) -> Vec<String> {
    let mut found = Vec::new();
    let needle = format!("{root}.");
    let mut base = 0usize;
    while let Some(idx) = src[base..].find(&needle) {
        let at = base + idx;
        let before_ok = at == 0
            || !src[..at]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.');
        base = at + needle.len();
        if !before_ok {
            continue;
        }
        let seg: String = src[base..]
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !seg.is_empty() {
            found.push(seg);
        }
    }
    found
}

/// The series names an expression reads, as written.
///
/// Only literal first arguments — `series_sum("a.b", ...)` — which is what a
/// cross-stream read is. A computed name is not addressed here and is left to
/// the runtime, where it still returns 0 for an unmatched name.
///
/// Scans the source rather than the compiled tree because both callers hold the
/// source and one of them (the compiler's waterfall check) runs before anything
/// is compiled. It lives here rather than in either caller so the compiler and
/// the engine read the same names: they refuse and explain the same reference,
/// and two scanners cannot drift into disagreeing about what a model says.
pub fn series_references(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = src.as_bytes();
    for func in ["series_sum", "series_avg"] {
        let mut from = 0usize;
        while let Some(rel) = src[from..].find(func) {
            let after = from + rel + func.len();
            from = after;
            let mut i = after;
            while i < bytes.len() && (bytes[i] as char).is_whitespace() {
                i += 1;
            }
            if i >= bytes.len() || bytes[i] != b'(' {
                continue;
            }
            i += 1;
            while i < bytes.len() && (bytes[i] as char).is_whitespace() {
                i += 1;
            }
            if i >= bytes.len() || bytes[i] != b'"' {
                continue;
            }
            i += 1;
            let start = i;
            while i < bytes.len() && bytes[i] != b'"' {
                i += 1;
            }
            if i <= bytes.len() {
                out.push(src[start..i].to_string());
            }
        }
    }
    out
}

/// Does `name` match `pattern`? The one selector dialect.
///
/// `<prefix>.*` matches `<prefix>` ITSELF and every name beneath `<prefix>.`;
/// anything else is an exact name. There is no mid-pattern `*`, no `?` and no
/// regex — a selector has to stay greppable.
///
/// Matching the bare prefix is the part that was inconsistent, and it is the
/// part that matters. Pack rules emit `<name>{{contract.dot_suffix}}`, which
/// expands to a BARE name for an unsuffixed contract and a suffixed one
/// otherwise, so a selector that skips the bare form silently drops the
/// unsuffixed instance of whatever it selects.
///
/// Two implementations of this existed and disagreed on exactly that point.
/// `EnvAdapter::matching_series` matched the bare prefix deliberately;
/// `cfdl-metrics` matched `stream.<prefix>.` against the whole KEY, so whether
/// the bare instance was reached depended on the key format rather than on any
/// decision:
///
/// - `sum_stream_totals` keys end in `.total`, so `stream.<p>.total` supplied
///   the separating dot itself and the bare instance was included by accident.
/// - `wal_years` keys do not, so `stream.<p>` failed the prefix test and the
///   bare instance was silently dropped. `domain.credit.wal_years` selects
///   sched_principal, prepay, bullet and recoveries this way and goldens ship
///   all four bare, so an unsuffixed pool reported a weighted average life over
///   a subset of its own principal.
///
/// Neither was caught: none of the affected fixtures runs with `--pack`, so
/// `domain_metrics` is absent from every golden that would have shown it.
///
/// Selectors match NAMES. The key format a caller stores them under
/// (`stream.<name>`, `stream.<name>.total`) is the caller's business, which is
/// what stops the accident above from being load-bearing again.
/// One resolved quantile call site: the slice a model actually asked for, and
/// what it came to.
///
/// This is the audit record for a NONLINEAR input. Publishing the declaration
/// alone would say a price stack existed; it would not say that the top 2% of
/// hours averaged 340.00 and that this is the number which struck the revenue.
/// A reviewer cannot check the second from the first without redoing the
/// integral by hand.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantileCall {
    /// The quantile named at the call site.
    pub quantile: String,
    /// `quantile_at`, `quantile_mean` or `quantile_of`.
    pub function: String,
    /// The literal arguments after the name, in source order.
    pub args: Vec<f64>,
    /// What the call resolves to. `None` when an argument is not a literal —
    /// the call is still listed, because a silently omitted call site would
    /// read as a model that never made one.
    pub value: Option<f64>,
}

/// Every quantile call site in `src`, resolved against `quantiles` where the
/// arguments are literals.
///
/// Walks the parsed expression rather than scanning text, so a name inside a
/// comment or a string cannot be mistaken for a call. Returns an empty vector
/// for a source that does not parse: the caller has already reported that as
/// E5009, and reporting it twice would be noise.
pub fn quantile_calls(src: &str, quantiles: &BTreeMap<String, QuantileDef>) -> Vec<QuantileCall> {
    let Ok(compiled) = compile_expr(src) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    collect_quantile_calls(&compiled.expr, quantiles, &mut out);
    out
}

fn collect_quantile_calls(
    expr: &cfdl_calc::Expr,
    quantiles: &BTreeMap<String, QuantileDef>,
    out: &mut Vec<QuantileCall>,
) {
    use cfdl_calc::ExprKind;
    match &expr.kind {
        ExprKind::Call { name, args } => {
            if matches!(
                name.as_str(),
                "quantile_at" | "quantile_mean" | "quantile_of"
            ) {
                if let Some(call) = resolve_quantile_call(name, args, quantiles) {
                    out.push(call);
                }
            }
            for a in args {
                collect_quantile_calls(a, quantiles, out);
            }
        }
        ExprKind::Unary { expr, .. } => collect_quantile_calls(expr, quantiles, out),
        ExprKind::Binary { lhs, rhs, .. } => {
            collect_quantile_calls(lhs, quantiles, out);
            collect_quantile_calls(rhs, quantiles, out);
        }
        _ => {}
    }
}

fn resolve_quantile_call(
    name: &str,
    args: &[cfdl_calc::Expr],
    quantiles: &BTreeMap<String, QuantileDef>,
) -> Option<QuantileCall> {
    use cfdl_calc::ExprKind;
    let ExprKind::Str(quantile) = &args.first()?.kind else {
        // A computed quantile name. Nothing to record against a declaration.
        return None;
    };
    let literals: Option<Vec<f64>> = args[1..]
        .iter()
        .map(|a| match &a.kind {
            ExprKind::Number(n) => n.to_f64(),
            _ => None,
        })
        .collect();
    let def = quantiles.get(quantile);
    let value = match (literals.as_ref(), def) {
        (Some(nums), Some(def)) => match (name, nums.as_slice()) {
            ("quantile_at", [x]) => quantile_eval(def, *x),
            ("quantile_mean", [a, b]) => quantile_slice_mean(def, *a, *b),
            ("quantile_of", [v]) => quantile_invert(def, *v),
            _ => None,
        },
        _ => None,
    };
    Some(QuantileCall {
        quantile: quantile.clone(),
        function: name.to_string(),
        args: literals.unwrap_or_default(),
        value,
    })
}

pub fn selector_matches(pattern: &str, name: &str) -> bool {
    match pattern.strip_suffix(".*") {
        Some(prefix) => name == prefix || name.starts_with(&format!("{prefix}.")),
        None => name == pattern,
    }
}

/// Does `name` match any of `patterns`? Empty patterns match nothing.
pub fn selector_matches_any(patterns: &[String], name: &str) -> bool {
    patterns.iter().any(|p| selector_matches(p, name))
}

pub fn eval(compiled: &CompiledExpr, env: &ExprEnv) -> Result<Value, ExprError> {
    eval_with_mode(compiled, env, env.mode)
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
            // `prev` alone is this recurrence's own previous value.
            // `prev.<name>` is another one's — and a FIELD's name is its entity
            // path, so `prev.asset.tlb.balance` is the whole remainder rather
            // than its first segment.
            //
            // The dotted spelling rather than a `prev <path>` prefix: `prev`
            // is an ordinary identifier, so a prefix form would be two operands
            // side by side, which is exactly where an expression ENDS. Reusing
            // the dot keeps one rule for where expressions stop and one
            // spelling for "the period before".
            let rest: Vec<&str> = parts.collect();
            if rest.is_empty() {
                return self.env.prev_self.as_ref().and_then(domain_to_calc);
            }
            return self
                .env
                .prev_states
                .get(&rest.join("."))
                .and_then(domain_to_calc);
        }
        if root == "remaining" && parts.next().is_none() {
            return self.env.remaining.as_ref().and_then(domain_to_calc);
        }
        if root == "available" && parts.next().is_none() {
            return self.env.available.as_ref().and_then(domain_to_calc);
        }
        // `asset.tlb.balance` IS `entity.asset.tlb.balance`.
        //
        // An entity's properties are bound under its family, so both spellings
        // name the same read. The bare one is what the object model already
        // says out loud — an asset has a balance — and it is what everyone
        // writing a waterfall reached for first, including this project's own
        // documentation. Supporting it removes the difference rather than
        // teaching it.
        //
        // Only a declared family is aliased, and only when that family is
        // actually bound, so no other root changes meaning.
        let family_alias = matches!(root, "asset" | "party" | "contract" | "reference")
            && matches!(self.env.entity.get(root), Some(Value::Map(_)));

        let (map, first) = if family_alias {
            (&self.env.entity, root)
        } else {
            let m = match root {
                "paid" => &self.env.paid,
                "owed" => &self.env.owed,
                "model" => &self.env.model,
                "time" => &self.env.time,
                "entity" => &self.env.entity,
                "cfg" => &self.env.cfg,
                "obs" => &self.env.obs,
                "inputs" => &self.env.inputs,
                _ => return None,
            };
            (m, parts.next()?)
        };
        // The `entity` root is open-world: entity state fields may not exist
        // until an event sets them, and expressions like
        // `entity.status != \"refinanced\"` must evaluate (to null) before
        // that. Other roots stay strict so typos are hard errors.
        let open_world = root == "entity" || family_alias;
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

    fn quantile_at(&self, name: &str, share: Decimal) -> Option<Decimal> {
        let q = self.env.quantiles.get(name)?;
        quantile_eval(q, to_f64(share)?).and_then(Decimal::from_f64)
    }

    fn quantile_mean(&self, name: &str, from: Decimal, to: Decimal) -> Option<Decimal> {
        let q = self.env.quantiles.get(name)?;
        quantile_slice_mean(q, to_f64(from)?, to_f64(to)?).and_then(Decimal::from_f64)
    }

    fn quantile_of(&self, name: &str, value: Decimal) -> Option<Decimal> {
        let q = self.env.quantiles.get(name)?;
        quantile_invert(q, to_f64(value)?).and_then(Decimal::from_f64)
    }
}

fn to_f64(d: Decimal) -> Option<f64> {
    use rust_decimal::prelude::ToPrimitive;
    d.to_f64()
}

/// The quantile function at `share`, clamped flat outside [first, last].
///
/// `step` reads the last point at or below the share, which is the same
/// flat-forward rule a `step` curve uses on dates.
pub(crate) fn quantile_eval(q: &QuantileDef, share: f64) -> Option<f64> {
    let pts = &q.points;
    if pts.is_empty() {
        return None;
    }
    if share <= pts[0].0 {
        return Some(pts[0].1);
    }
    let last = pts[pts.len() - 1];
    if share >= last.0 {
        return Some(last.1);
    }
    let idx = pts.partition_point(|(x, _)| *x <= share);
    let prev = pts[idx - 1];
    if q.interpolation == "linear" {
        let next = pts[idx];
        let frac = (share - prev.0) / (next.0 - prev.0);
        Some(prev.1 + (next.1 - prev.1) * frac)
    } else {
        Some(prev.1)
    }
}

/// The mean value over the share slice `[from, to]` — a PARTIAL EXPECTATION
/// divided by the slice width, which is what a payoff over a tail actually
/// needs.
///
/// Computed as the exact integral of the interpolated function rather than by
/// sampling it: rectangles under `step`, trapezoids under `linear`. That is
/// why the declaration carries no quadrature choice — the interpolation
/// already decides it, and the answer is exact for that shape rather than an
/// approximation of it.
///
/// A zero-width slice is the point value, not a division by zero.
pub(crate) fn quantile_slice_mean(q: &QuantileDef, from: f64, to: f64) -> Option<f64> {
    if q.points.is_empty() || !from.is_finite() || !to.is_finite() {
        return None;
    }
    let (lo, hi) = if from <= to { (from, to) } else { (to, from) };
    if (hi - lo).abs() < f64::EPSILON {
        return quantile_eval(q, lo);
    }
    // Integrate segment by segment over the breakpoints inside the slice, so
    // every piece is integrated under the rule its own segment declares.
    let mut edges: Vec<f64> = vec![lo];
    for (x, _) in &q.points {
        if *x > lo && *x < hi {
            edges.push(*x);
        }
    }
    edges.push(hi);
    let mut area = 0.0;
    for w in edges.windows(2) {
        let (a, b) = (w[0], w[1]);
        let width = b - a;
        if width <= 0.0 {
            continue;
        }
        area += if q.interpolation == "linear" {
            // Trapezoid: exact for a segment that is linear by declaration.
            (quantile_eval(q, a)? + quantile_eval(q, b)?) / 2.0 * width
        } else {
            // Rectangle: a step segment holds the value it opened with. Read
            // just inside the segment so a breakpoint at `a` takes the new
            // step rather than the one it replaced.
            quantile_eval(q, a + width / 2.0)? * width
        };
    }
    Some(area / (hi - lo))
}

/// The share at or below which `value` sits — the CDF, and the inverse of
/// `quantile_eval`.
///
/// This is what turns a stated threshold (a lease breakpoint, a tranche
/// attachment point) into a share, so a slice can be taken against a value the
/// modeller knows rather than a percentile they would have to work out.
/// Well-defined because values are non-decreasing in share, which the compiler
/// enforces.
pub(crate) fn quantile_invert(q: &QuantileDef, value: f64) -> Option<f64> {
    let pts = &q.points;
    if pts.is_empty() {
        return None;
    }
    if value <= pts[0].1 {
        return Some(pts[0].0);
    }
    let last = pts[pts.len() - 1];
    if value >= last.1 {
        return Some(last.0);
    }
    let idx = pts.partition_point(|(_, v)| *v <= value);
    let prev = pts[idx - 1];
    let next = pts[idx];
    if q.interpolation == "linear" {
        let span = next.1 - prev.1;
        if span.abs() < f64::EPSILON {
            return Some(prev.0);
        }
        Some(prev.0 + (next.0 - prev.0) * (value - prev.1) / span)
    } else {
        // A step holds its value across the segment, so the share at which
        // `value` is first reached is the segment's own opening share.
        Some(prev.0)
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
        // Exact lookups stay a map hit rather than a scan; the glob case
        // delegates so there is one dialect. See `selector_matches`.
        if name.ends_with(".*") {
            self.env
                .series
                .iter()
                .filter(|(key, _)| selector_matches(name, key))
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
    // The quantile hooks are trait methods, so the trait must be in scope for
    // a test to call them on the adapter directly.
    use cfdl_calc::Env as _;

    #[test]
    fn selector_glob_matches_the_bare_prefix_and_its_children() {
        // The whole point. A pack rule emitting `<name>{{contract.dot_suffix}}`
        // produces the bare name for an unsuffixed contract, so a selector that
        // matched only children would drop it.
        assert!(selector_matches(
            "credit.pool.interest.*",
            "credit.pool.interest"
        ));
        assert!(selector_matches(
            "credit.pool.interest.*",
            "credit.pool.interest.p"
        ));
        assert!(selector_matches(
            "credit.pool.interest.*",
            "credit.pool.interest.a.b"
        ));
    }

    #[test]
    fn selector_glob_does_not_match_a_sibling_sharing_a_text_prefix() {
        // `.*` is a path-segment boundary, not a string prefix: an extra
        // segment is required, so `interest_accrued` is a different name.
        assert!(!selector_matches(
            "credit.pool.interest.*",
            "credit.pool.interest_accrued"
        ));
        assert!(!selector_matches("cre.pct_rent.*", "cre.pct_rent_extra"));
    }

    #[test]
    fn selector_without_a_glob_is_an_exact_name() {
        assert!(selector_matches("energy.ppa.revenue", "energy.ppa.revenue"));
        assert!(!selector_matches(
            "energy.ppa.revenue",
            "energy.ppa.revenue.plant_a"
        ));
    }

    #[test]
    fn selector_any_is_a_disjunction_and_empty_matches_nothing() {
        let patterns = vec!["cre.opex.line".to_string(), "cre.opex.line.*".to_string()];
        assert!(selector_matches_any(&patterns, "cre.opex.line"));
        assert!(selector_matches_any(&patterns, "cre.opex.line"));
        assert!(selector_matches_any(&patterns, "cre.opex.line.taxes"));
        assert!(!selector_matches_any(&patterns, "cre.vacancy.loss"));
        assert!(!selector_matches_any(&[], "cre.opex.line"));
    }

    #[test]
    fn literal_series_names_are_not_computed() {
        let c = compile_expr(r#"series_sum("a.b", 0, time.t) + series_avg("c.*", 0, 3)"#)
            .expect("compile");
        assert!(uses_series(&c));
        assert!(!has_computed_series_name(&c));
    }

    #[test]
    fn a_variable_series_name_is_computed() {
        let c = compile_expr("series_sum(inputs.which, 0, time.t)").expect("compile");
        assert!(uses_series(&c));
        assert!(has_computed_series_name(&c));
    }

    #[test]
    fn one_computed_name_among_literals_still_counts() {
        let c =
            compile_expr(r#"series_sum("a.b", 0, 1) + series_sum(pick, 0, 1)"#).expect("compile");
        assert!(has_computed_series_name(&c));
        // A series call nested inside another call's arguments is still found.
        let nested = compile_expr("max(0, series_avg(pick, 0, 1))").expect("compile");
        assert!(has_computed_series_name(&nested));
    }

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
    fn unknown_variable_is_its_own_error_kind() {
        // Carries EXPR_UNKNOWN_NAME rather than EXPR_EVAL, so a caller can
        // refuse an unresolvable name while still tolerating arithmetic that
        // failed for an ordinary reason.
        let compiled = compile_expr("nope.missing + 1").expect("compile");
        let err = eval(&compiled, &ExprEnv::empty()).unwrap_err();
        assert_eq!(err.code, EXPR_UNKNOWN_NAME);
        assert!(err.message.contains("nope.missing"));
    }

    #[test]
    fn arithmetic_failure_keeps_the_generic_code() {
        let compiled = compile_expr("1 / 0").expect("compile");
        if let Err(err) = eval(&compiled, &ExprEnv::empty()) {
            assert_eq!(err.code, "EXPR_EVAL");
        }
    }

    fn quantile_env(interp: &str) -> ExprEnv {
        let mut env = ExprEnv::empty();
        env.quantiles.insert(
            "p".to_string(),
            QuantileDef {
                interpolation: interp.to_string(),
                points: vec![(0.0, 10.0), (0.5, 20.0), (1.0, 60.0)],
            },
        );
        env
    }

    #[test]
    fn quantile_calls_records_every_site_including_the_unresolvable() {
        let mut defs = BTreeMap::new();
        defs.insert(
            "p".to_string(),
            quantile_env("linear").quantiles["p"].clone(),
        );

        // Nested inside arithmetic and inside another call, to show the walk
        // reaches call sites a flat scan of the top level would miss.
        let calls = quantile_calls(
            "max(quantile_mean(\"p\", 0.5, 1.0) - quantile_at(\"p\", 0.0), 0)",
            &defs,
        );
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].function, "quantile_mean");
        assert_eq!(calls[0].value, Some(40.0));
        assert_eq!(calls[1].value, Some(10.0));

        // A computed argument cannot be resolved, and the site is still
        // listed. Dropping it would read as a model that never made the call.
        let calls = quantile_calls("quantile_mean(\"p\", inputs.lo, 1.0)", &defs);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].value, None);

        // A name that is not a literal has no declaration to record against.
        assert!(quantile_calls("quantile_at(some.name, 0.5)", &defs).is_empty());

        // An unparseable source is the caller's E5009, not ours to repeat.
        assert!(quantile_calls("quantile_at(\"p\", ", &defs).is_empty());
    }

    #[test]
    fn quantile_mean_is_the_exact_integral_not_a_sample() {
        // Linear: the slice 0.5..1.0 is a trapezoid between 20 and 60, so its
        // mean is 40 exactly. Sampling the midpoint would also give 40 here,
        // which is why the asymmetric slice below is the real test.
        let env = quantile_env("linear");
        let got = EnvAdapter { env: &env }.quantile_mean("p", dec(0.5), dec(1.0));
        assert_eq!(got.and_then(|d| d.to_f64()), Some(40.0));

        // 0.0..1.0 spans BOTH segments, whose slopes differ. The exact area is
        // (10+20)/2*0.5 + (20+60)/2*0.5 = 7.5 + 20 = 27.5. A single trapezoid
        // across the whole range would say 35 — the error this integrates
        // segment by segment to avoid.
        let got = EnvAdapter { env: &env }.quantile_mean("p", dec(0.0), dec(1.0));
        assert_eq!(got.and_then(|d| d.to_f64()), Some(27.5));
    }

    #[test]
    fn quantile_mean_under_step_is_rectangles() {
        // A step holds the value it opened with: 10 across 0.0..0.5, 20 across
        // 0.5..1.0, so the whole-range mean is 15 rather than linear's 27.5.
        // Same points, same slice, different declared shape.
        let env = quantile_env("step");
        let got = EnvAdapter { env: &env }.quantile_mean("p", dec(0.0), dec(1.0));
        assert_eq!(got.and_then(|d| d.to_f64()), Some(15.0));
    }

    #[test]
    fn quantile_of_inverts_quantile_at() {
        let env = quantile_env("linear");
        let a = EnvAdapter { env: &env };
        // Round-trip on a declared point and on an interpolated one.
        assert_eq!(
            a.quantile_of("p", dec(20.0)).and_then(|d| d.to_f64()),
            Some(0.5)
        );
        assert_eq!(
            a.quantile_at("p", dec(0.5)).and_then(|d| d.to_f64()),
            Some(20.0)
        );
        let share = a.quantile_of("p", dec(40.0)).unwrap();
        assert_eq!(
            a.quantile_at("p", share).and_then(|d| d.to_f64()),
            Some(40.0)
        );
    }

    #[test]
    fn quantile_clamps_flat_outside_its_range() {
        let env = quantile_env("linear");
        let a = EnvAdapter { env: &env };
        assert_eq!(
            a.quantile_at("p", dec(-1.0)).and_then(|d| d.to_f64()),
            Some(10.0)
        );
        assert_eq!(
            a.quantile_at("p", dec(2.0)).and_then(|d| d.to_f64()),
            Some(60.0)
        );
        // And the inverse clamps to the share axis, not past it.
        assert_eq!(
            a.quantile_of("p", dec(1000.0)).and_then(|d| d.to_f64()),
            Some(1.0)
        );
        assert_eq!(
            a.quantile_of("p", dec(0.0)).and_then(|d| d.to_f64()),
            Some(0.0)
        );
    }

    #[test]
    fn a_curve_and_a_quantile_do_not_resolve_against_each_other() {
        // Different axes. Reaching for one through the other's function finds
        // nothing, which surfaces as an evaluation error rather than a number
        // read off the wrong axis.
        let env = quantile_env("linear");
        let a = EnvAdapter { env: &env };
        assert!(a.quantile_at("sofr", dec(0.5)).is_none());
        let mut env2 = ExprEnv::empty();
        env2.curves.insert(
            "sofr".to_string(),
            CurveDef {
                interpolation: "step".to_string(),
                points: vec![],
            },
        );
        assert!(EnvAdapter { env: &env2 }
            .quantile_at("sofr", dec(0.5))
            .is_none());
    }

    fn dec(v: f64) -> Decimal {
        Decimal::from_f64(v).unwrap()
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
