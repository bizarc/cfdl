use crate::date::{CalcDate, DayCount};
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
        "cpr_to_smm" => {
            // SMM = 1 - (1 - CPR)^(1/12). Fractional exponent: f64_escape.
            let cpr = num(one(name, args, span)?)?;
            let smm = 1.0 - (1.0 - to_f64(cpr, span)?).powf(1.0 / 12.0);
            Ok(Value::Number(from_f64(smm, span)?))
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
