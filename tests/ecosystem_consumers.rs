//! Optional **ecosystem interop** bench — gated behind the `ecosystem-tests` feature.
//!
//! ```text
//! cargo test --features ecosystem-tests
//! ```
//!
//! ## What this is (and what it is NOT)
//!
//! This is an *interoperability* check, not a correctness oracle. It compiles fixtures with
//! zic-rs and loads the resulting TZif bytes with [`tz-rs`](https://crates.io/crates/tz-rs) —
//! a third-party, pure-Rust TZif **reader** — then asserts the offset / DST flag / abbreviation
//! a real consumer observes at the exact instants that expose each semantic trap in the
//! behaviour ledger (`docs/reference-zic-semantics.md`).
//!
//! The correctness hierarchy is unchanged (see `docs/consumer-testbench.md`):
//!   1. reference `zic` / `zdump` behaviour  — the binding contract;
//!   2. RFC 9636 structural validity;
//!   3. zic-rs's own parser/semantic tests;
//!   4. **this** consumer bench — evidence that generated TZif is *readable & usable* by a real
//!      Rust consumer, never a substitute for (1). A disagreement here is investigated against
//!      the `zic`/`zdump` oracle first; "tz-rs says X" is not authority over reference `zic`.
//!
//! The whole file is `#![cfg(feature = "ecosystem-tests")]`, so the default `cargo test` (and
//! the default dependency graph) never compile it or `tz-rs`.
#![cfg(feature = "ecosystem-tests")]

use std::path::PathBuf;

use tzcompile::model::calendar::days_from_civil;
use tzcompile::{compile_zone_to_bytes, load_database};

fn fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// Compile a fixture zone with zic-rs and return its TZif bytes (the same bytes the `compile`
/// command would write).
fn tzif(source: &str, zone: &str) -> Vec<u8> {
    let db = load_database(&[fixture(source)]).expect("load database");
    compile_zone_to_bytes(&db, zone).expect("compile to TZif bytes")
}

/// A UT instant from a civil date/time, via zic-rs's own calendar (no hard-coded magic
/// numbers, no dependency on the host clock).
fn instant(y: i32, mon: u8, d: u8, hh: i64, mm: i64, ss: i64) -> i64 {
    days_from_civil(y, mon, d) * 86_400 + hh * 3_600 + mm * 60 + ss
}

/// What the consumer (`tz-rs`) observes at `ts`: `(ut_offset_seconds, is_dst, abbreviation)`.
fn observed(tzif: &[u8], ts: i64) -> (i32, bool, String) {
    let tz = tz::TimeZone::from_tz_data(tzif).expect("tz-rs parses zic-rs TZif");
    let t = tz
        .find_local_time_type(ts)
        .expect("tz-rs resolves a local time type");
    (
        t.ut_offset(),
        t.is_dst(),
        t.time_zone_designation().to_string(),
    )
}

#[test]
fn tzrs_reads_fixed_offset_zones() {
    let utc = tzif("fixtures/minimal/utc.zi", "Etc/UTC");
    assert_eq!(
        observed(&utc, instant(2020, 6, 1, 0, 0, 0)),
        (0, false, "UTC".into())
    );

    let fixed = tzif("fixtures/minimal/fixed.zi", "Test/Fixed");
    assert_eq!(
        observed(&fixed, instant(2020, 6, 1, 0, 0, 0)),
        (-18000, false, "EST".into())
    );
}

#[test]
fn tzrs_reads_finite_dst_transition_to_the_second() {
    let z = tzif("fixtures/minimal/dst.zi", "Test/Simple");
    // Spring forward 2020-03-08 07:00:00Z: the last EST second, then EDT.
    assert_eq!(
        observed(&z, instant(2020, 3, 8, 6, 59, 59)),
        (-18000, false, "EST".into())
    );
    assert_eq!(
        observed(&z, instant(2020, 3, 8, 7, 0, 0)),
        (-14400, true, "EDT".into())
    );
    // Fall back 2020-11-01 06:00:00Z: the last EDT second, then EST.
    assert_eq!(
        observed(&z, instant(2020, 11, 1, 5, 59, 59)),
        (-14400, true, "EDT".into())
    );
    assert_eq!(
        observed(&z, instant(2020, 11, 1, 6, 0, 0)),
        (-18000, false, "EST".into())
    );
}

#[test]
fn tzrs_projects_recurring_footer_beyond_explicit_transitions() {
    // Test/Eastern's explicit transitions stop at RECUR_HI (2037); 2040 is governed purely by
    // the POSIX footer `EST5EDT,M3.2.0,M11.1.0`. That tz-rs gets it right is direct evidence
    // that zic-rs's footer is correct and consumer-usable, not just the explicit window.
    let z = tzif("fixtures/minimal/eastern.zi", "Test/Eastern");
    assert_eq!(
        observed(&z, instant(2040, 1, 15, 12, 0, 0)),
        (-18000, false, "EST".into()),
        "footer-projected winter 2040 -> EST"
    );
    assert_eq!(
        observed(&z, instant(2040, 7, 15, 12, 0, 0)),
        (-14400, true, "EDT".into()),
        "footer-projected summer 2040 -> EDT"
    );
}

#[test]
fn tzrs_sees_the_mid_dst_boundary_one_hour_trap() {
    // The MidDst era ends mid-DST at 1990-07-01 04:00:00Z: EDT -> AST, both utoff -14400 but a
    // real type change (DST flag + abbreviation differ). A consumer must observe the change.
    let z = tzif("fixtures/minimal/multi_mid.zi", "Test/MidDst");
    assert_eq!(
        observed(&z, instant(1990, 7, 1, 3, 59, 59)),
        (-14400, true, "EDT".into())
    );
    assert_eq!(
        observed(&z, instant(1990, 7, 1, 4, 0, 0)),
        (-14400, false, "AST".into())
    );
}

#[test]
fn tzrs_reads_europe_london_history_and_footer() {
    // The first real IANA slice. Check a historical summer (BST), and the footer-projected
    // future (2030: GMT in winter, BST in summer) — proving tz-rs reads zic-rs's EU footer.
    let z = tzif(
        "fixtures/iana-slices/europe_london_2026b.zi",
        "Europe/London",
    );
    assert_eq!(
        observed(&z, instant(1980, 7, 1, 12, 0, 0)),
        (3600, true, "BST".into()),
        "historical British Summer Time"
    );
    assert_eq!(
        observed(&z, instant(2030, 1, 15, 12, 0, 0)),
        (0, false, "GMT".into()),
        "footer-projected winter -> GMT"
    );
    assert_eq!(
        observed(&z, instant(2030, 7, 15, 12, 0, 0)),
        (3600, true, "BST".into()),
        "footer-projected summer -> BST"
    );
}
