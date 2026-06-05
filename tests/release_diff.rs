//! T16.6a — `release-diff` integration tests.
//!
//! Synthetic OLD/NEW `.zi` pairs exercise the structural axis (always available, no oracle) and — when
//! a `zdump` oracle is on `PATH` — the behaviour axis split into past/future windows. The behaviour-axis
//! tests auto-skip when `zdump` is absent (the project's oracle-availability discipline), exactly like
//! `tests/diagnostic_parity.rs`.

use std::process::Command;

use tzcompile::release_diff::{
    build_release_diff, OracleFailureScope, ReleaseChangeKind, ReleaseDiffOptions,
    ReleaseDiffReport,
};

/// Write a `.zi` source to a uniquely-named temp file and load it as a [`Database`]. The path is unique
/// per call (a process-wide atomic counter) so parallel tests never clobber each other's input.
fn db_from(_label: &str, source: &str) -> tzcompile::model::Database {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("zic-rs-rdtest-{}-{}", std::process::id(), n));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("in.zi");
    std::fs::write(&path, source).unwrap();
    tzcompile::load_database(&[path]).unwrap()
}

/// Build a report with the behaviour axis OFF (structural only — no oracle needed, fully deterministic).
fn diff_structural(old_src: &str, new_src: &str) -> ReleaseDiffReport {
    let old_db = db_from("old", old_src);
    let new_db = db_from("new", new_src);
    build_release_diff(
        &old_db,
        &new_db,
        &ReleaseDiffOptions {
            horizon: (1900, 2040),
            split: 2025,
            zone_filter: None,
            zdump_program: None,
        },
    )
    .unwrap()
}

fn kind_of(report: &ReleaseDiffReport, name: &str) -> Option<ReleaseChangeKind> {
    report
        .rows
        .iter()
        .find(|r| r.name == name)
        .map(|r| r.change_kind)
}

fn has_zdump() -> bool {
    Command::new("zdump")
        .arg("--version")
        .output()
        .map(|_| true)
        .unwrap_or(false)
}

// ── structural axis (no oracle) ──────────────────────────────────────────────────────────────────

#[test]
fn identical_release_is_unchanged() {
    let src = "Zone Test/Z -5:00 - EST\n";
    let report = diff_structural(src, src);
    assert_eq!(
        kind_of(&report, "Test/Z"),
        Some(ReleaseChangeKind::Unchanged)
    );
}

#[test]
fn zone_added_and_removed() {
    let old = "Zone Test/Keep -5:00 - EST\nZone Test/Gone 3:00 - PLUS3\n";
    let new = "Zone Test/Keep -5:00 - EST\nZone Test/New 4:00 - PLUS4\n";
    let report = diff_structural(old, new);
    assert_eq!(kind_of(&report, "Test/New"), Some(ReleaseChangeKind::Added));
    assert_eq!(
        kind_of(&report, "Test/Gone"),
        Some(ReleaseChangeKind::Removed)
    );
    assert_eq!(
        kind_of(&report, "Test/Keep"),
        Some(ReleaseChangeKind::Unchanged)
    );
}

#[test]
fn link_unchanged_then_retargeted() {
    let old = "Zone Test/A 1:00 - PLUS1\nZone Test/B 2:00 - PLUS2\nLink Test/A Test/Alias\n";
    let new = "Zone Test/A 1:00 - PLUS1\nZone Test/B 2:00 - PLUS2\nLink Test/B Test/Alias\n";
    let unchanged = diff_structural(old, old);
    assert_eq!(
        kind_of(&unchanged, "Test/Alias"),
        Some(ReleaseChangeKind::Unchanged)
    );
    let retargeted = diff_structural(old, new);
    assert_eq!(
        kind_of(&retargeted, "Test/Alias"),
        Some(ReleaseChangeKind::LinkChanged)
    );
}

#[test]
fn changed_zone_without_oracle_is_behaviour_unassessed() {
    // A future-year rule extension changes the compiled bytes; with no zdump oracle, the structural
    // delta is recorded but behaviour is honestly NOT assessed (never silently "no change").
    let old = "Rule R 1990 2030 - Mar lastSun 2:00 1:00 D\n\
               Rule R 1990 2030 - Oct lastSun 2:00 0 S\n\
               Zone Test/Z -5:00 R E%sT\n";
    let new = "Rule R 1990 2035 - Mar lastSun 2:00 1:00 D\n\
               Rule R 1990 2035 - Oct lastSun 2:00 0 S\n\
               Zone Test/Z -5:00 R E%sT\n";
    let report = diff_structural(old, new);
    assert_eq!(
        kind_of(&report, "Test/Z"),
        Some(ReleaseChangeKind::BehaviourUnassessed)
    );
    // oracle absence is surfaced, not silent.
    assert!(matches!(
        report.oracle_mode,
        tzcompile::manifest::OracleMode::Unavailable(_)
    ));
}

#[test]
fn json_shape_is_well_formed() {
    let report = diff_structural("Zone Test/Z -5:00 - EST\n", "Zone Test/Z 3:00 - PLUS3\n");
    let json = report.to_json();
    assert!(json.contains("\"schema\": \"zic-rs-release-diff-v1\""));
    assert!(json.contains("\"oracle_mode\""));
    assert!(json.contains("\"summary\""));
    assert!(json.contains("\"identifiers\""));
    assert!(json.contains("\"non_claim\""));
}

// ── totality ─────────────────────────────────────────────────────────────────────────────────────

#[test]
fn oracle_failure_scope_totality() {
    // The two scopes have distinct, stable labels.
    assert_eq!(
        OracleFailureScope::GlobalToolUnavailable.as_str(),
        "global_tool_unavailable"
    );
    assert_eq!(
        OracleFailureScope::RowOrIdentifierFailure.as_str(),
        "row_or_identifier_failure"
    );
    assert_ne!(
        OracleFailureScope::GlobalToolUnavailable.as_str(),
        OracleFailureScope::RowOrIdentifierFailure.as_str()
    );
}

#[test]
fn unresolvable_zdump_is_global_unavailable_up_front() {
    // T17.3: a zdump program that cannot be resolved makes the whole behaviour axis unavailable
    // (GlobalToolUnavailable) — decided ONCE up front, not by poisoning the run on the first row.
    // The structural rows are still produced; the byte-differing one is honestly BehaviourUnassessed.
    let old_db = db_from("old", "Zone Test/Z -5:00 - EST\n");
    let new_db = db_from("new", "Zone Test/Z 3:00 - PLUS3\n");
    let report = build_release_diff(
        &old_db,
        &new_db,
        &ReleaseDiffOptions {
            horizon: (1900, 2040),
            split: 2025,
            zone_filter: None,
            zdump_program: Some("zic-rs-definitely-not-a-real-zdump".into()),
        },
    )
    .unwrap();
    match &report.oracle_mode {
        tzcompile::manifest::OracleMode::Unavailable(reason) => {
            assert!(
                reason.contains("global_tool_unavailable"),
                "expected a global-tool-unavailable reason, got: {reason}"
            );
        }
        other => panic!("expected Unavailable, got {other:?}"),
    }
    assert_eq!(
        kind_of(&report, "Test/Z"),
        Some(ReleaseChangeKind::BehaviourUnassessed)
    );
}

#[test]
fn change_kind_totality() {
    use std::collections::BTreeSet;
    let labels: BTreeSet<&str> = ReleaseChangeKind::ALL.iter().map(|k| k.as_str()).collect();
    assert_eq!(
        labels.len(),
        ReleaseChangeKind::ALL.len(),
        "labels must be unique"
    );
    // every label round-trips and is non-empty
    for k in ReleaseChangeKind::ALL {
        assert!(!k.as_str().is_empty());
    }
    // kind_counts seeds every variant (so a 0-count kind is still reported, never hidden).
    let report = ReleaseDiffReport {
        oracle_mode: tzcompile::manifest::OracleMode::NotRun,
        horizon: (1900, 2040),
        split: 2025,
        rows: Vec::new(),
        errors: Vec::new(),
    };
    assert_eq!(report.kind_counts().len(), ReleaseChangeKind::ALL.len());
}

// ── behaviour axis (oracle-gated; auto-skip when zdump absent) ─────────────────────────────────────

fn diff_with_oracle(old_src: &str, new_src: &str) -> ReleaseDiffReport {
    let old_db = db_from("old", old_src);
    let new_db = db_from("new", new_src);
    build_release_diff(
        &old_db,
        &new_db,
        &ReleaseDiffOptions {
            horizon: (1900, 2040),
            split: 2025,
            zone_filter: None,
            zdump_program: Some("zdump".into()),
        },
    )
    .unwrap()
}

#[test]
fn behaviour_future_only() {
    if !has_zdump() {
        eprintln!("note: zdump not on PATH — skipping behaviour-axis test (oracle skip)");
        return;
    }
    // Identical through 2030; NEW extends the rule to 2035. The only divergence is post-split (2025).
    let old = "Rule R 1990 2030 - Mar lastSun 2:00 1:00 D\n\
               Rule R 1990 2030 - Oct lastSun 2:00 0 S\n\
               Zone Test/Z -5:00 R E%sT\n";
    let new = "Rule R 1990 2035 - Mar lastSun 2:00 1:00 D\n\
               Rule R 1990 2035 - Oct lastSun 2:00 0 S\n\
               Zone Test/Z -5:00 R E%sT\n";
    let report = diff_with_oracle(old, new);
    assert_eq!(
        kind_of(&report, "Test/Z"),
        Some(ReleaseChangeKind::BehaviorFuture),
        "future-only rule extension must classify as behavior_future"
    );
}

#[test]
fn split_year_change_is_attributed_to_the_future_window_not_double_counted() {
    // T17.3 — pin the `--split` boundary semantics so a later reviewer never wonders whether a change
    // in the split year counts as past, future, or both. OLD is a fixed zone with no transitions; NEW
    // introduces transitions ONLY in the split year (2025). With windows [lo, split] and [split, hi],
    // the past window has no change (the 2025 transitions are not below 2025), so the change is
    // attributed to the FUTURE window — `behavior_future`, NOT `behavior_past_and_future`
    // (i.e. the split-year transition is not double-counted into both windows).
    if !has_zdump() {
        eprintln!("note: zdump not on PATH — skipping split-boundary test (oracle skip)");
        return;
    }
    // OLD and NEW are byte-for-byte identical before 2025 (same zone, same rule set, no rule active
    // pre-2025); they differ ONLY in the SAVE amount of the 2025 summer transition (1h vs 2h). So the
    // sole behavioural difference sits in the split year — exactly the boundary case.
    let old = "Rule X 2025 only - Jun 1 0:00 1:00 D\n\
               Rule X 2025 only - Sep 1 0:00 0 S\n\
               Zone Test/B 1:00 X PLUS%s\n";
    let new = "Rule X 2025 only - Jun 1 0:00 2:00 D\n\
               Rule X 2025 only - Sep 1 0:00 0 S\n\
               Zone Test/B 1:00 X PLUS%s\n";
    let report = diff_with_oracle(old, new);
    assert_eq!(
        kind_of(&report, "Test/B"),
        Some(ReleaseChangeKind::BehaviorFuture),
        "a split-year (2025) change must land in the future window only, never double-counted"
    );
}

#[test]
fn behaviour_past_only() {
    if !has_zdump() {
        eprintln!("note: zdump not on PATH — skipping behaviour-axis test (oracle skip)");
        return;
    }
    // Both rules END in 2020 (well before split 2025, no recurring footer → future identical); NEW adds
    // earlier transitions (FROM 1985 vs 1990), so only the past window differs.
    let old = "Rule R 1990 2020 - Mar lastSun 2:00 1:00 D\n\
               Rule R 1990 2020 - Oct lastSun 2:00 0 S\n\
               Zone Test/Z -5:00 R E%sT\n";
    let new = "Rule R 1985 2020 - Mar lastSun 2:00 1:00 D\n\
               Rule R 1985 2020 - Oct lastSun 2:00 0 S\n\
               Zone Test/Z -5:00 R E%sT\n";
    let report = diff_with_oracle(old, new);
    assert_eq!(
        kind_of(&report, "Test/Z"),
        Some(ReleaseChangeKind::BehaviorPast),
        "earlier-transitions-only change must classify as behavior_past"
    );
    assert!(matches!(
        report.oracle_mode,
        tzcompile::manifest::OracleMode::ReferenceZdump
    ));
}
