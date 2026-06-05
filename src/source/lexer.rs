//! The lexer: bytes → [`Line`]s of [`Field`]s.
//!
//! This implements `zic`'s field-splitting rules faithfully (see `zic(8)` "FILES"):
//!
//! * Fields are separated by runs of whitespace (space, tab, form-feed, CR, VT).
//! * A `#` that is **not inside quotes** begins a comment running to end of line.
//! * A double-quoted run is one field; inside it, whitespace and `#` are literal. There is
//!   no backslash escaping in tzdata, and a `"` only delimits — there is no way to embed a
//!   literal `"` in a field, which matches `zic`.
//! * Blank lines (after comment removal) are dropped.
//!
//! Security / admissibility hardening (untrusted input), matching reference `zic`'s `inputline`/
//! `getfields`: we reject embedded NUL bytes, enforce the 2048-byte maximum line length (the pinned
//! `_POSIX2_LINE_MAX`, including the newline), require every line to end with a newline (a final
//! unterminated line is fatal, like reference — T14.2), and reject an unclosed double-quote (odd
//! number of quotation marks). A hostile or corrupt source cannot push us into pathological territory,
//! and zic-rs is never *more* lenient than reference `zic` at the lexical layer.

use std::path::Path;

use crate::diagnostics::{Diagnostic, DiagnosticCode};
use crate::error::{Error, Result};

use super::records::{Field, Line};

/// Maximum source line length in bytes, including the trailing newline — matches `zic`.
pub const MAX_LINE_LEN: usize = 2048;

/// Tokenise an entire source file into logical lines.
///
/// Returns a hard [`Error`] on the first structural problem (these are not recoverable in a
/// way that keeps later line/column numbers trustworthy): invalid UTF-8, a NUL byte, an
/// over-length line, a final line not terminated by a newline, or an unterminated quoted field.
/// Tokenise UTF-8 tzdata source — the modern source contract. Thin wrapper over [`tokenize_with`].
pub fn tokenize(bytes: &[u8], file: &Path) -> Result<Vec<Line>> {
    tokenize_with(bytes, file, false)
}

/// Refuse a non-UTF-8 byte that lies *outside* a `#` comment — i.e. in a semantics-bearing field.
/// (LEGACY-SOURCE.1 guardrail: legacy Latin-1 replay admits non-UTF-8 only in comments.)
fn reject_non_utf8_outside_comments(bytes: &[u8], file: &Path) -> Result<()> {
    for (idx, line) in bytes.split(|&b| b == b'\n').enumerate() {
        let line_no = idx + 1;
        if let Err(e) = std::str::from_utf8(line) {
            // index of the first invalid byte within this line, vs the first `#` (comment start)
            let bad = e.valid_up_to();
            let comment_at = line.iter().position(|&b| b == b'#');
            let in_comment = comment_at.is_some_and(|h| bad >= h);
            if !in_comment {
                return Err(Error::from(Diagnostic::error(
                    DiagnosticCode::InvalidValue,
                    "non-UTF-8 byte outside a comment: legacy Latin-1 replay admits non-UTF-8 only in `#` comments, never in a semantics-bearing field",
                    file,
                    line_no,
                )));
            }
        }
    }
    Ok(())
}

/// Tokenise tzdata source. `legacy_latin1` opts into **bounded** historical-source replay
/// (LEGACY-SOURCE.1): when the bytes are not valid UTF-8, admit ISO-8859-1 (Latin-1) **only if every
/// non-UTF-8 byte lies inside a `#` comment** — pre-2013 tzdb carries accented author/place names in
/// comments. A non-UTF-8 byte in a semantics-bearing field is still refused. The modern default
/// (`legacy_latin1 == false`) is unchanged: any invalid UTF-8 is a hard `ZIC012`.
pub fn tokenize_with(bytes: &[u8], file: &Path, legacy_latin1: bool) -> Result<Vec<Line>> {
    let latin1_owned;
    let text: &str = match std::str::from_utf8(bytes) {
        Ok(t) => t,
        // Bounded legacy replay: Latin-1 is lossless (each byte → U+0000..=U+00FF) and comment bytes are
        // stripped by the lexer, so the *compiled output* is unaffected — only comment text differs.
        Err(_) if legacy_latin1 => {
            reject_non_utf8_outside_comments(bytes, file)?;
            latin1_owned = bytes.iter().map(|&b| b as char).collect::<String>();
            &latin1_owned
        }
        // Modern contract: any invalid UTF-8 is a hard error.
        Err(_) => {
            return Err(Error::from(Diagnostic::error(
                DiagnosticCode::InvalidValue,
                "source is not valid UTF-8",
                file,
                0,
            )));
        }
    };

    // Reference `zic`'s `inputline` requires every line to end with `\n`: when it reaches EOF with
    // content already accumulated (`linelen > 0`) it reports "unterminated line" and exits. Mirror that
    // exactly — a non-empty input whose final line is not newline-terminated is fatal. Computed here but
    // *checked inside the loop, after the per-line NUL/overlong checks*, so those take precedence within
    // that same final line (reference accumulates byte-by-byte and would hit a NUL/overlong first). An
    // input ending in `\n` has an empty final segment and is fine; empty input is fine (clean EOF).
    let unterminated_final_line = !text.is_empty() && !text.ends_with('\n');
    let last_number = text.split('\n').count();

    let mut lines = Vec::new();
    // Split on '\n'; tolerate a trailing '\r' (CRLF inputs) per line.
    for (idx, raw_line) in text.split('\n').enumerate() {
        let number = idx + 1;
        let raw_line = raw_line.strip_suffix('\r').unwrap_or(raw_line);

        // The +1 accounts for the newline that `split` consumed, matching the documented
        // limit which counts the newline. The final element from `split` has no newline,
        // but counting it the same way is conservative and harmless.
        if raw_line.len() + 1 > MAX_LINE_LEN {
            return Err(Error::from(Diagnostic::error(
                DiagnosticCode::OverlongInputLine,
                format!("line exceeds {MAX_LINE_LEN}-byte limit"),
                file,
                number,
            )));
        }
        if raw_line.as_bytes().contains(&0) {
            return Err(Error::from(Diagnostic::error(
                DiagnosticCode::NulByteInInput,
                "line contains a NUL byte",
                file,
                number,
            )));
        }
        if unterminated_final_line && number == last_number {
            return Err(Error::from(Diagnostic::error(
                DiagnosticCode::UnterminatedInputLine,
                "input line is not terminated by a newline",
                file,
                number,
            )));
        }

        let fields = split_fields(raw_line, file, number)?;
        if !fields.is_empty() {
            lines.push(Line { number, fields });
        }
    }
    Ok(lines)
}

/// Split one physical line into fields, honouring comments and double-quoting.
fn split_fields(line: &str, file: &Path, number: usize) -> Result<Vec<Field>> {
    let bytes = line.as_bytes();
    let mut fields = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        // Skip leading whitespace between fields.
        if is_ws(bytes[i]) {
            i += 1;
            continue;
        }
        // Unquoted '#' starts a comment: the rest of the line is gone.
        if bytes[i] == b'#' {
            break;
        }

        let start = i;
        let mut text = String::new();
        // Accumulate one field, which may interleave quoted and unquoted runs (e.g.
        // `ab"c d"e` is the single field `abc de`, matching zic's character-wise scan).
        while i < bytes.len() {
            match bytes[i] {
                b'"' => {
                    // Consume a quoted run up to the closing quote.
                    i += 1;
                    let qstart = i;
                    while i < bytes.len() && bytes[i] != b'"' {
                        i += 1;
                    }
                    if i >= bytes.len() {
                        return Err(Error::from(
                            Diagnostic::error(
                                DiagnosticCode::UnterminatedQuote,
                                "unterminated double-quoted field (odd number of quotation marks)",
                                file,
                                number,
                            )
                            .with_span(start, line.len()),
                        ));
                    }
                    text.push_str(&line[qstart..i]);
                    i += 1; // skip closing quote
                }
                b'#' => break, // a '#' outside quotes ends the field (and line).
                b if is_ws(b) => break,
                _ => {
                    text.push(bytes[i] as char);
                    i += 1;
                }
            }
        }
        fields.push(Field::new(text, start));
    }
    Ok(fields)
}

/// tzdata whitespace: space, tab, newline (already split), CR, form-feed, vertical tab.
fn is_ws(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\r' | 0x0c | 0x0b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn lex(s: &str) -> Vec<Line> {
        tokenize(s.as_bytes(), &PathBuf::from("t.zi")).unwrap()
    }

    // --- LEGACY-SOURCE.1: bounded Latin-1 historical-source replay ---

    /// Pre-2013 tzdb carries non-UTF-8 (Latin-1) bytes in `#` comments (accented author/place names).
    /// `--legacy-latin1` admits them; the comment is stripped, so the resulting records are unaffected.
    #[test]
    fn legacy_latin1_admits_non_utf8_in_comments() {
        let src = b"# note about La Naci\xf3n (Latin-1 \xf3)\nZone Test/X 0 - UTC\n";
        let p = PathBuf::from("t.zi");
        // modern contract (default): fail closed
        assert!(tokenize(src, &p).is_err(), "default must reject non-UTF-8");
        // legacy replay: admitted, and the Zone record survives intact
        let lines = tokenize_with(src, &p, true).expect("legacy mode admits Latin-1 in comments");
        assert!(
            lines
                .iter()
                .any(|l| l.fields.first().map(|f| f.text.as_str()) == Some("Zone")),
            "the Zone record must parse after the comment is stripped"
        );
    }

    /// The modern UTF-8 contract is preserved: invalid UTF-8 is a hard error unless legacy mode is on.
    #[test]
    fn modern_invalid_utf8_fails_closed() {
        let src = b"# caf\xe9\nZone Test/X 0 - UTC\n";
        assert!(tokenize(src, &PathBuf::from("t.zi")).is_err());
    }

    /// Guardrail: even in legacy mode, a non-UTF-8 byte in a **semantics-bearing field** (not a comment)
    /// is refused — legacy replay only admits Latin-1 inside `#` comments.
    #[test]
    fn legacy_rejects_non_utf8_outside_comment() {
        let src = b"Zone Test/X\xe9 0 - UTC\n"; // bad byte in the zone-name field, no `#`
        let e = tokenize_with(src, &PathBuf::from("t.zi"), true).unwrap_err();
        assert!(
            format!("{e}").contains("outside a comment"),
            "semantics-bearing non-UTF-8 must be refused even in legacy mode: {e}"
        );
    }

    #[test]
    fn basic_split_and_comments() {
        let ls = lex("Zone Etc/UTC 0 - UTC  # trailing comment\n# whole-line\n\n");
        assert_eq!(ls.len(), 1);
        let f: Vec<&str> = ls[0].fields.iter().map(|f| f.text.as_str()).collect();
        assert_eq!(f, vec!["Zone", "Etc/UTC", "0", "-", "UTC"]);
    }

    #[test]
    fn quoting_preserves_spaces_and_hash() {
        let ls = lex("Zone \"weird name #1\" 0 - X\n");
        assert_eq!(ls[0].fields[1].text, "weird name #1");
    }

    #[test]
    fn unterminated_quote_errors() {
        // Odd number of quotation marks → the dedicated lexical class (T14.2), no longer the
        // generic `InvalidValue`. (Input is newline-terminated so this isn't the newline rule.)
        let e = tokenize(b"Zone \"oops\n", &PathBuf::from("t.zi")).unwrap_err();
        assert!(matches!(
            e.diagnostic().map(|d| d.code),
            Some(DiagnosticCode::UnterminatedQuote)
        ));
    }

    #[test]
    fn missing_final_newline_rejected() {
        // Reference `zic` (`inputline`) is fatal at EOF with content but no trailing newline (T14.2).
        let e = tokenize(b"Zone Etc/UTC 0 - UTC", &PathBuf::from("t.zi")).unwrap_err();
        let d = e.diagnostic().unwrap();
        assert_eq!(d.code, DiagnosticCode::UnterminatedInputLine);
        assert_eq!(d.line, 1, "reported on the unterminated final line");
    }

    #[test]
    fn missing_final_newline_reported_on_the_last_line() {
        // An earlier well-terminated line passes; the unterminated tail is line 2.
        let e = tokenize(
            b"Zone Etc/UTC 0 - UTC\nLink Etc/UTC UTC",
            &PathBuf::from("t.zi"),
        )
        .unwrap_err();
        let d = e.diagnostic().unwrap();
        assert_eq!(d.code, DiagnosticCode::UnterminatedInputLine);
        assert_eq!(d.line, 2);
    }

    #[test]
    fn final_newline_accepted_unchanged() {
        // The valid, newline-terminated form still parses to exactly one line.
        let ls = lex("Zone Etc/UTC 0 - UTC\n");
        assert_eq!(ls.len(), 1);
    }

    #[test]
    fn empty_input_is_not_unterminated() {
        // Empty input is a clean EOF, not an unterminated line (matches reference `linelen == 0`).
        assert!(tokenize(b"", &PathBuf::from("t.zi")).unwrap().is_empty());
    }

    #[test]
    fn nul_takes_precedence_over_missing_newline() {
        // Reference accumulates byte-by-byte, so a NUL on the final unterminated line is hit before
        // EOF — NUL wins. Our per-line NUL check runs before the newline check, matching that order.
        let e = tokenize(b"Zone\0X", &PathBuf::from("t.zi")).unwrap_err();
        assert_eq!(e.diagnostic().unwrap().code, DiagnosticCode::NulByteInInput);
    }

    #[test]
    fn nul_rejected() {
        assert!(tokenize(b"Zone\0X\n", &PathBuf::from("t.zi")).is_err());
    }

    #[test]
    fn overlong_line_rejected() {
        let long = format!("Zone {}\n", "x".repeat(MAX_LINE_LEN));
        assert!(tokenize(long.as_bytes(), &PathBuf::from("t.zi")).is_err());
    }
}
