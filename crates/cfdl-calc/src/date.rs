use std::fmt;

/// Proleptic Gregorian calendar date. No timezone, no time-of-day — underwriting
/// schedules are date-resolution by construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CalcDate {
    year: i32,
    month: u32, // 1..=12
    day: u32,   // 1..=31
}

impl fmt::Display for CalcDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

pub fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

impl CalcDate {
    pub fn new(year: i32, month: u32, day: u32) -> Option<Self> {
        if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
            return None;
        }
        Some(Self { year, month, day })
    }

    pub fn year(&self) -> i32 {
        self.year
    }

    pub fn month(&self) -> u32 {
        self.month
    }

    pub fn day(&self) -> u32 {
        self.day
    }

    /// Days since the civil epoch 1970-01-01 (Howard Hinnant's algorithm).
    pub fn to_epoch_days(&self) -> i64 {
        let y = if self.month <= 2 {
            self.year - 1
        } else {
            self.year
        } as i64;
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = y - era * 400; // [0, 399]
        let mp = (self.month as i64 + 9) % 12; // [0, 11], Mar=0
        let doy = (153 * mp + 2) / 5 + self.day as i64 - 1; // [0, 365]
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
        era * 146097 + doe - 719468
    }

    pub fn from_epoch_days(days: i64) -> Self {
        let z = days + 719468;
        let era = if z >= 0 { z } else { z - 146096 } / 146097;
        let doe = z - era * 146097; // [0, 146096]
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
        let mp = (5 * doy + 2) / 153; // [0, 11]
        let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
        let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
        let year = (if m <= 2 { y + 1 } else { y }) as i32;
        Self {
            year,
            month: m,
            day: d,
        }
    }

    pub fn add_days(&self, days: i64) -> Self {
        Self::from_epoch_days(self.to_epoch_days() + days)
    }

    /// Excel `edate` semantics: shift by whole months, clamping to month end.
    pub fn add_months(&self, months: i32) -> Self {
        let total = self.year as i64 * 12 + (self.month as i64 - 1) + months as i64;
        let year = total.div_euclid(12) as i32;
        let month = (total.rem_euclid(12) + 1) as u32;
        let day = self.day.min(days_in_month(year, month));
        Self { year, month, day }
    }

    /// Excel `eomonth` semantics: end of the month `months` away.
    pub fn end_of_month(&self, months: i32) -> Self {
        let shifted = self.add_months(months);
        Self {
            year: shifted.year,
            month: shifted.month,
            day: days_in_month(shifted.year, shifted.month),
        }
    }

    pub fn days_between(&self, later: &CalcDate) -> i64 {
        later.to_epoch_days() - self.to_epoch_days()
    }
}

/// Day-count conventions per ISDA/SIFMA definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DayCount {
    /// 30/360 US (Bond Basis) with the standard end-of-month adjustments.
    Thirty360US,
    Act360,
    Act365Fixed,
}

impl DayCount {
    pub fn parse(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "30/360" | "30/360 us" | "bond" => Some(Self::Thirty360US),
            "act/360" | "actual/360" => Some(Self::Act360),
            "act/365" | "act/365f" | "actual/365" => Some(Self::Act365Fixed),
            _ => None,
        }
    }

    /// Year fraction from `start` to `end` (as f64 numerator/denominator math is
    /// exact here: day counts are integers, denominators are 360/365).
    pub fn year_frac(&self, start: &CalcDate, end: &CalcDate) -> (i64, i64) {
        match self {
            DayCount::Thirty360US => {
                let mut d1 = start.day as i64;
                let mut d2 = end.day as i64;
                // US convention adjustments.
                if d1 == 31 {
                    d1 = 30;
                }
                if d2 == 31 && d1 == 30 {
                    d2 = 30;
                }
                let days = (end.year as i64 - start.year as i64) * 360
                    + (end.month as i64 - start.month as i64) * 30
                    + (d2 - d1);
                (days, 360)
            }
            DayCount::Act360 => (start.days_between(end), 360),
            DayCount::Act365Fixed => (start.days_between(end), 365),
        }
    }
}
