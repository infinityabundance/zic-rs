//! A tiny, dependency-free JSON string escaper shared by the deterministic JSON emitters
//! (`manifest`, `report`).
//!
//! The project deliberately hand-rolls its JSON rather than pulling in `serde_json`, so the
//! one piece every emitter needs — correct string escaping — lives here once. It implements
//! exactly the escapes RFC 8259 requires: the two mandatory (`"` and `\`), the common control
//! shorthands (`\n`/`\r`/`\t`), and `\u00XX` for any other C0 control character. Everything
//! else (including non-ASCII) is emitted verbatim as UTF-8, which is valid JSON.

/// Return `s` as a quoted, escaped JSON string literal (including the surrounding `"`).
pub(crate) fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_mandatory_and_control_chars() {
        assert_eq!(escape("a\"b\\c"), "\"a\\\"b\\\\c\"");
        assert_eq!(escape("line\nbreak\ttab"), "\"line\\nbreak\\ttab\"");
        assert_eq!(escape("\u{0001}"), "\"\\u0001\"");
        // Non-ASCII passes through as UTF-8 (valid JSON).
        assert_eq!(escape("café"), "\"café\"");
    }
}
