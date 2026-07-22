//! cfdl-calc — the CFDL expression engine.
//!
//! Design contract (LAUNCH_PLAN.md §6, Workstream B):
//! - Bare, Excel-familiar expression syntax: `base_rent * (1 + escalation)^years`,
//!   `if(dscr < 1.20, cash_trap, distribute)`.
//! - snake_case function vocabulary (`pmt`, `eomonth`, `year_frac`).
//! - Decimal-first numerics (`rust_decimal`) for money math; float64 is used ONLY
//!   for transcendental operations (fractional `^`, iterative solvers) and those
//!   conversion points are explicit in the code (`f64_escape`).
//! - `Mode::ExcelCompat` evaluates arithmetic in IEEE-754 float64 to reproduce
//!   Excel's representation artifacts, so benchmark harnesses can both match Excel
//!   references and quantify decimal-vs-float differences.
//! - No loops, no recursion, no I/O: every expression terminates, and the crate is
//!   wasm-clean (no filesystem, no threads, no tokio).
//! - Every AST node carries a byte-offset `Span` for precise diagnostics and LSP use.

mod date;
mod eval;
mod funcs;
mod parser;
mod token;
mod value;

pub use date::CalcDate;
pub use eval::{eval, Env, MapEnv, Mode};
pub use parser::{parse, BinOp, Expr, ExprKind, UnOp};
pub use token::Span;
pub use value::Value;

use std::fmt;

/// Error with an optional byte-offset span into the expression source.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub struct CalcError {
    pub message: String,
    pub span: Option<Span>,
}

impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.span {
            Some(s) => write!(f, "{} (at {}..{})", self.message, s.start, s.end),
            None => write!(f, "{}", self.message),
        }
    }
}

impl CalcError {
    pub fn new(message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}

/// Parse and evaluate an expression in one step.
pub fn eval_str(source: &str, env: &dyn Env, mode: Mode) -> Result<Value, CalcError> {
    let expr = parse(source)?;
    eval(&expr, env, mode)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    fn n(source: &str) -> Decimal {
        let env = MapEnv::new();
        match eval_str(source, &env, Mode::Decimal).unwrap() {
            Value::Number(d) => d,
            other => panic!("expected number, got {other:?}"),
        }
    }

    fn b(source: &str) -> bool {
        let env = MapEnv::new();
        match eval_str(source, &env, Mode::Decimal).unwrap() {
            Value::Bool(v) => v,
            other => panic!("expected bool, got {other:?}"),
        }
    }

    #[test]
    fn decimal_arithmetic_is_exact() {
        // The canonical float trap: 0.1 + 0.2 == 0.3 must hold in decimal mode.
        assert!(b("0.1 + 0.2 == 0.3"));
        assert_eq!(n("1.10 * 3"), dec("3.30"));
    }

    #[test]
    fn excel_compat_mode_reproduces_float_artifacts() {
        let env = MapEnv::new();
        let v = eval_str("0.1 + 0.2 == 0.3", &env, Mode::ExcelCompat).unwrap();
        // IEEE-754: 0.1 + 0.2 != 0.3 exactly.
        assert_eq!(v, Value::Bool(false));
    }

    #[test]
    fn precedence_and_power() {
        assert_eq!(n("2 + 3 * 4"), dec("14"));
        assert_eq!(n("(2 + 3) * 4"), dec("20"));
        // `^` binds tighter than unary minus and is right-associative.
        assert_eq!(n("2 ^ 3 ^ 2"), dec("512"));
        assert_eq!(n("-2 ^ 2"), dec("-4"));
        assert_eq!(n("10 - 4 - 3"), dec("3"));
        assert_eq!(n("7 % 4"), dec("3"));
    }

    #[test]
    fn fractional_power_uses_f64_escape() {
        // 9 ^ 0.5 = 3; comes back through the documented float escape.
        let v = n("9 ^ 0.5");
        assert!((v - dec("3")).abs() < dec("0.0000000001"), "got {v}");
    }

    #[test]
    fn comparisons_and_logic() {
        assert!(b("1 < 2 and 2 <= 2"));
        assert!(b("3 != 4"));
        assert!(b("3 <> 4")); // Excel-style alias
        assert!(b("not (1 > 2)"));
        assert!(b("true or 1/0 > 0")); // short-circuit: rhs never evaluated
    }

    #[test]
    fn variables_resolve_dotted_paths() {
        let mut env = MapEnv::new();
        env.set("time.t", Value::Number(dec("6")));
        env.set("escalation", Value::Number(dec("0.03")));
        let v = eval_str("(1 + escalation) ^ 2", &env, Mode::Decimal).unwrap();
        assert_eq!(v, Value::Number(dec("1.0609")));
        assert!(matches!(
            eval_str("time.t >= 6", &env, Mode::Decimal).unwrap(),
            Value::Bool(true)
        ));
    }

    #[test]
    fn unknown_variable_reports_span() {
        let env = MapEnv::new();
        let err = eval_str("1 + missing_var", &env, Mode::Decimal).unwrap_err();
        assert!(err.message.contains("missing_var"));
        let span = err.span.unwrap();
        assert_eq!(&"1 + missing_var"[span.start..span.end], "missing_var");
    }

    #[test]
    fn if_and_aggregates() {
        assert_eq!(n("if(1 < 2, 10, 20)"), dec("10"));
        assert_eq!(n("min(3, 1, 2)"), dec("1"));
        assert_eq!(n("max(3, 1, 2)"), dec("3"));
        assert_eq!(n("sum(1, 2, 3.5)"), dec("6.5"));
        assert_eq!(n("avg(2, 4)"), dec("3"));
        assert_eq!(n("abs(-4.2)"), dec("4.2"));
    }

    #[test]
    fn round_matches_excel_half_away_from_zero() {
        assert_eq!(n("round(2.5, 0)"), dec("3"));
        assert_eq!(n("round(-2.5, 0)"), dec("-3"));
        assert_eq!(n("round(1.005, 2)"), dec("1.01"));
        assert_eq!(n("round_down(1.999, 2)"), dec("1.99"));
        assert_eq!(n("round_up(1.001, 2)"), dec("1.01"));
    }

    #[test]
    fn pmt_matches_excel_reference() {
        // Excel: PMT(0.005, 360, 100000) = -599.5505251527... (30yr, 6%/12, $100k)
        let v = n("round(pmt(0.005, 360, 100000), 4)");
        assert_eq!(v, dec("-599.5505"));
        // Zero-rate degenerate case: PMT(0, 12, 1200) = -100
        assert_eq!(n("pmt(0, 12, 1200)"), dec("-100"));
        // Annuity-due: Excel PMT(0.005, 360, 100000, 0, 1) = -596.5678
        assert_eq!(
            n("round(pmt(0.005, 360, 100000, 0, 1), 4)"),
            dec("-596.5677")
        );
    }

    #[test]
    fn pv_fv_match_excel_reference() {
        // Excel: FV(0.005, 120, -100) = 16387.9346635...
        assert_eq!(n("round(fv(0.005, 120, -100), 4)"), dec("16387.9347"));
        // Excel: PV(0.01, 36, -500) = 15053.7524611...
        assert_eq!(n("round(pv(0.01, 36, -500), 4)"), dec("15053.7525"));
    }

    #[test]
    fn rate_solver_converges() {
        // Excel: RATE(360, -599.5505251527, 100000) = 0.005
        let v = n("round(rate(360, -599.5505251527, 100000), 8)");
        assert_eq!(v, dec("0.005"));
    }

    #[test]
    fn cpr_to_smm_reference() {
        // SMM = 1 - (1 - CPR)^(1/12); CPR 6% -> SMM 0.51430%
        assert_eq!(n("round(cpr_to_smm(0.06), 7)"), dec("0.0051430"));
    }

    #[test]
    fn date_functions() {
        let env = MapEnv::new();
        // eomonth / edate
        let v = eval_str("eomonth(date(2026, 1, 15), 1)", &env, Mode::Decimal).unwrap();
        assert_eq!(v, Value::Date(CalcDate::new(2026, 2, 28).unwrap()));
        let v = eval_str("edate(date(2026, 1, 31), 1)", &env, Mode::Decimal).unwrap();
        assert_eq!(v, Value::Date(CalcDate::new(2026, 2, 28).unwrap()));
        // date subtraction yields day counts (2028 is a leap year)
        assert_eq!(n("date(2028, 3, 1) - date(2028, 2, 1)"), dec("29"));
        // date + days
        let v = eval_str("date(2026, 12, 30) + 3", &env, Mode::Decimal).unwrap();
        assert_eq!(v, Value::Date(CalcDate::new(2027, 1, 2).unwrap()));
    }

    #[test]
    fn year_frac_conventions() {
        // 30/360 US: Jan 15 -> Jul 15 is exactly half a year.
        assert_eq!(
            n("year_frac(date(2026, 1, 15), date(2026, 7, 15), \"30/360\")"),
            dec("0.5")
        );
        // ACT/360: 181 actual days / 360
        let v = n("round(year_frac(date(2026, 1, 15), date(2026, 7, 15), \"act/360\"), 8)");
        assert_eq!(v, dec("0.50277778"));
        // ACT/365
        let v = n("round(year_frac(date(2026, 1, 15), date(2026, 7, 15), \"act/365\"), 8)");
        assert_eq!(v, dec("0.49589041"));
    }

    #[test]
    fn division_by_zero_is_an_error() {
        let env = MapEnv::new();
        let err = eval_str("1 / 0", &env, Mode::Decimal).unwrap_err();
        assert!(err.message.contains("division by zero"));
    }

    #[test]
    fn wrong_arity_reports_function_name() {
        let env = MapEnv::new();
        let err = eval_str("pmt(0.005)", &env, Mode::Decimal).unwrap_err();
        assert!(err.message.contains("pmt"), "{}", err.message);
    }
}
