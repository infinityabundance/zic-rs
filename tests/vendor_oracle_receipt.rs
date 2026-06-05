//! T16.5 — external vendor-oracle **receipt admission**. The core repo admits *receipts* (verifies the
//! contract + rules); the QEMU/VM lab that *generates* them lives outside the repo. Tests pin: finite
//! vocabularies, the admission rules (never `Admitted` by assumption), the committed sample, the
//! hash-scope distinction, and the enforcing non-claim.

use tzcompile::vendor_oracle::{
    OracleExitDisposition, OracleRunStatus, ReceiptAdmission, ReceiptHashScope,
    ReferencePlatformStatus, VendorOracleReceipt,
};

/// **(T16.5)** The receipt vocabularies are finite + render distinct, stable, snake-case ids.
#[test]
fn vendor_oracle_receipt_schema_is_finite_vocab() {
    let platform = [
        ReferencePlatformStatus::Admitted,
        ReferencePlatformStatus::Unavailable,
        ReferencePlatformStatus::DocumentationOnly,
        ReferencePlatformStatus::SkippedWithReason,
        ReferencePlatformStatus::InadmissibleUnpinned,
        ReferencePlatformStatus::PendingExternalReceipt,
    ];
    let run = [
        OracleRunStatus::Completed,
        OracleRunStatus::SkippedUnavailable,
        OracleRunStatus::TimedOut,
        OracleRunStatus::Killed,
        OracleRunStatus::FailedToStart,
    ];
    let exit = [
        OracleExitDisposition::Success,
        OracleExitDisposition::NonzeroWithOutput,
        OracleExitDisposition::NonzeroNoOutput,
        OracleExitDisposition::TerminatedBySignal,
        OracleExitDisposition::TimedOut,
        OracleExitDisposition::NotRun,
    ];
    let distinct = |v: Vec<&str>| {
        let mut s = v.clone();
        s.sort_unstable();
        s.dedup();
        s.len() == v.len()
    };
    assert!(distinct(platform.iter().map(|p| p.as_str()).collect()));
    assert!(distinct(run.iter().map(|r| r.as_str()).collect()));
    assert!(distinct(exit.iter().map(|e| e.as_str()).collect()));
}

/// **(T16.5)** The committed minimal sample is **admissible** and round-trips to the committed fixture
/// (the canonical schema example a vendor lab fills in).
#[test]
fn minimal_sample_receipt_validates_and_matches_fixture() {
    let sample = VendorOracleReceipt::minimal_sample();
    assert_eq!(sample.admit(), ReceiptAdmission::Admitted);
    let golden = include_str!("../fixtures/vendor-oracle/minimal-sample-receipt.json");
    assert_eq!(
        sample.to_json().trim_end(),
        golden.trim_end(),
        "sample receipt drifted from the committed fixture"
    );
    // The emitted receipt embeds its own admission verdict.
    assert!(sample.to_json().contains("\"admission\": \"admitted\""));
}

/// **(T16.5)** A receipt with no platform is inadmissible (never admitted by assumption).
#[test]
fn receipt_with_missing_platform_rejected() {
    let mut r = VendorOracleReceipt::minimal_sample();
    r.platform = String::new();
    assert_eq!(r.admit(), ReceiptAdmission::InadmissibleMissingPlatform);
}

/// **(T16.5)** A receipt naming a fixture set the core does not know is inadmissible.
#[test]
fn receipt_with_unknown_fixture_set_rejected() {
    let mut r = VendorOracleReceipt::minimal_sample();
    r.fixture_set = "some-unknown-corpus-v9".to_string();
    assert_eq!(r.admit(), ReceiptAdmission::InadmissibleUnknownFixtureSet);
}

/// **(T16.5)** A nonzero exit (even with output) is **not admitted** the same as a clean success.
#[test]
fn receipt_with_nonzero_exit_marked_not_admitted() {
    let mut r = VendorOracleReceipt::minimal_sample();
    r.exit_disposition = OracleExitDisposition::NonzeroWithOutput;
    assert_eq!(r.admit(), ReceiptAdmission::NotAdmittedExitNotSuccess);
    assert!(!r.admit().is_admitted());
}

/// **(T16.5)** A timed-out / incomplete run is **not** admitted — and timeout is its own operational
/// result, never silently "unavailable" or "mismatch".
#[test]
fn receipt_with_incomplete_run_not_admitted() {
    let mut r = VendorOracleReceipt::minimal_sample();
    r.run_status = OracleRunStatus::TimedOut;
    assert_eq!(r.admit(), ReceiptAdmission::NotAdmittedRunIncomplete);
}

/// **(T16.5)** A platform cannot be admitted by assumption: a pending or unpinned platform status never
/// yields `Admitted`, regardless of a clean run/exit.
#[test]
fn platform_status_cannot_be_admitted_by_assumption() {
    let mut r = VendorOracleReceipt::minimal_sample();
    r.platform_status = ReferencePlatformStatus::PendingExternalReceipt;
    assert_eq!(r.admit(), ReceiptAdmission::PendingExternalReceipt);
    r.platform_status = ReferencePlatformStatus::InadmissibleUnpinned;
    assert_eq!(r.admit(), ReceiptAdmission::InadmissibleUnpinnedPlatform);
}

/// **(T16.5-core)** Strict admission: a `documentation_only` (or `unavailable`/`skipped`) platform is
/// **not** admitted even with a clean run/exit — platforms are admitted by evidence, never by status alone.
#[test]
fn documentation_only_platform_not_admitted() {
    use tzcompile::vendor_oracle::ReferencePlatformStatus;
    let mut r = VendorOracleReceipt::minimal_sample();
    r.platform_status = ReferencePlatformStatus::DocumentationOnly;
    assert_eq!(
        r.admit(),
        ReceiptAdmission::NotAdmittedPlatformNotAdmissible
    );
}

/// **(T16.5-core)** Strict admission requires conformance-grade evidence: an identified binary, exact
/// argv, a declared hash scope, and resolved verdicts. Each missing piece blocks admission with its
/// own typed reason.
#[test]
fn strict_admission_requires_conformance_grade_evidence() {
    use tzcompile::vendor_oracle::{
        ClassLocationVerdict, OracleInvocationIdentity, ReceiptHashScope,
    };
    // Missing binary identity.
    let mut r = VendorOracleReceipt::minimal_sample();
    r.zic_binary_sha256 = None;
    assert_eq!(
        r.admit(),
        ReceiptAdmission::NotAdmittedMissingBinaryIdentity
    );
    // Non-exact argv (a shell template is not conformance-grade).
    let mut r = VendorOracleReceipt::minimal_sample();
    r.invocation = OracleInvocationIdentity::TemplateOnly;
    assert_eq!(r.admit(), ReceiptAdmission::NotAdmittedNonExactArgv);
    // Undeclared hash scope.
    let mut r = VendorOracleReceipt::minimal_sample();
    r.hash_scope = ReceiptHashScope::Unknown;
    assert_eq!(r.admit(), ReceiptAdmission::NotAdmittedHashScopeUndeclared);
    // A non-matching verdict not covered by a declared known-divergence.
    let mut r = VendorOracleReceipt::minimal_sample();
    r.class_location_verdicts
        .push(("some_fixture".to_string(), ClassLocationVerdict::Divergence));
    assert_eq!(r.admit(), ReceiptAdmission::NotAdmittedUnresolvedVerdict);
    // ...but the same divergence becomes admissible when explicitly declared known.
    r.known_divergences.push("some_fixture".to_string());
    assert_eq!(r.admit(), ReceiptAdmission::Admitted);
}

/// **(T16.5b)** Ingestion is now implemented, but **scoped**: the boundary const records that the reader
/// is a no-dep, receipt-scoped, fail-closed reader — explicitly **not** a general JSON library.
#[test]
fn ingestion_is_scoped_not_a_general_json_library() {
    let boundary = tzcompile::vendor_oracle::EXTERNAL_JSON_INGESTION;
    assert!(boundary.contains("implemented_t16_5b"));
    assert!(boundary.contains("not_a_general_json_library"));
}

// --- T16.5b: external receipt JSON ingestion (no-dep, fail-closed, receipt-scoped) ---

/// **(T16.5b)** The committed sample JSON ingests through the no-dep reader and admits — and a full
/// **round-trip** (emit → parse → re-emit) is byte-stable.
#[test]
fn external_sample_receipt_ingests_and_admits() {
    let golden = include_str!("../fixtures/vendor-oracle/minimal-sample-receipt.json");
    let parsed = VendorOracleReceipt::from_json(golden).expect("sample must ingest");
    assert_eq!(parsed.admit(), ReceiptAdmission::Admitted);
    // Round-trip: the parsed receipt re-emits to the same canonical bytes.
    assert_eq!(parsed.to_json().trim_end(), golden.trim_end());
}

/// **(T16.5b)** Fail-closed: malformed JSON, missing fields, unknown enum values, wrong types, and
/// unrecognised fields are each a typed **parse error** — never a silently-coerced receipt.
#[test]
fn ingestion_fails_closed_on_bad_input() {
    use tzcompile::vendor_oracle::ReceiptParseError;
    let golden = include_str!("../fixtures/vendor-oracle/minimal-sample-receipt.json");
    // Malformed JSON.
    assert_eq!(
        VendorOracleReceipt::from_json("{ not json"),
        Err(ReceiptParseError::MalformedJson)
    );
    // Unknown enum value (fail-closed, never coerced).
    let bad_enum = golden.replace("\"completed\"", "\"sorta_completed\"");
    assert_eq!(
        VendorOracleReceipt::from_json(&bad_enum),
        Err(ReceiptParseError::UnknownEnumValue("oracle_run_status"))
    );
    // Missing required field (drop "platform").
    let missing = golden.replace("\"platform\": \"freebsd_14_x86_64\",\n  ", "");
    assert!(matches!(
        VendorOracleReceipt::from_json(&missing),
        Err(ReceiptParseError::MissingField("platform"))
    ));
    // Unrecognised top-level field (strict v1).
    let extra = golden.replacen("{\n", "{\n  \"surprise_field\": 1,\n", 1);
    assert_eq!(
        VendorOracleReceipt::from_json(&extra),
        Err(ReceiptParseError::UnknownField)
    );
    // Wrong type (argv as a string, not an array).
    let wrong = golden.replace(
        "\"argv\": [\"zic\", \"-v\", \"-d\", \"out\", \"fixture.zi\"]",
        "\"argv\": \"zic\"",
    );
    assert_eq!(
        VendorOracleReceipt::from_json(&wrong),
        Err(ReceiptParseError::WrongType("argv"))
    );
    // Wrong schema id.
    let wrong_schema = golden.replace("vendor-oracle-receipt-v1", "vendor-oracle-receipt-v2");
    assert_eq!(
        VendorOracleReceipt::from_json(&wrong_schema),
        Err(ReceiptParseError::WrongSchema)
    );
}

/// **(T16.5b)** The load-bearing distinction: a **parse failure is not an inadmissible receipt**. A
/// well-formed receipt that simply fails the rules parses cleanly (`Ok`) and is then non-admitted by
/// `admit()` — two different return paths, never conflated.
#[test]
fn parse_failure_is_distinct_from_inadmissible_receipt() {
    let golden = include_str!("../fixtures/vendor-oracle/minimal-sample-receipt.json");
    // A clean parse of a receipt whose run timed out: Ok(...) from the reader, NotAdmitted from admit().
    let timed_out = golden.replace("\"completed\"", "\"timed_out\"").replace(
        "\"admission\": \"admitted\"",
        "\"admission\": \"not_admitted_run_incomplete\"",
    );
    let r =
        VendorOracleReceipt::from_json(&timed_out).expect("a timed-out receipt still PARSES fine");
    assert_eq!(r.admit(), ReceiptAdmission::NotAdmittedRunIncomplete);
    assert!(!r.admit().is_admitted());
}

/// **(T16.5b)** The reader did not turn the core repo into a VM lab: the QEMU-external non-claim is still
/// advertised + guard-backed (ingestion is just reading bytes, never running platforms).
#[test]
fn qemu_lab_still_not_in_core_after_ingestion() {
    use tzcompile::manifest::NEGATIVE_CAPABILITIES;
    assert!(NEGATIVE_CAPABILITIES
        .iter()
        .any(|n| n.as_str() == "does_not_ship_or_operate_vendor_qemu_labs_in_core_repo"));
    // And ingestion is pure parsing — it never resolves/executes anything (scoped reader, not a VM).
    assert!(tzcompile::vendor_oracle::EXTERNAL_JSON_INGESTION.contains("implemented_t16_5b"));
}

/// **(T16.5)** `ReceiptHashScope` distinguishes the stable **claim** hash from the full **artifact** hash
/// (so a claim id does not churn on volatile run metadata like timestamps).
#[test]
fn receipt_hash_scope_distinguishes_stable_verdict_from_full_receipt() {
    assert_eq!(
        ReceiptHashScope::StableVerdictOnly.as_str(),
        "stable_verdict_only"
    );
    assert_eq!(
        ReceiptHashScope::FullReceiptIncludingRunMetadata.as_str(),
        "full_receipt_including_run_metadata"
    );
    assert_ne!(
        ReceiptHashScope::StableVerdictOnly.as_str(),
        ReceiptHashScope::FullReceiptIncludingRunMetadata.as_str()
    );
}

/// **(T16.5)** The enforcing non-claim is present + guard-backed: the core repo does not ship/operate
/// vendor QEMU labs (it admits receipts only).
#[test]
fn qemu_lab_not_in_core_negative_capability_present() {
    use tzcompile::manifest::{NegativeCapability, NEGATIVE_CAPABILITIES};
    let target = "does_not_ship_or_operate_vendor_qemu_labs_in_core_repo";
    let nc = NEGATIVE_CAPABILITIES
        .iter()
        .find(|n| n.as_str() == target)
        .expect("the QEMU-external non-claim must be advertised");
    assert!(
        !NegativeCapability::enforced_by(*nc).is_empty(),
        "the non-claim must name its enforcing guard"
    );
    // And the support-report advertises it.
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("in.zi");
    std::fs::write(&p, "Zone Etc/UTC 0:00 - UTC\n").unwrap();
    let db = tzcompile::load_database(std::slice::from_ref(&p)).unwrap();
    let j = tzcompile::report::build_support_report(&db, None).to_json();
    assert!(j.contains(target), "support-report must advertise {target}");
}
