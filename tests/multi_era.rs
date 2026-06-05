//! Multi-era (`UNTIL` continuation) zone tests — the T3.1 milestone.
//!
//! These pin the five cross-era correctness traps and the recurring-final-era anchor
//! behaviour discovered against reference `zic` (see `docs/reference-zic-semantics.md`). The
//! test *names* are deliberately descriptive so a reviewer sees exactly which subtle zic
//! behaviour each one guards. Library-level assertions always run; oracle assertions
//! auto-skip when `zic`/`zdump` are absent.

use std::path::PathBuf;

use tzcompile::compare::{compare_zone, reference_zic, CompareMode};
use tzcompile::model::calendar::{civil_from_days, year_of_unix};
use tzcompile::{compile_zone, compile_zone_to_bytes, load_database};

/// `(year, month, day)` of a UT transition instant — for asserting *which* rule governed a
/// historical transition (e.g. the finite April onset vs. the footer's March rule).
fn ymd(at: i64) -> (i32, u8, u8) {
    civil_from_days(at.div_euclid(86_400))
}

fn fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn compile(source: &str, zone: &str) -> tzcompile::tzif::TzifData {
    let db = load_database(&[fixture(source)]).expect("load");
    compile_zone(&db, zone).expect("compile")
}

/// `UNTIL` belongs to the *ending* era's context, and `1990 Jul 1 0:00` wall is converted
/// while DST is active, so the prevailing save (+1h) is subtracted: the boundary lands at
/// 1990-07-01 04:00Z (= local 00:00 − stdoff(−5h) − save(1h)), not 05:00Z.
#[test]
fn until_wall_time_uses_prevailing_save_when_dst_active() {
    let d = compile("fixtures/minimal/multi_mid.zi", "Test/MidDst");
    // 1990-07-01 04:00:00 UTC.
    let boundary = 646_804_800;
    let t = d
        .transitions
        .iter()
        .find(|t| t.at == boundary)
        .expect("era boundary transition at 1990-07-01 04:00Z");
    let ty = &d.types[t.type_index as usize];
    assert_eq!(
        (ty.utoff, ty.is_dst, ty.abbr.as_str()),
        (-14400, false, "AST")
    );
}

/// The MidDst boundary switches EDT → AST: both have `utoff = −14400`, but the DST flag and
/// abbreviation differ, so it is a *real* type change and must not be de-duplicated.
#[test]
fn equal_utoff_boundary_type_change_is_not_deduped() {
    let d = compile("fixtures/minimal/multi_mid.zi", "Test/MidDst");
    let boundary = 646_804_800;
    let pos = d
        .transitions
        .iter()
        .position(|t| t.at == boundary)
        .expect("boundary transition present");
    let after = &d.types[d.transitions[pos].type_index as usize];
    let before = &d.types[d.transitions[pos - 1].type_index as usize];
    assert_eq!(
        before.utoff, after.utoff,
        "equal UT offset across the boundary"
    );
    assert!(
        before.is_dst && !after.is_dst,
        "DST flag changes (EDT -> AST)"
    );
    assert_eq!((before.abbr.as_str(), after.abbr.as_str()), ("EDT", "AST"));
}

/// The POSIX footer is synthesised from the *final* era only (here the fixed `-6:00 CST`
/// era), never an earlier era.
#[test]
fn footer_comes_from_final_era() {
    let d = compile("fixtures/minimal/multi_ff.zi", "Test/FF");
    assert_eq!(d.footer, "CST6");
}

/// A single-era recurring zone does **not** back-fill DST before the rule's FROM year: the
/// first transition is in 2007 (the US rule's FROM), not earlier.
#[test]
fn single_era_recurring_rule_does_not_backfill_before_from_year() {
    let d = compile("fixtures/minimal/eastern.zi", "Test/Eastern");
    let first = d.transitions.first().expect("at least one transition");
    assert!(
        year_of_unix(first.at) >= 2007,
        "first transition should be 2007+, got {}",
        year_of_unix(first.at)
    );
}

/// A *continuation* recurring final era anchors the footer at the era start, so the
/// recurrence projects from there. The compiled file has exactly one explicit transition
/// (the anchor at 2000-01-01 05:00Z) plus the recurring footer.
#[test]
fn final_continuation_recurring_era_footer_projects_from_era_start() {
    let d = compile("fixtures/minimal/multi_fr.zi", "Test/FR");
    assert_eq!(d.footer, "EST5EDT,M3.2.0,M11.1.0");
    assert_eq!(
        d.transitions.len(),
        1,
        "anchor only — footer covers the tail"
    );
    assert_eq!(d.transitions[0].at, 946_702_800); // 2000-01-01 05:00:00 UTC
}

/// Same anchoring even when the rule's FROM year (2015) is well after the era start (2000):
/// the recurrence is projected from the era start, not the FROM year.
#[test]
fn final_continuation_from_2015_projects_from_era_start() {
    let d = compile("fixtures/minimal/multi_q.zi", "Test/Q");
    assert_eq!(d.footer, "EST5EDT,M3.2.0,M11.1.0");
    assert_eq!(d.transitions.len(), 1);
    assert_eq!(d.transitions[0].at, 946_702_800); // anchor = era start, not 2015
}

/// The final recurring era emits one anchor transition, not a "fat" per-year expansion
/// (which would be dozens of transitions and would wrongly pin pre-recurrence years).
#[test]
fn final_recurring_era_emits_anchor_transition_not_fat_year_expansion() {
    let d = compile("fixtures/minimal/multi_fr.zi", "Test/FR");
    assert_eq!(d.transitions.len(), 1);
}

/// The anchor stops the footer projecting backwards into the previous era: type 0 (the
/// pre-anchor state) is the first era's standard `EST`, and the lone transition is the
/// era-start anchor.
#[test]
fn footer_anchor_prevents_projection_into_previous_era() {
    let d = compile("fixtures/minimal/multi_fr.zi", "Test/FR");
    assert_eq!((d.types[0].utoff, d.types[0].is_dst), (-18000, false));
    assert_eq!(d.types[0].abbr, "EST");
    assert_eq!(d.transitions[0].at, 946_702_800);
}

// --- T3.2c: effective-in-era rule classification (the Europe/London gate). ---

/// A multi-era final era whose rule set is "mixed" only by *raw* membership — finite rows
/// that all end before the era starts, plus recurring rows — must be classified by its
/// *effective in-era* activations. Here the final era starts 1996 and Rule E's finite rows
/// end in 1995, so in-era the set is recurring-only: one forced anchor at the era start
/// (1996-01-01 00:00Z) + the recurring footer, never the old blanket "mixed" rejection.
#[test]
fn final_era_ignores_pre_era_finite_rows_when_classifying_recurring_tail() {
    let d = compile("fixtures/minimal/final_effective.zi", "Test/FinalEffective");
    assert_eq!(d.footer, "GMT0BST,M3.5.0/1,M10.5.0");
    assert_eq!(
        d.transitions.len(),
        1,
        "effective recurring-only final era: anchor only, footer covers the tail"
    );
    assert_eq!(d.transitions[0].at, 820_454_400); // 1996-01-01 00:00:00 UTC (era start)
}

/// A *single-era* zone mixing finite (April-onset, 1977..1995) and recurring (March-onset,
/// 1996..max) rows must emit the **finite history with its real April onsets** — not collapse
/// the historical years into the footer's March rule. (Reference `zic` emits a slimmer
/// explicit set; zic-rs emits a fat one; both are zdump-equivalent — see the slim/fat note in
/// docs/reference-zic-semantics.md. This test guards the *behaviour*: history is honoured.)
#[test]
fn single_era_mixed_historical_rules_are_checked_before_footer_tail() {
    let d = compile("fixtures/minimal/mixed.zi", "Test/Mixed");
    assert_eq!(d.footer, "GMT0BST,M3.5.0/1,M10.5.0");
    // The 1980 spring onset is governed by the finite rule (Apr Sun>=1), not the footer (Mar).
    let onset_1980 = d
        .transitions
        .iter()
        .find(|t| year_of_unix(t.at) == 1980 && d.types[t.type_index as usize].is_dst)
        .expect("a 1980 DST onset transition");
    assert_eq!(
        ymd(onset_1980.at).1,
        4,
        "1980 onset is in April (finite rule), not March"
    );
    // And there is no transition in March 1980 (which the footer's M3 rule would imply).
    assert!(
        !d.transitions.iter().any(|t| {
            let (y, m, _) = ymd(t.at);
            y == 1980 && m == 3
        }),
        "no March 1980 transition — the recurring footer must not govern the finite period"
    );
    // From 1996 the recurring rule takes over: the onset moves to March.
    let onset_2000 = d
        .transitions
        .iter()
        .find(|t| year_of_unix(t.at) == 2000 && d.types[t.type_index as usize].is_dst)
        .expect("a 2000 DST onset transition");
    assert_eq!(
        ymd(onset_2000.at).1,
        3,
        "from 1996 the onset is in March (recurring rule)"
    );
}

/// The genuinely **mixed-in-era** case (T4.1): a final era whose finite rows are *still active
/// inside the era* (here from 1970, before Rule E's 1977..1995 finite rows) alongside the
/// recurring rows. zic-rs now expands the finite history explicitly and projects the recurring
/// tail via the footer — the same path as the single-era `Test/Mixed`, and what admits
/// America/New_York. The footer comes from the perpetual (`1996 max`) rows.
#[test]
fn mixed_in_era_finite_and_recurring_final_era_is_supported() {
    let d = compile("fixtures/minimal/mixed_in_era.zi", "Test/MixedInEra");
    assert_eq!(d.footer, "GMT0BST,M3.5.0/1,M10.5.0");
    // Finite history present (a 1980 onset, governed by the 1977..1995 finite rule) ...
    assert!(
        d.transitions
            .iter()
            .any(|t| year_of_unix(t.at) == 1980 && d.types[t.type_index as usize].is_dst),
        "1980 finite-rule DST onset must be emitted explicitly"
    );
    // ... and the recurring tail is governed by the footer beyond the explicit window.
    assert!(
        d.transitions.len() > 2,
        "explicit finite history, not a lone anchor"
    );
}

// --- T3.2a: obsolete `FROM = minimum` (coerced to 1900, as reference `zic`). ---

/// `FROM = minimum` is the obsolete spelling reference `zic` coerces to **1900** (not an
/// infinite past). The compiled zone's first transition is therefore in 1900, and the recurring
/// footer comes from the perpetual rule (`Apr lastSun` / `Oct lastSun`).
#[test]
fn from_minimum_lowers_first_transition_to_1900() {
    let d = compile("fixtures/minimal/minrule.zi", "Test/MinRule");
    assert_eq!(d.footer, "EST5EDT,M4.5.0,M10.5.0");
    let first = d.transitions.first().expect("at least one transition");
    assert_eq!(
        year_of_unix(first.at),
        1900,
        "first transition is in 1900, not earlier"
    );
}

// --- T3.2b: inline-save eras (RULES = a constant clock value). ---

/// Find the local-time-type with the given abbreviation.
fn type_with<'a>(
    d: &'a tzcompile::tzif::TzifData,
    abbr: &str,
) -> &'a tzcompile::tzif::LocalTimeType {
    d.types
        .iter()
        .find(|t| t.abbr == abbr)
        .unwrap_or_else(|| panic!("no type with abbr {abbr:?} in {:?}", d.types))
}

/// Compile a source string and return the compile error message (expects failure).
fn compile_err(source: &str, zone: &str) -> String {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("t.zi");
    std::fs::write(&p, source).unwrap();
    let db = load_database(&[p]).expect("parses");
    compile_zone(&db, zone)
        .expect_err("should fail closed")
        .to_string()
}

/// A literal-FORMAT inline-save era is one fixed type at `STDOFF + SAVE`, marked daylight.
/// `8:00 0:30 HKWT` → utoff 30600, is_dst, abbr `HKWT`.
#[test]
fn inline_save_literal_format_marks_dst_and_adds_save_to_offset() {
    let d = compile("fixtures/minimal/inline.zi", "Test/InlineLit");
    let t = type_with(&d, "HKWT");
    assert_eq!((t.utoff, t.is_dst), (30600, true));
}

/// A `%z`-FORMAT inline-save era renders its abbreviation from the **total** effective offset
/// (`STDOFF + SAVE`), not from `STDOFF`. `7:00 0:20 %z` → utoff 26400, abbr `+0720`.
#[test]
fn inline_save_percent_z_uses_effective_total_offset_for_abbreviation() {
    let d = compile("fixtures/minimal/inline.zi", "Test/InlineZ");
    let t = type_with(&d, "+0720");
    assert_eq!((t.utoff, t.is_dst), (26400, true));
}

/// An inline-save era works as a *middle* era of a multi-era zone: the boundary transitions into
/// `HKWT` and back out to `HKT` are both present (HKT → HKWT → HKT).
#[test]
fn inline_save_works_across_multi_era_boundaries() {
    let d = compile("fixtures/minimal/inline.zi", "Test/InlineLit");
    assert_eq!(
        d.transitions.len(),
        2,
        "into HKWT (1941) and back to HKT (1945)"
    );
    let kinds: Vec<_> = d
        .transitions
        .iter()
        .map(|t| d.types[t.type_index as usize].abbr.as_str())
        .collect();
    assert_eq!(kinds, vec!["HKWT", "HKT"]);
}

#[test]
fn inline_save_percent_s_fails_closed_until_pinned() {
    let msg = compile_err("Zone T/PS 8:00 1:00 E%sT\n", "T/PS");
    assert!(msg.contains("%s"), "expected a %s diagnostic, got: {msg}");
}

#[test]
fn inline_save_slash_format_fails_closed_until_pinned() {
    let msg = compile_err("Zone T/SL 8:00 1:00 STD/DST\n", "T/SL");
    assert!(
        msg.to_lowercase().contains("slash") || msg.contains('/'),
        "got: {msg}"
    );
}

/// Negative inline `SAVE` is **valid signed state** (law 7), pinned against reference `zic`'s
/// Europe/Prague `1 -1 GMT` era. `8:00 -1:00 XYZ` → one fixed type at the signed effective offset
/// `STDOFF + SAVE` = 7:00, `is_dst` (save ≠ 0), literal abbr `XYZ`. Only `%s`/slash inline still fail.
#[test]
fn inline_save_negative_renders_signed_effective_offset() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("t.zi");
    std::fs::write(&p, "Zone T/NG 8:00 -1:00 XYZ\n").unwrap();
    let db = load_database(&[p]).expect("parses");
    let d = compile_zone(&db, "T/NG").expect("negative inline SAVE now compiles");
    let ty = d
        .types
        .iter()
        .find(|t| t.abbr == "XYZ")
        .expect("the inline-save type");
    assert_eq!((ty.utoff, ty.is_dst), (25200, true)); // 8:00 + (-1:00) = 7:00; save != 0 → dst
}

// --- Oracle: every multi-era fixture must match reference `zic` under `zdump` over a
// declared horizon. Auto-skips when `zic`/`zdump` are absent. ---

fn oracle(source: &str, zone: &str, lo: i32, hi: i32) {
    if !reference_zic::is_available("zic") || !reference_zic::is_available("zdump") {
        eprintln!("skipping oracle: reference `zic`/`zdump` not on PATH");
        return;
    }
    let inputs = vec![fixture(source)];
    let db = load_database(&inputs).unwrap();
    let work = tempfile::tempdir().unwrap();
    let mode = CompareMode::Zdump {
        program: "zdump".to_string(),
        lo,
        hi,
    };
    let cmp = compare_zone(&db, &inputs, zone, "zic", work.path(), &mode).unwrap();
    assert!(cmp.is_match(), "{}", cmp.summary());
}

#[test]
fn oracle_fixed_to_fixed() {
    oracle("fixtures/minimal/multi_ff.zi", "Test/FF", 1960, 1990);
}

#[test]
fn oracle_fixed_to_recurring() {
    oracle("fixtures/minimal/multi_fr.zi", "Test/FR", 1995, 2099);
}

#[test]
fn oracle_from_2015_continuation() {
    oracle("fixtures/minimal/multi_q.zi", "Test/Q", 2000, 2018);
}

#[test]
fn oracle_era_ends_mid_dst() {
    oracle("fixtures/minimal/multi_mid.zi", "Test/MidDst", 1980, 2000);
}

#[test]
fn oracle_final_effective_recurring_only() {
    oracle(
        "fixtures/minimal/final_effective.zi",
        "Test/FinalEffective",
        1990,
        2040,
    );
}

#[test]
fn oracle_single_era_mixed_finite_and_recurring() {
    // Horizon spans the FINITE period (1976..1997) so a regression that wrongly let the
    // footer govern history would be caught — not just the recurring tail.
    oracle("fixtures/minimal/mixed.zi", "Test/Mixed", 1976, 1997);
}

/// T4.0 — first real IANA-zone slice. The canonical (record-keys-only de-abbreviated)
/// Europe/London from tzdb 2026b must match reference `zic` under `zdump` across its full
/// history (LMT → GMT/BST eras, the 1968–71 fixed-BST and earlier BDST double-DST period,
/// the recurring EU footer). Auto-skips when `zic`/`zdump` are absent.
#[test]
fn europe_london_2026b_matches_reference_zic_over_1830_2045() {
    oracle(
        "fixtures/iana-slices/europe_london_2026b.zi",
        "Europe/London",
        1830,
        2045,
    );
}

/// `FROM = minimum` really starts at 1900 (early horizon proves the lower bound) ...
#[test]
fn oracle_from_minimum_matches_reference_zic_over_1899_1910() {
    oracle("fixtures/minimal/minrule.zi", "Test/MinRule", 1899, 1910);
}

/// ... and the recurring tail still matches reference `zic` (late horizon).
#[test]
fn oracle_from_minimum_matches_reference_zic_over_2019_2040() {
    oracle("fixtures/minimal/minrule.zi", "Test/MinRule", 2019, 2040);
}

#[test]
fn oracle_inline_save_literal_matches_reference_zic() {
    oracle("fixtures/minimal/inline.zi", "Test/InlineLit", 1940, 1946);
}

#[test]
fn oracle_inline_save_percent_z_matches_reference_zic() {
    oracle("fixtures/minimal/inline.zi", "Test/InlineZ", 1930, 1940);
}

#[test]
fn oracle_inline_save_single_era_matches_reference_zic() {
    oracle("fixtures/minimal/inline.zi", "Test/InlineSolo", 1990, 1995);
}

// --- "%z unlock": a `%z` FORMAT on a no-rules era renders the numeric standard offset. ---

/// `0:30 - %z` → a single fixed type `utoff=1800, is_dst=false, abbr="+0030"`, footer
/// `<+0030>-0:30` (the numeric abbreviation is angle-bracket-quoted in the POSIX TZ string).
#[test]
fn percent_z_on_no_rules_era_renders_numeric_offset() {
    let d = compile("fixtures/minimal/percent_z_norules.zi", "Test/PZSolo");
    assert_eq!(d.types.len(), 1);
    assert_eq!((d.types[0].utoff, d.types[0].is_dst), (1800, false));
    assert_eq!(d.types[0].abbr, "+0030");
    assert_eq!(d.footer, "<+0030>-0:30");
}

/// `%s` and the `STD/DST` slash form on a no-rules era stay fail-closed (no LETTER / no DST).
#[test]
fn percent_s_and_slash_on_no_rules_era_still_fail_closed() {
    assert!(compile_err("Zone Z 5:00 - E%sT\n", "Z").contains("%s"));
    assert!(
        compile_err("Zone Z 5:00 - A/B\n", "Z")
            .to_lowercase()
            .contains("slash")
            || compile_err("Zone Z 5:00 - A/B\n", "Z").contains('/')
    );
}

#[test]
fn oracle_percent_z_no_rules_matches_reference_zic() {
    oracle(
        "fixtures/minimal/percent_z_norules.zi",
        "Test/PZSolo",
        1900,
        2010,
    );
    oracle(
        "fixtures/minimal/percent_z_norules.zi",
        "Test/PZMulti",
        1900,
        2010,
    );
}

// --- Behaviour verification against the real installed `tzdata.zi` (auto-skip if absent). ---

const SYS_TZDATA: &str = "/usr/share/zoneinfo/tzdata.zi";

/// `Some(is_match)` from the `zdump` oracle for a system-`tzdata.zi` zone, or `None` if the file
/// or reference tools are unavailable (portable-CI skip).
fn system_zdump_match(zone: &str, lo: i32, hi: i32) -> Option<bool> {
    if !std::path::Path::new(SYS_TZDATA).exists()
        || !reference_zic::is_available("zic")
        || !reference_zic::is_available("zdump")
    {
        eprintln!("skipping: system tzdata.zi or reference zic/zdump unavailable");
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

/// The `%z`-no-rules unlock is *behaviour-correct on real zones*, not merely compile-clean: a
/// regional sample of zones the unlock newly admitted matches reference `zic` under `zdump`.
#[test]
fn percent_z_unlock_sample_zones_match_reference_zic() {
    for z in [
        "Africa/Lagos",
        "Africa/Nairobi",
        "Africa/Bissau",
        "Asia/Jakarta",
    ] {
        if let Some(m) = system_zdump_match(z, 1900, 2040) {
            assert!(m, "{z} (newly %z-unlocked) must zdump-match reference zic");
        }
    }
}

/// T5 fix #1 — the standard-`LETTER` temporal-state bug is fixed: `Pacific/Auckland` now
/// **behaviour-verifies** against reference `zic`. New Zealand's `1946 Jan 1 0 0 S` rule lands
/// exactly on the +11:30→+12:00 era boundary and flips the standard abbreviation NZMT→NZST; the
/// generic fix (a boundary-coincident activation, `ut <= s`, seeds the boundary state — `%s`/the
/// standard LETTER is *temporal* state) renders NZST after 1946 as reference `zic` does. See
/// reference-zic-semantics.md §10. Auto-skips without the system `tzdata.zi` / reference tools.
#[test]
fn pacific_auckland_1946_changes_nzmt_to_nzst_and_matches_reference_zic() {
    if let Some(m) = system_zdump_match("Pacific/Auckland", 1900, 2040) {
        assert!(
            m,
            "Pacific/Auckland must zdump-match reference zic after the standard-letter fix"
        );
    }
}

/// T5 fix #2 — the *final recurring era* anchor now seeds its boundary state from any rule
/// activation that fires at-or-before the era start, the same `ut <= s` rule the general path
/// uses. `Europe/Lisbon`'s final era `0 E WE%sT` begins 1996-03-31 01:00u, which is exactly Rule
/// E's last-Sunday-of-March spring-forward, so the era-start anchor must be WEST (isdst, +1h),
/// not the era's standard WET. Before the fix the shape-(a) short-circuit emitted the standard
/// state and `zdump` showed WET at the boundary. The same fix corrects `America/St_Johns`. See
/// reference-zic-semantics.md §10. Auto-skips without the system `tzdata.zi` / reference tools.
#[test]
fn europe_lisbon_final_era_boundary_coincident_dst_matches_reference_zic() {
    if let Some(m) = system_zdump_match("Europe/Lisbon", 1900, 2040) {
        assert!(
            m,
            "Europe/Lisbon must zdump-match reference zic: the 1996 era-start anchor is a \
             boundary-coincident spring-forward (WEST), not standard WET"
        );
    }
}

/// T5 fix #2 also corrects `America/St_Johns` (a different zone hitting the same final-recurring
/// era boundary-coincident activation). Guards against a regression that fixes Lisbon but not it.
#[test]
fn america_st_johns_matches_reference_zic_after_anchor_fix() {
    if let Some(m) = system_zdump_match("America/St_Johns", 1900, 2040) {
        assert!(
            m,
            "America/St_Johns must zdump-match reference zic after the final-era anchor fix"
        );
    }
}

/// T5 #3 — **boundary-coincident standard-clock activation absorption** (the generic mechanism, not
/// a "Russia fix"). Hermetic: a synthetic zone whose era boundary (`3:00 ... 1985 Mar lastSun
/// 2:00s`) coincides with the new era's spring-forward (`2:00 ... Rule BS Mar lastSun 2:00s`) where
/// the new era's standard offset is 1h smaller. Reference `zic` makes the boundary itself the DST
/// transition (UT offset continuous, isdst 0→1); the bug emitted a spurious 1h standard interval.
/// No system tzdata needed — uses reference `zic` on the checked-in fixture.
#[test]
fn era_boundary_absorbs_new_era_standard_clock_spring_forward() {
    oracle(
        "fixtures/minimal/boundary_spring.zi",
        "Test/BoundarySpring",
        1980,
        1996,
    );
}

/// T5 #3 on real zones — the Russia cluster (~30 zones) collapsed when this one mechanism landed.
/// Each: an era boundary whose standard offset drops 1h coincides with Rule Russia's
/// `lastSun Mar 02:00s` spring, so reference `zic` springs *at* the boundary (continuous UT offset,
/// isdst flips) while the bug inserted a standard hour. Auto-skips without system `tzdata.zi`.
#[test]
fn europe_moscow_1991_matches_reference_zic() {
    if let Some(m) = system_zdump_match("Europe/Moscow", 1900, 2040) {
        assert!(
            m,
            "Europe/Moscow must zdump-match reference zic (1991 boundary spring)"
        );
    }
}

#[test]
fn europe_volgograd_boundary_spring_forward_matches_reference_zic() {
    if let Some(m) = system_zdump_match("Europe/Volgograd", 1900, 2040) {
        assert!(
            m,
            "Europe/Volgograd must zdump-match reference zic (1988/1992 boundary springs)"
        );
    }
}

#[test]
fn asia_novosibirsk_1991_matches_reference_zic() {
    if let Some(m) = system_zdump_match("Asia/Novosibirsk", 1900, 2040) {
        assert!(
            m,
            "Asia/Novosibirsk must zdump-match reference zic (1991 boundary spring)"
        );
    }
}

/// Guard the *negative* property across several cluster zones at once: the offset-drop boundary must
/// not create a spurious standard hour. Loops the three representatives the diagnosis pinned.
#[test]
fn russia_boundary_offset_drop_does_not_create_spurious_standard_hour() {
    for z in ["Europe/Moscow", "Europe/Volgograd", "Asia/Novosibirsk"] {
        if let Some(m) = system_zdump_match(z, 1985, 1995) {
            assert!(
                m,
                "{z}: offset-drop era boundary must not insert a standard hour"
            );
        }
    }
}

/// T5 #4 — **a ruled era inherits the rule set's prevailing state from a prior activation that
/// precedes the era's local expansion window.** Hermetic: Rule PC turns DST on in 1970 and off in
/// 1980; two `B%sT` eras resume Rule PC (one just after the onset, one across an intervening fixed
/// era) and both must start in DST (BDT, +6) — the intervening era does not reset the rule's
/// timeline. Without the fix the era seeds *standard* (BST). Uses reference `zic` on the fixture.
#[test]
fn ruled_era_inherits_prior_active_rule_state() {
    oracle(
        "fixtures/minimal/prior_active_carry.zi",
        "Test/PriorCarry",
        1968,
        1982,
    );
}

/// T5 #4 on real zones — one generic fix cleared 5 (Phoenix/Bermuda/Manila/Brussels/Macau). Each is
/// a ruled era resuming a rule whose last change predates its window: Phoenix's 1944 `US M%sT` era
/// must start in War time (MWT) — Rule US set War in 1942, cleared 1945. Auto-skips without tzdata.
#[test]
fn america_phoenix_1944_inherits_war_time_matches_reference_zic() {
    if let Some(m) = system_zdump_match("America/Phoenix", 1900, 2040) {
        assert!(
            m,
            "America/Phoenix 1944 era must inherit War time (MWT) from the 1942 rule activation"
        );
    }
}

/// Bermuda's 1930 `-4 Be A%sT` era must render `AST`, not `AT`: Rule Be's last pre-1930 activation
/// (1918) carries `LETTER = S`, which an in-window-only seed misses.
#[test]
fn atlantic_bermuda_1930_inherits_standard_letter_matches_reference_zic() {
    if let Some(m) = system_zdump_match("Atlantic/Bermuda", 1900, 2040) {
        assert!(
            m,
            "Atlantic/Bermuda 1930 era must inherit LETTER=S (AST) from the 1918 rule activation"
        );
    }
}

/// Manila's 1945 era and Brussels' 1944 era both resume a rule that is mid-DST from 1941 / 1940 —
/// the prior-active save+is_dst must carry across the intervening (occupation / other-rule) era.
#[test]
fn asia_manila_and_europe_brussels_inherit_prior_dst_match_reference_zic() {
    for z in ["Asia/Manila", "Europe/Brussels"] {
        if let Some(m) = system_zdump_match(z, 1900, 2040) {
            assert!(
                m,
                "{z} must inherit the prior-active DST state at the ruled-era start"
            );
        }
    }
}

/// T5 #5 — **clock-reference normalization at era boundaries.** Hermetic: the era ends at a wall
/// `UNTIL` (`1990 Mar lastSun 2:00`) that, at save=0, names the same instant as Rule Y's
/// standard-ref spring (`Mar lastSun 2:00s`), while the standard offset drops 6→5. The boundary must
/// itself be the spring (GDT, +6); the bug emitted standard then sprang an hour later because the
/// absorption test required exact w/s/u reference equality. Uses reference `zic` on the fixture.
#[test]
fn era_boundary_normalizes_wall_standard_clock_reference() {
    oracle(
        "fixtures/minimal/boundary_clockref.zi",
        "Test/BoundaryClockRef",
        1985,
        1996,
    );
}

/// T5 #5 on real zones — the absorption sub-fix (wall `UNTIL` vs standard rule AT, equivalent at
/// save=0 across an offset-drop boundary). Tashkent/Ashgabat (`…Mar 31 2` wall vs Rule R `2s`) and
/// Anadyr (`…Apr 1 0s` standard vs Rule R `Apr 1 0` wall) all spring *at* the boundary in reference
/// `zic`. Auto-skips without system `tzdata.zi`.
#[test]
fn central_asia_offset_drop_boundaries_normalize_clock_reference() {
    for z in ["Asia/Tashkent", "Asia/Ashgabat", "Asia/Anadyr"] {
        if let Some(m) = system_zdump_match(z, 1900, 2040) {
            assert!(
                m,
                "{z}: wall/standard boundary spring must be absorbed (no spurious std hour)"
            );
        }
    }
}

/// T5 #5 on real zones — the era-end-break sub-fix (compare resolved UT, not naive local seconds).
/// `Europe/Simferopol` 2014 ends `2 E EE%sT 2014 Mar 30 2` (wall) while Rule E springs `Mar lastSu
/// 1u` (universal): the spring belongs to the *next* era in UT and must not be emitted.
/// `Europe/Warsaw` 1918 has a Rule c fall-back (`2s`) coincident with a wall `UNTIL` (`S 16 3`).
#[test]
fn europe_mixed_reference_era_end_breaks_match_reference_zic() {
    for z in ["Europe/Simferopol", "Europe/Warsaw"] {
        if let Some(m) = system_zdump_match(z, 1900, 2040) {
            assert!(
                m,
                "{z}: mixed-reference era-end must compare resolved UT, not naive local"
            );
        }
    }
}

#[test]
fn oracle_mixed_in_era_final_era_matches_reference_zic() {
    oracle(
        "fixtures/minimal/mixed_in_era.zi",
        "Test/MixedInEra",
        1970,
        2040,
    );
}

/// T4.1 — second real IANA-zone slice. America/New_York exercises the genuinely *mixed-in-era*
/// final era (Rule US: finite DST 1967..2006 + recurring 2007..max) plus the WWII war-time eras.
/// Matches reference `zic` under `zdump` across its full history. Auto-skips without `zic`/`zdump`.
#[test]
fn america_new_york_2026b_matches_reference_zic_over_1883_2040() {
    oracle(
        "fixtures/iana-slices/america_new_york_2026b.zi",
        "America/New_York",
        1883,
        2040,
    );
}

/// zic-rs output for the canonical and zishrink-abbreviated New York slices must be identical
/// (proves the record-key prefix parsing is faithful for this zone too — no external `zic`).
#[test]
fn zic_rs_compiles_abbreviated_and_canonical_new_york_identically() {
    let bytes = |rel: &str| {
        let db = load_database(&[fixture(rel)]).expect("load");
        compile_zone_to_bytes(&db, "America/New_York").expect("compile")
    };
    assert_eq!(
        bytes("fixtures/iana-slices/america_new_york_2026b.zi"),
        bytes("fixtures/iana-slices/america_new_york_2026b.abbrev.zi"),
    );
}

/// Faithfulness proof for the canonicalised slice: reference `zic` compiles the canonical
/// (record-keys-only de-abbreviated) slice and the verbatim abbreviated `tzdata.zi` extract to
/// **byte-identical** TZif. This shows the only-expand-R/Z/L transform changes nothing zic
/// cares about — the fixture is a faithful canonicalisation, not a hand-shaped approximation.
/// Auto-skips when `zic` is absent.
#[test]
fn canonical_london_slice_is_byte_identical_to_abbreviated_extract_under_reference_zic() {
    if !reference_zic::is_available("zic") {
        eprintln!("skipping faithfulness proof: reference `zic` not on PATH");
        return;
    }
    let compile_with_zic = |rel: &str| -> Vec<u8> {
        let out = tempfile::tempdir().unwrap();
        let status = std::process::Command::new("zic")
            .arg("-d")
            .arg(out.path())
            .arg(fixture(rel))
            .status()
            .expect("run reference zic");
        assert!(status.success(), "reference zic failed on {rel}");
        std::fs::read(out.path().join("Europe/London")).expect("compiled Europe/London")
    };
    let canonical = compile_with_zic("fixtures/iana-slices/europe_london_2026b.zi");
    let abbreviated = compile_with_zic("fixtures/iana-slices/europe_london_2026b.abbrev.zi");
    assert_eq!(
        canonical, abbreviated,
        "canonical slice and abbreviated extract must compile byte-identically under reference zic"
    );
}

/// zic-rs now parses the **zishrink-abbreviated** record keys (`R`/`Z`/`L`) used by the
/// installed `tzdata.zi`, so it can read the abbreviated extract directly. Its own output for
/// the abbreviated extract and the (record-keys-expanded) canonical slice must be byte-identical
/// — proving the prefix-matching is faithful, not a lossy convenience. No external `zic` needed.
#[test]
fn zic_rs_compiles_abbreviated_and_canonical_london_identically() {
    let bytes = |rel: &str| {
        let db = load_database(&[fixture(rel)]).expect("load");
        compile_zone_to_bytes(&db, "Europe/London").expect("compile")
    };
    assert_eq!(
        bytes("fixtures/iana-slices/europe_london_2026b.zi"),
        bytes("fixtures/iana-slices/europe_london_2026b.abbrev.zi"),
        "zic-rs output must be identical for canonical and zishrink-abbreviated London input"
    );
}

/// Smoke test: the installed single-file `/usr/share/zoneinfo/tzdata.zi` (the real zishrink
/// source) parses end-to-end, and `Europe/London` read straight from it compiles to the same
/// bytes as our pinned slice. **Auto-skips** when the system file is absent, so default CI stays
/// portable; this is an opportunistic check on the real-world source, not a pinned oracle.
#[test]
fn europe_london_direct_tzdata_zi_parse_smoke() {
    let sys = std::path::Path::new("/usr/share/zoneinfo/tzdata.zi");
    if !sys.exists() {
        eprintln!("skipping: /usr/share/zoneinfo/tzdata.zi not present");
        return;
    }
    let db = load_database(&[sys.to_path_buf()]).expect("installed tzdata.zi parses end-to-end");
    let from_system = compile_zone_to_bytes(&db, "Europe/London").expect("compile from tzdata.zi");
    let pinned_db =
        load_database(&[fixture("fixtures/iana-slices/europe_london_2026b.zi")]).unwrap();
    let from_pinned = compile_zone_to_bytes(&pinned_db, "Europe/London").unwrap();
    // Equal only when the system tzdb matches our pinned 2026b slice; if a newer tzdb changed
    // London, just assert it still parses+compiles (don't pin behaviour to the host's version).
    if from_system != from_pinned {
        eprintln!("note: system tzdata.zi differs from pinned 2026b London (newer tzdb?); parsed+compiled OK");
    }
}
