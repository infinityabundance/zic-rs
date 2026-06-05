//! Deep `zic`-semantics audit surface — one expressive test per **named law** in
//! [`docs/zic-deep-semantics.md`](../docs/zic-deep-semantics.md).
//!
//! These do not add new coverage so much as make the existing coverage **reviewable by law**: a
//! maintainer can open this file and see, in one place, which obscure `zic` behaviour each test
//! guards and which reference source proves it. Fixture-based tests reuse the curated fixtures in
//! `fixtures/minimal/`; behaviour tests run the real `zdump` oracle over the system `tzdata.zi` and
//! **auto-skip** when the reference tools or the database are absent (keeps CI portable). The
//! detailed per-zone evidence lives in `docs/reference-zic-semantics.md`.

use std::path::PathBuf;

use tzcompile::compare::{compare_zone, reference_zic, CompareMode};
use tzcompile::{compile_zone, load_database};

const SYS_TZDATA: &str = "/usr/share/zoneinfo/tzdata.zi";

fn fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn compiled(source: &str, zone: &str) -> tzcompile::tzif::TzifData {
    let db = load_database(&[fixture(source)]).expect("load");
    compile_zone(&db, zone).expect("compile")
}

/// `Some(is_match)` from the `zdump` oracle for a fixture zone, or `None` if reference tools absent.
fn fixture_oracle(source: &str, zone: &str, lo: i32, hi: i32) -> Option<bool> {
    if !reference_zic::is_available("zic") || !reference_zic::is_available("zdump") {
        eprintln!("skipping: reference `zic`/`zdump` not on PATH");
        return None;
    }
    let inputs = vec![fixture(source)];
    let db = load_database(&inputs).unwrap();
    let work = tempfile::tempdir().unwrap();
    let mode = CompareMode::Zdump {
        program: "zdump".to_string(),
        lo,
        hi,
    };
    Some(
        compare_zone(&db, &inputs, zone, "zic", work.path(), &mode)
            .unwrap()
            .is_match(),
    )
}

/// `Some(is_match)` for a system-`tzdata.zi` zone, or `None` if the file / tools are unavailable.
fn system_match(zone: &str, lo: i32, hi: i32) -> Option<bool> {
    if !std::path::Path::new(SYS_TZDATA).exists()
        || !reference_zic::is_available("zic")
        || !reference_zic::is_available("zdump")
    {
        eprintln!("skipping: system tzdata.zi or reference tools unavailable");
        return None;
    }
    let inputs = vec![PathBuf::from(SYS_TZDATA)];
    let db = load_database(&inputs).unwrap();
    let work = tempfile::tempdir().unwrap();
    let mode = CompareMode::Zdump {
        program: "zdump".to_string(),
        lo,
        hi,
    };
    Some(
        compare_zone(&db, &inputs, zone, "zic", work.path(), &mode)
            .unwrap()
            .is_match(),
    )
}

fn assert_all_match(zones: &[&str], lo: i32, hi: i32) {
    for z in zones {
        if let Some(m) = system_match(z, lo, hi) {
            assert!(m, "{z}: must zdump-match reference zic over {lo}..{hi}");
        }
    }
}

// ── Law 1 — a Zone is a state machine over eras (covered structurally below) ──────────────────

/// **Law 2 — `UNTIL` is outgoing-era state, not an absolute literal.** `Test/MidDst`'s era ends at
/// `1990 Jul 1 0:00` *wall*, converted through the outgoing era while DST is active, so the
/// boundary lands at 1990-07-01 04:00Z (= local − stdoff(−5) − save(1)), switching EDT → AST.
#[test]
fn until_uses_outgoing_era_context() {
    let d = compiled("fixtures/minimal/multi_mid.zi", "Test/MidDst");
    let t = d
        .transitions
        .iter()
        .find(|t| t.at == 646_804_800)
        .expect("era boundary at 1990-07-01 04:00Z");
    let ty = &d.types[t.type_index as usize];
    assert_eq!(
        (ty.utoff, ty.is_dst, ty.abbr.as_str()),
        (-14400, false, "AST")
    );
}

/// **Law 3 — `Rule AT` / `Zone UNTIL` wall/standard/universal frames must be normalized.** The five
/// residual zones each mix references across an era boundary (wall UNTIL vs `2s`/`1u` rule AT); the
/// fix compares resolved instants, not raw local seconds. (T5 #5.)
#[test]
fn wall_standard_universal_refs_are_normalized() {
    assert_all_match(
        &[
            "Asia/Tashkent",
            "Asia/Ashgabat",
            "Asia/Anadyr",
            "Europe/Simferopol",
            "Europe/Warsaw",
        ],
        1900,
        2040,
    );
}

/// **Law 4 — era-boundary ↔ rule-transition coincidences collapse to ONE transition** ("Menominee
/// law"), via exact normalized clock-frame equality (no proximity tolerance). Moscow/Tashkent spring
/// *at* the offset-drop boundary rather than a standard hour then a separate spring. (T5 #3/#5.)
#[test]
fn era_boundary_coincidences_collapse_to_one_transition_without_tolerance() {
    assert_all_match(&["Europe/Moscow", "Asia/Tashkent"], 1900, 2040);
}

/// **Law 5 — a named rule set has its own timeline across eras.** Phoenix's 1944 `US M%sT` era
/// starts in War time (Rule US set War in 1942, cleared 1945); Bermuda's 1930 era inherits `LETTER=S`
/// (AST) from a 1918 activation. (T5 #4.)
#[test]
fn prior_rule_state_carries_across_eras() {
    assert_all_match(&["America/Phoenix", "Atlantic/Bermuda"], 1900, 2040);
}

/// **Law 6 — `LETTER/S` is temporal rule state.** New Zealand's `1946 Jan 1 0 0 S` rule (SAVE=0)
/// changes the standard abbreviation NZMT → NZST at the era boundary — a `SAVE=0` row that still
/// changes the letter. (T5 #1.)
#[test]
fn letter_s_is_temporal_state() {
    assert_all_match(&["Pacific/Auckland"], 1900, 2040);
}

/// **Law 6 (corollary) — a `SAVE=0` rule can change the abbreviation.** Same Auckland proof, named
/// for the specific trap a "save + bool" rule model would miss.
#[test]
fn save_zero_rule_can_change_abbreviation() {
    assert_all_match(&["Pacific/Auckland"], 1940, 1960);
}

/// **Law 8/9 — inline `SAVE` builds a fixed type from STDOFF+SAVE; `%z` renders the EFFECTIVE
/// offset.** `Test/InlineZ` (`7:00 0:20 %z`) → abbr `+0720` from the *total* offset (26400s, dst),
/// not from STDOFF alone. (T3.2b.)
#[test]
fn inline_save_percent_z_uses_effective_offset() {
    let d = compiled("fixtures/minimal/inline.zi", "Test/InlineZ");
    let ty = d
        .types
        .iter()
        .find(|t| t.abbr == "+0720")
        .expect("an inline-save type rendered as +0720");
    assert_eq!((ty.utoff, ty.is_dst), (26400, true));
}

/// **Law 13 — the POSIX footer is the post-tail behaviour contract.** `Test/Eastern`'s recurring US
/// rule is summarised exactly as `EST5EDT,M3.2.0,M11.1.0`; `zdump` projects future transitions from
/// it. (T3.)
#[test]
fn footer_drives_post_tail_behavior() {
    let d = compiled("fixtures/minimal/eastern.zi", "Test/Eastern");
    assert_eq!(d.footer, "EST5EDT,M3.2.0,M11.1.0");
}

/// **Compatibility law — `FROM = minimum` lowers to 1900** (obsolete synonym, not infinite past).
/// `Test/MinRule` matches reference `zic` across the 1899..1910 boundary. (T3.2a.)
#[test]
fn minimum_lowers_to_1900() {
    if let Some(m) = fixture_oracle("fixtures/minimal/minrule.zi", "Test/MinRule", 1899, 1910) {
        assert!(
            m,
            "FROM=minimum must compile as 1900 and match reference zic"
        );
    }
}

/// **Law 7 — `SAVE` is signed state; negative inline `SAVE` is now SUPPORTED.** `effective_offset =
/// STDOFF + SAVE` and `is_dst = (save != 0)`, exactly as reference `zic` renders Europe/Prague's
/// 1946–47 `1 -1 GMT` era (→ `GMT`, isdst=1, gmtoff 0). Implemented as first-class signed SAVE (not a
/// per-zone exception); cleared the Prague fail-closed bucket → 339/0/2. `%s`/slash inline still fail.
#[test]
fn negative_inline_save_is_signed_state_matches_reference_zic() {
    // Hermetic: the signed effective offset on a crafted era.
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("t.zi");
    std::fs::write(&p, "Zone Test/Neg 8:00 -1:00 XYZ\n").unwrap();
    let db = load_database(&[p]).expect("parses");
    let d = compile_zone(&db, "Test/Neg").expect("negative inline SAVE now compiles");
    assert!(
        d.types.iter().any(|t| t.utoff == 25200 && t.is_dst),
        "8:00 + (-1:00) = 7:00, save != 0 → dst"
    );
    // Real zone: Europe/Prague behaviour-matches reference zic over the horizon.
    assert_all_match(&["Europe/Prague"], 1900, 2040);
}

/// **Law 10 — neighbouring-month `ON` / non-POSIX recurring day form, now SUPPORTED.**
/// `Asia/Gaza`/`Asia/Hebron`'s perpetual Palestine rule uses `Sat<=30`; `zic` re-anchors it onto a
/// clean nth-weekday with the skipped days folded into the transition time (`Sat<=30 02:00` →
/// `M3.4.4/50` = 4th Thursday + 50h), which needs the **v3** footer. Both zones now compile (v3) and
/// zdump-match reference `zic` — the last canonical-zone behaviour bucket → **341/341**. Auto-skips
/// without system tzdata.
#[test]
fn neighboring_month_on_form_matches_reference_zic() {
    assert_all_match(&["Asia/Gaza", "Asia/Hebron"], 1900, 2040);
}
