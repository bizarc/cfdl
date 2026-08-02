use crate::date::{CalcDate, DayCount, HolidayCalendar, RollConvention};
use crate::eval::{from_f64, pow_mode, powi_decimal, to_f64, Mode};
use crate::token::Span;
use crate::value::Value;
use crate::CalcError;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::{Decimal, RoundingStrategy};

type Arg<'a> = (Value, Span);

pub fn call(name: &str, args: &[Arg], span: Span, mode: Mode) -> Result<Value, CalcError> {
    match name {
        "abs" => {
            let x = num(one(name, args, span)?)?;
            Ok(Value::Number(x.abs()))
        }
        "min" => fold_nums(name, args, span, |a, b| if b < a { b } else { a }),
        "max" => fold_nums(name, args, span, |a, b| if b > a { b } else { a }),
        "sum" => {
            let mut acc = Decimal::ZERO;
            for a in args {
                acc = acc
                    .checked_add(num(a)?)
                    .ok_or_else(|| CalcError::new("numeric overflow in sum", Some(span)))?;
            }
            Ok(Value::Number(acc))
        }
        "avg" => {
            if args.is_empty() {
                return Err(CalcError::new(
                    "avg expects at least 1 argument",
                    Some(span),
                ));
            }
            let mut acc = Decimal::ZERO;
            for a in args {
                acc = acc
                    .checked_add(num(a)?)
                    .ok_or_else(|| CalcError::new("numeric overflow in avg", Some(span)))?;
            }
            Ok(Value::Number(acc / Decimal::from(args.len() as i64)))
        }
        // Excel ROUND: half away from zero (documented; NOT banker's rounding).
        "round" => {
            let (x, digits) = num_and_digits(name, args, span)?;
            Ok(Value::Number(x.round_dp_with_strategy(
                digits,
                RoundingStrategy::MidpointAwayFromZero,
            )))
        }
        "round_down" => {
            let (x, digits) = num_and_digits(name, args, span)?;
            Ok(Value::Number(
                x.round_dp_with_strategy(digits, RoundingStrategy::ToZero),
            ))
        }
        "round_up" => {
            let (x, digits) = num_and_digits(name, args, span)?;
            Ok(Value::Number(x.round_dp_with_strategy(
                digits,
                RoundingStrategy::AwayFromZero,
            )))
        }
        // Function form of `^`, kept for continuity with the earlier CEL dialect.
        "pow" => {
            let [base, exp] = exactly::<2>(name, args, span)?;
            Ok(Value::Number(pow_mode(num(base)?, num(exp)?, span, mode)?))
        }
        "clamp" => {
            let [x, lo, hi] = exactly::<3>(name, args, span)?;
            let (x, lo, hi) = (num(x)?, num(lo)?, num(hi)?);
            if lo > hi {
                return Err(CalcError::new(
                    format!("clamp: lower bound {lo} exceeds upper bound {hi}"),
                    Some(span),
                ));
            }
            Ok(Value::Number(x.max(lo).min(hi)))
        }
        "pmt" => annuity(name, args, span, mode, Annuity::Pmt),
        "pv" => annuity(name, args, span, mode, Annuity::Pv),
        "fv" => annuity(name, args, span, mode, Annuity::Fv),
        "nper" => nper(args, span),
        "rate" => rate(args, span),
        // MACRS GDS half-year convention percentages (IRS Pub 946).
        // macrs_rate(year, life): 0-based recovery year; 0 beyond the table.
        "macrs_rate" => {
            let [year, life] = exactly::<2>(name, args, span)?;
            let (year, life) = (int(year)?, int(life)?);
            let table: &[&str] = match life {
                5 => &["0.20", "0.32", "0.192", "0.1152", "0.1152", "0.0576"],
                7 => &[
                    "0.1429", "0.2449", "0.1749", "0.1249", "0.0893", "0.0892", "0.0893", "0.0446",
                ],
                15 => &[
                    "0.05", "0.095", "0.0855", "0.077", "0.0693", "0.0623", "0.059", "0.059",
                    "0.0591", "0.059", "0.0591", "0.059", "0.0591", "0.059", "0.0591", "0.0295",
                ],
                20 => &[
                    "0.0375", "0.07219", "0.06677", "0.06177", "0.05713", "0.05285", "0.04888",
                    "0.04522", "0.04462", "0.04461", "0.04462", "0.04461", "0.04462", "0.04461",
                    "0.04462", "0.04461", "0.04462", "0.04461", "0.04462", "0.04461", "0.02231",
                ],
                other => {
                    return Err(CalcError::new(
                        format!(
                            "macrs_rate: unsupported recovery period {other} (use 5, 7, 15, 20)"
                        ),
                        Some(span),
                    ))
                }
            };
            let rate = if year < 0 {
                return Err(CalcError::new(
                    "macrs_rate: year must be >= 0".to_string(),
                    Some(span),
                ));
            } else {
                table
                    .get(year as usize)
                    .map(|s| s.parse::<Decimal>().expect("table literal parses"))
                    .unwrap_or(Decimal::ZERO)
            };
            Ok(Value::Number(rate))
        }
        // ln(x) / exp(x): natural logarithm and its inverse.
        //
        // THESE EXIST TO TURN A CUMULATIVE PRODUCT INTO A CUMULATIVE SUM.
        // A survival factor under a varying hazard, or revenue under a growth
        // path that decays, is PROD(1 + r_i) — which has no closed form and
        // cannot be built by pow(1 + r, t), since that applies one period's
        // rate as though it had held throughout. But `series_sum` already
        // aggregates a stream over a period window, so
        //
        //     PROD(1 + r_i)  ==  exp(series_sum("ln_one_plus_r", 0, t))
        //
        // with a helper stream carrying ln(1 + r_t). That is a phase-2 stream
        // reading a phase-1 stream, which the engine already supports — no
        // per-period state and no backward reference required.
        //
        // PRECISION. Both escape to f64, as `pow` already does for fractional
        // exponents and `cpr_to_smm` does for its root. They are NOT
        // decimal-exact. Prefer a closed form wherever one exists; reach for
        // these when the alternative is not being able to express the quantity
        // at all. docs/03_expression_environment.md says the same.
        "ln" => {
            let x = num(one(name, args, span)?)?;
            if x <= Decimal::ZERO {
                return Err(CalcError::new(
                    format!("ln: argument must be greater than zero, got {x}"),
                    Some(span),
                ));
            }
            from_f64(to_f64(x, span)?.ln(), span).map(Value::Number)
        }
        "exp" => {
            let x = num(one(name, args, span)?)?;
            from_f64(to_f64(x, span)?.exp(), span).map(Value::Number)
        }
        // round_to(x, step): round x to the nearest multiple of step.
        //
        // HALF AWAY FROM ZERO, deliberately, and not the banker's rounding that
        // round_dp defaults to elsewhere in this file. Every convention this
        // exists for is statutory or contractual — a production tax credit
        // published to the nearest 0.1 cent, a tariff block, a tranche
        // denomination — and those all round halves up. The choice is invisible
        // at the call site, so it is stated here.
        //
        // A STEP rather than a decimal count, because the ticks that matter are
        // not all powers of ten: an eighth, a quarter-cent, a 25-unit lot.
        //
        // Note this rounds ONE value. It does not express a recurrence — "each
        // year is last year's rounded figure, escalated" needs a stream to read
        // its own prior period, which the language cannot do. See
        // docs/13_feature_backlog.md.
        "round_to" => {
            let [x, step] = exactly::<2>(name, args, span)?;
            let (x, step) = (num(x)?, num(step)?);
            if step <= Decimal::ZERO {
                return Err(CalcError::new(
                    format!("round_to: step must be greater than zero, got {step}"),
                    Some(span),
                ));
            }
            let quotient =
                (x / step).round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero);
            Ok(Value::Number(quotient * step))
        }
        // Excel IPMT/PPMT (ordinary annuity only; due=1 not yet supported).
        "ipmt" => interest_split(name, args, span, mode, Split::Interest),
        "ppmt" => interest_split(name, args, span, mode, Split::Principal),
        "cpr_to_smm" => {
            // SMM = 1 - (1 - CPR)^(1/12). Fractional exponent: f64_escape.
            let cpr = num(one(name, args, span)?)?;
            let smm = 1.0 - (1.0 - to_f64(cpr, span)?).powf(1.0 / 12.0);
            Ok(Value::Number(from_f64(smm, span)?))
        }
        // The cadence-general form of cpr_to_smm: the periodic mortality
        // equivalent to an annual CPR on a grid of `ppy` periods per year.
        //
        // A separate name rather than an optional second argument on
        // cpr_to_smm, because `cpr_to_smm(0.06, 4)` would read as "single
        // MONTHLY mortality, quarterly" — a lie in the name. cpr_to_smm keeps
        // meaning exactly what packs/credit/README.md says it means, forever.
        //
        // Note this is a root, not a division: CPR is an effective annual
        // survival rate, so it converts by (1-x)^(1/ppy). Note rates are
        // nominal and convert by division. Collapsing the two would be the
        // next silent-wrong-answer bug.
        "cpr_to_periodic" => {
            let [cpr, ppy] = exactly::<2>(name, args, span)?;
            let periods = to_f64(num(ppy)?, span)?;
            if periods <= 0.0 {
                return Err(CalcError::new(
                    format!("{name}: periods per year must be positive, got {periods}"),
                    Some(span),
                ));
            }
            let rate = 1.0 - (1.0 - to_f64(num(cpr)?, span)?).powf(1.0 / periods);
            Ok(Value::Number(from_f64(rate, span)?))
        }
        "date" => {
            let [y, m, d] = exactly::<3>(name, args, span)?;
            let (y, m, d) = (int(y)?, int(m)?, int(d)?);
            CalcDate::new(y as i32, m as u32, d as u32)
                .map(Value::Date)
                .ok_or_else(|| {
                    CalcError::new(format!("invalid date {y:04}-{m:02}-{d:02}"), Some(span))
                })
        }
        // parse_date("2027-07-01" | "2027-07"): ISO date text to a date value.
        "parse_date" => {
            let arg = one(name, args, span)?;
            let Value::Text(raw) = &arg.0 else {
                return Err(CalcError::new(
                    format!("parse_date expects text, got {}", arg.0.type_name()),
                    Some(arg.1),
                ));
            };
            let parts: Vec<&str> = raw.split('-').collect();
            let parsed = match parts.as_slice() {
                [y, m, d] => (y.parse::<i32>(), m.parse::<u32>(), d.parse::<u32>()),
                [y, m] => (y.parse::<i32>(), m.parse::<u32>(), Ok(1)),
                _ => {
                    return Err(CalcError::new(
                        format!("parse_date: invalid ISO date `{raw}`"),
                        Some(arg.1),
                    ))
                }
            };
            match parsed {
                (Ok(y), Ok(m), Ok(d)) => CalcDate::new(y, m, d).map(Value::Date).ok_or_else(|| {
                    CalcError::new(format!("parse_date: invalid date `{raw}`"), Some(arg.1))
                }),
                _ => Err(CalcError::new(
                    format!("parse_date: invalid ISO date `{raw}`"),
                    Some(arg.1),
                )),
            }
        }
        // Whole calendar months from d1 to d2 (day-of-month ignored):
        // months_between(date(2026,1,15), date(2026,3,1)) == 2.
        "months_between" => {
            let [d1, d2] = exactly::<2>(name, args, span)?;
            let (a, b) = (date(d1)?, date(d2)?);
            let months =
                (b.year() as i64 - a.year() as i64) * 12 + (b.month() as i64 - a.month() as i64);
            Ok(Value::Number(Decimal::from(months)))
        }
        // Whole days from d1 to d2, the day-grid counterpart of
        // months_between. `d2 - d1` already yields days; this is the named
        // form, so a pack lowering rule can count elapsed periods on a daily
        // calendar the same way it does on a monthly one.
        "days_between" => {
            let [d1, d2] = exactly::<2>(name, args, span)?;
            let (a, b) = (date(d1)?, date(d2)?);
            Ok(Value::Number(Decimal::from(a.days_between(&b))))
        }
        "edate" => {
            let [d, m] = exactly::<2>(name, args, span)?;
            Ok(Value::Date(date(d)?.add_months(int(m)? as i32)))
        }
        "eomonth" => {
            let [d, m] = exactly::<2>(name, args, span)?;
            Ok(Value::Date(date(d)?.end_of_month(int(m)? as i32)))
        }
        "year_frac" => {
            let [d1, d2, basis] = exactly::<3>(name, args, span)?;
            let convention = match &basis.0 {
                Value::Text(s) => DayCount::parse(s).ok_or_else(|| {
                    CalcError::new(
                        format!("unknown day-count basis `{s}` (use \"30/360\", \"act/360\", \"act/365\")"),
                        Some(basis.1),
                    )
                })?,
                other => {
                    return Err(CalcError::new(
                        format!("year_frac basis must be text, got {}", other.type_name()),
                        Some(basis.1),
                    ))
                }
            };
            let (days, denom) = convention.year_frac(&date(d1)?, &date(d2)?);
            Ok(Value::Number(Decimal::from(days) / Decimal::from(denom)))
        }
        "is_business_day" => {
            let [d, cal] = exactly::<2>(name, args, span)?;
            Ok(Value::Bool(calendar(cal)?.is_business_day(&date(d)?)))
        }
        "roll" => {
            let [d, conv, cal] = exactly::<3>(name, args, span)?;
            let convention = roll_convention(conv)?;
            Ok(Value::Date(calendar(cal)?.roll(&date(d)?, convention)))
        }
        "add_business_days" => {
            let [d, n, cal] = exactly::<3>(name, args, span)?;
            Ok(Value::Date(
                calendar(cal)?.add_business_days(&date(d)?, int(n)?),
            ))
        }
        other => Err(CalcError::new(
            format!("unknown function `{other}`"),
            Some(span),
        )),
    }
}

// ---------- annuity family (Excel sign conventions) ----------
//
// Cash-flow identity: pv*f + pmt*(1 + rate*due)*(f - 1)/rate + fv = 0,
// where f = (1 + rate)^nper. Decimal-exact when nper is a whole number;
// fractional nper takes the documented f64_escape.

enum Annuity {
    Pmt,
    Pv,
    Fv,
}

fn annuity(
    name: &str,
    args: &[Arg],
    span: Span,
    mode: Mode,
    which: Annuity,
) -> Result<Value, CalcError> {
    if args.len() < 3 || args.len() > 5 {
        return Err(CalcError::new(
            format!(
                "{name} expects 3 to 5 arguments (rate, nper, amount, [fv|pv], [due]), got {}",
                args.len()
            ),
            Some(span),
        ));
    }
    let rate = num(&args[0])?;
    let nper = num(&args[1])?;
    let third = num(&args[2])?;
    let fourth = args.get(3).map(num).transpose()?.unwrap_or(Decimal::ZERO);
    let due = match args.get(4).map(num).transpose()?.unwrap_or(Decimal::ZERO) {
        d if d == Decimal::ZERO => Decimal::ZERO,
        d if d == Decimal::ONE => Decimal::ONE,
        d => {
            return Err(CalcError::new(
                format!("{name} `due` must be 0 or 1, got {d}"),
                Some(args[4].1),
            ))
        }
    };

    if rate.is_zero() {
        if nper.is_zero() {
            return Err(CalcError::new(
                format!("{name}: nper must not be zero"),
                Some(span),
            ));
        }
        return Ok(Value::Number(match which {
            Annuity::Pmt => -(third + fourth) / nper, // third=pv, fourth=fv
            Annuity::Pv => -(fourth + third * nper),  // third=pmt, fourth=fv
            Annuity::Fv => -(fourth + third * nper),  // third=pmt, fourth=pv
        }));
    }

    let one_plus = Decimal::ONE + rate;
    let f = if mode == Mode::Decimal && nper.fract() == Decimal::ZERO {
        let n = nper
            .to_i64()
            .ok_or_else(|| CalcError::new(format!("{name}: nper out of range"), Some(args[1].1)))?;
        powi_decimal(one_plus, n, span)?
    } else {
        // f64_escape: fractional nper or excel_compat mode.
        from_f64(to_f64(one_plus, span)?.powf(to_f64(nper, span)?), span)?
    };
    let due_factor = Decimal::ONE + rate * due;
    let annuity_factor = due_factor * (f - Decimal::ONE) / rate;

    let result = match which {
        // third=pv, fourth=fv
        Annuity::Pmt => -(third * f + fourth) / annuity_factor,
        // third=pmt, fourth=fv
        Annuity::Pv => -(fourth + third * annuity_factor) / f,
        // third=pmt, fourth=pv
        Annuity::Fv => -(fourth * f + third * annuity_factor),
    };
    Ok(Value::Number(result))
}

fn nper(args: &[Arg], span: Span) -> Result<Value, CalcError> {
    if args.len() < 3 || args.len() > 5 {
        return Err(CalcError::new(
            format!(
                "nper expects 3 to 5 arguments (rate, pmt, pv, [fv], [due]), got {}",
                args.len()
            ),
            Some(span),
        ));
    }
    let rate = to_f64(num(&args[0])?, span)?;
    let pmt = to_f64(num(&args[1])?, span)?;
    let pv = to_f64(num(&args[2])?, span)?;
    let fv = args
        .get(3)
        .map(num)
        .transpose()?
        .map(|d| to_f64(d, span))
        .transpose()?
        .unwrap_or(0.0);
    let due = args
        .get(4)
        .map(num)
        .transpose()?
        .map(|d| to_f64(d, span))
        .transpose()?
        .unwrap_or(0.0);
    if rate == 0.0 {
        if pmt == 0.0 {
            return Err(CalcError::new(
                "nper: pmt must not be zero when rate is 0",
                Some(span),
            ));
        }
        return Ok(Value::Number(from_f64(-(pv + fv) / pmt, span)?));
    }
    // Iterative-free closed form (logarithms): documented f64_escape.
    let adj = pmt * (1.0 + rate * due) / rate;
    let numerator = adj - fv;
    let denominator = pv + adj;
    if numerator <= 0.0 || denominator == 0.0 || numerator / denominator <= 0.0 {
        return Err(CalcError::new(
            "nper: no solution for these cash flows",
            Some(span),
        ));
    }
    let n = (numerator / denominator).ln() / (1.0 + rate).ln();
    Ok(Value::Number(from_f64(n, span)?))
}

/// Newton's method in f64 (documented f64_escape); IRR-class solvers have no
/// industry standard — solver and tolerance are part of the documented contract.
fn rate(args: &[Arg], span: Span) -> Result<Value, CalcError> {
    if args.len() < 3 || args.len() > 6 {
        return Err(CalcError::new(
            format!(
                "rate expects 3 to 6 arguments (nper, pmt, pv, [fv], [due], [guess]), got {}",
                args.len()
            ),
            Some(span),
        ));
    }
    let nper = to_f64(num(&args[0])?, span)?;
    let pmt = to_f64(num(&args[1])?, span)?;
    let pv = to_f64(num(&args[2])?, span)?;
    let fv = args
        .get(3)
        .map(num)
        .transpose()?
        .map(|d| to_f64(d, span))
        .transpose()?
        .unwrap_or(0.0);
    let due = args
        .get(4)
        .map(num)
        .transpose()?
        .map(|d| to_f64(d, span))
        .transpose()?
        .unwrap_or(0.0);
    let guess = args
        .get(5)
        .map(num)
        .transpose()?
        .map(|d| to_f64(d, span))
        .transpose()?
        .unwrap_or(0.1);

    let f = |r: f64| -> f64 {
        if r.abs() < 1e-14 {
            return pv + pmt * nper + fv;
        }
        let g = (1.0 + r).powf(nper);
        pv * g + pmt * (1.0 + r * due) * (g - 1.0) / r + fv
    };
    let mut r = guess;
    for _ in 0..100 {
        let y = f(r);
        if y.abs() < 1e-12 {
            return Ok(Value::Number(from_f64(r, span)?));
        }
        let h = 1e-8_f64.max(r.abs() * 1e-8);
        let dy = (f(r + h) - f(r - h)) / (2.0 * h);
        if dy == 0.0 || !dy.is_finite() {
            break;
        }
        let next = r - y / dy;
        if !next.is_finite() || next <= -1.0 {
            break;
        }
        if (next - r).abs() < 1e-14 {
            return Ok(Value::Number(from_f64(next, span)?));
        }
        r = next;
    }
    Err(CalcError::new("rate: solver did not converge", Some(span)))
}

enum Split {
    Interest,
    Principal,
}

/// Excel IPMT/PPMT: interest / principal portion of payment `per` (1-based)
/// on a level-pay annuity. Decimal-exact for whole-number periods.
fn interest_split(
    name: &str,
    args: &[Arg],
    span: Span,
    mode: Mode,
    which: Split,
) -> Result<Value, CalcError> {
    if args.len() < 4 || args.len() > 5 {
        return Err(CalcError::new(
            format!(
                "{name} expects 4 or 5 arguments (rate, per, nper, pv, [fv]), got {}",
                args.len()
            ),
            Some(span),
        ));
    }
    let rate = num(&args[0])?;
    let per = int(&args[1])?;
    let nper = num(&args[2])?;
    let pv = num(&args[3])?;
    let fv = args.get(4).map(num).transpose()?.unwrap_or(Decimal::ZERO);
    if per < 1 {
        return Err(CalcError::new(
            format!("{name}: per must be >= 1"),
            Some(args[1].1),
        ));
    }
    if rate.is_zero() {
        // No interest; payment is pure principal.
        let payment = -(pv + fv) / nper;
        return Ok(Value::Number(match which {
            Split::Interest => Decimal::ZERO,
            Split::Principal => payment,
        }));
    }
    let n = nper
        .to_i64()
        .ok_or_else(|| CalcError::new(format!("{name}: nper out of range"), Some(args[2].1)))?;
    let one_plus = Decimal::ONE + rate;
    let f = if mode == Mode::Decimal {
        powi_decimal(one_plus, n, span)?
    } else {
        from_f64(to_f64(one_plus, span)?.powf(n as f64), span)?
    };
    let payment = -(pv * f + fv) * rate / (f - Decimal::ONE);
    // Outstanding balance after (per - 1) payments.
    let fk = if mode == Mode::Decimal {
        powi_decimal(one_plus, per - 1, span)?
    } else {
        from_f64(to_f64(one_plus, span)?.powf((per - 1) as f64), span)?
    };
    let balance = pv * fk + payment * (fk - Decimal::ONE) / rate;
    let interest = -(balance * rate);
    Ok(Value::Number(match which {
        Split::Interest => interest,
        Split::Principal => payment - interest,
    }))
}

// ---------- argument helpers ----------

fn one<'a>(name: &str, args: &'a [Arg], span: Span) -> Result<&'a Arg<'a>, CalcError> {
    if args.len() != 1 {
        return Err(CalcError::new(
            format!("{name} expects 1 argument, got {}", args.len()),
            Some(span),
        ));
    }
    Ok(&args[0])
}

fn exactly<'a, const N: usize>(
    name: &str,
    args: &'a [Arg],
    span: Span,
) -> Result<[&'a Arg<'a>; N], CalcError> {
    if args.len() != N {
        return Err(CalcError::new(
            format!("{name} expects {N} arguments, got {}", args.len()),
            Some(span),
        ));
    }
    let mut out = [&args[0]; N];
    for (i, slot) in out.iter_mut().enumerate() {
        *slot = &args[i];
    }
    Ok(out)
}

fn num(arg: &Arg) -> Result<Decimal, CalcError> {
    match &arg.0 {
        Value::Number(d) => Ok(*d),
        other => Err(CalcError::new(
            format!("expected number, got {}", other.type_name()),
            Some(arg.1),
        )),
    }
}

fn int(arg: &Arg) -> Result<i64, CalcError> {
    let d = num(arg)?;
    if d.fract() != Decimal::ZERO {
        return Err(CalcError::new(
            format!("expected whole number, got {d}"),
            Some(arg.1),
        ));
    }
    d.to_i64()
        .ok_or_else(|| CalcError::new(format!("number out of range: {d}"), Some(arg.1)))
}

fn date(arg: &Arg) -> Result<CalcDate, CalcError> {
    match &arg.0 {
        Value::Date(d) => Ok(*d),
        other => Err(CalcError::new(
            format!("expected date, got {}", other.type_name()),
            Some(arg.1),
        )),
    }
}

fn num_and_digits(name: &str, args: &[Arg], span: Span) -> Result<(Decimal, u32), CalcError> {
    if args.is_empty() || args.len() > 2 {
        return Err(CalcError::new(
            format!("{name} expects 1 or 2 arguments, got {}", args.len()),
            Some(span),
        ));
    }
    let x = num(&args[0])?;
    let digits = match args.get(1) {
        Some(a) => {
            let d = int(a)?;
            if !(0..=28).contains(&d) {
                return Err(CalcError::new(
                    format!("{name} digits must be 0..=28, got {d}"),
                    Some(a.1),
                ));
            }
            d as u32
        }
        None => 0,
    };
    Ok((x, digits))
}

fn fold_nums(
    name: &str,
    args: &[Arg],
    span: Span,
    pick: fn(Decimal, Decimal) -> Decimal,
) -> Result<Value, CalcError> {
    if args.is_empty() {
        return Err(CalcError::new(
            format!("{name} expects at least 1 argument"),
            Some(span),
        ));
    }
    let mut acc = num(&args[0])?;
    for a in &args[1..] {
        acc = pick(acc, num(a)?);
    }
    Ok(Value::Number(acc))
}

fn calendar(arg: &Arg) -> Result<HolidayCalendar, CalcError> {
    match &arg.0 {
        Value::Text(s) => HolidayCalendar::parse(s).ok_or_else(|| {
            CalcError::new(
                format!("unknown calendar `{s}` (use \"weekend\", \"us\", \"target\", \"uk\")"),
                Some(arg.1),
            )
        }),
        other => Err(CalcError::new(
            format!("expected calendar name text, got {}", other.type_name()),
            Some(arg.1),
        )),
    }
}

fn roll_convention(arg: &Arg) -> Result<RollConvention, CalcError> {
    match &arg.0 {
        Value::Text(s) => RollConvention::parse(s).ok_or_else(|| {
            CalcError::new(
                format!(
                    "unknown roll convention `{s}` (use \"none\", \"following\", \"modified_following\", \"preceding\", \"modified_preceding\")"
                ),
                Some(arg.1),
            )
        }),
        other => Err(CalcError::new(
            format!("expected roll convention text, got {}", other.type_name()),
            Some(arg.1),
        )),
    }
}

/// `series_sum(name, from_t, to_t)` / `series_avg(...)`: cross-stream series
/// aggregation resolved by the host via `Env::series_aggregate`.
pub(crate) fn series_call(
    name: &str,
    args: &[Arg],
    span: Span,
    env: &dyn crate::eval::Env,
) -> Result<Value, CalcError> {
    let [series, from, to] = exactly::<3>(name, args, span)?;
    let series_name = match &series.0 {
        Value::Text(s) => s.clone(),
        other => {
            return Err(CalcError::new(
                format!(
                    "{name} expects a series name text, got {}",
                    other.type_name()
                ),
                Some(series.1),
            ))
        }
    };
    let (from, to) = (int(from)?, int(to)?);
    let mean = name == "series_avg";
    env.series_aggregate(&series_name, from, to, mean)
        .map(Value::Number)
        .ok_or_else(|| {
            CalcError::new(
                format!("{name}: series `{series_name}` is not available in this context"),
                Some(series.1),
            )
        })
}

/// `curve_value(name, date)`: named curve lookup resolved by the host via
/// `Env::curve_value` (interpolation is the curve's own declaration).
pub(crate) fn curve_call(
    name: &str,
    args: &[Arg],
    span: Span,
    env: &dyn crate::eval::Env,
) -> Result<Value, CalcError> {
    let [curve, date] = exactly::<2>(name, args, span)?;
    let curve_name = match &curve.0 {
        Value::Text(s) => s.clone(),
        other => {
            return Err(CalcError::new(
                format!(
                    "{name} expects a curve name text, got {}",
                    other.type_name()
                ),
                Some(curve.1),
            ))
        }
    };
    let date = match &date.0 {
        Value::Date(d) => *d,
        other => {
            return Err(CalcError::new(
                format!("{name} expects a date, got {}", other.type_name()),
                Some(date.1),
            ))
        }
    };
    env.curve_value(&curve_name, date)
        .map(Value::Number)
        .ok_or_else(|| {
            CalcError::new(
                format!("{name}: curve `{curve_name}` is not available in this context"),
                Some(curve.1),
            )
        })
}

/// Does the expression call any of the given function names? Used by the
/// engine to split stream evaluation into phases.
pub fn expr_calls_any(expr: &crate::Expr, names: &[&str]) -> bool {
    use crate::ExprKind;
    match &expr.kind {
        ExprKind::Call { name, args } => {
            names.contains(&name.as_str()) || args.iter().any(|a| expr_calls_any(a, names))
        }
        ExprKind::Unary { expr, .. } => expr_calls_any(expr, names),
        ExprKind::Binary { lhs, rhs, .. } => {
            expr_calls_any(lhs, names) || expr_calls_any(rhs, names)
        }
        _ => false,
    }
}
