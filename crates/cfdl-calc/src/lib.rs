//! cfdl-calc — the CFDL expression engine.
//!
//! Design contract:
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

pub use date::{CalcDate, DayCount, HolidayCalendar, RollConvention};
pub use eval::{eval, Env, MapEnv, Mode};
pub use funcs::expr_calls_any;
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
    fn cpr_to_periodic_generalises_cpr_to_smm() {
        // The whole point of the pair: cpr_to_smm is the ppy = 12 case, and
        // must stay bit-identical to it, or converting the credit pack to
        // cpr_to_periodic would move every monthly number.
        for cpr in ["0.0", "0.02", "0.06", "0.25", "0.9"] {
            assert_eq!(
                n(&format!("cpr_to_periodic({cpr}, 12)")),
                n(&format!("cpr_to_smm({cpr})")),
                "cpr_to_periodic({cpr}, 12) must equal cpr_to_smm({cpr})"
            );
        }
    }

    #[test]
    fn cpr_to_periodic_survives_a_full_year_at_any_cadence() {
        // A periodic mortality is correct when compounding it over one year of
        // its own periods reproduces the annual CPR. This holds by definition
        // for a root and would not for a division, which is why CPR converts
        // differently from a note rate.
        for (ppy, periods) in [(1, 1), (4, 4), (12, 12), (365, 365)] {
            let survival =
                format!("round(1 - pow(1 - cpr_to_periodic(0.06, {ppy}), {periods}), 8)");
            assert_eq!(n(&survival), dec("0.06"), "ppy = {ppy} must recover CPR");
        }
    }

    #[test]
    fn days_between_counts_whole_days_and_signs() {
        assert_eq!(
            n("days_between(date(2026,1,1), date(2026,1,31))"),
            dec("30")
        );
        assert_eq!(
            n("days_between(date(2026,1,31), date(2026,1,1))"),
            dec("-30")
        );
        // 2028 is a leap year, so February contributes 29 days.
        assert_eq!(
            n("days_between(date(2028,1,1), date(2029,1,1))"),
            dec("366")
        );
        // Agrees with the `-` operator, which is the same underlying count.
        assert_eq!(
            n("days_between(date(2026,3,1), date(2026,6,1))"),
            n("date(2026,6,1) - date(2026,3,1)")
        );
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
    fn business_day_calendars_match_known_dates() {
        let env = MapEnv::new();
        let b = |src: &str| match eval_str(src, &env, Mode::Decimal).unwrap() {
            Value::Bool(v) => v,
            other => panic!("expected bool, got {other:?}"),
        };
        // 2026-07-21 is a Tuesday.
        assert!(b("is_business_day(date(2026, 7, 21), \"weekend\")"));
        // 2026-07-03 (Friday) is the observed US Independence Day (Jul 4 is Sat).
        assert!(!b("is_business_day(date(2026, 7, 3), \"us\")"));
        assert!(b("is_business_day(date(2026, 7, 3), \"target\")"));
        // MLK Day 2026: 3rd Monday of January = Jan 19.
        assert!(!b("is_business_day(date(2026, 1, 19), \"us\")"));
        // Thanksgiving 2026: 4th Thursday of November = Nov 26.
        assert!(!b("is_business_day(date(2026, 11, 26), \"us\")"));
        // Easter 2026 is April 5 -> Good Friday Apr 3 (TARGET + UK closed, US open).
        assert!(!b("is_business_day(date(2026, 4, 3), \"target\")"));
        assert!(!b("is_business_day(date(2026, 4, 3), \"uk\")"));
        assert!(b("is_business_day(date(2026, 4, 3), \"us\")"));
        // UK spring bank holiday 2026: last Monday of May = May 25.
        assert!(!b("is_business_day(date(2026, 5, 25), \"uk\")"));
    }

    #[test]
    fn roll_conventions_match_isda_semantics() {
        let env = MapEnv::new();
        let d = |src: &str| match eval_str(src, &env, Mode::Decimal).unwrap() {
            Value::Date(v) => v,
            other => panic!("expected date, got {other:?}"),
        };
        // 2026-05-31 is a Sunday: following -> Jun 1; modified_following stays
        // in May -> Fri May 29.
        assert_eq!(
            d("roll(date(2026, 5, 31), \"following\", \"weekend\")"),
            CalcDate::new(2026, 6, 1).unwrap()
        );
        assert_eq!(
            d("roll(date(2026, 5, 31), \"modified_following\", \"weekend\")"),
            CalcDate::new(2026, 5, 29).unwrap()
        );
        // Business day is unchanged under any convention.
        assert_eq!(
            d("roll(date(2026, 7, 21), \"modified_following\", \"us\")"),
            CalcDate::new(2026, 7, 21).unwrap()
        );
        // add_business_days skips the observed July 4 holiday:
        // Thu Jul 2 + 2 business days = Tue Jul 7 (skips Fri 3 observed, weekend).
        assert_eq!(
            d("add_business_days(date(2026, 7, 2), 2, \"us\")"),
            CalcDate::new(2026, 7, 7).unwrap()
        );
    }

    #[test]
    fn year_frac_30e_360() {
        // Aug 31 -> Feb 28: 30E/360 counts d1=30, d2=28.
        let v = n("round(year_frac(date(2026, 8, 31), date(2027, 2, 28), \"30e/360\"), 8)");
        // days = (2027-2026)*360 + (2-8)*30 + (28-30) = 360 - 180 - 2 = 178
        assert_eq!(v, dec("0.49444444"));
    }

    #[test]
    fn macrs_rates_match_irs_pub_946() {
        // 5-year GDS half-year convention (Table A-1).
        assert_eq!(n("macrs_rate(0, 5)"), dec("0.20"));
        assert_eq!(n("macrs_rate(1, 5)"), dec("0.32"));
        assert_eq!(n("macrs_rate(5, 5)"), dec("0.0576"));
        assert_eq!(n("macrs_rate(6, 5)"), dec("0")); // beyond the table
                                                     // 7-year year 3 = 12.49%.
        assert_eq!(n("macrs_rate(3, 7)"), dec("0.1249"));
        // Tables sum to 100%.
        for life in [5_i32, 7, 15, 20] {
            let mut total = Decimal::ZERO;
            for year in 0..=21 {
                total += n(&format!("macrs_rate({year}, {life})"));
            }
            assert_eq!(total, dec("1"), "life {life}");
        }
    }

    #[test]
    fn ipmt_ppmt_match_excel() {
        // Excel: IPMT(0.005, 1, 360, 100000) = -500.00 exactly.
        assert_eq!(n("round(ipmt(0.005, 1, 360, 100000), 6)"), dec("-500"));
        // Excel: PPMT(0.005, 1, 360, 100000) = -99.5505251527
        assert_eq!(n("round(ppmt(0.005, 1, 360, 100000), 4)"), dec("-99.5505"));
        // Excel: IPMT(0.005, 2, 360, 100000) = -499.5022473742
        assert_eq!(n("round(ipmt(0.005, 2, 360, 100000), 4)"), dec("-499.5022"));
        // ipmt + ppmt = pmt for every period.
        let total = n("round(ipmt(0.005, 120, 360, 100000) + ppmt(0.005, 120, 360, 100000), 6)");
        let pmt = n("round(pmt(0.005, 360, 100000), 6)");
        assert_eq!(total, pmt);
        // Zero-rate degenerate case.
        assert_eq!(n("ipmt(0, 5, 12, 1200)"), dec("0"));
        assert_eq!(n("ppmt(0, 5, 12, 1200)"), dec("-100"));
    }

    #[test]
    fn parse_date_and_months_between() {
        let env = MapEnv::new();
        let v = eval_str("parse_date(\"2027-07-01\")", &env, Mode::Decimal).unwrap();
        assert_eq!(v, Value::Date(CalcDate::new(2027, 7, 1).unwrap()));
        let v = eval_str("parse_date(\"2027-07\")", &env, Mode::Decimal).unwrap();
        assert_eq!(v, Value::Date(CalcDate::new(2027, 7, 1).unwrap()));
        // Lease-anniversary anchoring: months since lease start.
        assert_eq!(
            n("months_between(parse_date(\"2027-07\"), date(2029, 7, 1))"),
            dec("24")
        );
        assert_eq!(
            n("months_between(date(2026, 3, 1), date(2026, 1, 1))"),
            dec("-2")
        );
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

#[cfg(test)]
mod excel_compat_stability {
    use super::*;

    fn both(src: &str) -> (f64, f64) {
        let env = MapEnv::new();
        let num = |m| match eval_str(src, &env, m).unwrap() {
            Value::Number(x) => x.to_string().parse::<f64>().unwrap(),
            other => panic!("expected number, got {other:?}"),
        };
        (num(Mode::Decimal), num(Mode::ExcelCompat))
    }

    #[test]
    fn credit_pool_arithmetic_is_mode_stable() {
        // Does running the credit pack in excel_compat change its answers?
        // No — and this pins that, because it is not obvious. The pack looks
        // float-sensitive: 360 periods of compounding through pow().
        //
        // It is stable because the shapes that actually diverge are absent
        // here. Decimal mode already routes fractional exponents through the
        // f64 escape, so cpr_to_periodic is identical in both. What remains is
        // integer-exponent pow and multiplication, where 28 decimal digits and
        // f64's ~16 both far exceed whole-dollar precision. The divergence
        // excel_compat exists to reproduce — 0.1 + 0.2 != 0.3, and equality
        // comparisons on accumulated sums — the pack never performs.
        //
        // Worst observed is ~4e-14 relative, roughly ten orders of magnitude
        // inside benchmarks/credit/sifma_cash_flow_a's 0.51 tolerance.
        let r = "0.08 / 12";
        let smm = "cpr_to_periodic(0.11361512828387077, 12)";
        let k = format!("((1 - {smm}) - {smm})");
        let cases = [
            smm.to_string(),
            k.clone(),
            format!("100000000 * (1 - {smm}) * ({r})"),
            format!("100000000 * (1 - ({r}) / (pow(1 + {r}, 360) - 1)) * {smm}"),
            format!("pow({k}, 348)"),
            format!(
                "100000000 * ((pow(1 + {r}, 360) - pow(1 + {r}, 347)) / (pow(1 + {r}, 360) - 1)) \
                 * pow({k}, 347) * (1 - {smm}) * ({r})"
            ),
        ];
        for src in &cases {
            let (decimal, excel) = both(src);
            let rel = if decimal == 0.0 {
                (excel - decimal).abs()
            } else {
                (excel - decimal).abs() / decimal.abs()
            };
            assert!(
                rel < 1e-12,
                "mode divergence {rel:e} exceeds 1e-12 for `{src}` \
                 (decimal {decimal}, excel_compat {excel})"
            );
        }
    }
}
