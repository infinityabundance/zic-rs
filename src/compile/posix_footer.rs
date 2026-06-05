//! Synthesis of the POSIX `TZ` string that forms a TZif footer (RFC 9636 §3.3, and
//! POSIX.1 "Other") .
//!
//! The footer tells a reader how to behave for instants *after* the last explicit
//! transition. Getting it exactly right matters: in slim files it is the only thing
//! describing the far future. Our policy is **exact-or-nothing** — we synthesise a footer
//! only for shapes we can represent precisely, and otherwise refuse (rather than emit an
//! approximate rule). In T1 the only supported shape is a constant offset with no DST.
//!
//! ## The sign trap
//!
//! A POSIX `TZ` offset is "the value you ADD to local time to get UT" — the opposite sign
//! of the TZif `tt_utoff` ("seconds east of UT"). So a zone at UT−5 (`utoff = -18000`) has
//! POSIX offset `+5`, written `EST5`. UT itself (`utoff = 0`) is `UTC0`. This inversion is
//! a classic source of off-by-a-sign bugs, hence this very explicit helper.

/// Build the POSIX `TZ` string for a constant-offset, no-DST zone (e.g. `EST5`, `UTC0`).
pub fn fixed_offset(abbr: &str, utoff: i32) -> String {
    let mut s = String::new();
    s.push_str(&quote_abbr(abbr));
    s.push_str(&format_posix_offset(utoff));
    s
}

/// Format a TZif `utoff` (seconds east of UT) as a POSIX offset string (sign inverted).
///
/// POSIX form is `[-]hh[:mm[:ss]]` with no zero-padding on the hours and the sign meaning
/// "west is positive". We therefore negate `utoff` first.
pub fn format_posix_offset(utoff: i32) -> String {
    let posix = -(utoff as i64); // invert sign: east-of-UT utoff -> west-positive POSIX
    let neg = posix < 0;
    let abs = posix.unsigned_abs();
    let h = abs / 3600;
    let m = (abs % 3600) / 60;
    let sec = abs % 60;

    let mut out = String::new();
    if neg {
        out.push('-');
    }
    out.push_str(&h.to_string());
    if m != 0 || sec != 0 {
        out.push_str(&format!(":{m:02}"));
        if sec != 0 {
            out.push_str(&format!(":{sec:02}"));
        }
    }
    out
}

/// Quote an abbreviation for inclusion in a POSIX `TZ` string if it is not purely
/// alphabetic. POSIX requires `<...>` around abbreviations containing digits or `+`/`-`
/// (e.g. numeric zone names like `+05`). Plain alphabetic names are emitted bare.
fn quote_abbr(abbr: &str) -> String {
    if !abbr.is_empty() && abbr.bytes().all(|b| b.is_ascii_alphabetic()) {
        abbr.to_string()
    } else {
        format!("<{abbr}>")
    }
}

/// The two recurring rules and offsets needed to describe an annually-repeating DST regime
/// as a POSIX `TZ` string. All `*_wall` times are already converted to **local wall** time
/// (the caller does the wall/standard/universal conversion, since it knows the prevailing
/// offsets — see `compile::transitions`).
#[derive(Debug, Clone, Copy)]
pub struct Recurring<'a> {
    pub std_abbr: &'a str,
    pub std_utoff: i32,
    pub dst_abbr: &'a str,
    pub dst_utoff: i32,
    /// When DST begins.
    pub dst_month: u8,
    pub dst_on: crate::model::calendar::OnDay,
    pub dst_at_wall: i32,
    /// When DST ends (standard resumes).
    pub std_month: u8,
    pub std_on: crate::model::calendar::OnDay,
    pub std_at_wall: i32,
}

/// Synthesise the POSIX `TZ` string for an annually-recurring DST regime, or `None` if it
/// cannot be represented *exactly* (POSIX only expresses "nth weekday" / "last weekday" of a
/// month, so e.g. `Sun<=N` and fixed-day rules are refused — the caller then fails closed
/// rather than emit an approximate footer).
///
/// Form: `std[offset]dst[offset],start[/time],end[/time]` where, matching `zic`:
/// * the DST offset is **omitted** when it is the default one hour ahead of standard;
/// * each `/time` is **omitted** when it is the POSIX default `02:00:00`.
///
/// Returns `(tz_string, min_version)` — the TZif version byte the footer requires (`b'2'`, or
/// `b'3'` per `zic.c`'s `compat >= 2013` rule: a non-zero re-anchor day-shift or a negative folded
/// transition time; see the version note in the body and `date_rule`). `None` if the day form is
/// not expressible at all.
pub fn recurring(r: &Recurring<'_>) -> Option<(String, u8)> {
    let mut s = String::new();
    s.push_str(&quote_abbr(r.std_abbr));
    s.push_str(&format_posix_offset(r.std_utoff));
    s.push_str(&quote_abbr(r.dst_abbr));
    // Emit the DST offset only when it differs from the POSIX default (std minus one hour,
    // i.e. one hour *ahead* of standard → utoff = std_utoff + 3600).
    if r.dst_utoff - r.std_utoff != 3600 {
        s.push_str(&format_posix_offset(r.dst_utoff));
    }
    // Each `ON` form yields an `Mm.w.d` rule plus a **day-shift** folded into the transition time
    // (non-zero only when `zic` re-anchors a `>=N`/`<=N` form onto a clean nth-weekday — that
    // shift can push the time past 24h).
    let (dst_rule, dst_shift) = date_rule(r.dst_month, r.dst_on)?;
    let (std_rule, std_shift) = date_rule(r.std_month, r.std_on)?;
    let dst_tod = r.dst_at_wall + dst_shift;
    let std_tod = r.std_at_wall + std_shift;
    s.push(',');
    s.push_str(&dst_rule);
    s.push_str(&time_suffix(dst_tod));
    s.push(',');
    s.push_str(&std_rule);
    s.push_str(&time_suffix(std_tod));
    // Content-driven TZif version, pinned to `zic.c`'s `stringrule`/`stringzone` (tzcode 2026b):
    // `version = compat < 2013 ? '2' : '3'`, and `compat` reaches **2013** (⇒ v3) exactly when a
    // `weekday>=N`/`weekday<=N` `ON` form is re-anchored onto a clean nth-weekday with a **non-zero
    // day-shift** (`wdayoff != 0`, our `*_shift != 0`), OR the folded transition time is **negative**.
    // A folded time `>= 24h` *alone* is only `compat = 1994` — still **v2** (Africa/Cairo's literal
    // `lastThu 24:00` → `/24`, v2). So the day-shift, not the displayed time's range, is the trigger:
    // America/Santiago & Pacific/Easter (`Sun>=2`, `wdayoff = 1`) are v3 even though their `/24`/`/22`
    // land in range; Asia/Gaza (`Sat<=30`, `wdayoff = 2`) is v3 via the shift (its `/50` is incidental);
    // America/Nuuk (`/-1`) is v3 via the negative time. See docs/structural-parity.md.
    let needs_v3 = dst_shift != 0 || std_shift != 0 || dst_tod < 0 || std_tod < 0;
    let version = if needs_v3 { b'3' } else { b'2' };
    Some((s, version))
}

/// A POSIX `Mm.w.d` date rule from a month and an `ON` day spec, plus a **day-shift** (seconds) to
/// fold into the transition time — or `None` if the form is not expressible as an nth/last weekday.
/// `w` is the week (1..4, or 5 = "last"); `d` is `0=Sunday`.
///
/// `zic` re-anchors a `weekday >= N` / `weekday <= N` form whose day is not a clean week boundary
/// onto an *earlier* weekday and pushes the skipped days into the transition time (which can exceed
/// 24h — the v3 case). For `weekday W <= N` (not the last day of the month): `wdayoff = N mod 7`,
/// `d = W − wdayoff` (mod 7), `week = N / 7`, and the time gains `wdayoff` days. Example: Asia/Gaza's
/// `Sat<=30 @02:00` → `wdayoff = 2`, `d = Thu(4)`, `week = 4`, `+2 days` → `M3.4.4/50`. The `>=N`
/// form is symmetric (`wdayoff = (N−1) mod 7`, `week = 1 + (N−1)/7`); for `wdayoff = 0` it reduces to
/// the plain `M{month}.{(N−1)/7+1}.{W}` we already emit for US/EU rules.
fn date_rule(month: u8, on: crate::model::calendar::OnDay) -> Option<(String, i32)> {
    use crate::model::calendar::OnDay;
    // Leap-year month lengths (matches `zic`'s `len_months[1]`), for the `<=N` "last day" check.
    const LEN: [u8; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let (week, weekday, shift_days): (i32, i32, i32) = match on {
        // `lastSun` etc. → week 5 ("last"), no shift.
        OnDay::Last(wd) => (5, wd as i32, 0),
        // `weekday >= N`.
        OnDay::OnAfter(wd, n) => {
            let wdayoff = (n as i32 - 1).rem_euclid(7);
            (1 + (n as i32 - 1) / 7, wd as i32 - wdayoff, wdayoff)
        }
        // `weekday <= N`. `N == last day of month` is week 5 with no shift.
        OnDay::OnBefore(wd, n) => {
            if (1..=12).contains(&month) && n == LEN[(month - 1) as usize] {
                (5, wd as i32, 0)
            } else {
                let wdayoff = (n as i32).rem_euclid(7);
                (n as i32 / 7, wd as i32 - wdayoff, wdayoff)
            }
        }
        // A fixed numeric day has no nth/last-weekday form (Julian `Jn`/`n` forms are not emitted).
        OnDay::Day(_) => return None,
    };
    // POSIX weeks are 1..5; a `<=N` with N < 7 would give week 0 — refuse rather than emit garbage.
    if !(1..=5).contains(&week) {
        return None;
    }
    Some((
        format!("M{month}.{week}.{}", weekday.rem_euclid(7)),
        shift_days * 86400,
    ))
}

/// The `/time` suffix for a transition, omitted when it is the POSIX default of `02:00:00`.
fn time_suffix(wall_seconds: i32) -> String {
    const POSIX_DEFAULT: i32 = 2 * 3600;
    if wall_seconds == POSIX_DEFAULT {
        return String::new();
    }
    format!("/{}", format_posix_time(wall_seconds))
}

/// Format a transition time-of-day as POSIX `h[:mm[:ss]]` (sign-preserving).
fn format_posix_time(seconds: i32) -> String {
    let neg = seconds < 0;
    let abs = seconds.unsigned_abs();
    let h = abs / 3600;
    let m = (abs % 3600) / 60;
    let s = abs % 60;
    let mut out = String::new();
    if neg {
        out.push('-');
    }
    out.push_str(&h.to_string());
    if m != 0 || s != 0 {
        out.push_str(&format!(":{m:02}"));
        if s != 0 {
            out.push_str(&format!(":{s:02}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc_and_est() {
        assert_eq!(fixed_offset("UTC", 0), "UTC0");
        assert_eq!(fixed_offset("EST", -18000), "EST5");
    }

    #[test]
    fn east_offset_is_negative() {
        // Tokyo-ish: UT+9 -> POSIX "-9".
        assert_eq!(format_posix_offset(9 * 3600), "-9");
    }

    #[test]
    fn fractional_hours() {
        // UT+5:30 (utoff +19800) -> POSIX "-5:30".
        assert_eq!(format_posix_offset(19800), "-5:30");
        // UT-3:30 (utoff -12600) -> POSIX "3:30".
        assert_eq!(format_posix_offset(-12600), "3:30");
    }

    #[test]
    fn numeric_abbr_is_quoted() {
        assert_eq!(fixed_offset("+05", 18000), "<+05>-5");
    }

    #[test]
    fn recurring_us_eastern() {
        use crate::model::calendar::{OnDay, Weekday};
        let r = Recurring {
            std_abbr: "EST",
            std_utoff: -18000,
            dst_abbr: "EDT",
            dst_utoff: -14400,
            dst_month: 3,
            dst_on: OnDay::OnAfter(Weekday::Sun, 8), // second Sunday in March
            dst_at_wall: 2 * 3600,                   // 02:00 → default, omitted
            std_month: 11,
            std_on: OnDay::OnAfter(Weekday::Sun, 1), // first Sunday in November
            std_at_wall: 2 * 3600,
        };
        assert_eq!(
            recurring(&r).unwrap(),
            ("EST5EDT,M3.2.0,M11.1.0".to_string(), b'2')
        );
    }

    #[test]
    fn recurring_london_emits_nondefault_times_and_lastsun() {
        use crate::model::calendar::{OnDay, Weekday};
        // GMT/BST: lastSun, 01:00 wall start, 02:00 default end → "GMT0BST,M3.5.0/1,M10.5.0".
        let r = Recurring {
            std_abbr: "GMT",
            std_utoff: 0,
            dst_abbr: "BST",
            dst_utoff: 3600,
            dst_month: 3,
            dst_on: OnDay::Last(Weekday::Sun),
            dst_at_wall: 3600, // 01:00 → /1
            std_month: 10,
            std_on: OnDay::Last(Weekday::Sun),
            std_at_wall: 2 * 3600, // 02:00 → omitted
        };
        assert_eq!(
            recurring(&r).unwrap(),
            ("GMT0BST,M3.5.0/1,M10.5.0".to_string(), b'2')
        );
    }

    #[test]
    fn recurring_sat_leq_30_uses_v3_extended_time_footer() {
        use crate::model::calendar::{OnDay, Weekday};
        // Asia/Gaza's perpetual Palestine rule: `Sat<=30 02:00` in March and October. `zic`
        // re-anchors it onto the 4th Thursday + 50h (Thu + 2 days + 2h = the Saturday ≤ 30), which
        // needs the v3 footer extension. Footer: `EET-2EEST,M3.4.4/50,M10.4.4/50`, version `3`.
        let r = Recurring {
            std_abbr: "EET",
            std_utoff: 7200,
            dst_abbr: "EEST",
            dst_utoff: 10800,
            dst_month: 3,
            dst_on: OnDay::OnBefore(Weekday::Sat, 30),
            dst_at_wall: 2 * 3600,
            std_month: 10,
            std_on: OnDay::OnBefore(Weekday::Sat, 30),
            std_at_wall: 2 * 3600,
        };
        assert_eq!(
            recurring(&r).unwrap(),
            ("EET-2EEST,M3.4.4/50,M10.4.4/50".to_string(), b'3')
        );
    }

    #[test]
    fn recurring_dayshift_forces_v3_even_when_time_in_range() {
        use crate::model::calendar::{OnDay, Weekday};
        // The America/Santiago / Pacific/Easter case: `Sun>=2` re-anchors to Saturday of week 1
        // with a non-zero day-shift (`wdayoff = (2-1)%7 = 1`). Per `zic.c` `stringrule`, a non-zero
        // day-shift sets `compat = 2013` ⇒ TZif **v3**, *regardless* of where the folded transition
        // time lands. Here it folds back to 23:00 (in `0..24h`) — the old range-only heuristic would
        // have wrongly said v2; the day-shift is the real trigger.
        let r = Recurring {
            std_abbr: "X5",
            std_utoff: -18000,
            dst_abbr: "X4",
            dst_utoff: -14400,
            dst_month: 9,
            dst_on: OnDay::OnAfter(Weekday::Sun, 2),
            dst_at_wall: -3600, // -1h + 1-day shift = 23:00, in range
            std_month: 4,
            std_on: OnDay::OnAfter(Weekday::Sun, 2),
            std_at_wall: -3600,
        };
        let (footer, version) = recurring(&r).unwrap();
        assert_eq!(version, b'3', "day-shift must force v3: {footer}");
        assert!(
            footer.contains("M9.1.6/23"),
            "in-range folded time: {footer}"
        );
    }

    #[test]
    fn recurring_last_weekday_24h_stays_v2() {
        use crate::model::calendar::{OnDay, Weekday};
        // The Africa/Cairo case: a clean *last*-weekday form (no day-shift) with a literal `24:00`
        // AT. Per `zic.c`, a folded time `>= 24h` *alone* is only `compat = 1994` — still **v2**.
        // The day-shift is zero, so the 24h time does NOT bump the version.
        let r = Recurring {
            std_abbr: "EET",
            std_utoff: 7200,
            dst_abbr: "EEST",
            dst_utoff: 10800,
            dst_month: 4,
            dst_on: OnDay::Last(Weekday::Fri),
            dst_at_wall: 0,
            std_month: 10,
            std_on: OnDay::Last(Weekday::Thu),
            std_at_wall: 24 * 3600, // last Thursday 24:00 → compat 1994, not 2013
        };
        let (footer, version) = recurring(&r).unwrap();
        assert_eq!(version, b'2', "24h with no day-shift stays v2: {footer}");
        assert!(footer.contains("/24"), "{footer}");
    }

    #[test]
    fn recurring_refuses_fixed_numeric_day() {
        use crate::model::calendar::{OnDay, Weekday};
        // A fixed numeric `ON` day (no weekday) has no `Mm.w.d` form → fail closed (exact-or-fail).
        let r = Recurring {
            std_abbr: "EST",
            std_utoff: -18000,
            dst_abbr: "EDT",
            dst_utoff: -14400,
            dst_month: 3,
            dst_on: OnDay::OnAfter(Weekday::Sun, 8),
            dst_at_wall: 2 * 3600,
            std_month: 10,
            std_on: OnDay::Day(25), // fixed day-of-month, not a weekday rule
            std_at_wall: 2 * 3600,
        };
        assert!(recurring(&r).is_none());
    }

    #[test]
    fn recurring_emits_explicit_dst_offset_when_nondefault() {
        use crate::model::calendar::{OnDay, Weekday};
        // A 30-minute DST saving is not the POSIX default, so the DST offset is emitted.
        // std UT+10:30 (utoff 37800) -> "<+1030>-10:30"; dst +11:00 (utoff 39600) -> "-11".
        let r = Recurring {
            std_abbr: "+1030",
            std_utoff: 37800,
            dst_abbr: "+11",
            dst_utoff: 39600,
            dst_month: 10,
            dst_on: OnDay::OnAfter(Weekday::Sun, 1),
            dst_at_wall: 2 * 3600,
            std_month: 4,
            std_on: OnDay::OnAfter(Weekday::Sun, 1),
            std_at_wall: 3 * 3600,
        };
        assert_eq!(
            recurring(&r).unwrap(),
            ("<+1030>-10:30<+11>-11,M10.1.0,M4.1.0/3".to_string(), b'2')
        );
    }
}
