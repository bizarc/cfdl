// Extracted from lib.rs — one stage of the engine. See docs/13 §7.44.
use super::*;

/// For each timeline period, the period whose amount is paid there.
///
/// `None` means no payment. For an annuity due the two indices coincide. For
/// an ordinary annuity the payment lands one interval after the period that
/// earned it, so the amount must still be evaluated against the earning
/// period — `time.t` inside the amount expression refers to accrual, not to
/// settlement.
pub(crate) fn schedule_accruals(
    schedule: &IrSchedule,
    timeline: &[Date],
) -> Result<Vec<Vec<usize>>, EngineError> {
    // A state-anchored schedule has no dates until the walk finds the
    // entries; it opens empty and `anchored_accruals` fills it per entry.
    if schedule.kind == "StateEnter" {
        return Ok(vec![Vec::new(); timeline.len()]);
    }
    let mut out = vec![Vec::new(); timeline.len()];
    apply_schedule_indices(schedule, timeline, &mut out)?;
    Ok(out)
}

/// The accruals of a state-anchored schedule, given the entries known so far
/// (`docs/28` §6.2). Each entry opens its own window of `anchor_periods`
/// grid periods — a re-entered state re-anchors, and a self-edge is an entry
/// — and within the window the schedule behaves exactly as `every <interval>
/// from <entry> to <window end>`: one synthesized Every per entry, so
/// intervals, placement, day rules and payment terms all mean what they mean
/// everywhere else.
pub(crate) fn anchored_accruals(
    schedule: &IrSchedule,
    entries: &[usize],
    timeline: &[Date],
) -> Result<Vec<Vec<usize>>, EngineError> {
    let mut out = vec![Vec::new(); timeline.len()];
    let window = schedule.anchor_periods.unwrap_or(0).max(0) as usize;
    if window == 0 {
        return Ok(out);
    }
    for &entry in entries {
        if entry >= timeline.len() {
            continue;
        }
        let last = (entry + window - 1).min(timeline.len() - 1);
        let synthesized = IrSchedule {
            kind: "Every".to_string(),
            on: None,
            every: schedule.every.clone(),
            placement: schedule.placement,
            net_days: schedule.net_days,
            net_months: schedule.net_months,
            from: Some(timeline[entry].to_string()),
            to: Some(timeline[last].to_string()),
            on_rule: schedule.on_rule.clone(),
            phase: None,
            convention: schedule.convention.clone(),
            calendar: schedule.calendar.clone(),
            except_dates: schedule.except_dates.clone(),
            also_dates: schedule.also_dates.clone(),
            anchor_entity: None,
            anchor_state: None,
            anchor_periods: None,
        };
        apply_schedule_indices(&synthesized, timeline, &mut out)?;
    }
    // Two overlapping windows must not double an accrual.
    for cell in &mut out {
        cell.sort_unstable();
        cell.dedup();
    }
    Ok(out)
}

/// Calendar days in the period beginning `date` on `calendar`.
///
/// Derived from the calendar rather than the timeline so it is available
/// wherever the expression environment is built, including the projection
/// tail. Actual days, not a nominal 30 — that is the whole point: an
/// Actual/360 accrual pays more in a 31-day month than a 30-day one.
pub(crate) fn days_in_period(calendar: &str, date: &Date) -> f64 {
    let next = match calendar {
        "daily" => return 1.0,
        "monthly" => date.add_months(1),
        "quarterly" => date.add_months(3),
        "annual" => date.add_months(12),
        _ => date.add_months(1),
    };
    days_between(date, &next).max(1) as f64
}

pub(crate) fn periods_per_year(calendar: &str) -> f64 {
    match calendar {
        "daily" => 365.0,
        "monthly" => 12.0,
        "quarterly" => 4.0,
        "annual" => 1.0,
        _ => 1.0,
    }
}

/// How far through its period a payment falls, per docs/12_payment_timing.md.
///
/// A payment belongs to the period that earned it; this says where inside that
/// period the cash sits, and so how far it is discounted. One mechanism covers
/// every case — an annuity due sits at the start, an ordinary annuity at the
/// end, and a day rule at its own point in between.
/// The placement a schedule states, or `None` when it states none and the
/// form's default applies. The ONE place that resolution lives.
pub(crate) fn placement_of(schedule: &IrSchedule) -> Option<Placement> {
    schedule.placement
}

pub(crate) fn discount_offset(schedule: &IrSchedule, calendar: &str) -> f64 {
    // WHERE IN ITS PERIOD THE CASH SITS. One axis, three positions, and the
    // only thing that differs between forms is which position they default to
    // when the model does not say.
    //
    // A one-shot happens on its stated date, so it defaults to that period's
    // START: a purchase on 2026-01 settles then and is not discounted for a
    // period it never waited through. A recurrence defaults to its period's
    // END — an ordinary annuity, where the interval elapses and then payment
    // falls.
    //
    // That one-shot default is right for an acquisition and wrong for a
    // disposal: a reversion is taken at the END of the holding period, so a
    // year-5 sale must discount five periods rather than four. On a monthly
    // model the gap is one month; on an annual one it is a whole year, and 9%
    // of the reversion at 12%. `end` is how a model says so.
    let one_shot = schedule.kind == "OnDate";
    match placement_of(schedule) {
        Some(Placement::Start) => return 0.0,
        Some(Placement::Mid) => return 0.5,
        Some(Placement::End) => return 1.0,
        // Unstated: a one-shot opens its period, a recurrence closes it — and
        // a recurrence's close may be refined by a day rule below.
        None if one_shot => return 0.0,
        None => {}
    }
    match schedule.on_rule.as_ref() {
        // `on day <n>`: n days into the period, so the divisor is the period's
        // own length. It was a literal 30 until an annual model wanted the
        // mid-period convention and got it from `on day 15` — which is right
        // only on a monthly grid. On a quarterly or annual calendar `day 15`
        // is 15 days into a quarter or a year, not half of one, and on a daily
        // calendar the payment date IS the period, so it clamps to its end.
        Some(rule) if rule.kind == "DayOfMonth" => {
            (rule.day.clamp(1, 31) as f64 / (365.0 / periods_per_year(calendar))).min(1.0)
        }
        // End of month is the period end, same as the default.
        _ => 1.0,
    }
}

/// Occurrence dates from `from`, stepping by `interval`, up to and including `to`.
pub(crate) fn occurrences(
    from: &Date,
    to: &Date,
    interval: &str,
) -> Result<Vec<Date>, EngineError> {
    // Guard against a zero step producing an unbounded loop.
    let step_months = match interval {
        "monthly" => Some(1),
        "quarterly" => Some(3),
        "annual" => Some(12),
        "daily" | "weekly" => None,
        other => {
            return Err(EngineError::Schedule(format!(
                "unsupported schedule interval: {other}"
            )))
        }
    };
    let step_days = match interval {
        "daily" => Some(1),
        "weekly" => Some(7),
        _ => None,
    };

    let mut out = Vec::new();
    let advance = |d: &Date| match (step_months, step_days) {
        (Some(m), _) => d.add_months(m),
        (_, Some(days)) => d.add_days(days),
        _ => d.clone(),
    };
    let mut cursor = from.clone();
    let last = to.clone();
    // A monthly stream over a century is ~1200 occurrences; this ceiling only
    // exists so a malformed range cannot spin.
    let limit = 100_000;
    while cursor <= last && out.len() < limit {
        out.push(cursor.clone());
        let next = advance(&cursor);
        if next == cursor {
            break;
        }
        cursor = next;
    }
    Ok(out)
}

/// Move an occurrence within its own interval per `on day <n>` / `on eom`.
pub(crate) fn place_in_interval(occurrence: &Date, on_rule: Option<&IrOnRule>) -> Date {
    match on_rule {
        Some(rule) if rule.kind == "EndOfMonth" => Date {
            year: occurrence.year,
            month: occurrence.month,
            day: days_in_month(occurrence.year, occurrence.month),
        },
        Some(rule) if rule.kind == "DayOfMonth" => Date {
            year: occurrence.year,
            month: occurrence.month,
            // Clamp so day 31 in a 30-day month lands on the 30th rather than
            // rolling into the next period.
            day: (rule.day.max(1) as u32).min(days_in_month(occurrence.year, occurrence.month)),
        },
        _ => occurrence.clone(),
    }
}

/// The timeline bucket containing `date`: the last period starting at or
/// before it. An occurrence mid-period belongs to that period, so an exact
/// match is not required.
pub(crate) fn period_index(timeline: &[Date], date: &Date) -> Option<usize> {
    if timeline.first().is_some_and(|first| date < first) {
        return None;
    }
    timeline.iter().rposition(|d| d <= date)
}

/// Populate `out[payment] = Some(accrual)` for every occurrence.
///
/// Mirrors `apply_schedule`, but records which period earned each payment
/// rather than accumulating an amount. `OnDate` and the explicit `also`/
/// `except` dates are point events, so their accrual and payment periods are
/// the same.
/// The last day of the period at `idx` — the day before the next one starts.
///
/// A period's spacing is taken from its neighbours rather than the calendar,
/// so this works on any cadence. The final period borrows the spacing of the
/// one before it.
pub(crate) fn period_end(timeline: &[Date], idx: usize) -> Date {
    match timeline.get(idx + 1) {
        Some(next) => next.add_days(-1).max(timeline[idx].clone()),
        None => {
            let span = timeline
                .get(idx.wrapping_sub(1))
                .map(|prev| days_between(prev, &timeline[idx]))
                .unwrap_or(30)
                .max(1);
            timeline[idx].add_days(span - 1)
        }
    }
}

/// Whole days from `from` to `to`, negative if `to` precedes it.
pub(crate) fn days_between(from: &Date, to: &Date) -> i32 {
    (to.to_epoch_days() - from.to_epoch_days()) as i32
}

/// Whether a date falls past the last period the timeline models.
///
/// The final period spans from its start date up to the next one that would
/// have followed, so a date inside that span still belongs to it. Spacing is
/// taken from the last two periods rather than the calendar, which keeps this
/// independent of the cadence.
pub(crate) fn beyond_timeline(timeline: &[Date], date: &Date) -> bool {
    let (Some(last), Some(prev)) = (
        timeline.last(),
        timeline.get(timeline.len().wrapping_sub(2)),
    ) else {
        return false;
    };
    let span_days = days_between(prev, last).max(1);
    *date >= last.add_days(span_days)
}

pub(crate) fn apply_schedule_indices(
    schedule: &IrSchedule,
    timeline: &[Date],
    out: &mut [Vec<usize>],
) -> Result<(), EngineError> {
    let roll = schedule_roll(schedule)?;
    match schedule.kind.as_str() {
        "OnDate" => {
            if let Some(on) = &schedule.on {
                let target = roll_date(&Date::parse(on)?, roll);
                if let Some(idx) = timeline.iter().position(|d| *d == target) {
                    out[idx].push(idx);
                }
            }
        }
        "Every" => {
            let default_from = timeline
                .first()
                .map(|d| d.to_string())
                .unwrap_or_else(|| "1970-01-01".to_string());
            let default_to = timeline
                .last()
                .map(|d| d.to_string())
                .unwrap_or_else(|| "1970-01-01".to_string());
            let from = schedule.from.as_deref().unwrap_or(default_from.as_str());
            let to = schedule.to.as_deref().unwrap_or(default_to.as_str());
            let from_date = Date::parse(from)?;
            let to_date = Date::parse(to)?;
            let interval = schedule.every.as_deref().unwrap_or("monthly");

            // Accruals run over [from, to]; each settles at its own end for an
            // ordinary annuity, or at its start for an annuity due.
            let starts = occurrences(&from_date, &to_date, interval)?;
            for (k, start) in starts.iter().enumerate() {
                let accrual_idx = match period_index(timeline, start) {
                    Some(i) => i,
                    None => continue,
                };
                // An annuity due pays as its interval opens. An ordinary
                // annuity pays as it closes — the last calendar period the
                // interval covers, which for an annual interval on a monthly
                // grid is its twelfth month, not its first.
                let pay_idx = if placement_of(schedule) == Some(Placement::Start) {
                    accrual_idx
                } else {
                    let next = starts
                        .get(k + 1)
                        .cloned()
                        .unwrap_or_else(|| step_once(start, interval));
                    // The interval closes in the last period before the
                    // next one opens. Taking that directly, rather than
                    // stepping back from the next interval's index, keeps the
                    // final interval correct when it closes at or past the
                    // end of the timeline.
                    timeline
                        .iter()
                        .rposition(|d| *d < next)
                        .unwrap_or(accrual_idx)
                };
                // The billing date, then the payment terms, then the roll.
                // Order matters: the due date is N days after billing, and it
                // is that due date which moves off a weekend — not the bill.
                //
                // Billing happens when the period closes, not when it opens:
                // January's electricity is invoiced at the end of January, so
                // net-30 falls in early March, not late January. A day rule
                // (`on day 15`, `on eom`) names the billing date explicitly
                // and overrides that.
                let has_terms = schedule.net_days.is_some_and(|d| d != 0)
                    || schedule.net_months.is_some_and(|m| m != 0);
                let billed = match (has_terms, schedule.on_rule.as_ref()) {
                    (true, None) => period_end(timeline, pay_idx),
                    _ => place_in_interval(&timeline[pay_idx], schedule.on_rule.as_ref()),
                };
                // Months step by the calendar, not by 30 days: a six-month lag
                // is six months, and the two diverge once billing is not at a
                // month end.
                let due = match (schedule.net_days, schedule.net_months) {
                    (_, Some(m)) if m != 0 => billed.add_months(m as i32),
                    (Some(d), _) if d != 0 => billed.add_days(d as i32),
                    _ => billed,
                };
                let rolled = roll_date(&due, roll);

                // period_index clamps a date past the end to the last period,
                // which would pile deferred cash into the final bucket and
                // overstate it. A payment that falls outside the modeled
                // horizon is a modeling error, not a rounding one.
                if beyond_timeline(timeline, &rolled) {
                    return Err(EngineError::Schedule(format!(
                        "a payment accruing in period {} settles on {} under these payment terms, past the end of the model timeline. Extend the timeline so the cash has a period to land in.",
                        accrual_idx + 1,
                        rolled
                    )));
                }
                let settled = period_index(timeline, &rolled).unwrap_or(pay_idx);
                out[settled].push(accrual_idx);
            }
        }
        other => {
            return Err(EngineError::Schedule(format!(
                "unsupported schedule kind: {other}"
            )));
        }
    }
    for raw in &schedule.except_dates {
        let target = roll_date(&Date::parse(raw)?, roll);
        if let Some(idx) = timeline.iter().position(|d| *d == target) {
            out[idx].clear();
        }
    }
    for raw in &schedule.also_dates {
        let target = roll_date(&Date::parse(raw)?, roll);
        if let Some(idx) = timeline.iter().position(|d| *d == target) {
            out[idx].push(idx);
        }
    }
    Ok(())
}

/// Resolve the schedule's business-day roll, if any. A convention without a
/// calendar defaults to the weekend-only calendar; a calendar without a
/// convention defaults to `following`.
pub(crate) fn schedule_roll(
    schedule: &IrSchedule,
) -> Result<Option<(cfdl_calc::RollConvention, cfdl_calc::HolidayCalendar)>, EngineError> {
    if schedule.convention.is_none() && schedule.calendar.is_none() {
        return Ok(None);
    }
    let convention = match schedule.convention.as_deref() {
        None => cfdl_calc::RollConvention::Following,
        Some(raw) => cfdl_calc::RollConvention::parse(raw)
            .ok_or_else(|| EngineError::Schedule(format!("unknown roll convention: {raw}")))?,
    };
    let calendar = match schedule.calendar.as_deref() {
        None => cfdl_calc::HolidayCalendar::Weekend,
        Some(raw) => cfdl_calc::HolidayCalendar::parse(raw)
            .ok_or_else(|| EngineError::Schedule(format!("unknown holiday calendar: {raw}")))?,
    };
    Ok(Some((convention, calendar)))
}

pub(crate) fn roll_date(
    date: &Date,
    roll: Option<(cfdl_calc::RollConvention, cfdl_calc::HolidayCalendar)>,
) -> Date {
    let Some((convention, calendar)) = roll else {
        return date.clone();
    };
    let Some(calc) = cfdl_calc::CalcDate::new(date.year, date.month, date.day) else {
        return date.clone();
    };
    let rolled = calendar.roll(&calc, convention);
    Date {
        year: rolled.year(),
        month: rolled.month(),
        day: rolled.day(),
    }
}

pub(crate) fn timeline_dates(
    start: &str,
    calendar: &str,
    periods: usize,
) -> Result<Vec<Date>, EngineError> {
    let start = Date::parse(start)?;
    let mut out = Vec::with_capacity(periods);
    for idx in 0..periods {
        let date = match calendar {
            "daily" => start.add_days(idx as i32),
            "monthly" => start.add_months(idx as i32),
            "quarterly" => start.add_months((idx as i32) * 3),
            "annual" => start.add_months((idx as i32) * 12),
            _ => start.add_months(idx as i32),
        };
        out.push(date);
    }
    Ok(out)
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Date {
    pub(crate) year: i32,
    pub(crate) month: u32,
    pub(crate) day: u32,
}

impl Date {
    pub fn parse(value: &str) -> Result<Self, EngineError> {
        let parts = value.split('-').collect::<Vec<_>>();
        match parts.as_slice() {
            [y, m, d] => {
                let year = y
                    .parse::<i32>()
                    .map_err(|_| EngineError::InvalidDate(value.to_string()))?;
                let month = m
                    .parse::<u32>()
                    .map_err(|_| EngineError::InvalidDate(value.to_string()))?;
                let day = d
                    .parse::<u32>()
                    .map_err(|_| EngineError::InvalidDate(value.to_string()))?;
                if month == 0 || month > 12 {
                    return Err(EngineError::InvalidDate(value.to_string()));
                }
                if day == 0 || day > days_in_month(year, month) {
                    return Err(EngineError::InvalidDate(value.to_string()));
                }
                Ok(Self { year, month, day })
            }
            _ => Err(EngineError::InvalidDate(value.to_string())),
        }
    }

    /// Days since 1970-01-01, negative before it.
    ///
    /// Hinnant's civil-from-days, the standard branch-free formulation. Used
    /// for date differences; `add_days` still walks day by day, which is fine
    /// for the small offsets payment terms produce.
    pub(crate) fn to_epoch_days(&self) -> i64 {
        let y = if self.month <= 2 {
            self.year - 1
        } else {
            self.year
        } as i64;
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = y - era * 400;
        let m = self.month as i64;
        let d = self.day as i64;
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    pub(crate) fn add_days(&self, days: i32) -> Self {
        if days == 0 {
            return self.clone();
        }
        // Stepping backwards used to be a silent no-op, so any caller asking
        // for a day before a date got the date itself.
        if days < 0 {
            let mut out = self.clone();
            for _ in 0..(-days) {
                if out.day > 1 {
                    out.day -= 1;
                } else {
                    if out.month == 1 {
                        out.month = 12;
                        out.year -= 1;
                    } else {
                        out.month -= 1;
                    }
                    out.day = days_in_month(out.year, out.month);
                }
            }
            return out;
        }
        let mut out = self.clone();
        for _ in 0..days {
            let dim = days_in_month(out.year, out.month);
            if out.day < dim {
                out.day += 1;
            } else {
                out.day = 1;
                if out.month == 12 {
                    out.month = 1;
                    out.year += 1;
                } else {
                    out.month += 1;
                }
            }
        }
        out
    }

    pub(crate) fn add_months(&self, months: i32) -> Self {
        let total_months = self.year * 12 + (self.month as i32 - 1) + months;
        let year = total_months.div_euclid(12);
        let month = total_months.rem_euclid(12) as u32 + 1;
        let day = self.day.min(days_in_month(year, month));
        Self { year, month, day }
    }
}

pub(crate) fn days_in_month(year: i32, month: u32) -> u32 {
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

pub(crate) fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}
