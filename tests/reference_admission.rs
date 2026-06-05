//! T16.3 — `ReferenceLocatorKind` + `SignatureTrustModel` (where a reference came from × how it is
//! trusted). The central rule, enforced as code not prose: **only a versioned archive with
//! integrity-pinned trust can back a *sealed* release claim**; a live-PATH binary is exploration-grade.
//! And trust is never overstated — `fingerprint_anchored` is not `web_of_trust_validated`, and
//! `hash_only` (integrity) is never rendered as "signature verified" (authenticity).

use tzcompile::manifest::{
    ReferenceAdmission, ReferenceLocatorKind, SignatureTrustModel, ADMITTED_2026B_REFERENCE,
};

/// **(T16.3)** Both vocabularies are finite + render distinct, stable, snake-case tokens.
#[test]
fn locator_and_trust_are_finite_vocab() {
    let locators = [
        ReferenceLocatorKind::VersionedArchive,
        ReferenceLocatorKind::LiveCurrentDirectory,
        ReferenceLocatorKind::LocalCachedCopy,
        ReferenceLocatorKind::DistroSourcePackage,
        ReferenceLocatorKind::Unknown,
    ];
    let trusts = [
        SignatureTrustModel::FingerprintAnchored,
        SignatureTrustModel::WebOfTrustValidated,
        SignatureTrustModel::PlatformKeyring,
        SignatureTrustModel::HashOnly,
        SignatureTrustModel::Unsigned,
        SignatureTrustModel::Unknown,
    ];
    // Distinct ids.
    let mut ls: Vec<&str> = locators.iter().map(|l| l.as_str()).collect();
    ls.sort_unstable();
    ls.dedup();
    assert_eq!(ls.len(), locators.len(), "locator ids must be distinct");
    let mut ts: Vec<&str> = trusts.iter().map(|t| t.as_str()).collect();
    ts.sort_unstable();
    ts.dedup();
    assert_eq!(ts.len(), trusts.len(), "trust ids must be distinct");
}

/// **(T16.3)** A versioned archive with integrity-pinned trust backs a sealed claim — this is the
/// admitted 2026b reference (versioned + fingerprint-anchored).
#[test]
fn versioned_archive_can_support_sealed_claim() {
    assert!(ADMITTED_2026B_REFERENCE.supports_sealed_claim());
    assert_eq!(
        ADMITTED_2026B_REFERENCE.locator,
        ReferenceLocatorKind::VersionedArchive
    );
    assert_eq!(
        ADMITTED_2026B_REFERENCE.trust,
        SignatureTrustModel::FingerprintAnchored
    );
}

/// **(T16.3)** A live-PATH binary — whatever the host has right now — can **never** back a sealed claim,
/// no matter how it is trusted: the locator alone disqualifies it (it moves under your feet).
#[test]
fn live_current_directory_cannot_support_sealed_claim() {
    for trust in [
        SignatureTrustModel::FingerprintAnchored,
        SignatureTrustModel::HashOnly,
        SignatureTrustModel::Unknown,
    ] {
        let a = ReferenceAdmission {
            locator: ReferenceLocatorKind::LiveCurrentDirectory,
            trust,
        };
        assert!(
            !a.supports_sealed_claim(),
            "live_current_directory must never back a sealed claim (trust={})",
            trust.as_str()
        );
    }
}

/// **(T16.3)** An `Unknown` locator never admits a sealed release claim, regardless of trust.
#[test]
fn unknown_locator_does_not_admit_release() {
    let a = ReferenceAdmission {
        locator: ReferenceLocatorKind::Unknown,
        trust: SignatureTrustModel::FingerprintAnchored,
    };
    assert!(!a.supports_sealed_claim());
}

/// **(T16.3)** `fingerprint_anchored` is rendered exactly — never collapsed into the weaker
/// `web_of_trust_validated`. The reader is told *what kind* of trust they are getting.
#[test]
fn fingerprint_anchored_not_rendered_as_web_of_trust() {
    assert_eq!(
        SignatureTrustModel::FingerprintAnchored.as_str(),
        "fingerprint_anchored"
    );
    assert_ne!(
        SignatureTrustModel::FingerprintAnchored.as_str(),
        SignatureTrustModel::WebOfTrustValidated.as_str()
    );
}

/// **(T16.3)** `hash_only` proves integrity, not authenticity — it must never read as "signature
/// verified". Its rendered token is `hash_only`, and it carries integrity but is a *distinct* model from
/// the authenticity-bearing ones.
#[test]
fn hash_only_not_rendered_as_signature_verified() {
    assert_eq!(SignatureTrustModel::HashOnly.as_str(), "hash_only");
    assert_ne!(SignatureTrustModel::HashOnly.as_str(), "signature_verified");
    // Integrity yes; but it is not the same claim as fingerprint-anchored authenticity.
    assert!(SignatureTrustModel::HashOnly.pins_integrity());
    assert_ne!(
        SignatureTrustModel::HashOnly.as_str(),
        SignatureTrustModel::FingerprintAnchored.as_str()
    );
}

/// **(T16.3)** Unsigned / unknown trust never pins integrity → never backs a sealed claim even from a
/// versioned archive (the bytes are not pinned).
#[test]
fn unsigned_or_unknown_trust_does_not_seal_even_from_archive() {
    for trust in [SignatureTrustModel::Unsigned, SignatureTrustModel::Unknown] {
        let a = ReferenceAdmission {
            locator: ReferenceLocatorKind::VersionedArchive,
            trust,
        };
        assert!(
            !a.supports_sealed_claim(),
            "an unverified archive ({}) is not sealed-claim-grade",
            trust.as_str()
        );
    }
}
