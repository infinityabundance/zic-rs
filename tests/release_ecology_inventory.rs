//! T16.1 — release-ecology & downstream-contract **inventory witness** (the inventory's executable twin).
//!
//! T16.1 is inventory-first (like T13.1/T14.1/T15.1): it changes no behaviour. This witness makes the
//! inventory in `docs/tzdb-release-ecology.md` **executable from day one** — it pins the *current
//! non-claims* so that every later T16.x addition is a **deliberate, visible flip**, not silent drift.
//!
//! Two kinds of assertion:
//!   1. The **current admission state** of each ecology surface (a typed table — `NotClaimedYet` /
//!      `AdmittedScoped` / `DocumentationOnly`), mirroring the T13.6 reference-platform matrix.
//!   2. The not-yet-built report fields are **genuinely absent** today (the "flips deliberately" baseline),
//!      while the surfaces that *are* claimed (the T15.5-remainder oracle identity) are present.

use tzcompile::load_database;
use tzcompile::report::build_support_report;
use tzcompile::semantic_witness::build_semantic_witness_report;

const SRC: &str = "Zone Etc/UTC 0:00 - UTC\nLink Etc/UTC UTC\n";

fn support_json() -> String {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("in.zi");
    std::fs::write(&p, SRC).unwrap();
    let db = load_database(std::slice::from_ref(&p)).unwrap();
    build_support_report(&db, None).to_json()
}

/// Build a semantic-report on the **oracle-unavailable** path (bogus tool names), so the test runs
/// without reference `zic`/`zdump` while still emitting the `oracle_identity` shape.
fn semantic_json() -> String {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("in.zi");
    std::fs::write(&p, SRC).unwrap();
    let inputs = vec![p];
    let db = load_database(&inputs).unwrap();
    build_semantic_witness_report(
        &db,
        &["Etc/UTC".to_string()],
        "zic-rs-no-such-zic",
        "zic-rs-no-such-zdump",
        &inputs,
        dir.path(),
    )
    .unwrap()
    .to_json()
}

/// The current admission state of a release/ecology surface (T16.1 inventory vocabulary; T16.2 added
/// `EmittedEvidenceMostlyUnknown` for the first surface whose *surface* is built but whose *evidence* is
/// honestly mostly unknown — `ReferenceBuildProfile`).
#[derive(Debug, PartialEq, Eq)]
enum EcologyAdmission {
    /// Named in the inventory with an owner type + execution milestone, but **not built / not claimed yet**.
    NotClaimedYet,
    /// Admitted, but **scoped** to a single pinned subject (e.g. only `upstream_iana_2026b`).
    AdmittedScoped,
    /// The surface is **emitted** (typed, in a report), but its evidence is honestly mostly
    /// `unknown_unmeasured`/`inferred_forbidden` — the surface exists, the *vibes* do not (T16.2).
    EmittedEvidenceMostlyUnknown,
    /// The surface is **emitted** with genuine typed evidence that distinguishes the *sealed* reference
    /// (versioned + fingerprint-anchored → sealed-claim-grade) from the *live* oracle (unknown trust →
    /// exploration only) — `ReferenceLocatorKind` / `SignatureTrustModel` (T16.3).
    EmittedScopedReference,
    /// The surface is **built as its own report** (a separate proof surface, not part of the conformance
    /// reports) — the auxiliary-table validator `zic-rs-aux-table-validation-v1` (T16.4).
    BuiltSeparateReport,
}

/// **(T16.1)** The inventory is executable: each surface has a recorded current admission state, and the
/// only `AdmittedScoped` rows are the genuinely-pinned ones (the 2026b reference). Nothing is admitted by
/// assumption; everything else is honestly `NotClaimedYet` with an owning milestone (see the doc table).
#[test]
fn release_ecology_inventory_current_state() {
    let inventory: &[(&str, EcologyAdmission, &str)] = &[
        // T16.2 flipped this from NotClaimedYet → emitted (surface built, evidence honest-unknown).
        (
            "reference_build_profile",
            EcologyAdmission::EmittedEvidenceMostlyUnknown,
            "T16.2 (surface) + T16.5 (richer capture)",
        ),
        // T16.3 flipped these two → emitted with genuine sealed-vs-live evidence.
        (
            "reference_locator_kind",
            EcologyAdmission::EmittedScopedReference,
            "T16.3",
        ),
        (
            "signature_trust_model",
            EcologyAdmission::EmittedScopedReference,
            "T16.3",
        ),
        (
            "release_intake_provenance",
            EcologyAdmission::NotClaimedYet,
            "T16.6",
        ),
        // T16.4: the aux-table validator is now built as its own report surface.
        (
            "auxiliary_table_validator",
            EcologyAdmission::BuiltSeparateReport,
            "T16.4",
        ),
        (
            "install_ecology_layout",
            EcologyAdmission::NotClaimedYet,
            "T16.4",
        ),
        // The one admitted reference is scoped to a single pinned release — never a blanket admission.
        (
            "vendor_platform_oracle",
            EcologyAdmission::AdmittedScoped,
            "T16.5 (only upstream_iana_2026b)",
        ),
        (
            "generated_transform_provenance",
            EcologyAdmission::NotClaimedYet,
            "T16.2/T17",
        ),
    ];
    // Exactly one surface is admitted (scoped) today — the upstream reference oracle. Never a blanket one.
    let admitted: Vec<&str> = inventory
        .iter()
        .filter(|(_, a, _)| *a == EcologyAdmission::AdmittedScoped)
        .map(|(s, _, _)| *s)
        .collect();
    assert_eq!(
        admitted,
        vec!["vendor_platform_oracle"],
        "only the scoped upstream reference oracle is admitted; never a blanket admission"
    );
    // Exactly one surface is emitted-but-honest-unknown — `ReferenceBuildProfile` (the T16.2 flip).
    let emitted: Vec<&str> = inventory
        .iter()
        .filter(|(_, a, _)| *a == EcologyAdmission::EmittedEvidenceMostlyUnknown)
        .map(|(s, _, _)| *s)
        .collect();
    assert_eq!(
        emitted,
        vec!["reference_build_profile"],
        "T16.2 emitted exactly the ReferenceBuildProfile surface (evidence honest-unknown)"
    );
    // T16.3 emitted the locator + trust surfaces with genuine sealed-vs-live evidence.
    let emitted_scoped: Vec<&str> = inventory
        .iter()
        .filter(|(_, a, _)| *a == EcologyAdmission::EmittedScopedReference)
        .map(|(s, _, _)| *s)
        .collect();
    assert_eq!(
        emitted_scoped,
        vec!["reference_locator_kind", "signature_trust_model"],
        "T16.3 emitted exactly the locator + signature-trust surfaces"
    );
    // T16.4 built the aux-table validator as its own report surface.
    let built: Vec<&str> = inventory
        .iter()
        .filter(|(_, a, _)| *a == EcologyAdmission::BuiltSeparateReport)
        .map(|(s, _, _)| *s)
        .collect();
    assert_eq!(
        built,
        vec!["auxiliary_table_validator"],
        "T16.4 built exactly the aux-table validator surface"
    );
    // Every still-NotClaimedYet surface names an owning execution milestone (no orphaned gaps).
    for (surface, adm, milestone) in inventory {
        if *adm == EcologyAdmission::NotClaimedYet {
            assert!(
                milestone.starts_with("T16") || milestone.contains("T17"),
                "ecology surface {surface} has no owning execution milestone"
            );
        }
    }
}

/// **(T16.4 category boundary)** The aux-table validator is a **separate report surface**, not part of the
/// oracle/reference-identity surface — so its fields must **not** appear in the semantic-report
/// `oracle_identity`. (`BuiltSeparateReport` ≠ `EmittedScopedReference`: the validator prevents category
/// drift between release-ecology evidence and reference-oracle identity.)
#[test]
fn auxiliary_tables_not_emitted_as_reference_identity() {
    let sem = semantic_json();
    // The aux-table report's distinctive fields never leak into oracle_identity.
    for table_field in [
        "zone_universe",
        "country_code_authority",
        "aux-table-validation",
        "table_diagnostic_code_space",
    ] {
        assert!(
            !sem.contains(table_field),
            "aux-table field {table_field:?} must not appear in oracle_identity (separate surface):\n{sem}"
        );
    }
    // The oracle_identity *does* carry the reference-admission surface (the contrast).
    assert!(sem.contains("\"reference_admission\""));
}

/// **(T16.1, updated at T16.2)** The **not-yet-built** ecology report fields are still **genuinely absent**
/// — so each future T16.x addition flips them deliberately (the T15.1 shape-witness discipline). Note
/// `reference_build_profile` / `time_t_model` / `runtime_leap_support` are **no longer** in this list:
/// T16.2 emitted them (the deliberate flip), so they moved to the T16.2 presence tests below.
#[test]
fn ecology_report_fields_are_absent_until_their_milestone() {
    let s = support_json();
    let sem = semantic_json();
    // The aux-table validator (T16.4) is a *separate* report surface — its fields must NOT leak into the
    // conformance reports (support/semantic). `release_intake_provenance` is still genuinely unbuilt.
    for absent in [
        "release_intake_provenance",
        "zone_universe",
        "country_code_authority",
    ] {
        assert!(
            !s.contains(absent) && !sem.contains(absent),
            "ecology field {absent:?} must be absent until its T16.x milestone (deliberate flip):\nsupport={s}\nsemantic={sem}"
        );
    }
}

// --- T16.2: ReferenceBuildProfile capture (ecology evidence, not vibes) ---

/// **(T16.2)** The `reference_build_profile` is present in the semantic report and every axis carries an
/// explicit `disposition` — so a reader can tell measured-fact from honest-don't-know from refuse-to-guess.
#[test]
fn reference_build_profile_present_and_dispositioned() {
    let sem = semantic_json();
    assert!(
        sem.contains("\"reference_build_profile\""),
        "T16.2 reference_build_profile missing:\n{sem}"
    );
    for axis in [
        "\"source_release\"",
        "\"binary_sha256\"",
        "\"reference_platform\"",
        "\"build_flags\"",
        "\"time_t_model\"",
        "\"runtime_leap_support\"",
        "\"tzdir_resolution_policy\"",
        "\"locale\"",
        "\"warning_thresholds\"",
    ] {
        assert!(
            sem.contains(axis),
            "build-profile axis {axis} missing:\n{sem}"
        );
    }
    // Every axis is a `{ disposition, value }` object — no bare scalars that could read as guesses.
    assert!(sem.contains("\"disposition\""));
}

/// **(T16.2)** The build flags and `time_t` model are **never inferred** from the host/version — they
/// render `inferred_forbidden` (the typed non-inference is the claim). This is the "evidence not vibes"
/// guard: the report must not say `time_t_model: signed_64` just because the host is 64-bit.
#[test]
fn build_flags_and_time_t_are_inferred_forbidden_not_guessed() {
    let sem = semantic_json();
    // Both must be inferred_forbidden, and must NOT carry a value (no host/version guess).
    assert!(
        sem.contains(
            "\"build_flags\": { \"disposition\": \"inferred_forbidden\", \"value\": null }"
        ),
        "build_flags must be inferred_forbidden (never guessed from version):\n{sem}"
    );
    assert!(
        sem.contains(
            "\"time_t_model\": { \"disposition\": \"inferred_forbidden\", \"value\": null }"
        ),
        "time_t_model must be inferred_forbidden (never inferred from host width):\n{sem}"
    );
    // `runtime_leap_support` / `warning_thresholds` are honest unknowns, not silent assumptions.
    assert!(sem.contains("\"runtime_leap_support\": { \"disposition\": \"unknown_unmeasured\""));
    assert!(sem.contains("\"warning_thresholds\": { \"disposition\": \"unknown_unmeasured\""));
}

// --- T16.3: ReferenceLocatorKind + SignatureTrustModel (sealed vs exploration) ---

/// **(T16.3)** The semantic report emits `reference_admission` distinguishing the **live oracle** it
/// actually ran (a `live_current_directory` binary, `unknown` trust → **not** sealed) from the project's
/// **sealed reference** (the versioned + fingerprint-anchored 2026b archive → sealed). The central rule
/// is machine-visible: only the versioned archive supports a sealed claim.
#[test]
fn reference_admission_distinguishes_live_oracle_from_sealed_reference() {
    let sem = semantic_json();
    assert!(sem.contains("\"reference_admission\""), "missing:\n{sem}");
    // The live oracle is exploration-grade — never sealed.
    assert!(sem.contains(
        "\"live_oracle\": { \"locator\": \"live_current_directory\", \"signature_trust\": \"unknown\", \"supports_sealed_claim\": false }"
    ), "live oracle must be unsealed:\n{sem}");
    // The sealed reference is the versioned, fingerprint-anchored archive — sealed.
    assert!(sem.contains(
        "\"sealed_reference\": { \"locator\": \"versioned_archive\", \"signature_trust\": \"fingerprint_anchored\", \"supports_sealed_claim\": true }"
    ), "sealed reference must be versioned+fingerprint-anchored+sealed:\n{sem}");
    // Honesty: fingerprint-anchored is never downgraded to web-of-trust, and nothing claims "signature_verified".
    assert!(!sem.contains("web_of_trust_validated"));
    assert!(!sem.contains("signature_verified"));
}

/// **(T16.2)** The genuinely-known axis — the data-path policy — is `Known`, proving zic-rs hands the
/// oracle an explicit compiled-file path (not a zone name against system zoneinfo). And `reference_platform`
/// is finite-vocabulary (`std::env::consts::OS`), consistent with the flat `reference_platform` field.
#[test]
fn known_axes_are_known_and_platform_is_finite_vocab() {
    let sem = semantic_json();
    assert!(sem.contains(
        "\"tzdir_resolution_policy\": { \"disposition\": \"known\", \"value\": \"explicit_tzif_path_argument\" }"
    ));
    // reference_platform inside the profile is from the finite std::env::consts::OS set.
    let plat = std::env::consts::OS;
    assert!(
        [
            "linux",
            "macos",
            "windows",
            "freebsd",
            "netbsd",
            "openbsd",
            "dragonfly",
            "solaris",
            "android",
            "ios",
            "other"
        ]
        .contains(&plat),
        "unexpected platform token {plat}"
    );
    assert!(sem.contains(&format!(
        "\"reference_platform\": {{ \"disposition\": \"known\", \"value\": \"{plat}\" }}"
    )));
}

/// **(T16.1)** What *is* claimed today must stay claimed: the T15.5-remainder oracle identity (the honest
/// "which binary" answer that `ReferenceBuildProfile` will *extend*, not replace) is present in the
/// semantic report. This pins the inventory's "current state" row as true, not aspirational.
#[test]
fn current_oracle_identity_surface_is_present() {
    let sem = semantic_json();
    for present in [
        "\"oracle_identity\"",
        "\"zic_binary_sha256\"",
        "\"zdump_binary_sha256\"",
        "\"zoneinfo_resolution\": \"explicit_tzif_path_argument\"",
        "\"reference_platform\"",
    ] {
        assert!(
            sem.contains(present),
            "current oracle-identity surface {present} missing:\n{sem}"
        );
    }
}
