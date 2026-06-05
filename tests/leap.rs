//! Leap-second emission (T11.3 Stationary · T11.4 Rolling · T11.5 Expires/v4).
//!
//! Reproduces the reference `zic -L` leap fixtures (`fixtures/leap/reference/`): compile a zone, apply
//! a parsed leap table, and assert the emitted TZif **leap-correction table** matches reference —
//! `leapcnt`, cumulative corrections, (adjusted) occurrence instants, the Rolling local-wall offset,
//! and the Expires→v4 no-op marker — while the ordinary transition/type arrays are **unchanged**
//! (leaps are orthogonal). Acceptance: *leap-source tables emit reference-matching TZif leap
//! correction entries without changing ordinary zone-source compilation or local-time-type semantics.*

use std::path::{Path, PathBuf};

use tzcompile::compile::apply_leaps;
use tzcompile::model::calendar::days_from_civil;
use tzcompile::source::parse_leap_source;
use tzcompile::{compile_zone, load_database, tzif};

fn fx(p: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(p)
}

fn parsed(path: PathBuf) -> tzif::ParsedTzif {
    tzif::parse(&std::fs::read(path).unwrap()).unwrap()
}

/// `Test/East5` (+5) with one Stationary leap → reference `stationary-east5.tzif`. Critically, the
/// leap occurrence is **UT-as-written** — *not* shifted by the +5h zone offset (that is what
/// distinguishes Stationary from Rolling).
#[test]
fn stationary_leap_emits_reference_table_ut_as_written() {
    let db = load_database(&[fx("fixtures/leap/src/east5.zi")]).unwrap();
    let table = parse_leap_source(
        &std::fs::read(fx("fixtures/leap/src/stationary.list")).unwrap(),
        Path::new("stationary.list"),
    )
    .unwrap();

    let mut data = compile_zone(&db, "Test/East5").unwrap();
    let (timecnt, typecnt) = (data.transitions.len(), data.types.len());
    apply_leaps(&mut data, &table, None).unwrap();

    // Orthogonality: applying leaps does not touch the transition/type arrays.
    assert_eq!(
        data.transitions.len(),
        timecnt,
        "leaps must not add transitions"
    );
    assert_eq!(data.types.len(), typecnt, "leaps must not add types");

    let ours = tzif::parse(&tzif::write_bytes(&data).unwrap()).unwrap();
    let reference = parsed(fx("fixtures/leap/reference/stationary-east5.tzif"));

    assert_eq!(
        ours.leaps, reference.leaps,
        "leap table must match reference"
    );
    assert_eq!(ours.version, reference.version, "version (v2, no expiry)");
    assert_eq!(ours.leaps.len(), 1);
    // UT-as-written: 2016-12-31 23:59:60 ≡ 2017-01-01 00:00:00 UT, NOT shifted by the zone's +5h.
    assert_eq!(ours.leaps[0].trans, days_from_civil(2017, 1, 1) * 86_400);
    assert_eq!(ours.leaps[0].corr, 1);
}

/// The full IANA `leapseconds` (27 Stationary `+1` leaps) compiled into `UTC` must reproduce
/// `right/UTC`'s leap table (cumulative `+1…+27`). Skips if the system file is absent.
#[test]
fn stationary_utc_full_table_matches_right_utc() {
    let leapfile = Path::new("/usr/share/zoneinfo/leapseconds");
    if !leapfile.exists() {
        eprintln!("skipping: /usr/share/zoneinfo/leapseconds not present");
        return;
    }
    let db = load_database(&[fx("fixtures/leap/src/utc.zi")]).unwrap();
    let table = parse_leap_source(&std::fs::read(leapfile).unwrap(), leapfile).unwrap();
    let mut data = compile_zone(&db, "Test/UTC").unwrap();
    apply_leaps(&mut data, &table, None).unwrap();

    let ours = tzif::parse(&tzif::write_bytes(&data).unwrap()).unwrap();
    let reference = parsed(fx("fixtures/leap/reference/stationary-utc.tzif"));

    assert_eq!(
        ours.leaps, reference.leaps,
        "27-leap table must match right/UTC"
    );
    assert_eq!(ours.version, reference.version, "v2 (no expiry/truncation)");
    // Cumulative corrections 1..=27.
    assert_eq!(ours.leaps.len(), 27);
    assert_eq!(ours.leaps[0].corr, 1);
    assert_eq!(ours.leaps[26].corr, 27);
}

/// `Test/East5` (+5) with one **Rolling** leap → reference `rolling-east5.tzif`. The occurrence is
/// `local-wall`, so it is stored as the Stationary instant **minus the +5h offset** (T11.4 Part A).
#[test]
fn rolling_leap_subtracts_zone_offset() {
    let db = load_database(&[fx("fixtures/leap/src/east5.zi")]).unwrap();
    let table = parse_leap_source(
        &std::fs::read(fx("fixtures/leap/src/rolling.list")).unwrap(),
        Path::new("rolling.list"),
    )
    .unwrap();
    let mut data = compile_zone(&db, "Test/East5").unwrap();
    apply_leaps(&mut data, &table, None).unwrap();

    let ours = tzif::parse(&tzif::write_bytes(&data).unwrap()).unwrap();
    let reference = parsed(fx("fixtures/leap/reference/rolling-east5.tzif"));
    assert_eq!(
        ours.leaps, reference.leaps,
        "Rolling leap table must match reference"
    );
    // Rolling = Stationary − offset: the +5h zone shifts the stored occurrence 18000 s earlier.
    assert_eq!(
        ours.leaps[0].trans,
        days_from_civil(2017, 1, 1) * 86_400 - 18_000
    );
    assert_eq!(ours.leaps[0].corr, 1);
}

/// `Test/UTC` with `Leap 2016…` + `Expires 2025` → reference `v4-expires.tzif`: the leap table gains
/// a **no-op expiry marker** and the file becomes **TZif v4** (content-triggered, T11.5).
#[test]
fn expires_emits_v4_with_noop_marker() {
    let db = load_database(&[fx("fixtures/leap/src/utc.zi")]).unwrap();
    let table = parse_leap_source(
        &std::fs::read(fx("fixtures/leap/src/expires.list")).unwrap(),
        Path::new("expires.list"),
    )
    .unwrap();
    let mut data = compile_zone(&db, "Test/UTC").unwrap();
    apply_leaps(&mut data, &table, None).unwrap();

    let ours = tzif::parse(&tzif::write_bytes(&data).unwrap()).unwrap();
    let reference = parsed(fx("fixtures/leap/reference/v4-expires.tzif"));
    assert_eq!(ours.version, b'4', "Expires triggers TZif v4");
    assert_eq!(ours.version, reference.version);
    assert_eq!(
        ours.leaps, reference.leaps,
        "v4 leap table (real + expiry) must match reference"
    );
    // 2 entries: the real 2017 leap, then the no-op expiry marker (correction unchanged).
    assert_eq!(ours.leaps.len(), 2);
    assert_eq!(ours.leaps[1].corr, ours.leaps[0].corr);
}

/// Applying no leaps leaves an ordinary zone byte-identical (the orthogonality guarantee at the
/// boundary): a normal compile has an empty leap table.
#[test]
fn ordinary_zone_has_no_leaps() {
    let db = load_database(&[fx("fixtures/leap/src/east5.zi")]).unwrap();
    let data = compile_zone(&db, "Test/East5").unwrap();
    assert!(data.leaps.is_empty());
    let ours = tzif::parse(&tzif::write_bytes(&data).unwrap()).unwrap();
    assert!(ours.leaps.is_empty());
    assert_eq!(ours.counts.leapcnt, 0);
}
