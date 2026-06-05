//! Tests for the `explain` evidence trace and link/canonical resolution (T3.5).

use std::path::PathBuf;

use tzcompile::compile::plan::explain;
use tzcompile::model::Database;
use tzcompile::{load_database, resolve_link_target};

fn fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn db_from(src: &str) -> Database {
    let mut db = Database::default();
    tzcompile::source::parse_into(src.as_bytes(), &PathBuf::from("t.zi"), &mut db).unwrap();
    db
}

// --- explain trace ---------------------------------------------------------------------

#[test]
fn explain_multi_era_trace_has_eras_types_transitions_and_equal_utoff_note() {
    let db = load_database(&[fixture("fixtures/minimal/multi_mid.zi")]).unwrap();
    let trace = explain(&db, "Test/MidDst").unwrap();
    // Decoded source eras.
    assert!(trace.contains("2 era(s)"));
    assert!(trace.contains("RULES rules R"));
    assert!(trace.contains("UNTIL 1990-07-1"));
    // Compiled types incl. the equal-utoff pair.
    assert!(trace.contains("abbr=\"AST\""));
    assert!(trace.contains("abbr=\"EDT\""));
    // A decoded transition instant and the equal-utoff note (the kept-type trap).
    assert!(trace.contains("1990-07-01 04:00:00Z"));
    assert!(trace.contains("utoff -4 is shared by distinct types"));
}

#[test]
fn explain_reports_link_alias_resolution() {
    let db = load_database(&[fixture("fixtures/minimal/utc.zi")]).unwrap();
    let trace = explain(&db, "UTC").unwrap();
    assert!(trace.contains("link alias -> canonical zone Etc/UTC"));
    assert!(trace.contains("aliases (links): UTC"));
    assert!(trace.contains("footer \"UTC0\""));
}

#[test]
fn explain_recurring_caps_transition_listing() {
    let db = load_database(&[fixture("fixtures/minimal/eastern.zi")]).unwrap();
    let trace = explain(&db, "Test/Eastern").unwrap();
    assert!(trace.contains("62 transition(s)"));
    assert!(trace.contains("more)")); // the "… (N more)" elision marker
    assert!(trace.contains("EST5EDT,M3.2.0,M11.1.0"));
}

#[test]
fn explain_surfaces_link_cycle() {
    let db = db_from("Link A B\nLink B A\n");
    let err = explain(&db, "B").unwrap_err();
    assert!(err.message.contains("cycle"), "got: {}", err.message);
    assert!(err.message.contains("B -> A -> B"));
}

#[test]
fn explain_surfaces_missing_link_target() {
    let db = db_from("Link Nowhere X\n");
    let err = explain(&db, "X").unwrap_err();
    assert!(err.message.contains("does not name a zone or link"));
}

// --- link/canonical resolution --------------------------------------------------------

#[test]
fn resolve_alias_to_canonical() {
    let db = db_from("Zone Etc/UTC 0 - UTC\nLink Etc/UTC UTC\n");
    assert_eq!(resolve_link_target(&db, "UTC").unwrap(), "Etc/UTC");
    // A real zone resolves to itself.
    assert_eq!(resolve_link_target(&db, "Etc/UTC").unwrap(), "Etc/UTC");
}

#[test]
fn resolve_multi_hop_chain() {
    let db = db_from("Zone Etc/UTC 0 - UTC\nLink Etc/UTC UTC\nLink UTC Universal\n");
    assert_eq!(resolve_link_target(&db, "Universal").unwrap(), "Etc/UTC");
}

#[test]
fn resolve_cycle_errors_with_path() {
    let db = db_from("Link A B\nLink B A\n");
    let err = resolve_link_target(&db, "B").unwrap_err();
    assert!(err.to_string().contains("cycle"));
}

#[test]
fn resolve_missing_target_errors() {
    let db = db_from("Link Nowhere X\n");
    let err = resolve_link_target(&db, "X").unwrap_err();
    assert!(err.to_string().contains("does not name a zone or link"));
}
