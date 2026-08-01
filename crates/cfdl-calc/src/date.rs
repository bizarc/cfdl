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

/// Day-count conventions, per the standard market definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DayCount {
    /// 30/360 US (Bond Basis) with the standard end-of-month adjustments.
    Thirty360US,
    /// 30E/360 (Eurobond Basis): both day-of-month values capped at 30.
    Thirty360E,
    Act360,
    Act365Fixed,
}

impl DayCount {
    pub fn parse(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "30/360" | "30/360 us" | "bond" => Some(Self::Thirty360US),
            "30e/360" | "eurobond" => Some(Self::Thirty360E),
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
            DayCount::Thirty360E => {
                let d1 = (start.day as i64).min(30);
                let d2 = (end.day as i64).min(30);
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

// ---------- business-day calendars & rolls ----------

/// Business-day roll conventions (ISDA definitions).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RollConvention {
    None,
    Following,
    ModifiedFollowing,
    Preceding,
    ModifiedPreceding,
}

impl RollConvention {
    pub fn parse(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "none" => Some(Self::None),
            "following" => Some(Self::Following),
            "modified_following" => Some(Self::ModifiedFollowing),
            "preceding" => Some(Self::Preceding),
            "modified_preceding" => Some(Self::ModifiedPreceding),
            _ => None,
        }
    }
}

/// Holiday calendars. Holidays are computed algorithmically (no data files),
/// which keeps the library wasm-clean and deterministic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HolidayCalendar {
    /// Saturdays and Sundays only.
    Weekend,
    /// US federal holidays (with Saturday->Friday / Sunday->Monday observance).
    UsFederal,
    /// TARGET2 (eurozone settlement) closing days.
    Target,
    /// England & Wales bank holidays (statutory rules; one-off proclamations
    /// such as royal events are NOT modeled).
    UkBank,
}

impl HolidayCalendar {
    pub fn parse(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "weekend" | "none" => Some(Self::Weekend),
            "us" | "us_federal" | "sifma" => Some(Self::UsFederal),
            "target" | "target2" | "eur" => Some(Self::Target),
            "uk" | "uk_bank" | "london" => Some(Self::UkBank),
            _ => None,
        }
    }

    pub fn is_business_day(&self, date: &CalcDate) -> bool {
        if date.weekday() >= 5 {
            return false;
        }
        match self {
            HolidayCalendar::Weekend => true,
            HolidayCalendar::UsFederal => !is_us_federal_holiday(date),
            HolidayCalendar::Target => !is_target_holiday(date),
            HolidayCalendar::UkBank => !is_uk_bank_holiday(date),
        }
    }

    pub fn roll(&self, date: &CalcDate, convention: RollConvention) -> CalcDate {
        if convention == RollConvention::None || self.is_business_day(date) {
            return *date;
        }
        match convention {
            RollConvention::None => *date,
            RollConvention::Following => self.next_business_day(date, 1),
            RollConvention::Preceding => self.next_business_day(date, -1),
            RollConvention::ModifiedFollowing => {
                let rolled = self.next_business_day(date, 1);
                if rolled.month() != date.month() {
                    self.next_business_day(date, -1)
                } else {
                    rolled
                }
            }
            RollConvention::ModifiedPreceding => {
                let rolled = self.next_business_day(date, -1);
                if rolled.month() != date.month() {
                    self.next_business_day(date, 1)
                } else {
                    rolled
                }
            }
        }
    }

    fn next_business_day(&self, date: &CalcDate, step: i64) -> CalcDate {
        let mut d = date.add_days(step);
        while !self.is_business_day(&d) {
            d = d.add_days(step);
        }
        d
    }

    pub fn add_business_days(&self, date: &CalcDate, n: i64) -> CalcDate {
        if n == 0 {
            // Convention: 0 business days rolls forward to a business day.
            return self.roll(date, RollConvention::Following);
        }
        let step = if n > 0 { 1 } else { -1 };
        let mut remaining = n.abs();
        let mut d = *date;
        while remaining > 0 {
            d = self.next_business_day(&d, step);
            remaining -= 1;
        }
        d
    }
}

impl CalcDate {
    /// Day of week: 0 = Monday .. 6 = Sunday (ISO).
    pub fn weekday(&self) -> u32 {
        // 1970-01-01 was a Thursday (ISO index 3).
        (self.to_epoch_days() + 3).rem_euclid(7) as u32
    }
}

/// Gregorian Easter Sunday (Anonymous/Meeus algorithm).
fn easter_sunday(year: i32) -> CalcDate {
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = ((h + l - 7 * m + 114) / 31) as u32;
    let day = (((h + l - 7 * m + 114) % 31) + 1) as u32;
    CalcDate::new(year, month, day).expect("Easter computation yields a valid date")
}

/// nth (1-based) weekday-of-month, e.g. 3rd Monday of January.
fn nth_weekday(year: i32, month: u32, weekday: u32, nth: u32) -> CalcDate {
    let first = CalcDate::new(year, month, 1).expect("first of month");
    let offset = (weekday + 7 - first.weekday()) % 7;
    first.add_days(offset as i64 + 7 * (nth as i64 - 1))
}

/// Last given weekday of a month.
fn last_weekday(year: i32, month: u32, weekday: u32) -> CalcDate {
    let last = CalcDate::new(year, month, days_in_month(year, month)).expect("last of month");
    last.add_days(-(((last.weekday() + 7 - weekday) % 7) as i64))
}

/// US federal holiday check, with observance shifts (Sat -> Fri, Sun -> Mon).
fn is_us_federal_holiday(date: &CalcDate) -> bool {
    let y = date.year();
    let observed = |m: u32, d: u32| -> [Option<CalcDate>; 2] {
        let actual = CalcDate::new(y, m, d).expect("fixed holiday");
        match actual.weekday() {
            5 => [Some(actual.add_days(-1)), None], // Sat observed Fri
            6 => [Some(actual.add_days(1)), None],  // Sun observed Mon
            _ => [Some(actual), None],
        }
    };
    let mut holidays: Vec<CalcDate> = Vec::with_capacity(12);
    for pair in [
        observed(1, 1),   // New Year's Day
        observed(6, 19),  // Juneteenth
        observed(7, 4),   // Independence Day
        observed(11, 11), // Veterans Day
        observed(12, 25), // Christmas
    ] {
        holidays.extend(pair.into_iter().flatten());
    }
    // New Year's Day of NEXT year observed on Dec 31 when Jan 1 is a Saturday.
    if let Some(next_ny) = CalcDate::new(y + 1, 1, 1) {
        if next_ny.weekday() == 5 {
            holidays.push(next_ny.add_days(-1));
        }
    }
    holidays.push(nth_weekday(y, 1, 0, 3)); // MLK: 3rd Monday of January
    holidays.push(nth_weekday(y, 2, 0, 3)); // Presidents Day: 3rd Monday of February
    holidays.push(last_weekday(y, 5, 0)); // Memorial Day: last Monday of May
    holidays.push(nth_weekday(y, 9, 0, 1)); // Labor Day: 1st Monday of September
    holidays.push(nth_weekday(y, 10, 0, 2)); // Columbus Day: 2nd Monday of October
    holidays.push(nth_weekday(y, 11, 3, 4)); // Thanksgiving: 4th Thursday of November
    holidays.contains(date)
}

/// TARGET2 closing days: New Year, Good Friday, Easter Monday, Labour Day,
/// Christmas, Boxing Day.
fn is_target_holiday(date: &CalcDate) -> bool {
    let y = date.year();
    let easter = easter_sunday(y);
    *date == CalcDate::new(y, 1, 1).expect("jan 1")
        || *date == CalcDate::new(y, 5, 1).expect("may 1")
        || *date == CalcDate::new(y, 12, 25).expect("dec 25")
        || *date == CalcDate::new(y, 12, 26).expect("dec 26")
        || *date == easter.add_days(-2) // Good Friday
        || *date == easter.add_days(1) // Easter Monday
}

/// England & Wales bank holidays (statutory): New Year (observed), Good
/// Friday, Easter Monday, early May / spring / summer bank holidays,
/// Christmas and Boxing Day with substitute days.
fn is_uk_bank_holiday(date: &CalcDate) -> bool {
    let y = date.year();
    let easter = easter_sunday(y);
    let mut holidays = vec![
        easter.add_days(-2),     // Good Friday
        easter.add_days(1),      // Easter Monday
        nth_weekday(y, 5, 0, 1), // Early May bank holiday
        last_weekday(y, 5, 0),   // Spring bank holiday
        last_weekday(y, 8, 0),   // Summer bank holiday
    ];
    // New Year's Day: substitute Monday if on a weekend.
    let ny = CalcDate::new(y, 1, 1).expect("jan 1");
    holidays.push(match ny.weekday() {
        5 => ny.add_days(2),
        6 => ny.add_days(1),
        _ => ny,
    });
    // Christmas + Boxing Day with substitute days.
    let christmas = CalcDate::new(y, 12, 25).expect("dec 25");
    let boxing = CalcDate::new(y, 12, 26).expect("dec 26");
    match christmas.weekday() {
        5 => {
            // Sat/Sun -> substitutes Mon 27 + Tue 28
            holidays.push(christmas.add_days(2));
            holidays.push(boxing.add_days(2));
        }
        6 => {
            holidays.push(christmas.add_days(1));
            holidays.push(boxing.add_days(1));
        }
        4 => {
            // Christmas Friday -> Boxing Day Saturday observed Monday
            holidays.push(christmas);
            holidays.push(boxing.add_days(2));
        }
        _ => {
            holidays.push(christmas);
            holidays.push(boxing);
        }
    }
    holidays.contains(date)
}
