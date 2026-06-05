//! Case-insensitive, unambiguous-prefix matching for the English names `zic` recognises:
//! month names, weekday names, and a handful of keywords (`minimum`, `maximum`, `only`).
//!
//! `zic`'s rule (see `zic(8)`): names may be given by any unambiguous prefix, case-
//! insensitively. `Ja` is January; `j` is rejected as ambiguous (June/July). We reproduce
//! that exactly, returning a dedicated diagnostic for the ambiguous case so authors get a
//! useful message rather than a generic "unknown".
//!
//! Used lightly in T1 (validation only) and fully by the T2 transition compiler when it
//! parses `Rule` `IN`/`ON` fields and `UNTIL` months.

use crate::diagnostics::DiagnosticCode;
use crate::model::calendar::Weekday;

/// Full month names, index 0 = January.
const MONTHS: [&str; 12] = [
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
];

/// Full weekday names, index 0 = Sunday (matches [`Weekday`] ordering).
const WEEKDAYS: [&str; 7] = [
    "sunday",
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "saturday",
];

/// Match `input` against `table` by unambiguous case-insensitive prefix.
///
/// Returns the matching index, or a diagnostic code describing why it failed: the input
/// matched several entries (ambiguous) or none (unknown).
fn match_prefix(
    input: &str,
    table: &[&str],
    ambiguous: DiagnosticCode,
    unknown: DiagnosticCode,
) -> std::result::Result<usize, (DiagnosticCode, String)> {
    let needle = input.to_ascii_lowercase();
    if needle.is_empty() {
        return Err((unknown, "empty name".to_string()));
    }
    let mut found: Option<usize> = None;
    for (i, name) in table.iter().enumerate() {
        if name.starts_with(&needle) {
            // An exact full-length match is unambiguous even if it is a prefix of nothing
            // longer; but since each table has distinct entries, a full match is unique.
            if *name == needle {
                return Ok(i);
            }
            if found.is_some() {
                return Err((ambiguous, format!("{input:?} is an ambiguous abbreviation")));
            }
            found = Some(i);
        }
    }
    found.ok_or_else(|| (unknown, format!("{input:?} is not a recognised name")))
}

/// Parse a month name (any unambiguous prefix) into 1..=12.
pub fn month(input: &str) -> std::result::Result<u8, (DiagnosticCode, String)> {
    match_prefix(
        input,
        &MONTHS,
        DiagnosticCode::InvalidMonth,
        DiagnosticCode::InvalidMonth,
    )
    .map(|i| (i + 1) as u8)
}

/// Parse a weekday name (any unambiguous prefix) into a [`Weekday`].
pub fn weekday(input: &str) -> std::result::Result<Weekday, (DiagnosticCode, String)> {
    match_prefix(
        input,
        &WEEKDAYS,
        DiagnosticCode::AmbiguousNameAbbreviation,
        DiagnosticCode::InvalidDayRule,
    )
    .map(|i| Weekday::from_index(i as u32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn month_prefixes() {
        assert_eq!(month("January").unwrap(), 1);
        assert_eq!(month("Jan").unwrap(), 1);
        assert_eq!(month("Ja").unwrap(), 1);
        assert_eq!(month("dec").unwrap(), 12);
        // 'j' is ambiguous (June/July) — but 'Ja' is January, 'Jun'/'Jul' disambiguate.
        assert!(month("j").is_err());
        assert!(month("Ju").is_err()); // June vs July
        assert_eq!(month("Jun").unwrap(), 6);
        assert_eq!(month("Jul").unwrap(), 7);
        assert!(month("Smarch").is_err());
    }

    #[test]
    fn weekday_prefixes() {
        assert_eq!(weekday("Sun").unwrap(), Weekday::Sun);
        assert_eq!(weekday("Su").unwrap(), Weekday::Sun);
        assert_eq!(weekday("Saturday").unwrap(), Weekday::Sat);
        assert_eq!(weekday("Th").unwrap(), Weekday::Thu);
        // 'S' is ambiguous (Sunday/Saturday).
        assert!(weekday("S").is_err());
    }
}
