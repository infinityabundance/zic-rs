//! T16.4 — auxiliary-table validator tests. The central law under test: these are **policy/index/
//! reference** artifacts validated for **table structural admissibility only** — never compile inputs,
//! never semantic witnesses, and the validator **names the universe** it resolves against (it resolves
//! *no* zone names — structural only).

use std::collections::BTreeSet;
use tzcompile::aux_tables::{
    iso3166_codes, validate_zone_table, AuxTableValidationReport, CountryCodeAuthority,
    InstallEcologyStatus, ZoneTableFinding, ZoneTableKind, ZoneTableStructuralVerdict,
};
use tzcompile::manifest::ArtifactCategory;

fn findings(v: &tzcompile::aux_tables::AuxTableValidation) -> Vec<ZoneTableFinding> {
    v.findings.iter().map(|(_, f)| *f).collect()
}

/// **(T16.4)** The four table kinds are a finite vocabulary with distinct ids + coverage statements.
#[test]
fn zone_table_kind_finite_vocab() {
    let kinds = [
        ZoneTableKind::ZoneTab,
        ZoneTableKind::Zone1970Tab,
        ZoneTableKind::ZonenowTab,
        ZoneTableKind::Iso3166Tab,
    ];
    let mut ids: Vec<&str> = kinds.iter().map(|k| k.as_str()).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), kinds.len());
}

/// **(T16.4)** The verdict is finite (`conformant`/`violation`) — never a bare `valid: true`.
#[test]
fn zone_table_structural_verdict_finite_vocab() {
    assert_eq!(
        ZoneTableStructuralVerdict::Conformant.as_str(),
        "conformant"
    );
    assert_eq!(ZoneTableStructuralVerdict::Violation.as_str(), "violation");
}

/// **(T16.4)** The category law: zone tables are `policy_input`, `iso3166.tab` is `reference_input` —
/// **never** `compile_input`. (The typed `zone.tab`-is-not-compile lesson from T12.5c.)
#[test]
fn tables_are_policy_or_reference_never_compile_input() {
    assert_eq!(
        ZoneTableKind::ZoneTab.artifact_category().as_str(),
        "policy_input"
    );
    assert_eq!(
        ZoneTableKind::Zone1970Tab.artifact_category().as_str(),
        "policy_input"
    );
    assert_eq!(
        ZoneTableKind::Iso3166Tab.artifact_category().as_str(),
        "reference_input"
    );
    for k in [
        ZoneTableKind::ZoneTab,
        ZoneTableKind::Zone1970Tab,
        ZoneTableKind::ZonenowTab,
        ZoneTableKind::Iso3166Tab,
    ] {
        assert_ne!(
            k.artifact_category().as_str(),
            ArtifactCategory::CompileInput.as_str(),
            "{} must not be a compile input",
            k.as_str()
        );
    }
}

/// **(T16.4)** A well-formed `zone.tab` with a country spanning many rows (e.g. `US`) is **conformant** —
/// duplicate country *codes* are legal; only a semantic-row duplicate (identical identity tuple) is flagged.
#[test]
fn zone_tab_multi_row_per_country_is_conformant() {
    let src = "# comment\n\
        US\t+404251-0740023\tAmerica/New_York\n\
        US\t+411745-0863730\tAmerica/Chicago\n\
        US\t+340308-1181434\tAmerica/Los_Angeles\n";
    let v = validate_zone_table(ZoneTableKind::ZoneTab, src.as_bytes(), None);
    assert_eq!(v.verdict, ZoneTableStructuralVerdict::Conformant, "{v:?}");
    assert_eq!(v.rows_checked, 3);
}

/// **(T16.4)** A **semantic** duplicate (same `(cc-set, coord, zone-name)` identity tuple, comments
/// excluded) IS a finding — distinct from legal multi-row-per-country. Differing comments still duplicate.
#[test]
fn semantic_duplicate_row_is_flagged() {
    let src = "AD\t+4230+00131\tEurope/Andorra\n\
               AD\t+4230+00131\tEurope/Andorra\t# a differing comment\n";
    let v = validate_zone_table(ZoneTableKind::ZoneTab, src.as_bytes(), None);
    assert!(findings(&v).contains(&ZoneTableFinding::DuplicateSemanticRow));
}

/// **(T16.4)** Empty zone-name (zone tables) and empty country-name (`iso3166.tab`) are flagged; and
/// `iso3166.tab` duplicate country codes are a semantic-row duplicate (codes are a unique reference).
#[test]
fn empty_name_and_iso3166_duplicate_code_are_flagged() {
    // Empty zone-name field.
    let v = validate_zone_table(ZoneTableKind::ZoneTab, b"AD\t+4230+00131\t\n", None);
    assert!(findings(&v).contains(&ZoneTableFinding::EmptyNameField));
    // iso3166 duplicate code + empty name.
    let v = validate_zone_table(
        ZoneTableKind::Iso3166Tab,
        b"AD\tAndorra\nAD\tAndorra Again\nAE\t\n",
        None,
    );
    assert!(findings(&v).contains(&ZoneTableFinding::DuplicateSemanticRow));
    assert!(findings(&v).contains(&ZoneTableFinding::EmptyNameField));
}

/// **(T16.4)** Bad column count / country code / coordinate are each rejected with the typed finding.
#[test]
fn structural_violations_are_typed() {
    // Too few columns.
    let v = validate_zone_table(ZoneTableKind::ZoneTab, b"AD\t+4230+00131\n", None);
    assert!(findings(&v).contains(&ZoneTableFinding::InvalidColumnCount));
    // Bad country code (lowercase / wrong length).
    let v = validate_zone_table(ZoneTableKind::ZoneTab, b"usa\t+4230+00131\tX/Y\n", None);
    assert!(findings(&v).contains(&ZoneTableFinding::InvalidCountryCode));
    // Bad coordinate (no sign / wrong digit count).
    let v = validate_zone_table(ZoneTableKind::ZoneTab, b"AD\t4230x00131\tX/Y\n", None);
    assert!(findings(&v).contains(&ZoneTableFinding::InvalidCoordinateFormat));
}

/// **(T16.4)** `zone1970.tab` country codes are cross-validated against the **same admitted release's**
/// `iso3166.tab` (the authority is declared), with comma-list set semantics.
#[test]
fn zone1970_country_codes_validate_against_iso3166() {
    let iso = iso3166_codes(b"AD\tAndorra\nAE\tUnited Arab Emirates\nOM\tOman\n");
    // A code not in the iso set (`ZZ`) is flagged; the authority is recorded.
    let bad = "AD,ZZ\t+4230+00131\tEurope/Andorra\n";
    let v = validate_zone_table(ZoneTableKind::Zone1970Tab, bad.as_bytes(), Some(&iso));
    assert!(findings(&v).contains(&ZoneTableFinding::InvalidCountryCode));
    assert_eq!(
        v.country_code_authority,
        CountryCodeAuthority::SameAdmittedReleaseIso3166Tab
    );
    // A clean multi-country row validates.
    let good = "AE,OM\t+2518+05518\tAsia/Dubai\n";
    let v = validate_zone_table(ZoneTableKind::Zone1970Tab, good.as_bytes(), Some(&iso));
    assert_eq!(v.verdict, ZoneTableStructuralVerdict::Conformant, "{v:?}");
}

/// **(T16.4.1)** `zone.tab` country codes are **also** cross-validated against the same-release
/// `iso3166.tab` (not only `zone1970.tab`) — a code absent from the authority is flagged, and the
/// authority is recorded as `same_admitted_release_iso3166_tab` when the iso table is supplied.
#[test]
fn zone_tab_country_codes_validate_against_same_release_iso3166() {
    let iso = iso3166_codes(b"AD\tAndorra\nAE\tUnited Arab Emirates\n");
    // A code not in the iso set (`ZZ`) is flagged.
    let v = validate_zone_table(
        ZoneTableKind::ZoneTab,
        b"ZZ\t+4230+00131\tEurope/Nowhere\n",
        Some(&iso),
    );
    assert!(findings(&v).contains(&ZoneTableFinding::InvalidCountryCode));
    assert_eq!(
        v.country_code_authority,
        CountryCodeAuthority::SameAdmittedReleaseIso3166Tab
    );
    // A known code validates clean.
    let v = validate_zone_table(
        ZoneTableKind::ZoneTab,
        b"AD\t+4230+00131\tEurope/Andorra\n",
        Some(&iso),
    );
    assert_eq!(v.verdict, ZoneTableStructuralVerdict::Conformant, "{v:?}");
    // Without an iso table, zone.tab is shape-checked only (authority not_cross_validated).
    let v = validate_zone_table(ZoneTableKind::ZoneTab, b"ZZ\t+4230+00131\tX/Y\n", None);
    assert_eq!(
        v.country_code_authority,
        CountryCodeAuthority::NotCrossValidated
    );
    assert_eq!(
        v.verdict,
        ZoneTableStructuralVerdict::Conformant,
        "shape-only ZZ is fine: {v:?}"
    );
}

/// **(T16.4)** A country code repeated *within* a `zone1970.tab` row is set-semantics-invalid.
#[test]
fn duplicate_code_within_row_is_flagged() {
    let v = validate_zone_table(
        ZoneTableKind::Zone1970Tab,
        b"US,US\t+404251-0740023\tAmerica/New_York\n",
        None,
    );
    assert!(findings(&v).contains(&ZoneTableFinding::DuplicateCodeInRow));
}

/// **(T16.4)** `zonenow.tab` declares **now/future agreement** coverage — not all-history equivalence —
/// and allows its `XX` placeholder code.
#[test]
fn zonenow_declares_now_future_scope_and_allows_xx() {
    assert_eq!(
        ZoneTableKind::ZonenowTab.coverage(),
        "now_future_agreement_only"
    );
    assert_ne!(ZoneTableKind::ZonenowTab.coverage(), "all_history");
    let v = validate_zone_table(
        ZoneTableKind::ZonenowTab,
        b"XX\t-1416-17042\tPacific/Pago_Pago\tMidway\n",
        None,
    );
    assert_eq!(v.verdict, ZoneTableStructuralVerdict::Conformant, "{v:?}");
}

/// **(T16.4)** Non-UTF-8 bytes are a typed finding, never a panic (bounds-safe).
#[test]
fn non_utf8_is_a_finding_not_a_panic() {
    let v = validate_zone_table(ZoneTableKind::ZoneTab, &[0xff, 0xfe, 0x00, 0x9f], None);
    assert_eq!(v.verdict, ZoneTableStructuralVerdict::Violation);
    assert!(findings(&v).contains(&ZoneTableFinding::NonUtf8));
}

/// **(T16.4)** The report names the **universe** it validates against (structural-only, no name
/// resolution) and carries the bounded non-claims (geodetic / license / release-identity) + the separate
/// table-diagnostic code space. The "name the universe" rule is machine-visible.
#[test]
fn report_names_universe_and_bounds_non_claims() {
    let v = validate_zone_table(
        ZoneTableKind::ZoneTab,
        b"AD\t+4230+00131\tEurope/Andorra\n",
        None,
    );
    let report = AuxTableValidationReport {
        tables: vec![v],
        install_ecology: InstallEcologyStatus::current(),
    };
    let j = report.to_json();
    assert!(j.contains("\"schema\": \"zic-rs-aux-table-validation-v1\""));
    assert!(j.contains("\"zone_universe\": \"not_resolved_structural_only\""));
    assert!(j.contains("\"table_diagnostic_code_space\": \"separate_table_codes\""));
    assert!(j.contains("geodetic")); // coordinate syntax does not claim geodetic accuracy
    assert!(j.contains("public-domain notice is not provenance"));
    assert!(j.contains("\"install_ecology_status\": \"compile_output_tree_only\""));
}

/// **(T16.4)** Install ecology is bounded — only a compile output tree under `--out`, no layout parity.
#[test]
fn install_ecology_is_bounded() {
    assert_eq!(
        InstallEcologyStatus::current().as_str(),
        "compile_output_tree_only"
    );
}

/// Smoke: validate the real system tables when present (auto-skips for portability). All four should be
/// structurally conformant on a sane install.
#[test]
fn system_tables_are_structurally_conformant() {
    let dir = std::path::Path::new("/usr/share/zoneinfo");
    let iso_path = dir.join("iso3166.tab");
    if !iso_path.exists() {
        eprintln!("skipping: no system zoneinfo tables");
        return;
    }
    let iso_bytes = std::fs::read(&iso_path).unwrap();
    let iso: BTreeSet<String> = iso3166_codes(&iso_bytes);
    for (kind, name) in [
        (ZoneTableKind::ZoneTab, "zone.tab"),
        (ZoneTableKind::Zone1970Tab, "zone1970.tab"),
        (ZoneTableKind::ZonenowTab, "zonenow.tab"),
    ] {
        let p = dir.join(name);
        if !p.exists() {
            continue;
        }
        let bytes = std::fs::read(&p).unwrap();
        // Cross-validate both zone.tab and zone1970.tab against the same-release iso3166.tab.
        let cross =
            matches!(kind, ZoneTableKind::ZoneTab | ZoneTableKind::Zone1970Tab).then_some(&iso);
        let v = validate_zone_table(kind, &bytes, cross);
        assert_eq!(
            v.verdict,
            ZoneTableStructuralVerdict::Conformant,
            "{name} not conformant: {:?}",
            v.findings
        );
    }
}
