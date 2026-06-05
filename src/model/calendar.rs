//! Proleptic Gregorian calendar arithmetic, implemented in-house.
//!
//! We deliberately do **not** depend on `time`/`chrono`/`jiff` for date math. `zic`'s
//! transition model has its own conventions (days that spill across month boundaries via
//! `Sun>=29`, year ranges back to the far past, `24:00`+ times), and the safest way to
//! match it exactly is to own the arithmetic and test it against reference `zic`.
//!
//! The civil<->days conversion is Howard Hinnant's well-known algorithm
//! (<http://howardhinnant.github.io/date_algorithms.html>): branch-free, valid across an
//! enormous range, with the epoch at 1970-01-01. It is the backbone of turning a
//! `(year, month, day, time)` tuple into a Unix timestamp.
//!
//! These primitives are exercised throughout the compiler: expanding rule activations across
//! their year span, converting each era's `UNTIL` and rule `AT` to a UT instant, the day-form
//! arithmetic behind recurring POSIX footers (`Sun>=N` → `M{m}.{week}.{wday}`), and decoding
//! compiled transition instants back to readable timestamps in `explain`. Each is unit-tested
//! directly and validated end-to-end against reference `zic` via the `zdump` oracle.

use crate::diagnostics::DiagnosticCode;

/// Days of the week, `Sunday = 0` .. `Saturday = 6` (the order `zic` uses).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weekday {
    Sun = 0,
    Mon = 1,
    Tue = 2,
    Wed = 3,
    Thu = 4,
    Fri = 5,
    Sat = 6,
}

impl Weekday {
    pub fn from_index(i: u32) -> Weekday {
        use Weekday::*;
        match i % 7 {
            0 => Sun,
            1 => Mon,
            2 => Tue,
            3 => Wed,
            4 => Thu,
            5 => Fri,
            _ => Sat,
        }
    }
}

/// The `ON` day specification of a `Rule` (or the day part of an `UNTIL`).
///
/// `zic` permits several forms; this captures all of them so the parser can round-trip
/// them and the T2 compiler can resolve them to a concrete day-of-month.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnDay {
    /// A fixed day-of-month, e.g. `5`.
    Day(u8),
    /// `lastSun`, `lastMon`, ... — the last given weekday in the month.
    Last(Weekday),
    /// `Sun>=8` — the first `weekday` on or after `day`.
    OnAfter(Weekday, u8),
    /// `Sun<=25` — the last `weekday` on or before `day`.
    OnBefore(Weekday, u8),
}

/// Is `year` a leap year in the proleptic Gregorian calendar?
pub fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Number of days in `month` (1..=12) of `year`.
pub fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap(year) => 29,
        2 => 28,
        _ => 0, // caller validates the month range; 0 makes misuse obvious.
    }
}

/// Days since the Unix epoch (1970-01-01) for a civil date. Howard Hinnant's algorithm.
///
/// Valid for any `year` in `i32`. `month` is 1..=12, `day` is 1..=31. The function does not
/// validate the day against the month length — callers that built the date legitimately
/// (or want `zic`'s spill behaviour) rely on the raw arithmetic.
pub fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let y = year as i64 - if month <= 2 { 1 } else { 0 };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400; // [0, 399]
    let m = month as i64;
    let d = day as i64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146097 + doe - 719468
}

/// Inverse of [`days_from_civil`]: a day count since 1970-01-01 back to `(year, month, day)`.
/// Howard Hinnant's `civil_from_days`. Used to bound the years an era spans when walking
/// multi-era zones (we know an era's start/end as UT instants and need the calendar years).
pub fn civil_from_days(days: i64) -> (i32, u8, u8) {
    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = z - era * 146097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let year = if m <= 2 { y + 1 } else { y };
    (year as i32, m as u8, d as u8)
}

/// The civil year containing a Unix timestamp (UT).
pub fn year_of_unix(seconds: i64) -> i32 {
    civil_from_days(seconds.div_euclid(86400)).0
}

/// Day of the week for a civil date.
pub fn weekday_of(year: i32, month: u8, day: u8) -> Weekday {
    // 1970-01-01 was a Thursday. Reduce modulo 7, keeping the result non-negative.
    let days = days_from_civil(year, month, day);
    let idx = (days + 4).rem_euclid(7); // +4 shifts epoch-Thursday to Sunday=0 indexing.
    Weekday::from_index(idx as u32)
}

/// Resolve an [`OnDay`] to a concrete day-of-month within `(year, month)`.
///
/// Returns the day as a (possibly out-of-month) value the way `zic` computes it; for the
/// `>=`/`<=` forms `zic` allows the result to land in a neighbouring month (e.g. `Sun>=29`
/// in a 30-day month), which the T2 compiler handles by normalising the date afterwards.
/// Here we return the raw day number and a month delta so callers can normalise.
pub fn resolve_on_day(
    on: OnDay,
    year: i32,
    month: u8,
) -> std::result::Result<ResolvedDay, (DiagnosticCode, String)> {
    let dim = days_in_month(year, month);
    let result = match on {
        OnDay::Day(d) => ResolvedDay {
            day: d as i32,
            month,
            year,
        },
        OnDay::Last(wd) => {
            // Walk back from the last day of the month to the wanted weekday.
            let last = dim;
            let last_wd = weekday_of(year, month, last) as i32;
            let want = wd as i32;
            let back = (last_wd - want).rem_euclid(7);
            ResolvedDay {
                day: last as i32 - back,
                month,
                year,
            }
        }
        OnDay::OnAfter(wd, d) => {
            let start_wd = weekday_at(year, month, d as i32) as i32;
            let want = wd as i32;
            let fwd = (want - start_wd).rem_euclid(7);
            normalise(year, month, d as i32 + fwd)
        }
        OnDay::OnBefore(wd, d) => {
            let start_wd = weekday_at(year, month, d as i32) as i32;
            let want = wd as i32;
            let back = (start_wd - want).rem_euclid(7);
            normalise(year, month, d as i32 - back)
        }
    };
    Ok(result)
}

/// A fully-resolved (and month-normalised) calendar day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedDay {
    pub year: i32,
    pub month: u8,
    pub day: i32,
}

/// Weekday at a day number that may be <1 or >days_in_month, via the day-count algorithm.
fn weekday_at(year: i32, month: u8, day: i32) -> Weekday {
    let days = days_from_civil(year, month, 1) + (day as i64 - 1);
    Weekday::from_index(((days + 4).rem_euclid(7)) as u32)
}

/// Normalise a `(year, month, day)` where `day` may have spilled outside `[1, dim]` into a
/// real calendar date. Handles the `Sun>=29`/`Sun<=2` spill cases `zic` allows.
fn normalise(mut year: i32, mut month: u8, mut day: i32) -> ResolvedDay {
    loop {
        if day < 1 {
            // Borrow from the previous month.
            month = if month == 1 {
                year -= 1;
                12
            } else {
                month - 1
            };
            day += days_in_month(year, month) as i32;
        } else {
            let dim = days_in_month(year, month) as i32;
            if day > dim {
                day -= dim;
                month = if month == 12 {
                    year += 1;
                    1
                } else {
                    month + 1
                };
            } else {
                return ResolvedDay { year, month, day };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leap_years() {
        assert!(is_leap(2000));
        assert!(!is_leap(1900));
        assert!(is_leap(2020));
        assert!(!is_leap(2021));
    }

    #[test]
    fn epoch_is_thursday() {
        assert_eq!(weekday_of(1970, 1, 1), Weekday::Thu);
        // A few independently-known anchors.
        assert_eq!(weekday_of(2000, 1, 1), Weekday::Sat);
        assert_eq!(weekday_of(2020, 3, 8), Weekday::Sun); // US DST start 2020.
    }

    #[test]
    fn days_from_civil_anchors() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(1970, 1, 2), 1);
        assert_eq!(days_from_civil(1969, 12, 31), -1);
        // 2020-03-08 07:00 UTC == 1583643120 s; that is day 18329.
        assert_eq!(days_from_civil(2020, 3, 8), 18329);
    }

    #[test]
    fn last_sunday() {
        // Last Sunday of March 2020 is the 29th.
        let r = resolve_on_day(OnDay::Last(Weekday::Sun), 2020, 3).unwrap();
        assert_eq!((r.month, r.day), (3, 29));
    }

    #[test]
    fn sun_ge_8_and_le_25() {
        // First Sunday on/after Mar 8 2020 is the 8th (it is a Sunday).
        let a = resolve_on_day(OnDay::OnAfter(Weekday::Sun, 8), 2020, 3).unwrap();
        assert_eq!((a.month, a.day), (3, 8));
        // Last Sunday on/before Oct 25 2020 is the 25th (it is a Sunday).
        let b = resolve_on_day(OnDay::OnBefore(Weekday::Sun, 25), 2020, 10).unwrap();
        assert_eq!((b.month, b.day), (10, 25));
    }

    #[test]
    fn spill_into_next_month() {
        // Sun>=29 in April 2021: Apr has 30 days; the first Sunday on/after Apr 29 2021
        // is May 2 (Apr 29 is Thu). Verifies normalisation across the boundary.
        let r = resolve_on_day(OnDay::OnAfter(Weekday::Sun, 29), 2021, 4).unwrap();
        assert_eq!((r.month, r.day), (5, 2));
    }
}
