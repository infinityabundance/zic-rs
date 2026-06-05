//! T14.3 — metamorphic source-ordering parity (contract-level; NO behaviour change).
//!
//! `zic(8)`: input lines may appear in **any order** *except* continuation lines, which must stay
//! attached to their `Zone`. Reference `zic` enforces this by re-sorting internally
//! (`associate()` → `qsort(rules, rcomp)`, `qsort(links, …)`, `qsort(attypes, atcomp)`), so record
//! order is part of *source identity* but never of *compiled behaviour*. This suite pins exactly that:
//!
//! * **Permitted transformations** (pinned to what reference treats as equivalent): permuting whole
//!   logical records (a `Zone`+its continuations is ONE record), adding/removing comments and blank
//!   lines, and normalizing **inter-field** whitespace. It is deliberately **not** a general
//!   formatter — leading whitespace is *not* touched (in `zic` a leading-whitespace line is a
//!   continuation), and quoted fields are left alone.
//! * **Invariant:** every zone's compiled TZif bytes and the link/alias set are **identical** across
//!   all permitted transformations; only the order-sensitive *source identity* (the raw bytes) may
//!   differ. Continuations are never detached (proven by a negative control). Diagnostics keep the
//!   same **class** under permutation (the line *number* legitimately moves — we assert class +
//!   offending entity, not literal line, per the metamorphic caution).
//!
//! No behaviour change: this only observes `load_database` + `compile_zone_to_bytes`.

use std::collections::{BTreeMap, BTreeSet};

use tzcompile::{compile_zone_to_bytes, load_database, DiagnosticCode};

/// Logical records of the base source. **A `Zone` with continuation lines is a single record** (the
/// header line + its indented continuations), so permuting the record list never detaches a
/// continuation — exactly the `zic(8)` order-independence law and its one exception.
const RECORDS: &[&str] = &[
    "Rule Ra 1970 1979 - Apr lastSun 2:00 1:00 D",
    "Rule Ra 1970 1979 - Oct lastSun 2:00 0 S",
    "Rule Rb 1980 max - Mar Sun>=8 2:00 1:00 D",
    "Rule Rb 1980 max - Nov Sun>=1 2:00 0 S",
    "Zone Etc/Alpha 1:00 Ra A%sT",
    "Zone Test/Multi -5:00 - EST 1980\n\t-6:00 Rb C%sT", // header + continuation (one record)
    "Zone Etc/Beta 9:00 - JST",
    "Link Etc/Alpha Alias/Alpha",
    "Link Test/Multi Alias/Multi",
];

/// Assemble a source from a record ordering (newline-terminated, as the lexer now requires — T14.2).
fn source_of(order: &[usize]) -> String {
    let mut s = String::new();
    for &i in order {
        s.push_str(RECORDS[i]);
        s.push('\n');
    }
    s
}

/// Compile `source` and return a name→TZif-bytes map for every zone (the compiled-behaviour fingerprint).
fn compiled_zones(source: &str) -> BTreeMap<String, Vec<u8>> {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("in.zi");
    std::fs::write(&p, source).unwrap();
    let db = load_database(std::slice::from_ref(&p)).expect("base/permuted source compiles");
    db.zones
        .iter()
        .map(|z| {
            (
                z.name.clone(),
                compile_zone_to_bytes(&db, &z.name).expect("zone compiles"),
            )
        })
        .collect()
}

/// The link/alias set as order-independent `(link_name, target)` pairs.
fn link_set(source: &str) -> BTreeSet<(String, String)> {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("in.zi");
    std::fs::write(&p, source).unwrap();
    let db = load_database(std::slice::from_ref(&p)).unwrap();
    db.links
        .iter()
        .map(|l| (l.link_name.clone(), l.target.clone()))
        .collect()
}

/// A few **deterministic** permutations (no RNG — reproducible): identity, reverse, rotate-by-1, and a
/// hand-picked shuffle that interleaves Rules/Zones/Links and moves the multi-era record around.
fn permutations() -> Vec<Vec<usize>> {
    let n = RECORDS.len();
    let identity: Vec<usize> = (0..n).collect();
    let reverse: Vec<usize> = (0..n).rev().collect();
    let rotate: Vec<usize> = (0..n).map(|i| (i + 1) % n).collect();
    // Links first, multi-era zone before its rules, standalone zones interleaved.
    let shuffle = vec![7, 5, 8, 4, 2, 0, 6, 3, 1];
    assert_eq!(
        shuffle.len(),
        n,
        "shuffle must be a permutation of all records"
    );
    vec![identity, reverse, rotate, shuffle]
}

#[test]
fn permuting_independent_records_preserves_every_zone_and_link() {
    let base_order: Vec<usize> = (0..RECORDS.len()).collect();
    let base_src = source_of(&base_order);
    let base_zones = compiled_zones(&base_src);
    let base_links = link_set(&base_src);
    assert_eq!(base_zones.len(), 3, "Etc/Alpha, Test/Multi, Etc/Beta");
    assert_eq!(base_links.len(), 2);

    for perm in permutations() {
        let src = source_of(&perm);
        // Compiled behaviour is byte-identical per zone — reference re-sorts, so order is not behaviour.
        assert_eq!(
            compiled_zones(&src),
            base_zones,
            "zone bytes changed under permutation {perm:?}"
        );
        // The link/alias set is order-independent too.
        assert_eq!(
            link_set(&src),
            base_links,
            "link set changed under permutation {perm:?}"
        );
    }
}

#[test]
fn source_identity_differs_when_order_changes_even_though_behaviour_does_not() {
    // The companion truth: *source identity* (raw bytes) DOES change with order — only behaviour is
    // invariant. (This is why the manifest's order-sensitive `aggregate_hash` may differ while the
    // canonical behaviour is unchanged — the two are deliberately distinct axes, T12.3.)
    let base = source_of(&(0..RECORDS.len()).collect::<Vec<_>>());
    let reversed = source_of(&(0..RECORDS.len()).rev().collect::<Vec<_>>());
    assert_ne!(base, reversed, "reordering must change the source bytes");
    // ...yet the compiled behaviour is identical (asserted in the test above).
}

#[test]
fn comments_and_blank_lines_do_not_change_semantics() {
    let base = source_of(&(0..RECORDS.len()).collect::<Vec<_>>());
    let base_zones = compiled_zones(&base);

    // Same records, but with full-line comments and blank lines sprinkled between them. (A comment is
    // never inserted *inside* the multi-era record, which would split the continuation.)
    let mut decorated = String::from("# leading comment\n\n");
    for &i in &(0..RECORDS.len()).collect::<Vec<_>>() {
        decorated.push_str("# ---\n");
        decorated.push_str(RECORDS[i]);
        decorated.push_str("\n\n");
    }
    assert_eq!(
        compiled_zones(&decorated),
        base_zones,
        "comments/blank lines changed the compiled output"
    );
}

#[test]
fn inter_field_whitespace_normalization_does_not_change_semantics() {
    let base = source_of(&(0..RECORDS.len()).collect::<Vec<_>>());
    let base_zones = compiled_zones(&base);

    // Re-space ONLY between fields: collapse/expand the spaces that already separate tokens into
    // tabs and multi-space runs. Leading whitespace is never introduced (that would make a command
    // line a continuation), and the tab-indented continuation line keeps its leading tab.
    let respaced: String = base
        .lines()
        .map(|line| {
            if line.starts_with('\t') {
                // continuation: keep the leading tab, re-space the remainder between fields.
                let rest = line.trim_start_matches('\t');
                format!(
                    "\t{}",
                    rest.split_whitespace().collect::<Vec<_>>().join("   ")
                )
            } else {
                line.split_whitespace().collect::<Vec<_>>().join("\t")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    assert_ne!(respaced, base, "the re-spacing must actually differ");
    assert_eq!(
        compiled_zones(&respaced),
        base_zones,
        "inter-field whitespace changed the compiled output"
    );
}

#[test]
fn detached_continuation_is_rejected_not_silently_reordered() {
    // The ONE exception to order-independence: a continuation line moved away from its `Zone` is not
    // a free permutation — it becomes a stray continuation. zic-rs rejects it with the structural
    // `ContinuationWithoutZone` class (T13.2), proving the metamorphic law's boundary is enforced.
    let detached = "\t-6:00 Rb C%sT\nZone Etc/Beta 9:00 - JST\n";
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("in.zi");
    std::fs::write(&p, detached).unwrap();
    let err = load_database(std::slice::from_ref(&p)).expect_err("detached continuation must fail");
    assert_eq!(
        err.diagnostic().map(|d| d.code),
        Some(DiagnosticCode::ContinuationWithoutZone),
    );
}

#[test]
fn diagnostic_class_is_stable_under_permutation_even_as_line_moves() {
    // A malformed record (unknown keyword) keeps the SAME diagnostic *class* when independent valid
    // records are reordered around it — even though its line *number* legitimately moves. We assert
    // class + that it is still flagged, NOT a literal line number (the metamorphic-stability caution).
    let bad = "Frobnicate X 0 - Q";
    let class_of = |src: &str| -> Option<DiagnosticCode> {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("in.zi");
        std::fs::write(&p, src).unwrap();
        load_database(std::slice::from_ref(&p))
            .err()
            .and_then(|e| e.diagnostic().map(|d| d.code))
    };

    // Bad line first vs last among valid records → different line number, same class.
    let first = format!("{bad}\n{}\n{}\n", RECORDS[4], RECORDS[6]);
    let last = format!("{}\n{}\n{bad}\n", RECORDS[4], RECORDS[6]);
    assert_eq!(class_of(&first), Some(DiagnosticCode::UnknownLineType));
    assert_eq!(
        class_of(&first),
        class_of(&last),
        "diagnostic class must be stable under permutation (line number may differ)"
    );
}
