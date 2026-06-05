//! T14.1 — input-text admissibility, as an **executable contract** (not just the doc table).
//!
//! Each row pins a reference `zic` admissibility rule and asserts zic-rs's *current* behaviour against
//! it via the public `load_database` API. This is the T13 lesson applied to T14 from day one: a
//! doctrine table becomes a machine-checked witness. It records **current** behaviour honestly — the
//! one known divergence (a missing final newline: reference is fatal, zic-rs is lenient) is asserted
//! *as lenient* and flagged for T14.2, so the test flips deliberately when that tightening lands,
//! rather than silently drifting. No behaviour change here — this only observes.
//!
//! Diagnostic codes are matched from the rendered error string (`…[ZICnnn_…]…`); admissibility
//! failures are the **lexical** layer (see `docs/zic-hostile-input-parity.md`).

use tzcompile::load_database;

/// What zic-rs *currently* does with a crafted input (the executable column of the admissibility table).
#[derive(Debug, PartialEq, Eq)]
enum Admit {
    /// Accepted (parses) — `load_database` returns `Ok`.
    Accepts,
    /// Rejected with a diagnostic whose code string contains this token.
    RejectsWith(&'static str),
    /// Rejected, but not via a coded `ZIC…` diagnostic (a generic/IO error path).
    RejectsUncoded,
}

/// Compile `bytes` and classify the outcome.
fn admit(dir: &std::path::Path, bytes: &[u8]) -> Admit {
    let p = dir.join("in.zi");
    std::fs::write(&p, bytes).unwrap();
    match load_database(std::slice::from_ref(&p)) {
        Ok(_) => Admit::Accepts,
        Err(e) => {
            let rendered = e.to_string();
            // The coded diagnostics we expect at the admissibility layer.
            for code in [
                "ZIC016_NUL_INPUT_BYTE",
                "ZIC017_OVERLONG_INPUT_LINE",
                "ZIC021_UNTERMINATED_INPUT_LINE",
                "ZIC022_UNTERMINATED_QUOTE",
            ] {
                if rendered.contains(code) {
                    return Admit::RejectsWith(code);
                }
            }
            Admit::RejectsUncoded
        }
    }
}

/// A valid 20-char zone line; padding it controls the raw line length for the cap test.
const VALID_ZONE: &str = "Zone Etc/UTC 0 - UTC";

#[test]
fn admissibility_table_matches_reference_pins() {
    let dir = tempfile::tempdir().unwrap();

    // 1. Line-length cap — pinned: reference `zic.c` `_POSIX2_LINE_MAX = 2048`; zic-rs `MAX_LINE_LEN
    //    = 2048`. Both accept ≤ 2047 content bytes and reject ≥ 2048. (The "511" manpage figure is
    //    stale — verified against the pinned 2026b source.)
    let line_2047 = format!("{VALID_ZONE}{}\n", " ".repeat(2047 - VALID_ZONE.len()));
    assert_eq!(
        admit(dir.path(), line_2047.as_bytes()),
        Admit::Accepts,
        "2047-byte line ok"
    );
    let line_2048 = format!("{VALID_ZONE}{}\n", " ".repeat(2048 - VALID_ZONE.len()));
    assert_eq!(
        admit(dir.path(), line_2048.as_bytes()),
        Admit::RejectsWith("ZIC017_OVERLONG_INPUT_LINE"),
        "2048-byte line rejected (matches reference cap)"
    );

    // 2. NUL byte — pinned reference "NUL input byte" (fatal). zic-rs → ZIC016.
    assert_eq!(
        admit(dir.path(), b"Zone Etc/UTC 0 - U\0T\n"),
        Admit::RejectsWith("ZIC016_NUL_INPUT_BYTE"),
    );

    // 3. Final newline present — accepted by both.
    assert_eq!(admit(dir.path(), b"Zone Etc/UTC 0 - UTC\n"), Admit::Accepts);

    // 4. **Missing final newline** — reference `zic` is FATAL ("unterminated line"). T14.1 pinned
    //    zic-rs's then-lenient behaviour and flagged the flip; **T14.2 made it fatal**, matching
    //    reference, with the dedicated `ZIC021_UNTERMINATED_INPUT_LINE` class. (This assertion is the
    //    deliberate flip the T14.1 witness was written to anticipate.)
    assert_eq!(
        admit(dir.path(), b"Zone Etc/UTC 0 - UTC"), // no trailing '\n'
        Admit::RejectsWith("ZIC021_UNTERMINATED_INPUT_LINE"),
        "missing final newline now rejected (T14.2), matching reference `zic`'s 'unterminated line'"
    );

    // 5. Unterminated quote (odd number of `\"`) — reference fatal "Odd number of quotation marks".
    //    zic-rs always failed closed; **T14.2 gave it the dedicated `ZIC022_UNTERMINATED_QUOTE` class**
    //    (was the generic `ZIC012_INVALID_VALUE`). Note the quoted line is newline-terminated so this
    //    isolates the quote rule from rule 4.
    assert_eq!(
        admit(dir.path(), b"Zone Etc/UTC 0 - \"UT\n"),
        Admit::RejectsWith("ZIC022_UNTERMINATED_QUOTE"),
        "odd quotes now carry the dedicated UnterminatedQuote code (T14.2)"
    );

    // 6. Quoted `#` — literal inside quotes (not a comment); accepted by both.
    assert_eq!(
        admit(dir.path(), b"Zone Etc/UTC 0 - \"U#T\"\n"),
        Admit::Accepts
    );

    // 7. `#` comment outside quotes — stripped; the zone still compiles.
    assert_eq!(
        admit(dir.path(), b"Zone Etc/UTC 0 - UTC # trailing comment\n"),
        Admit::Accepts
    );
}
