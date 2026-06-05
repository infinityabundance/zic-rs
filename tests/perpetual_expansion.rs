//! PERPETUAL-EXPANSION.1 — empty-footer fallback for non-POSIX final recurrence.
//!
//! When a zone's final recurring era cannot be reduced to one POSIX DST/standard footer pair (the live
//! case: historical **perpetual year-parity** rules like `1990 max even/odd`, which a POSIX `TZ` string
//! cannot encode), the default **fails closed** (`ZIC001`). `--legacy-empty-footer` instead emits the
//! explicit transitions zic-rs already expands through the replay horizon (`RECUR_HI` = 2037) plus an
//! **empty footer** (frozen beyond) — matching the historical `zic` oracle's behaviour over the declared
//! `[1980, 2037]` window (see `reports/perpetual-expansion/`). It is a **footer-emission policy** only:
//! it never changes transition generation, never the default, and never a footer-synthesising zone.

use std::path::PathBuf;

use tzcompile::source::LegacySource;
use tzcompile::{
    compile_zone_styled, compile_zone_to_bytes_styled, load_database, load_database_with,
    EmitOptions, EmitStyle,
};

fn fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// Default emission, with the empty-footer fallback toggled.
fn emit(empty_footer: bool) -> EmitOptions {
    EmitOptions {
        allow_empty_footer_on_legacy_nonposix_recurrence: empty_footer,
        ..EmitStyle::Default.into()
    }
}

fn load_legacy(rel: &str) -> tzcompile::model::Database {
    load_database_with(
        &[fixture(rel)],
        LegacySource {
            yearistype: true,
            ..Default::default()
        },
    )
    .expect("load legacy")
}

#[test]
fn default_fails_closed_on_nonposix_final_recurrence() {
    let db = load_legacy("fixtures/minimal/perpetual_yearistype.zi");
    // Off by default → a perpetual year-parity tail is refused (the safety property is preserved).
    let err = compile_zone_styled(&db, "Test/Perpetual", emit(false))
        .expect_err("non-POSIX perpetual recurrence must fail closed by default");
    assert!(
        err.to_string().contains("ZIC001"),
        "expected ZIC001 fail-closed, got: {err}"
    );
}

#[test]
fn legacy_empty_footer_emits_empty_footer() {
    let db = load_legacy("fixtures/minimal/perpetual_yearistype.zi");
    let tz = compile_zone_styled(&db, "Test/Perpetual", emit(true))
        .expect("empty-footer fallback compiles the non-POSIX zone");
    assert_eq!(
        tz.footer, "",
        "the fallback emits an EMPTY footer (frozen beyond)"
    );
    // The transitions are not anchor-only: the even/odd rules are explicitly expanded through the
    // replay horizon (decades of annual transitions), so behaviour is real within the window.
    assert!(
        tz.transitions.len() > 50,
        "explicit transitions expanded through RECUR_HI (got {})",
        tz.transitions.len()
    );
}

#[test]
fn legacy_empty_footer_does_not_change_posix_footer_zone() {
    // An ordinary POSIX-recurring zone synthesises a real footer; the flag must be a NO-OP.
    let db = load_legacy("fixtures/minimal/perpetual_yearistype.zi");
    let tz = compile_zone_styled(&db, "Test/Normal", emit(true)).expect("normal zone compiles");
    assert!(
        !tz.footer.is_empty(),
        "a POSIX-recurring zone keeps its real footer even with the flag set"
    );
    let off = compile_zone_to_bytes_styled(&db, "Test/Normal", emit(false)).expect("off");
    let on = compile_zone_to_bytes_styled(&db, "Test/Normal", emit(true)).expect("on");
    assert_eq!(
        off, on,
        "the flag does not change a footer-synthesising zone's bytes"
    );
}

#[test]
fn core_fixture_byte_unchanged_under_the_flag() {
    // A real canonical recurring zone (Europe/London) is byte-identical with the flag on vs off —
    // the flag only touches the synthesis-FAILURE path, which CORE.1 zones never reach.
    let db =
        load_database(&[fixture("fixtures/iana-slices/europe_london_2026b.zi")]).expect("load");
    let off = compile_zone_to_bytes_styled(&db, "Europe/London", emit(false)).expect("off");
    let on = compile_zone_to_bytes_styled(&db, "Europe/London", emit(true)).expect("on");
    assert_eq!(
        off, on,
        "Europe/London bytes must be identical with --legacy-empty-footer on vs off"
    );
    // And it has a real (non-empty) footer either way — it never takes the fallback.
    let tz = compile_zone_styled(&db, "Europe/London", emit(true)).expect("compile");
    assert!(!tz.footer.is_empty());
}
