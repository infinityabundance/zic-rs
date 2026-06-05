//! T15.4 — **RFC 9636 TZif structural validator**: byte-format integrity as its own first-class axis,
//! kept strictly separate from T15.3 semantic witnesses (selected behaviour) and from diagnostics.
//!
//! *Structural validity ≠ semantic match ≠ reader compatibility ≠ release admission.* The validator
//! therefore emits **several** typed verdicts, never one `valid: true`:
//!
//! * [`TzifStructuralVerdict`] — the bytes are internally well-formed (counted-array bounds, index
//!   safety, count consistency, ascending transitions — the security-relevant checks RFC 9636 calls out);
//! * [`PosixFooterVerdict`] — the v2+ footer is present and parseable (*projection*-match vs a reference
//!   is **semantic**, T15.3 — out of scope here);
//! * [`ReaderCompatibilityVerdict`] — known modern/legacy reader hazards (a well-formed file can still be
//!   hostile to old readers — extends `ZIC020`);
//! * [`LeapExpiryVerdict`] — leap-table expiration shape;
//! * [`TzifVersionVerdict`] — the version byte's interpretation.
//!
//! **Validate reference `zic` output too** (the first guard): a validator that only sees zic-rs's
//! own output risks validating its writer's assumptions. The `tzif-validate` command runs over both.
//! **Non-claims:** this is not a hardened security sandbox for arbitrary hostile binaries, and it does
//! not claim arbitrary-TZif round-trip preservation — it reuses the bounds-safe [`parse`]
//! (which returns `Err`, never panics, on malformed input — including, since T17.1, transition
//! type-index < typecnt enforced at the decode choke point) and adds the remaining RFC-9636 invariants
//! (indicator counts · ascending order · typecnt≥1; plus an explicit defence-in-depth type-index check).

use super::parse;
use super::validate::{
    indicator_pair_valid, isdst_byte_valid, rfc_designation_index_valid, utoff_structural_valid,
};
use crate::json::escape;

/// Whether the byte stream is internally well-formed per RFC 9636.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TzifStructuralVerdict {
    Conformant,
    /// One or more invariant violations (carried in [`TzifValidation::violations`]).
    Violation,
}

/// The POSIX footer's *parseability* (not its semantic future projection — that is T15.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PosixFooterVerdict {
    /// v1-only file: there is no footer to assess.
    NotApplicable,
    Absent,
    Parseable,
    ParseError,
}

/// Known reader-population hazards (a *separate* axis from structural validity).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReaderCompatibilityVerdict {
    NoKnownHazard,
    /// `timecnt > 1200` — pre-2014 clients may mishandle (mirrors `ZIC020`).
    LegacyTransitionCountHazard,
    /// v4 leap-expiration / truncation that strict older readers may reject.
    V4ReaderHazard,
    NotExercised,
    Unknown,
}

/// Leap-second-table expiration shape (the conformance axis beside leap/right/v4, T11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeapExpiryVerdict {
    NoLeapTable,
    LeapTableNoExpiration,
    /// A terminal no-op leap record (corr == previous corr) marks expiration (TZif v4).
    ExpirationPresent,
    NotApplicable,
}

/// The TZif version byte's interpretation (v4 is *not* "v3 plus extras").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TzifVersionVerdict {
    V1,
    V2,
    V3,
    V4,
    Unknown,
}

impl TzifStructuralVerdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            TzifStructuralVerdict::Conformant => "conformant",
            TzifStructuralVerdict::Violation => "violation",
        }
    }
}
impl PosixFooterVerdict {
    pub fn as_str(self) -> &'static str {
        match self {
            PosixFooterVerdict::NotApplicable => "not_applicable",
            PosixFooterVerdict::Absent => "absent",
            PosixFooterVerdict::Parseable => "parseable",
            PosixFooterVerdict::ParseError => "parse_error",
        }
    }
}
impl ReaderCompatibilityVerdict {
    pub fn as_str(self) -> &'static str {
        match self {
            ReaderCompatibilityVerdict::NoKnownHazard => "no_known_hazard",
            ReaderCompatibilityVerdict::LegacyTransitionCountHazard => {
                "legacy_transition_count_hazard"
            }
            ReaderCompatibilityVerdict::V4ReaderHazard => "v4_reader_hazard",
            ReaderCompatibilityVerdict::NotExercised => "not_exercised",
            ReaderCompatibilityVerdict::Unknown => "unknown",
        }
    }
}
impl LeapExpiryVerdict {
    pub fn as_str(self) -> &'static str {
        match self {
            LeapExpiryVerdict::NoLeapTable => "no_leap_table",
            LeapExpiryVerdict::LeapTableNoExpiration => "leap_table_no_expiration",
            LeapExpiryVerdict::ExpirationPresent => "expiration_present",
            LeapExpiryVerdict::NotApplicable => "not_applicable",
        }
    }
}
impl TzifVersionVerdict {
    pub fn as_str(self) -> &'static str {
        match self {
            TzifVersionVerdict::V1 => "v1",
            TzifVersionVerdict::V2 => "v2",
            TzifVersionVerdict::V3 => "v3",
            TzifVersionVerdict::V4 => "v4",
            TzifVersionVerdict::Unknown => "unknown",
        }
    }
}

/// The full set of typed verdicts for one TZif byte stream.
#[derive(Debug, Clone)]
pub struct TzifValidation {
    pub structural: TzifStructuralVerdict,
    pub footer: PosixFooterVerdict,
    pub reader_compatibility: ReaderCompatibilityVerdict,
    pub leap_expiry: LeapExpiryVerdict,
    pub version: TzifVersionVerdict,
    /// Human-readable invariant violations (empty iff `structural == Conformant`).
    pub violations: Vec<String>,
}

/// `1200 < timecnt` legacy-client threshold (pinned from `zic.c`; mirrors `ZIC020`).
const LEGACY_TIMECNT_THRESHOLD: u32 = 1200;

/// Validate a TZif byte stream against RFC 9636 structural invariants, returning typed verdicts.
///
/// Bounds-safe: it never panics on malformed input — the bounds-checked [`parse`] returns
/// `Err` for truncation / bad magic / out-of-range designation index (→ a structural `Violation`), and
/// the extra invariant checks here use `get`/comparisons, not indexing.
pub fn validate(bytes: &[u8]) -> TzifValidation {
    let parsed = match parse(bytes) {
        Ok(p) => p,
        Err(e) => {
            // Malformed at the byte level (magic/truncation/designation-index): a structural violation,
            // not a tool crash. Other axes are not assessable.
            return TzifValidation {
                structural: TzifStructuralVerdict::Violation,
                footer: PosixFooterVerdict::NotApplicable,
                reader_compatibility: ReaderCompatibilityVerdict::Unknown,
                leap_expiry: LeapExpiryVerdict::NotApplicable,
                version: TzifVersionVerdict::Unknown,
                violations: vec![format!("parse failed: {e}")],
            };
        }
    };
    let c = &parsed.counts;
    let mut v: Vec<String> = Vec::new();

    // RFC 9636 invariants. `parse` enforces the byte-level bounds (truncation, designation index, and —
    // since T17.1 — transition type-index < typecnt at the decode choke point); the rest are asserted here.
    // typecnt ≥ 1 (a reader needs at least one local-time-type).
    if c.typecnt == 0 {
        v.push("typecnt must be >= 1".into());
    }
    // Every transition's type index must be in range (the counted-array bounds-safety RFC flags).
    // Defence-in-depth: `parse` (T17.1) already rejects an out-of-range index, so on the current path a
    // successfully-`parse`d file cannot reach here with a bad index. Kept as an explicit RFC-invariant
    // assertion and a regression guard should the decode path ever stop enforcing it.
    for (i, t) in parsed.transitions.iter().enumerate() {
        if (t.type_index as u32) >= c.typecnt {
            v.push(format!(
                "transition[{i}] type index {} out of bounds (typecnt={})",
                t.type_index, c.typecnt
            ));
            break;
        }
    }
    // Indicator counts must be 0 or typecnt (RFC 9636 §3.2).
    if c.isutcnt != 0 && c.isutcnt != c.typecnt {
        v.push(format!(
            "isutcnt {} must be 0 or typecnt ({})",
            c.isutcnt, c.typecnt
        ));
    }
    if c.isstdcnt != 0 && c.isstdcnt != c.typecnt {
        v.push(format!(
            "isstdcnt {} must be 0 or typecnt ({})",
            c.isstdcnt, c.typecnt
        ));
    }
    // Transition times must be strictly ascending.
    if parsed.transitions.windows(2).any(|w| w[0].at >= w[1].at) {
        v.push("transition times are not strictly ascending".into());
    }

    // RFC 9636 §3.2 byte-value validity (T23.kani.3f.enforce). `parse` is intentionally
    // memory-safe-*lenient* — it does **not** reject on these byte values — so the strict FORMAT rules are
    // applied here over the raw bytes `parse` captured in `parsed.raw`, via the predicates proven by
    // T23.kani.3f.1–.4. These are **format-validity** checks, not memory-safety or semantic checks:
    // *memory-safe acceptance ≠ format-valid acceptance.* A file can `parse` (memory-safe) yet be a
    // structural `Violation` here.
    let raw = &parsed.raw;
    for (i, t) in parsed.types.iter().enumerate() {
        if !utoff_structural_valid(t.utoff) {
            v.push(format!(
                "ttinfo[{i}] utoff == i32::MIN — RFC 9636 §3.2 forbids -2^31"
            ));
        }
        if let Some(&isdst) = raw.isdst.get(i) {
            if !isdst_byte_valid(isdst) {
                v.push(format!("ttinfo[{i}] isdst byte {isdst} not in {{0,1}}"));
            }
        }
        if let Some(&didx) = raw.desigidx.get(i) {
            let charcnt = raw.designation.len();
            let idx = didx as usize;
            let has_nul = idx < charcnt && raw.designation[idx..].contains(&0);
            if !rfc_designation_index_valid(idx, charcnt, has_nul) {
                v.push(format!(
                    "ttinfo[{i}] designation index {didx} invalid for charcnt {charcnt} (RFC 9636 §3.2: idx<charcnt, charcnt!=0, NUL at/after idx)"
                ));
            }
        }
    }
    // Indicator octets: when both arrays are present (per-type), each must be 0/1 and `isut==1 ⇒ isstd==1`;
    // when only one is present (e.g. isutcnt==0), range-check it (a length mismatch is itself flagged by the
    // isutcnt/isstdcnt-must-be-0-or-typecnt check above).
    if raw.std_indicators.len() == raw.ut_indicators.len() {
        for (i, (&isut, &isstd)) in raw
            .ut_indicators
            .iter()
            .zip(raw.std_indicators.iter())
            .enumerate()
        {
            if !indicator_pair_valid(isut, isstd) {
                v.push(format!(
                    "indicator[{i}]: isut={isut} isstd={isstd} violates RFC 9636 §3.2 (each in {{0,1}}, isut==1 ⇒ isstd==1)"
                ));
            }
        }
    } else {
        for (i, &b) in raw.std_indicators.iter().enumerate() {
            if b > 1 {
                v.push(format!("isstd indicator[{i}] byte {b} not in {{0,1}}"));
            }
        }
        for (i, &b) in raw.ut_indicators.iter().enumerate() {
            if b > 1 {
                v.push(format!("isut indicator[{i}] byte {b} not in {{0,1}}"));
            }
        }
    }

    let structural = if v.is_empty() {
        TzifStructuralVerdict::Conformant
    } else {
        TzifStructuralVerdict::Violation
    };

    let version = match parsed.version {
        0 => TzifVersionVerdict::V1,
        b'2' => TzifVersionVerdict::V2,
        b'3' => TzifVersionVerdict::V3,
        b'4' => TzifVersionVerdict::V4,
        _ => TzifVersionVerdict::Unknown,
    };

    let footer = if matches!(version, TzifVersionVerdict::V1) {
        PosixFooterVerdict::NotApplicable
    } else if parsed.footer.is_empty() {
        // A v2+ file with no footer is unusual but not malformed; record it honestly.
        PosixFooterVerdict::Absent
    } else {
        // `parse` already round-tripped the footer text; treat its presence as parseable. (Deep POSIX-TZ
        // grammar validation is a future refinement; future-projection *match* is semantic, T15.3.)
        PosixFooterVerdict::Parseable
    };

    // Leap-expiry shape: a terminal no-op leap record (corr unchanged from the prior) is the v4 expiry marker.
    let leap_expiry = if parsed.leaps.is_empty() {
        LeapExpiryVerdict::NoLeapTable
    } else {
        let n = parsed.leaps.len();
        let expiry_marker = n >= 2 && parsed.leaps[n - 1].corr == parsed.leaps[n - 2].corr;
        if expiry_marker {
            LeapExpiryVerdict::ExpirationPresent
        } else {
            LeapExpiryVerdict::LeapTableNoExpiration
        }
    };

    let reader_compatibility = if c.timecnt > LEGACY_TIMECNT_THRESHOLD {
        ReaderCompatibilityVerdict::LegacyTransitionCountHazard
    } else if matches!(leap_expiry, LeapExpiryVerdict::ExpirationPresent)
        || matches!(version, TzifVersionVerdict::V4)
    {
        // v4 / leap-expiration: strict pre-v4 readers may reject.
        ReaderCompatibilityVerdict::V4ReaderHazard
    } else {
        ReaderCompatibilityVerdict::NoKnownHazard
    };

    TzifValidation {
        structural,
        footer,
        reader_compatibility,
        leap_expiry,
        version,
        violations: v,
    }
}

impl TzifValidation {
    /// Render the verdict block (all axes separate — never collapsed into one `valid`).
    pub fn to_json_fields(&self) -> String {
        let viol = self
            .violations
            .iter()
            .map(|s| escape(s))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "\"tzif_structural_verdict\": {}, \"posix_footer_verdict\": {}, \
             \"reader_compatibility_verdict\": {}, \"leap_expiry_verdict\": {}, \
             \"tzif_version_verdict\": {}, \"violations\": [{}]",
            escape(self.structural.as_str()),
            escape(self.footer.as_str()),
            escape(self.reader_compatibility.as_str()),
            escape(self.leap_expiry.as_str()),
            escape(self.version.as_str()),
            viol,
        )
    }
}

/// One validated artifact: which producer emitted it, the zone, and its verdicts.
#[derive(Debug, Clone)]
pub struct TzifValidationRow {
    pub producer: &'static str,
    pub zone: String,
    pub validation: TzifValidation,
}

/// A multi-zone, multi-producer validation report (`zic-rs-tzif-validation-v1`).
#[derive(Debug, Clone)]
pub struct TzifValidationReport {
    pub rows: Vec<TzifValidationRow>,
    /// Whether reference `zic` output was also validated (the guard against validating only our own
    /// writer's assumptions). `false` → reference `zic` unavailable; recorded, never silent.
    pub reference_validated: bool,
}

impl TzifValidationReport {
    pub fn to_json(&self) -> String {
        let mut s = String::new();
        s.push_str("{\n");
        s.push_str("  \"schema\": \"zic-rs-tzif-validation-v1\",\n");
        s.push_str(&format!(
            "  \"artifact_category\": {},\n",
            escape(crate::manifest::ArtifactCategory::StructuralValidationArtifact.as_str())
        ));
        s.push_str(&format!(
            "  \"reference_validated\": {},\n",
            self.reference_validated
        ));
        s.push_str(&format!(
            "  \"note\": {},\n",
            escape(
                "RFC 9636 byte-format integrity only — NOT semantic behaviour (semantic-report) and NOT a \
                 hardened security sandbox; does not claim arbitrary-TZif round-trip."
            )
        ));
        s.push_str("  \"validations\": [");
        for (i, row) in self.rows.iter().enumerate() {
            s.push_str(if i == 0 { "\n" } else { ",\n" });
            s.push_str(&format!(
                "    {{ \"producer\": {}, \"zone\": {}, {} }}",
                escape(row.producer),
                escape(&row.zone),
                row.validation.to_json_fields(),
            ));
        }
        s.push_str(if self.rows.is_empty() {
            "]\n"
        } else {
            "\n  ]\n"
        });
        s.push_str("}\n");
        s
    }
}

/// Validate zic-rs's compiled output for `zones`, and — the key guard — reference `zic`'s output too when
/// available (so the validator must respect the actual producer profile, not just zic-rs's assumptions).
pub fn build_validation_report(
    db: &crate::model::Database,
    zones: &[String],
    reference_zic: &str,
    inputs: &[std::path::PathBuf],
    work_dir: &std::path::Path,
) -> crate::error::Result<TzifValidationReport> {
    let mut rows = Vec::new();
    for zone in zones {
        if let Ok(bytes) = crate::compile_zone_to_bytes(db, zone) {
            rows.push(TzifValidationRow {
                producer: "zic_rs",
                zone: zone.clone(),
                validation: validate(&bytes),
            });
        }
    }
    let reference_validated = crate::compare::reference_zic::is_available(reference_zic);
    if reference_validated {
        let ref_root = work_dir.join("ref");
        crate::compare::reference_zic::compile_with_reference(reference_zic, inputs, &ref_root)?;
        for zone in zones {
            let p = crate::compare::reference_zic::compiled_path(&ref_root, zone);
            if let Ok(bytes) = std::fs::read(&p) {
                rows.push(TzifValidationRow {
                    producer: "reference_zic",
                    zone: zone.clone(),
                    validation: validate(&bytes),
                });
            }
        }
    }
    Ok(TzifValidationReport {
        rows,
        reference_validated,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tzif::{write_bytes, TzifData};

    fn valid_bytes() -> Vec<u8> {
        write_bytes(&TzifData::fixed(-18000, "EST", "EST5")).unwrap()
    }

    #[test]
    fn well_formed_zic_rs_output_is_conformant() {
        let r = validate(&valid_bytes());
        assert_eq!(
            r.structural,
            TzifStructuralVerdict::Conformant,
            "{:?}",
            r.violations
        );
        assert!(r.violations.is_empty());
        // A fixed zone is v2, footer present/parseable, no leap table, no known reader hazard.
        assert_eq!(r.version, TzifVersionVerdict::V2);
        assert_eq!(r.footer, PosixFooterVerdict::Parseable);
        assert_eq!(r.leap_expiry, LeapExpiryVerdict::NoLeapTable);
        assert_eq!(
            r.reader_compatibility,
            ReaderCompatibilityVerdict::NoKnownHazard
        );
    }

    #[test]
    fn bad_magic_is_a_violation_not_a_panic() {
        let r = validate(b"NOPEnotatzif................");
        assert_eq!(r.structural, TzifStructuralVerdict::Violation);
        assert!(r.violations[0].contains("parse failed"));
    }

    #[test]
    fn truncated_header_is_a_violation() {
        let r = validate(b"TZif2"); // magic + version, then truncated
        assert_eq!(r.structural, TzifStructuralVerdict::Violation);
    }

    #[test]
    fn empty_input_is_a_violation() {
        assert_eq!(validate(b"").structural, TzifStructuralVerdict::Violation);
    }

    /// A hand-built **v1-only** TZif whose single transition points at type index 5 when `typecnt == 1`.
    /// **Since T17.1** the transition-type-index bound is enforced at the source by `parse` (the choke
    /// point), so this input is rejected at decode and the validator surfaces it as a structural
    /// `Violation` via the parse-failure path — still a *verdict*, never a panic. (The validator also
    /// retains its own explicit type-index check as defence-in-depth / a regression guard should the
    /// decode path ever loosen; on the current path `parse` catches it first.)
    #[test]
    fn type_index_out_of_bounds_is_caught_as_a_violation() {
        let mut b: Vec<u8> = Vec::new();
        b.extend_from_slice(b"TZif"); // magic
        b.push(0); // version 0 (v1-only)
        b.extend_from_slice(&[0u8; 15]); // reserved
        for n in [0u32, 0, 0, 1, 1, 4] {
            // isutcnt, isstdcnt, leapcnt, timecnt=1, typecnt=1, charcnt=4
            b.extend_from_slice(&n.to_be_bytes());
        }
        b.extend_from_slice(&0i32.to_be_bytes()); // 1 transition time
        b.push(5); // transition type index = 5 → OUT OF BOUNDS (typecnt == 1)
        b.extend_from_slice(&[0, 0, 0, 0, 0, 0]); // 1 ttinfo: utoff=0, isdst=0, desigidx=0
        b.extend_from_slice(b"UTC\0"); // designation table (charcnt=4)

        let r = validate(&b);
        assert_eq!(r.structural, TzifStructuralVerdict::Violation);
        assert!(
            r.violations
                .iter()
                .any(|s| s.contains("type index 5 out of range")),
            "expected a type-index-bounds violation (now enforced by parse), got {:?}",
            r.violations
        );
    }

    /// Hand-build a **v1-only** TZif with exactly one local-time type and zero transitions, with
    /// caller-set `ttinfo` bytes / designation table / indicator arrays — so a test can make a file that
    /// `parse` accepts (memory-safe) yet that violates an RFC 9636 §3.2 byte-value rule.
    fn v1_one_type(
        utoff: i32,
        isdst: u8,
        desigidx: u8,
        designation: &[u8],
        std_ind: &[u8],
        ut_ind: &[u8],
    ) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(b"TZif");
        b.push(0); // version 0 (v1-only)
        b.extend_from_slice(&[0u8; 15]);
        // counts: isutcnt, isstdcnt, leapcnt, timecnt=0, typecnt=1, charcnt
        for n in [
            ut_ind.len() as u32,
            std_ind.len() as u32,
            0,
            0,
            1,
            designation.len() as u32,
        ] {
            b.extend_from_slice(&n.to_be_bytes());
        }
        // (no transition times / indices: timecnt == 0)
        b.extend_from_slice(&utoff.to_be_bytes()); // ttinfo[0].utoff
        b.push(isdst); // ttinfo[0].isdst
        b.push(desigidx); // ttinfo[0].desigidx
        b.extend_from_slice(designation);
        b.extend_from_slice(std_ind);
        b.extend_from_slice(ut_ind);
        b
    }

    #[test]
    fn v1_one_type_clean_is_conformant() {
        // Control: a clean one-type v1 file passes the new byte-value checks too.
        let r = validate(&v1_one_type(-18000, 1, 0, b"EST\0", &[0], &[0]));
        assert_eq!(
            r.structural,
            TzifStructuralVerdict::Conformant,
            "{:?}",
            r.violations
        );
    }

    // T23.kani.3f.enforce — each fixture **parses** (memory-safe) but is a **format-validity** `Violation`.
    // *memory-safe acceptance ≠ format-valid acceptance* — the predicates are T23.kani.3f.1–.4.
    #[test]
    fn isdst_byte_out_of_range_parses_but_is_a_format_violation() {
        let b = v1_one_type(0, 2, 0, b"UTC\0", &[], &[]); // isdst = 2
        assert!(
            parse(&b).is_ok(),
            "parse is memory-safe-lenient (does not reject isdst=2)"
        );
        let r = validate(&b);
        assert_eq!(r.structural, TzifStructuralVerdict::Violation);
        assert!(
            r.violations.iter().any(|s| s.contains("isdst byte 2")),
            "{:?}",
            r.violations
        );
    }

    #[test]
    fn designation_index_equals_charcnt_parses_but_is_a_format_violation() {
        // desigidx == charcnt (4): slice-safe (T23.kani.3b) but NOT RFC-valid (T23.kani.3f.1).
        let b = v1_one_type(0, 0, 4, b"UTC\0", &[], &[]);
        assert!(parse(&b).is_ok());
        let r = validate(&b);
        assert_eq!(r.structural, TzifStructuralVerdict::Violation);
        assert!(
            r.violations.iter().any(|s| s.contains("designation index")),
            "{:?}",
            r.violations
        );
    }

    #[test]
    fn designation_without_nul_parses_but_is_a_format_violation() {
        let b = v1_one_type(0, 0, 0, b"ABC", &[], &[]); // idx 0 < charcnt 3 but no NUL at/after
        assert!(parse(&b).is_ok());
        let r = validate(&b);
        assert_eq!(r.structural, TzifStructuralVerdict::Violation);
        assert!(
            r.violations.iter().any(|s| s.contains("designation index")),
            "{:?}",
            r.violations
        );
    }

    #[test]
    fn indicator_pair_ut_without_std_parses_but_is_a_format_violation() {
        let b = v1_one_type(0, 0, 0, b"UTC\0", &[0], &[1]); // isstd=0, isut=1 → isut=1 ⇒ isstd=1 violated
        assert!(parse(&b).is_ok());
        let r = validate(&b);
        assert_eq!(r.structural, TzifStructuralVerdict::Violation);
        assert!(
            r.violations.iter().any(|s| s.contains("isut=1 isstd=0")),
            "{:?}",
            r.violations
        );
    }

    #[test]
    fn indicator_byte_out_of_range_parses_but_is_a_format_violation() {
        let b = v1_one_type(0, 0, 0, b"UTC\0", &[2], &[]); // isstd indicator byte = 2 (only std present)
        assert!(parse(&b).is_ok());
        let r = validate(&b);
        assert_eq!(r.structural, TzifStructuralVerdict::Violation);
        assert!(
            r.violations.iter().any(|s| s.contains("isstd indicator")),
            "{:?}",
            r.violations
        );
    }

    #[test]
    fn utoff_i32_min_parses_but_is_a_format_violation() {
        let b = v1_one_type(i32::MIN, 0, 0, b"UTC\0", &[], &[]);
        assert!(parse(&b).is_ok());
        let r = validate(&b);
        assert_eq!(r.structural, TzifStructuralVerdict::Violation);
        assert!(
            r.violations.iter().any(|s| s.contains("utoff == i32::MIN")),
            "{:?}",
            r.violations
        );
    }
}
