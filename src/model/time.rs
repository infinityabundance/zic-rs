//! Parsing of tzdata time-ish fields: UT offsets (`STDOFF`), saved amounts (`SAVE` and
//! inline rules), and times-of-day (`AT`, and the time part of `UNTIL`).
//!
//! tzdata times are *signed, colon-separated* and can exceed 24 hours or go negative
//! (e.g. `-2:30`, `24:00`, `25:00`). They are stored as a whole number of seconds.
//! Fractional seconds are accepted and rounded to the nearest second (ties away from
//! zero), matching modern `zic`.

use crate::diagnostics::DiagnosticCode;

/// A signed UT offset in seconds (east of UTC positive), as used by `STDOFF`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Offset(pub i32);

/// Which clock an `AT`/`UNTIL` time is measured against. (Consumed by the T2 transition
/// compiler; parsed now so the model is complete.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeRef {
    /// Wall-clock / local time (suffix `w`, or none).
    Wall,
    /// Standard time (suffix `s`).
    Standard,
    /// Universal time (suffix `u`, `g`, or `z`).
    Universal,
}

/// A parsed time-of-day: signed seconds plus the clock it is measured against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeOfDay {
    pub seconds: i32,
    pub reference: TimeRef,
}

impl TimeOfDay {
    /// `-`, i.e. zero wall time.
    pub fn zero() -> Self {
        TimeOfDay {
            seconds: 0,
            reference: TimeRef::Wall,
        }
    }
}

/// A parsed `SAVE` value: the offset added during this rule, plus whether it counts as DST.
///
/// The DST flag follows `zic`: an explicit `s`/`d` suffix wins; otherwise zero is standard
/// and non-zero is daylight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Save {
    pub seconds: i32,
    pub is_dst: bool,
}

/// Parse a colon-separated `[-+]hh[:mm[:ss[.frac]]]` magnitude into seconds.
///
/// Returns `(seconds, rest)` where `rest` is whatever trailed the numeric part (a suffix
/// letter such as `w`/`s`/`u`/`d`), so callers can interpret field-specific suffixes.
fn parse_hms(input: &str) -> std::result::Result<(i32, &str), String> {
    let s = input;
    let (neg, body) = match s.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, s.strip_prefix('+').unwrap_or(s)),
    };

    // Split the numeric prefix from a trailing non-digit/non-colon/non-dot suffix.
    let split = body
        .find(|c: char| !(c.is_ascii_digit() || c == ':' || c == '.'))
        .unwrap_or(body.len());
    let (num, rest) = body.split_at(split);
    if num.is_empty() {
        return Err(format!("missing time value in {input:?}"));
    }

    let mut parts = num.split(':');
    let h: i64 = parse_u(parts.next().unwrap_or(""))?;
    let m: i64 = match parts.next() {
        Some(p) => parse_u(p)?,
        None => 0,
    };
    // The seconds field may carry a fractional part.
    let (sec, frac) = match parts.next() {
        Some(p) => match p.split_once('.') {
            Some((whole, frac)) => (parse_u(whole)?, frac),
            None => (parse_u(p)?, ""),
        },
        None => (0, ""),
    };
    if parts.next().is_some() {
        return Err(format!("too many ':' groups in time {input:?}"));
    }
    if m >= 60 || sec >= 60 {
        // zic is lenient about the hour count (it may exceed 24) but not minutes/seconds.
        return Err(format!("minutes/seconds out of range in {input:?}"));
    }

    // `zic` is lenient about the hour count (it may exceed 24), so `h` can be large enough that
    // `h * 3600` overflows i64 (with `overflow-checks=true` that is a panic, not a wrap). `m`/`sec`
    // are each < 60, so only `h * 3600` is at risk — use checked arithmetic and reject out-of-range
    // as a typed Err rather than panicking. The final `i32::try_from` still bounds the usable range.
    let mut total = h
        .checked_mul(3600)
        .and_then(|hs| hs.checked_add(m * 60 + sec))
        .ok_or_else(|| format!("time {input:?} out of range"))?;
    // Round fractional seconds to nearest, ties away from zero (matches modern zic).
    if !frac.is_empty() {
        if !frac.bytes().all(|b| b.is_ascii_digit()) {
            return Err(format!("invalid fractional seconds in {input:?}"));
        }
        let half = frac.as_bytes()[0] >= b'5';
        if half {
            total = total
                .checked_add(1)
                .ok_or_else(|| format!("time {input:?} out of range"))?;
        }
    }

    let total = if neg { -total } else { total };
    let total = i32::try_from(total).map_err(|_| format!("time {input:?} out of range"))?;
    Ok((total, rest))
}

fn parse_u(s: &str) -> std::result::Result<i64, String> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("invalid number {s:?}"));
    }
    s.parse::<i64>()
        .map_err(|_| format!("number {s:?} out of range"))
}

/// Parse a `STDOFF`/`UNTIL`-offset value (no DST flag, no clock suffix expected).
pub fn parse_offset(input: &str) -> std::result::Result<Offset, (DiagnosticCode, String)> {
    if input == "-" {
        return Ok(Offset(0));
    }
    let (sec, rest) = parse_hms(input).map_err(|m| (DiagnosticCode::InvalidValue, m))?;
    if !rest.is_empty() {
        return Err((
            DiagnosticCode::InvalidValue,
            format!("unexpected suffix {rest:?} in offset {input:?}"),
        ));
    }
    Ok(Offset(sec))
}

/// Parse an `AT` time (or the time part of `UNTIL`), including a `w`/`s`/`u`/`g`/`z` suffix.
pub fn parse_time_of_day(input: &str) -> std::result::Result<TimeOfDay, (DiagnosticCode, String)> {
    if input == "-" {
        return Ok(TimeOfDay::zero());
    }
    let (sec, rest) = parse_hms(input).map_err(|m| (DiagnosticCode::InvalidValue, m))?;
    let reference = match rest {
        "" | "w" => TimeRef::Wall,
        "s" => TimeRef::Standard,
        "u" | "g" | "z" => TimeRef::Universal,
        other => {
            return Err((
                DiagnosticCode::InvalidTimeSuffix,
                format!("invalid time suffix {other:?} in {input:?}"),
            ))
        }
    };
    Ok(TimeOfDay {
        seconds: sec,
        reference,
    })
}

/// Parse a `SAVE` value (or an inline zone `RULES` offset), honouring the `s`/`d` suffix.
pub fn parse_save(input: &str) -> std::result::Result<Save, (DiagnosticCode, String)> {
    if input == "-" {
        return Ok(Save {
            seconds: 0,
            is_dst: false,
        });
    }
    let (sec, rest) = parse_hms(input).map_err(|m| (DiagnosticCode::InvalidValue, m))?;
    let is_dst = match rest {
        // Explicit suffix wins over the zero/non-zero heuristic.
        "s" => false,
        "d" => true,
        "" => sec != 0,
        other => {
            return Err((
                DiagnosticCode::InvalidTimeSuffix,
                format!("invalid save suffix {other:?} in {input:?}"),
            ))
        }
    };
    Ok(Save {
        seconds: sec,
        is_dst,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offsets() {
        assert_eq!(parse_offset("0").unwrap(), Offset(0));
        assert_eq!(parse_offset("-").unwrap(), Offset(0));
        assert_eq!(parse_offset("-5:00").unwrap(), Offset(-18000));
        assert_eq!(parse_offset("5:30:15").unwrap(), Offset(19815));
        assert_eq!(parse_offset("1").unwrap(), Offset(3600));
        assert!(parse_offset("5:00s").is_err());
        assert!(parse_offset("1:99").is_err());
    }

    #[test]
    fn huge_hour_count_is_typed_err_not_overflow_panic() {
        // T23.cargo-fuzz.2 / F2 regression: `zic` is lenient on the hour count, so a value whose
        // `h * 3600` overflows i64 (the F2 seed used an offset like this) must be a typed Err, not
        // a multiply-with-overflow panic (the crate sets `overflow-checks=true` in all profiles).
        assert!(parse_offset("6326386204529102403").is_err());
        assert!(parse_offset("-0000000006326386204529102403").is_err());
        assert!(parse_offset("9223372036854775807").is_err()); // i64::MAX hours
                                                               // A legitimately large-but-representable hour count still parses.
        assert_eq!(parse_offset("25").unwrap(), Offset(90000));
    }

    #[test]
    fn times_with_suffix() {
        assert_eq!(parse_time_of_day("2:00").unwrap().reference, TimeRef::Wall);
        assert_eq!(
            parse_time_of_day("2:00s").unwrap().reference,
            TimeRef::Standard
        );
        assert_eq!(
            parse_time_of_day("2:00u").unwrap().reference,
            TimeRef::Universal
        );
        assert_eq!(parse_time_of_day("24:00").unwrap().seconds, 86400);
        assert_eq!(parse_time_of_day("-2:30").unwrap().seconds, -9000);
        assert!(parse_time_of_day("2:00x").is_err());
    }

    #[test]
    fn save_dst_flag() {
        assert!(!parse_save("0").unwrap().is_dst);
        assert!(parse_save("1:00").unwrap().is_dst);
        assert!(!parse_save("1:00s").unwrap().is_dst);
        assert!(parse_save("0d").unwrap().is_dst);
        assert_eq!(parse_save("-1:00").unwrap().seconds, -3600);
    }

    #[test]
    fn fractional_rounds_to_nearest() {
        assert_eq!(parse_time_of_day("0:00:00.4").unwrap().seconds, 0);
        assert_eq!(parse_time_of_day("0:00:00.5").unwrap().seconds, 1);
    }
}
