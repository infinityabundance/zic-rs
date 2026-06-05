//! Rendering a zone's `FORMAT` field into a concrete abbreviation.
//!
//! `zic` supports three `FORMAT` shapes (all resolved with the rule context of a given
//! moment):
//!
//! * `%s` — substitute the active rule's `LETTER` (e.g. `E%sT` → `EST` when LETTER is `S`,
//!   `EDT` when it is `D`; an empty LETTER yields `ET`);
//! * `STD/DST` — a slash-separated pair; pick the left half in standard time, the right in
//!   daylight time;
//! * `%z` — the numeric UT offset as `±hh[mm[ss]]`, shortest non-lossy form;
//! * otherwise the `FORMAT` is a literal abbreviation, used verbatim.

/// Render `format` for a moment whose active rule LETTER is `letter`, daylight flag
/// `is_dst`, and total UT offset `utoff` (seconds east).
pub fn render(format: &str, letter: &str, is_dst: bool, utoff: i32) -> String {
    if let Some((std, dst)) = format.split_once('/') {
        // STD/DST slash form: choose by the daylight flag.
        return if is_dst { dst } else { std }.to_string();
    }
    if format.contains("%s") {
        return format.replace("%s", letter);
    }
    if format.contains("%z") {
        return format.replace("%z", &numeric_offset(utoff));
    }
    format.to_string()
}

/// `%z` expansion: `±hh`, `±hhmm`, or `±hhmmss` — the shortest form that loses no precision.
fn numeric_offset(utoff: i32) -> String {
    let sign = if utoff < 0 { '-' } else { '+' };
    let abs = (utoff as i64).unsigned_abs();
    let h = abs / 3600;
    let m = (abs % 3600) / 60;
    let s = abs % 60;
    let mut out = format!("{sign}{h:02}");
    if m != 0 || s != 0 {
        out.push_str(&format!("{m:02}"));
        if s != 0 {
            out.push_str(&format!("{s:02}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_s() {
        assert_eq!(render("E%sT", "S", false, -18000), "EST");
        assert_eq!(render("E%sT", "D", true, -14400), "EDT");
        assert_eq!(render("E%sT", "", false, -18000), "ET");
    }

    #[test]
    fn slash_form() {
        assert_eq!(render("CET/CEST", "", false, 3600), "CET");
        assert_eq!(render("CET/CEST", "", true, 7200), "CEST");
    }

    #[test]
    fn literal() {
        assert_eq!(render("UTC", "", false, 0), "UTC");
    }

    #[test]
    fn percent_z() {
        assert_eq!(render("%z", "", false, -18000), "-05");
        assert_eq!(render("%z", "", true, 19800), "+0530");
        assert_eq!(render("%z", "", false, 0), "+00");
    }
}
