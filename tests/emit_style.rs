//! Tests for `--emit-style` (campaign T8-slim).
//!
//! The contract: `EmitStyle::Default` is the behaviour-matched, CORE.1-gated output (never altered
//! by this feature); `EmitStyle::ZicSlim` reproduces reference `zic`'s slim explicit-transition set
//! by dropping the footer-governed recurring tail; `EmitStyle::ZicFat` currently aliases `Default`.
//! Every mode is `zdump`-equivalent — slim only changes *how many* explicit transitions precede the
//! POSIX footer, never the behaviour the footer + transitions describe.
//!
//! The decisive cross-checks (slim `timecnt` == reference `zic`, behaviour 341/0/0 in both modes)
//! live in the full sweep + `structural-report --emit-style zic-slim`; these tests pin the
//! library-level invariants and auto-skip when the installed `tzdata.zi` is absent.

use std::path::PathBuf;

use tzcompile::{compile_zone_styled, load_database, EmitOptions, EmitStyle};

fn tzdata() -> Option<PathBuf> {
    let p = PathBuf::from("/usr/share/zoneinfo/tzdata.zi");
    p.exists().then_some(p)
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/minimal")
        .join(name)
}

/// Decode a parsed TZif to its behaviour-bearing sequence `(at, utoff, is_dst, abbr)` — robust to
/// type-table **index ordering** (zic-rs and `zic` order equal-rank types differently; behaviour and
/// counts are what we pin, not the index permutation).
fn decoded(p: &tzcompile::tzif::ParsedTzif) -> Vec<(i64, i32, bool, String)> {
    p.transitions
        .iter()
        .map(|t| {
            let ty = &p.types[t.type_index as usize];
            (t.at, ty.utoff, ty.is_dst, ty.abbr.clone())
        })
        .collect()
}

/// T10.4d acceptance: each reference `zic -r` microcase fixture
/// ([`fixtures/range/reference`]) must be reproduced — same decoded transition sequence, footer,
/// version, and `-00` placeholder. (Raw type-index order may differ; see [`decoded`].)
#[test]
fn range_microcases_match_reference_fixtures() {
    let Some(src) = tzdata() else {
        eprintln!("skipping: /usr/share/zoneinfo/tzdata.zi not present");
        return;
    };
    let db = load_database(&[src]).unwrap();
    let ref_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/range/reference");
    // (zone, lo, hi, fixture)
    let cases: &[(&str, Option<i64>, Option<i64>, &str)] = &[
        (
            "America/New_York",
            Some(946_684_800),
            None,
            "ny.lo-2000.tzif",
        ),
        (
            "Etc/UTC",
            Some(946_684_800),
            Some(1_577_836_800),
            "utc.win-2000-2020.tzif",
        ),
        (
            "Europe/London",
            Some(157_766_400),
            None,
            "london.lo-1975.tzif",
        ),
        ("Asia/Gaza", Some(1_577_836_800), None, "gaza.lo-2020.tzif"),
        (
            "Etc/GMT+5",
            Some(946_684_800),
            Some(1_577_836_800),
            "gmtplus5.win-2000-2020.tzif",
        ),
        // T10.4f regression: a multi-era zone with an interior special era (2010–11 constant-AEDT
        // inline-save one-off). Open-ended `-r @0` previously over-truncated the slim tail (ours=77 <
        // ref=82, a *non*-prefix → the footer wrongly projected a 2010 fall-back); the shape-(a)
        // forward-expansion fix keeps reference's transitions as a prefix.
        (
            "Antarctica/Macquarie",
            Some(0),
            None,
            "macquarie.lo-1970.tzif",
        ),
    ];
    for (zone, lo, hi, fixture) in cases {
        let ours_bytes = tzcompile::compile_zone_to_bytes_styled(
            &db,
            zone,
            ranged(EmitStyle::ZicSlim, *lo, *hi),
        )
        .unwrap();
        let ours = tzcompile::tzif::parse(&ours_bytes).unwrap();
        let ref_bytes = std::fs::read(ref_dir.join(fixture)).unwrap();
        let reference = tzcompile::tzif::parse(&ref_bytes).unwrap();
        assert_eq!(ours.version, reference.version, "{zone}: version");
        assert_eq!(ours.footer, reference.footer, "{zone}: footer");
        assert!(
            ours.types.iter().any(|t| t.abbr == "-00"),
            "{zone}: `-00` present"
        );
        // First transition: at exactly `lo`, to the prevailing real type (the `-r` start boundary).
        if let Some(l) = lo {
            assert_eq!(ours.transitions[0].at, *l, "{zone}: boundary at lo");
            let ot = &ours.types[ours.transitions[0].type_index as usize];
            let rt = &reference.types[reference.transitions[0].type_index as usize];
            assert_eq!(
                (ot.utoff, ot.is_dst),
                (rt.utoff, rt.is_dst),
                "{zone}: prevailing at lo"
            );
        }
        // Reference's decoded transitions must be an exact **prefix** of ours: ours never drops or
        // reorders anything reference keeps. Ours may keep a *few* extra explicit transitions at the
        // footer-takeover slim boundary (behaviourally null — the POSIX footer reproduces them; full
        // `zdump` parity is verified by the external profile sweep, all three profiles 341/341). We do
        // not claim byte/`timecnt` parity at that boundary, per the project's slim-residual policy
        // (e.g. `Europe/London` keeps one extra 1996 spring vs reference, same class as `Europe/Lisbon`).
        let (o, r) = (decoded(&ours), decoded(&reference));
        assert!(
            o.len() >= r.len() && o[..r.len()] == r[..],
            "{zone}: reference transitions must be a prefix of ours (ours={}, ref={})",
            o.len(),
            r.len()
        );
        assert!(
            o.len() - r.len() <= 2,
            "{zone}: only a small slim-boundary residual allowed (ours={}, ref={})",
            o.len(),
            r.len()
        );
    }
}

fn ranged(style: EmitStyle, lo: Option<i64>, hi: Option<i64>) -> EmitOptions {
    EmitOptions {
        style,
        redundant_until: None,
        range: Some(tzcompile::RangeSpec { lo, hi }),
        allow_empty_footer_on_legacy_nonposix_recurrence: false,
    }
}

// --- T10.4d: `-r` range truncation mechanics (deterministic, fixtures/minimal) -------------------

/// Start truncation adds a leading `-00` (type 0) and a boundary transition at exactly `lo` to the
/// **prevailing type at `lo`** — for a single-era rule zone the full stream is expanded, so this is
/// exact. `Test/Eastern` (Rule US 2007..max, EST/EDT).
#[test]
fn range_lo_adds_unspecified_and_boundary_at_lo() {
    let db = load_database(&[fixture("eastern.zi")]).unwrap();
    let lo = 1_262_304_000; // 2010-01-01 (winter → prevailing EST)
    let d = compile_zone_styled(
        &db,
        "Test/Eastern",
        ranged(EmitStyle::ZicSlim, Some(lo), None),
    )
    .unwrap();
    assert_eq!(
        d.types[0].abbr, "-00",
        "type 0 must be the `-00` placeholder"
    );
    assert_eq!(d.types[0].utoff, 0);
    let first = &d.transitions[0];
    assert_eq!(first.at, lo, "first transition is at exactly lo");
    assert_eq!(
        d.types[first.type_index as usize].abbr, "EST",
        "winter lo → prevailing EST"
    );
    assert!(!d.footer.is_empty(), "unbounded hi keeps the footer");
}

/// The load-bearing invariant: the boundary type at `lo` is the type **in effect at `lo`** — NOT the
/// first transition after `lo`. A summer `lo` (DST active) must boundary to EDT, even though the next
/// real transition is the autumn fall-back to EST.
#[test]
fn range_prevailing_type_at_lo_not_next_transition() {
    let db = load_database(&[fixture("eastern.zi")]).unwrap();
    let lo = 1_277_942_400; // 2010-07-01 (summer → prevailing EDT)
    let d = compile_zone_styled(
        &db,
        "Test/Eastern",
        ranged(EmitStyle::ZicSlim, Some(lo), None),
    )
    .unwrap();
    let first = &d.transitions[0];
    assert_eq!(first.at, lo);
    assert!(
        d.types[first.type_index as usize].is_dst,
        "summer lo → prevailing must be DST (EDT), not the next (EST) transition"
    );
    assert_eq!(d.types[first.type_index as usize].abbr, "EDT");
}

/// Bounded `hi` appends a trailing transition at exactly the parsed `@hi` back to `-00`, and clears
/// the footer (no proleptic rule past a bounded end).
#[test]
fn range_bounded_hi_trailing_unspecified_and_empty_footer() {
    let db = load_database(&[fixture("eastern.zi")]).unwrap();
    let (lo, hi) = (1_262_304_000, 1_356_998_400); // 2010-01-01 .. 2013-01-01
    let d = compile_zone_styled(
        &db,
        "Test/Eastern",
        ranged(EmitStyle::ZicSlim, Some(lo), Some(hi)),
    )
    .unwrap();
    let last = d.transitions.last().unwrap();
    assert_eq!(last.at, hi, "trailing transition at exactly the parsed @hi");
    assert_eq!(
        d.types[last.type_index as usize].abbr, "-00",
        "trailing type is `-00`"
    );
    assert!(d.footer.is_empty(), "bounded hi clears the footer");
}

/// The hardening invariant: for a nonzero fixed-offset zone, the synthetic `-00` (utoff 0) stays a
/// **distinct** type from the real offset — `-00` never masquerades as the real fixed offset.
/// `Test/Fixed` is `-5:00` (`EST`, utoff −18000) with no transitions.
#[test]
fn range_fixed_nonzero_offset_distinct_from_unspecified() {
    let db = load_database(&[fixture("fixed.zi")]).unwrap();
    let (lo, hi) = (946_684_800, 1_577_836_800);
    let d = compile_zone_styled(
        &db,
        "Test/Fixed",
        ranged(EmitStyle::ZicSlim, Some(lo), Some(hi)),
    )
    .unwrap();
    assert!(
        d.types.iter().any(|t| t.abbr == "-00" && t.utoff == 0),
        "the `-00` placeholder (utoff 0) must be present"
    );
    assert!(
        d.types.iter().any(|t| t.utoff == -18000 && t.abbr != "-00"),
        "the real -05 offset must be retained, distinct from -00"
    );
    // The in-range slot is the real offset, the out-of-range boundary is `-00`.
    assert_eq!(d.types[d.transitions[0].type_index as usize].utoff, -18000);
    assert_eq!(
        d.types[d.transitions.last().unwrap().type_index as usize].abbr,
        "-00"
    );
}

// --- T10.3: `-R @hi` redundant tail (deterministic, fixture-based — independent of `-b`) ---------

/// `-R @hi` under slim keeps the footer-governed transitions out to `@hi` — **more** than pure slim,
/// but **fewer** than fat (it does not "become fat"), and the footer + TZif version are unchanged.
/// `Test/Eastern` is purely recurring (Rule US 2007..max), so slim keeps just the anchor; `@2020`
/// keeps ~2007..2020; fat keeps ~2007..2037.
#[test]
fn redundant_until_widens_slim_without_reaching_fat() {
    let db = load_database(&[fixture("eastern.zi")]).unwrap();
    let z = "Test/Eastern";
    let slim = compile_zone_styled(&db, z, EmitStyle::ZicSlim).unwrap();
    let fat = compile_zone_styled(&db, z, EmitStyle::ZicFat).unwrap();
    let hi = 1_577_836_800; // 2020-01-01T00:00:00Z
    let slim_r = compile_zone_styled(
        &db,
        z,
        EmitOptions {
            style: EmitStyle::ZicSlim,
            redundant_until: Some(hi),
            range: None,
            allow_empty_footer_on_legacy_nonposix_recurrence: false,
        },
    )
    .unwrap();
    assert!(
        slim.transitions.len() < slim_r.transitions.len(),
        "`-R` must keep more than pure slim ({} vs {})",
        slim_r.transitions.len(),
        slim.transitions.len()
    );
    assert!(
        slim_r.transitions.len() < fat.transitions.len(),
        "`-R @2020` must not reach fat ({} vs {})",
        slim_r.transitions.len(),
        fat.transitions.len()
    );
    // It widens slim, never changes the footer or version (it is redundant with the footer).
    assert_eq!(slim_r.footer, slim.footer);
    assert_eq!(slim_r.version, slim.version);
    // Every kept transition is at or before the requested bound.
    assert!(slim_r.transitions.iter().all(|t| t.at <= hi));
}

/// `-R` is a no-op on a zone with no footer-governed tail (a fixed-offset zone): slim == slim+R.
#[test]
fn redundant_until_no_op_on_fixed_zone() {
    let db = load_database(&[fixture("fixed.zi")]).unwrap();
    let z = "Test/Fixed";
    let slim = compile_zone_styled(&db, z, EmitStyle::ZicSlim).unwrap();
    let slim_r = compile_zone_styled(
        &db,
        z,
        EmitOptions {
            style: EmitStyle::ZicSlim,
            redundant_until: Some(1_577_836_800),
            range: None,
            allow_empty_footer_on_legacy_nonposix_recurrence: false,
        },
    )
    .unwrap();
    assert_eq!(slim.transitions.len(), slim_r.transitions.len());
    assert_eq!(slim.footer, slim_r.footer);
}

/// `-R` is independent of `-b`: applied to the fat-style default it does nothing (everything is
/// already kept), so the output equals the plain default.
#[test]
fn redundant_until_is_no_op_under_fat_default() {
    let db = load_database(&[fixture("eastern.zi")]).unwrap();
    let z = "Test/Eastern";
    let def = compile_zone_styled(&db, z, EmitStyle::Default).unwrap();
    let def_r = compile_zone_styled(
        &db,
        z,
        EmitOptions {
            style: EmitStyle::Default,
            redundant_until: Some(1_577_836_800),
            range: None,
            allow_empty_footer_on_legacy_nonposix_recurrence: false,
        },
    )
    .unwrap();
    assert_eq!(def.transitions.len(), def_r.transitions.len());
}

/// For a genuinely *mixed-in-era* zone (finite history + a recurring tail), slim emits strictly
/// fewer explicit transitions than the default fat expansion, while the **footer and TZif version
/// are identical** — the dropped transitions are exactly the ones the footer reproduces.
#[test]
fn slim_is_fatter_zones_truncated_footer_unchanged() {
    let Some(src) = tzdata() else {
        eprintln!("skipping: /usr/share/zoneinfo/tzdata.zi not present");
        return;
    };
    let db = load_database(&[src]).unwrap();
    for zone in ["America/New_York", "Pacific/Auckland", "America/Adak"] {
        let def = compile_zone_styled(&db, zone, EmitStyle::Default).unwrap();
        let slim = compile_zone_styled(&db, zone, EmitStyle::ZicSlim).unwrap();
        assert!(
            slim.transitions.len() < def.transitions.len(),
            "{zone}: slim ({}) should drop the footer-governed tail vs default ({})",
            slim.transitions.len(),
            def.transitions.len()
        );
        // The footer is the post-tail contract — it must be byte-identical across styles, since
        // slim *relies* on it to reproduce the dropped transitions.
        assert_eq!(
            def.footer, slim.footer,
            "{zone}: footer must not change with style"
        );
        assert_eq!(
            def.version, slim.version,
            "{zone}: version must not change with style"
        );
        // Pruning must leave a consistent type table: every transition's type index is in range.
        assert!(
            slim.transitions
                .iter()
                .all(|t| (t.type_index as usize) < slim.types.len()),
            "{zone}: slim type indices must stay in range after pruning"
        );
        assert!(
            !slim.types.is_empty(),
            "{zone}: type 0 (initial) is always kept"
        );
    }
}

/// `ZicFat` currently aliases `Default` — identical output (documented in `EmitStyle`).
#[test]
fn zic_fat_aliases_default() {
    let Some(src) = tzdata() else {
        eprintln!("skipping: /usr/share/zoneinfo/tzdata.zi not present");
        return;
    };
    let db = load_database(&[src]).unwrap();
    for zone in ["America/New_York", "Europe/London", "Etc/UTC"] {
        let def = compile_zone_styled(&db, zone, EmitStyle::Default).unwrap();
        let fat = compile_zone_styled(&db, zone, EmitStyle::ZicFat).unwrap();
        assert_eq!(def.transitions.len(), fat.transitions.len(), "{zone}");
        assert_eq!(def.footer, fat.footer, "{zone}");
    }
}

/// A zone whose final era is *effectively recurring-only* (shape-(a)) already emits the minimal
/// anchor+footer, so slim is a no-op there — default and slim are identical.
#[test]
fn shape_a_zone_unchanged_by_slim() {
    let Some(src) = tzdata() else {
        eprintln!("skipping: /usr/share/zoneinfo/tzdata.zi not present");
        return;
    };
    let db = load_database(&[src]).unwrap();
    for zone in ["Europe/London", "Europe/Moscow"] {
        let def = compile_zone_styled(&db, zone, EmitStyle::Default).unwrap();
        let slim = compile_zone_styled(&db, zone, EmitStyle::ZicSlim).unwrap();
        assert_eq!(
            def.transitions.len(),
            slim.transitions.len(),
            "{zone}: already-slim shape-(a) zone must be unchanged by --emit-style zic-slim"
        );
    }
}

/// Determinism: compiling the same zone+style twice yields identical transition counts (the writer
/// has no time/RNG/host state). A light guard alongside the structural-report determinism checks.
#[test]
fn slim_is_deterministic() {
    let Some(src) = tzdata() else {
        eprintln!("skipping: /usr/share/zoneinfo/tzdata.zi not present");
        return;
    };
    let db = load_database(&[src]).unwrap();
    let a = compile_zone_styled(&db, "America/New_York", EmitStyle::ZicSlim).unwrap();
    let b = compile_zone_styled(&db, "America/New_York", EmitStyle::ZicSlim).unwrap();
    assert_eq!(a.transitions.len(), b.transitions.len());
    assert_eq!(a.footer, b.footer);
}
