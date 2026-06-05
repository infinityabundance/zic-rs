//! T16.5 — **external vendor-oracle receipt admission**.
//!
//! Doctrine (load-bearing): ***the core repo admits evidence; the external lab generates it.*** The
//! QEMU/VM/OS-install orchestration that runs a vendor `zic`/`zdump` against the T13/T14 diagnostic
//! fixture corpus lives **outside** this repository (an external `zic-rs-vendor-oracle-lab`). What lives
//! here is only: the typed **receipt contract** (`vendor-oracle-receipt-v1`), the **admission rules**
//! (`VendorOracleReceipt::admit`), canonical JSON **emission**, a minimal **sample** receipt, and the
//! enforcing non-claim. The core repo never runs a VM.
//!
//! **Scope (T16.5-core + T16.5b):** the core *defines* the typed receipt contract, *emits* its canonical
//! JSON, *ingests* external v1 receipts via a **no-dep, receipt-scoped, fail-closed** reader
//! ([`VendorOracleReceipt::from_json`] — T16.5b), and *admits* them with strict rules. The reader is
//! **not** a general JSON library: v1 object shape + known string enums only, unknown fields/values
//! rejected. **Ingestion is parsing, never execution** — a parse failure ([`ReceiptParseError`]) is *not*
//! an inadmissible receipt, and the reader runs no platforms. The seal is *"the core can admit a
//! receipt"*, never *"all vendor platforms are tested"* — platform rows are admitted only by evidence.

use crate::json::escape;

/// The admission status of a vendor/platform reference (T16.5) — never `Admitted` by assumption. A real
/// platform row reaches `Admitted` only via an admitted receipt that passes the rules; everything else is
/// an honest non-admitted state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferencePlatformStatus {
    Admitted,
    Unavailable,
    DocumentationOnly,
    SkippedWithReason,
    /// A binary exists but cannot be identified sufficiently to admit (the inadmissibility disposition).
    InadmissibleUnpinned,
    /// A receipt is expected from the external lab but has not been admitted yet.
    PendingExternalReceipt,
}

impl ReferencePlatformStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            ReferencePlatformStatus::Admitted => "admitted",
            ReferencePlatformStatus::Unavailable => "unavailable",
            ReferencePlatformStatus::DocumentationOnly => "documentation_only",
            ReferencePlatformStatus::SkippedWithReason => "skipped_with_reason",
            ReferencePlatformStatus::InadmissibleUnpinned => "inadmissible_unpinned",
            ReferencePlatformStatus::PendingExternalReceipt => "pending_external_receipt",
        }
    }
}

/// How the oracle *run* concluded operationally (T16.5). A **timeout is neither "unavailable" nor
/// "mismatch"** — it is its own operational result, so it can never be silently read as a verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OracleRunStatus {
    Completed,
    SkippedUnavailable,
    TimedOut,
    Killed,
    FailedToStart,
}

impl OracleRunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            OracleRunStatus::Completed => "completed",
            OracleRunStatus::SkippedUnavailable => "skipped_unavailable",
            OracleRunStatus::TimedOut => "timed_out",
            OracleRunStatus::Killed => "killed",
            OracleRunStatus::FailedToStart => "failed_to_start",
        }
    }
}

/// The oracle process's **exit disposition** (T16.5) — a parsed output with a nonzero exit is **not**
/// admitted the same way as a clean success.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OracleExitDisposition {
    Success,
    NonzeroWithOutput,
    NonzeroNoOutput,
    TerminatedBySignal,
    TimedOut,
    NotRun,
}

impl OracleExitDisposition {
    pub fn as_str(self) -> &'static str {
        match self {
            OracleExitDisposition::Success => "success",
            OracleExitDisposition::NonzeroWithOutput => "nonzero_with_output",
            OracleExitDisposition::NonzeroNoOutput => "nonzero_no_output",
            OracleExitDisposition::TerminatedBySignal => "terminated_by_signal",
            OracleExitDisposition::TimedOut => "timed_out",
            OracleExitDisposition::NotRun => "not_run",
        }
    }
}

/// Encoding metadata for any captured stderr (T16.5) — class/location comparison is primary, but raw
/// captures still need encoding so a text mismatch is never confused with a semantic one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StderrEncoding {
    Utf8,
    LocaleDependent,
    BytesOnly,
    Unknown,
}

impl StderrEncoding {
    pub fn as_str(self) -> &'static str {
        match self {
            StderrEncoding::Utf8 => "utf8",
            StderrEncoding::LocaleDependent => "locale_dependent",
            StderrEncoding::BytesOnly => "bytes_only",
            StderrEncoding::Unknown => "unknown",
        }
    }
}

/// What a receipt hash covers (T16.5) — the **claim identity** (stable verdict content) is separated from
/// the **artifact identity** (full receipt including volatile run metadata like timestamps), so a claim
/// hash does not churn on every run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiptHashScope {
    StableVerdictOnly,
    FullReceiptIncludingRunMetadata,
    Unknown,
}

impl ReceiptHashScope {
    pub fn as_str(self) -> &'static str {
        match self {
            ReceiptHashScope::StableVerdictOnly => "stable_verdict_only",
            ReceiptHashScope::FullReceiptIncludingRunMetadata => {
                "full_receipt_including_run_metadata"
            }
            ReceiptHashScope::Unknown => "unknown",
        }
    }
}

/// How the oracle was invoked (T16.5) — prefer **exact argv** over a command-string *template*, so
/// quoting/shell-interpretation ambiguity cannot creep into conformance-grade evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OracleInvocationIdentity {
    ArgvExact,
    TemplateOnly,
    ShellExpanded,
    Unknown,
}

impl OracleInvocationIdentity {
    pub fn as_str(self) -> &'static str {
        match self {
            OracleInvocationIdentity::ArgvExact => "argv_exact",
            OracleInvocationIdentity::TemplateOnly => "template_only",
            OracleInvocationIdentity::ShellExpanded => "shell_expanded",
            OracleInvocationIdentity::Unknown => "unknown",
        }
    }
}

/// The class/location comparison verdict for one diagnostic fixture (T16.5) — wording compared **last**,
/// per the T13 rule; this is the class+location axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassLocationVerdict {
    ClassLocationMatch,
    ClassMatchLocationDiff,
    Divergence,
    ReferenceOnly,
    CandidateOnly,
    NotCompared,
}

impl ClassLocationVerdict {
    pub fn as_str(self) -> &'static str {
        match self {
            ClassLocationVerdict::ClassLocationMatch => "class_location_match",
            ClassLocationVerdict::ClassMatchLocationDiff => "class_match_location_diff",
            ClassLocationVerdict::Divergence => "divergence",
            ClassLocationVerdict::ReferenceOnly => "reference_only",
            ClassLocationVerdict::CandidateOnly => "candidate_only",
            ClassLocationVerdict::NotCompared => "not_compared",
        }
    }
}

/// The fixture sets the core admits receipts *for* (T16.5). A receipt naming any other set is
/// inadmissible — the core never admits a receipt against an unknown corpus.
pub const KNOWN_FIXTURE_SETS: &[&str] = &["diagnostic-fixtures-v1"];

/// **The external-JSON ingestion scope (T16.5-core defined the contract; T16.5b shipped the reader).** The
/// core now ingests v1 receipts via a **no-dep, receipt-scoped, fail-closed** reader
/// ([`VendorOracleReceipt::from_json`]) — it is **not** a general JSON library (v1 object shape + known
/// string enums only; unknown fields/values rejected). Two load-bearing doctrine lines:
/// - *The core repository admits vendor-oracle evidence by typed receipt contract + a scoped reader. It
///   does not generate the vendor evidence and does not ship the VM lab.*
/// - *Ingestion is **parsing**, never execution: a parse failure is not an inadmissible receipt, and the
///   reader runs no platforms.*
pub const EXTERNAL_JSON_INGESTION: &str =
    "implemented_t16_5b__no_dep_receipt_scoped_fail_closed_reader__not_a_general_json_library";

/// The verifier's decision on a receipt (T16.5). `Admitted` is reached **only** when the platform is
/// named, the fixture set is known, the platform status is admissible, the run completed, and the exit
/// was a clean success — never by assumption.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiptAdmission {
    Admitted,
    NotAdmittedRunIncomplete,
    NotAdmittedExitNotSuccess,
    /// Platform status is `unavailable`/`documentation_only`/`skipped` — not admitted by assumption.
    NotAdmittedPlatformNotAdmissible,
    /// No `zic_binary_sha256` — the reference binary is not identified.
    NotAdmittedMissingBinaryIdentity,
    /// Invocation is not `argv_exact` (a shell template/expansion is not conformance-grade).
    NotAdmittedNonExactArgv,
    /// `receipt_hash_scope` is `unknown` — the claim/artifact hash scope was not declared.
    NotAdmittedHashScopeUndeclared,
    /// A class/location verdict is non-matching and not covered by a declared known-divergence.
    NotAdmittedUnresolvedVerdict,
    PendingExternalReceipt,
    InadmissibleMissingPlatform,
    InadmissibleUnknownFixtureSet,
    InadmissibleUnpinnedPlatform,
}

impl ReceiptAdmission {
    pub fn as_str(self) -> &'static str {
        match self {
            ReceiptAdmission::Admitted => "admitted",
            ReceiptAdmission::NotAdmittedRunIncomplete => "not_admitted_run_incomplete",
            ReceiptAdmission::NotAdmittedExitNotSuccess => "not_admitted_exit_not_success",
            ReceiptAdmission::NotAdmittedPlatformNotAdmissible => {
                "not_admitted_platform_not_admissible"
            }
            ReceiptAdmission::NotAdmittedMissingBinaryIdentity => {
                "not_admitted_missing_binary_identity"
            }
            ReceiptAdmission::NotAdmittedNonExactArgv => "not_admitted_non_exact_argv",
            ReceiptAdmission::NotAdmittedHashScopeUndeclared => {
                "not_admitted_hash_scope_undeclared"
            }
            ReceiptAdmission::NotAdmittedUnresolvedVerdict => "not_admitted_unresolved_verdict",
            ReceiptAdmission::PendingExternalReceipt => "pending_external_receipt",
            ReceiptAdmission::InadmissibleMissingPlatform => "inadmissible_missing_platform",
            ReceiptAdmission::InadmissibleUnknownFixtureSet => "inadmissible_unknown_fixture_set",
            ReceiptAdmission::InadmissibleUnpinnedPlatform => "inadmissible_unpinned_platform",
        }
    }
    pub fn is_admitted(self) -> bool {
        self == ReceiptAdmission::Admitted
    }
}

/// One externally-generated vendor/platform diagnostic-oracle receipt (`vendor-oracle-receipt-v1`). The
/// external lab fills this in from a real vendor `zic`/`zdump` run against the fixture corpus; the core
/// repo only **admits** it (verifies the contract + rules) and emits its canonical JSON form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VendorOracleReceipt {
    /// e.g. `freebsd_14_x86_64` (empty ⇒ inadmissible).
    pub platform: String,
    pub platform_status: ReferencePlatformStatus,
    pub run_status: OracleRunStatus,
    pub exit_disposition: OracleExitDisposition,
    pub stderr_encoding: StderrEncoding,
    pub hash_scope: ReceiptHashScope,
    pub invocation: OracleInvocationIdentity,
    /// The **exact argv** of the oracle invocation (preferred over a shell string).
    pub argv: Vec<String>,
    /// Which fixture corpus was run (must be in [`KNOWN_FIXTURE_SETS`]).
    pub fixture_set: String,
    pub zic_binary_sha256: Option<String>,
    pub zic_version_output: Option<String>,
    pub tzdb_source_release: String,
    /// Distro patch-stack identity, or `None` = not claimed (pristine/unknown).
    pub patch_stack: Option<String>,
    /// Per-fixture class/location verdicts (the T13 comparison axis).
    pub class_location_verdicts: Vec<(String, ClassLocationVerdict)>,
    pub known_divergences: Vec<String>,
    pub skipped_with_reason: Option<String>,
    pub inadmissible_reason: Option<String>,
}

impl VendorOracleReceipt {
    /// Verify the receipt against the **strict** admission rules (T16.5-core). `Admitted` is reached
    /// **only** when *every* condition holds — never by assumption. Precedence (first failing wins):
    /// missing platform → unknown fixture set → platform status (unpinned / pending / not-admissible) →
    /// run incomplete → exit not success → missing binary identity → non-exact argv → hash scope
    /// undeclared → an unresolved (non-match, not-known-divergence) class/location verdict.
    pub fn admit(&self) -> ReceiptAdmission {
        if self.platform.trim().is_empty() {
            return ReceiptAdmission::InadmissibleMissingPlatform;
        }
        if !KNOWN_FIXTURE_SETS.contains(&self.fixture_set.as_str()) {
            return ReceiptAdmission::InadmissibleUnknownFixtureSet;
        }
        match self.platform_status {
            ReferencePlatformStatus::InadmissibleUnpinned => {
                return ReceiptAdmission::InadmissibleUnpinnedPlatform
            }
            ReferencePlatformStatus::PendingExternalReceipt => {
                return ReceiptAdmission::PendingExternalReceipt
            }
            // Never admitted by assumption: `unavailable`/`documentation_only`/`skipped` cannot seal.
            ReferencePlatformStatus::Unavailable
            | ReferencePlatformStatus::DocumentationOnly
            | ReferencePlatformStatus::SkippedWithReason => {
                return ReceiptAdmission::NotAdmittedPlatformNotAdmissible
            }
            ReferencePlatformStatus::Admitted => {}
        }
        if self.run_status != OracleRunStatus::Completed {
            return ReceiptAdmission::NotAdmittedRunIncomplete;
        }
        if self.exit_disposition != OracleExitDisposition::Success {
            return ReceiptAdmission::NotAdmittedExitNotSuccess;
        }
        // Conformance-grade evidence: identified binary + exact argv + a declared hash scope.
        if self.zic_binary_sha256.is_none() {
            return ReceiptAdmission::NotAdmittedMissingBinaryIdentity;
        }
        if self.invocation != OracleInvocationIdentity::ArgvExact {
            return ReceiptAdmission::NotAdmittedNonExactArgv;
        }
        if self.hash_scope == ReceiptHashScope::Unknown {
            return ReceiptAdmission::NotAdmittedHashScopeUndeclared;
        }
        // Every class/location verdict must match, or be a fixture explicitly listed as a known divergence.
        let all_resolved = self.class_location_verdicts.iter().all(|(fixture, v)| {
            *v == ClassLocationVerdict::ClassLocationMatch
                || self.known_divergences.iter().any(|d| d == fixture)
        });
        if !all_resolved {
            return ReceiptAdmission::NotAdmittedUnresolvedVerdict;
        }
        ReceiptAdmission::Admitted
    }

    /// Emit the canonical `vendor-oracle-receipt-v1` JSON (deterministic; the interchange form).
    pub fn to_json(&self) -> String {
        let opt = |o: &Option<String>| match o {
            Some(v) => escape(v),
            None => "null".to_string(),
        };
        let mut argv = String::from("[");
        for (i, a) in self.argv.iter().enumerate() {
            if i > 0 {
                argv.push_str(", ");
            }
            argv.push_str(&escape(a));
        }
        argv.push(']');
        let mut verdicts = String::from("[");
        for (i, (fixture, v)) in self.class_location_verdicts.iter().enumerate() {
            if i > 0 {
                verdicts.push_str(", ");
            }
            verdicts.push_str(&format!(
                "{{ \"fixture\": {}, \"verdict\": {} }}",
                escape(fixture),
                escape(v.as_str())
            ));
        }
        verdicts.push(']');
        let mut divergences = String::from("[");
        for (i, d) in self.known_divergences.iter().enumerate() {
            if i > 0 {
                divergences.push_str(", ");
            }
            divergences.push_str(&escape(d));
        }
        divergences.push(']');
        format!(
            "{{\n  \"schema\": \"vendor-oracle-receipt-v1\",\n  \
             \"admission\": {},\n  \"platform\": {},\n  \"platform_status\": {},\n  \
             \"oracle_run_status\": {},\n  \"oracle_exit_disposition\": {},\n  \
             \"stderr_encoding\": {},\n  \"receipt_hash_scope\": {},\n  \
             \"oracle_invocation_identity\": {},\n  \"argv\": {},\n  \"fixture_set\": {},\n  \
             \"zic_binary_sha256\": {},\n  \"zic_version_output\": {},\n  \"tzdb_source_release\": {},\n  \
             \"patch_stack_identity\": {},\n  \"class_location_verdicts\": {},\n  \
             \"known_divergences\": {},\n  \"skipped_with_reason\": {},\n  \"inadmissible_reason\": {}\n}}\n",
            escape(self.admit().as_str()),
            escape(&self.platform),
            escape(self.platform_status.as_str()),
            escape(self.run_status.as_str()),
            escape(self.exit_disposition.as_str()),
            escape(self.stderr_encoding.as_str()),
            escape(self.hash_scope.as_str()),
            escape(self.invocation.as_str()),
            argv,
            escape(&self.fixture_set),
            opt(&self.zic_binary_sha256),
            opt(&self.zic_version_output),
            escape(&self.tzdb_source_release),
            opt(&self.patch_stack),
            verdicts,
            divergences,
            opt(&self.skipped_with_reason),
            opt(&self.inadmissible_reason),
        )
    }

    /// A minimal **admissible** sample receipt (the canonical schema example). Used by the shape test and
    /// emittable as documentation. Represents a clean FreeBSD run that the core would admit.
    pub fn minimal_sample() -> Self {
        VendorOracleReceipt {
            platform: "freebsd_14_x86_64".to_string(),
            platform_status: ReferencePlatformStatus::Admitted,
            run_status: OracleRunStatus::Completed,
            exit_disposition: OracleExitDisposition::Success,
            stderr_encoding: StderrEncoding::Utf8,
            hash_scope: ReceiptHashScope::StableVerdictOnly,
            invocation: OracleInvocationIdentity::ArgvExact,
            argv: vec![
                "zic".to_string(),
                "-v".to_string(),
                "-d".to_string(),
                "out".to_string(),
                "fixture.zi".to_string(),
            ],
            fixture_set: "diagnostic-fixtures-v1".to_string(),
            zic_binary_sha256: Some("0".repeat(64)),
            zic_version_output: Some("zic (FreeBSD) 2026b".to_string()),
            tzdb_source_release: "2026b".to_string(),
            patch_stack: None,
            class_location_verdicts: vec![
                (
                    "unknown_line_type".to_string(),
                    ClassLocationVerdict::ClassLocationMatch,
                ),
                (
                    "duplicate_zone".to_string(),
                    ClassLocationVerdict::ClassLocationMatch,
                ),
            ],
            known_divergences: Vec::new(),
            skipped_with_reason: None,
            inadmissible_reason: None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// T16.5b — external receipt JSON **ingestion** (a no-dep reader, scoped to `vendor-oracle-receipt-v1`).
//
// This is **not** a general JSON library: it parses exactly the receipt's known object shape and known
// string enums, and **fails closed** on anything else (malformed JSON, missing/extra fields, wrong types,
// unknown enum values). The load-bearing distinction: a **parse failure** ([`ReceiptParseError`]) is *not*
// the same as **inadmissible evidence** — a receipt may parse cleanly and still be non-admitted by
// [`VendorOracleReceipt::admit`]. QEMU/VM orchestration remains entirely outside the core repo.
// ─────────────────────────────────────────────────────────────────────────────────────────────────

/// Why an external receipt could not be turned into a typed [`VendorOracleReceipt`] (T16.5b). Distinct
/// from admission: parse errors mean "this is not a well-formed v1 receipt", never "the evidence is bad".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReceiptParseError {
    /// The bytes are not well-formed JSON, or not a top-level object.
    MalformedJson,
    /// `schema` is missing or not `vendor-oracle-receipt-v1`.
    WrongSchema,
    /// A required field is absent.
    MissingField(&'static str),
    /// A field has the wrong JSON type (e.g. a string where an array was required).
    WrongType(&'static str),
    /// A string enum carried a value outside its finite vocabulary (fail-closed; never coerced).
    UnknownEnumValue(&'static str),
    /// An unrecognised top-level field (strict v1: we refuse receipts carrying claims we don't model).
    UnknownField,
}

impl std::fmt::Display for ReceiptParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReceiptParseError::MalformedJson => {
                write!(f, "malformed JSON (not a v1 receipt object)")
            }
            ReceiptParseError::WrongSchema => write!(f, "schema is not vendor-oracle-receipt-v1"),
            ReceiptParseError::MissingField(n) => write!(f, "missing required field: {n}"),
            ReceiptParseError::WrongType(n) => write!(f, "field has wrong type: {n}"),
            ReceiptParseError::UnknownEnumValue(n) => write!(f, "unknown enum value in field: {n}"),
            ReceiptParseError::UnknownField => write!(f, "unrecognised field (strict v1 receipt)"),
        }
    }
}

/// A minimal JSON value — the *only* shapes a receipt uses. Numbers are kept as raw text (receipts have
/// no numeric fields, but the grammar accepts them so a stray number fails at the *field* layer, not the
/// tokenizer). This type is private and receipt-scoped on purpose.
#[derive(Debug, Clone, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Num(String),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

/// A tiny recursive-descent JSON parser (RFC 8259 subset; fail-closed). Scoped to receipt ingestion — it
/// is intentionally *not* exposed as a general JSON facility.
struct JsonReader<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> JsonReader<'a> {
    fn new(s: &'a str) -> Self {
        JsonReader {
            b: s.as_bytes(),
            i: 0,
        }
    }
    fn ws(&mut self) {
        while self.i < self.b.len() && matches!(self.b[self.i], b' ' | b'\t' | b'\n' | b'\r') {
            self.i += 1;
        }
    }
    fn peek(&self) -> Option<u8> {
        self.b.get(self.i).copied()
    }
    fn parse(&mut self) -> Option<Json> {
        self.ws();
        let v = self.value()?;
        self.ws();
        // Reject trailing junk (fail-closed): the whole input must be exactly one value.
        if self.i == self.b.len() {
            Some(v)
        } else {
            None
        }
    }
    fn value(&mut self) -> Option<Json> {
        self.ws();
        match self.peek()? {
            b'{' => self.object(),
            b'[' => self.array(),
            b'"' => self.string().map(Json::Str),
            b't' => self.lit(b"true").then_some(Json::Bool(true)),
            b'f' => self.lit(b"false").then_some(Json::Bool(false)),
            b'n' => self.lit(b"null").then_some(Json::Null),
            b'-' | b'0'..=b'9' => self.number(),
            _ => None,
        }
    }
    fn lit(&mut self, kw: &[u8]) -> bool {
        if self.b[self.i..].starts_with(kw) {
            self.i += kw.len();
            true
        } else {
            false
        }
    }
    fn number(&mut self) -> Option<Json> {
        let start = self.i;
        if self.peek() == Some(b'-') {
            self.i += 1;
        }
        let mut saw_digit = false;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || matches!(c, b'.' | b'e' | b'E' | b'+' | b'-') {
                saw_digit |= c.is_ascii_digit();
                self.i += 1;
            } else {
                break;
            }
        }
        if !saw_digit {
            return None;
        }
        Some(Json::Num(
            std::str::from_utf8(&self.b[start..self.i])
                .ok()?
                .to_string(),
        ))
    }
    fn string(&mut self) -> Option<String> {
        if self.peek() != Some(b'"') {
            return None;
        }
        self.i += 1;
        let mut out = String::new();
        loop {
            let c = self.peek()?;
            self.i += 1;
            match c {
                b'"' => return Some(out),
                b'\\' => {
                    let e = self.peek()?;
                    self.i += 1;
                    match e {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            // Exactly 4 hex digits → a BMP scalar (receipts emit only `\u00XX` controls).
                            if self.i + 4 > self.b.len() {
                                return None;
                            }
                            let hex = std::str::from_utf8(&self.b[self.i..self.i + 4]).ok()?;
                            let cp = u32::from_str_radix(hex, 16).ok()?;
                            out.push(char::from_u32(cp)?);
                            self.i += 4;
                        }
                        _ => return None,
                    }
                }
                // A raw control byte inside a string is invalid JSON (fail-closed).
                0x00..=0x1f => return None,
                // Otherwise copy the UTF-8 byte(s) through. `c` is the lead byte; copy continuation bytes.
                _ => {
                    let lead = c;
                    let extra = match lead {
                        0x00..=0x7f => 0,
                        0xc0..=0xdf => 1,
                        0xe0..=0xef => 2,
                        0xf0..=0xf7 => 3,
                        _ => return None,
                    };
                    let begin = self.i - 1;
                    if begin + 1 + extra > self.b.len() {
                        return None;
                    }
                    self.i = begin + 1 + extra;
                    out.push_str(std::str::from_utf8(&self.b[begin..self.i]).ok()?);
                }
            }
        }
    }
    fn array(&mut self) -> Option<Json> {
        self.i += 1; // '['
        let mut items = Vec::new();
        self.ws();
        if self.peek() == Some(b']') {
            self.i += 1;
            return Some(Json::Arr(items));
        }
        loop {
            items.push(self.value()?);
            self.ws();
            match self.peek()? {
                b',' => {
                    self.i += 1;
                    continue;
                }
                b']' => {
                    self.i += 1;
                    return Some(Json::Arr(items));
                }
                _ => return None,
            }
        }
    }
    fn object(&mut self) -> Option<Json> {
        self.i += 1; // '{'
        let mut pairs = Vec::new();
        self.ws();
        if self.peek() == Some(b'}') {
            self.i += 1;
            return Some(Json::Obj(pairs));
        }
        loop {
            self.ws();
            let key = self.string()?;
            self.ws();
            if self.peek()? != b':' {
                return None;
            }
            self.i += 1;
            let val = self.value()?;
            pairs.push((key, val));
            self.ws();
            match self.peek()? {
                b',' => {
                    self.i += 1;
                    continue;
                }
                b'}' => {
                    self.i += 1;
                    return Some(Json::Obj(pairs));
                }
                _ => return None,
            }
        }
    }
}

/// The set of field names a v1 receipt may carry (strict: any other key fails closed). `admission` is
/// accepted but **ignored** on ingest — the producer's self-assessment is never trusted; we recompute it.
const RECEIPT_FIELDS: &[&str] = &[
    "schema",
    "admission",
    "platform",
    "platform_status",
    "oracle_run_status",
    "oracle_exit_disposition",
    "stderr_encoding",
    "receipt_hash_scope",
    "oracle_invocation_identity",
    "argv",
    "fixture_set",
    "zic_binary_sha256",
    "zic_version_output",
    "tzdb_source_release",
    "patch_stack_identity",
    "class_location_verdicts",
    "known_divergences",
    "skipped_with_reason",
    "inadmissible_reason",
];

impl VendorOracleReceipt {
    /// Ingest an external `vendor-oracle-receipt-v1` JSON document into the typed contract (T16.5b),
    /// **fail-closed**. Returns a [`ReceiptParseError`] for anything that is not a well-formed v1 receipt
    /// — *distinct* from the admission verdict (call [`Self::admit`] on the parsed receipt to decide that).
    /// The reader is receipt-scoped and never relaxes: unknown fields and unknown enum values are rejected.
    pub fn from_json(input: &str) -> Result<Self, ReceiptParseError> {
        let root = JsonReader::new(input)
            .parse()
            .ok_or(ReceiptParseError::MalformedJson)?;
        let obj = match root {
            Json::Obj(pairs) => pairs,
            _ => return Err(ReceiptParseError::MalformedJson),
        };
        // Strict v1: reject any unrecognised field (we refuse to silently ignore unmodelled claims).
        for (k, _) in &obj {
            if !RECEIPT_FIELDS.contains(&k.as_str()) {
                return Err(ReceiptParseError::UnknownField);
            }
        }
        let get = |name: &'static str| obj.iter().find(|(k, _)| k == name).map(|(_, v)| v);
        let req = |name: &'static str| get(name).ok_or(ReceiptParseError::MissingField(name));

        // schema must be exactly the v1 id.
        match req("schema")? {
            Json::Str(s) if s == "vendor-oracle-receipt-v1" => {}
            _ => return Err(ReceiptParseError::WrongSchema),
        }

        let as_str = |name: &'static str, v: &Json| -> Result<String, ReceiptParseError> {
            match v {
                Json::Str(s) => Ok(s.clone()),
                _ => Err(ReceiptParseError::WrongType(name)),
            }
        };
        let opt_str = |name: &'static str| -> Result<Option<String>, ReceiptParseError> {
            match get(name) {
                None | Some(Json::Null) => Ok(None),
                Some(Json::Str(s)) => Ok(Some(s.clone())),
                Some(_) => Err(ReceiptParseError::WrongType(name)),
            }
        };
        let str_array = |name: &'static str| -> Result<Vec<String>, ReceiptParseError> {
            match req(name)? {
                Json::Arr(items) => items.iter().map(|it| as_str(name, it)).collect(),
                _ => Err(ReceiptParseError::WrongType(name)),
            }
        };

        let platform = as_str("platform", req("platform")?)?;
        let platform_status =
            parse_platform_status(&as_str("platform_status", req("platform_status")?)?)
                .ok_or(ReceiptParseError::UnknownEnumValue("platform_status"))?;
        let run_status = parse_run_status(&as_str("oracle_run_status", req("oracle_run_status")?)?)
            .ok_or(ReceiptParseError::UnknownEnumValue("oracle_run_status"))?;
        let exit_disposition = parse_exit_disposition(&as_str(
            "oracle_exit_disposition",
            req("oracle_exit_disposition")?,
        )?)
        .ok_or(ReceiptParseError::UnknownEnumValue(
            "oracle_exit_disposition",
        ))?;
        let stderr_encoding =
            parse_stderr_encoding(&as_str("stderr_encoding", req("stderr_encoding")?)?)
                .ok_or(ReceiptParseError::UnknownEnumValue("stderr_encoding"))?;
        let hash_scope =
            parse_hash_scope(&as_str("receipt_hash_scope", req("receipt_hash_scope")?)?)
                .ok_or(ReceiptParseError::UnknownEnumValue("receipt_hash_scope"))?;
        let invocation = parse_invocation(&as_str(
            "oracle_invocation_identity",
            req("oracle_invocation_identity")?,
        )?)
        .ok_or(ReceiptParseError::UnknownEnumValue(
            "oracle_invocation_identity",
        ))?;
        let argv = str_array("argv")?;
        let fixture_set = as_str("fixture_set", req("fixture_set")?)?;

        // class_location_verdicts: array of { fixture, verdict } (verdict from the finite vocab).
        let mut class_location_verdicts = Vec::new();
        match req("class_location_verdicts")? {
            Json::Arr(items) => {
                for it in items {
                    let pairs = match it {
                        Json::Obj(p) => p,
                        _ => return Err(ReceiptParseError::WrongType("class_location_verdicts")),
                    };
                    let fixture = pairs
                        .iter()
                        .find(|(k, _)| k == "fixture")
                        .map(|(_, v)| as_str("class_location_verdicts", v))
                        .ok_or(ReceiptParseError::MissingField(
                            "class_location_verdicts.fixture",
                        ))??;
                    let verdict_s = pairs
                        .iter()
                        .find(|(k, _)| k == "verdict")
                        .map(|(_, v)| as_str("class_location_verdicts", v))
                        .ok_or(ReceiptParseError::MissingField(
                            "class_location_verdicts.verdict",
                        ))??;
                    let verdict = parse_class_location_verdict(&verdict_s).ok_or(
                        ReceiptParseError::UnknownEnumValue("class_location_verdicts.verdict"),
                    )?;
                    class_location_verdicts.push((fixture, verdict));
                }
            }
            _ => return Err(ReceiptParseError::WrongType("class_location_verdicts")),
        }

        Ok(VendorOracleReceipt {
            platform,
            platform_status,
            run_status,
            exit_disposition,
            stderr_encoding,
            hash_scope,
            invocation,
            argv,
            fixture_set,
            zic_binary_sha256: opt_str("zic_binary_sha256")?,
            zic_version_output: opt_str("zic_version_output")?,
            tzdb_source_release: as_str("tzdb_source_release", req("tzdb_source_release")?)?,
            patch_stack: opt_str("patch_stack_identity")?,
            class_location_verdicts,
            known_divergences: str_array("known_divergences")?,
            skipped_with_reason: opt_str("skipped_with_reason")?,
            inadmissible_reason: opt_str("inadmissible_reason")?,
        })
    }
}

fn parse_platform_status(s: &str) -> Option<ReferencePlatformStatus> {
    Some(match s {
        "admitted" => ReferencePlatformStatus::Admitted,
        "unavailable" => ReferencePlatformStatus::Unavailable,
        "documentation_only" => ReferencePlatformStatus::DocumentationOnly,
        "skipped_with_reason" => ReferencePlatformStatus::SkippedWithReason,
        "inadmissible_unpinned" => ReferencePlatformStatus::InadmissibleUnpinned,
        "pending_external_receipt" => ReferencePlatformStatus::PendingExternalReceipt,
        _ => return None,
    })
}
fn parse_run_status(s: &str) -> Option<OracleRunStatus> {
    Some(match s {
        "completed" => OracleRunStatus::Completed,
        "skipped_unavailable" => OracleRunStatus::SkippedUnavailable,
        "timed_out" => OracleRunStatus::TimedOut,
        "killed" => OracleRunStatus::Killed,
        "failed_to_start" => OracleRunStatus::FailedToStart,
        _ => return None,
    })
}
fn parse_exit_disposition(s: &str) -> Option<OracleExitDisposition> {
    Some(match s {
        "success" => OracleExitDisposition::Success,
        "nonzero_with_output" => OracleExitDisposition::NonzeroWithOutput,
        "nonzero_no_output" => OracleExitDisposition::NonzeroNoOutput,
        "terminated_by_signal" => OracleExitDisposition::TerminatedBySignal,
        "timed_out" => OracleExitDisposition::TimedOut,
        "not_run" => OracleExitDisposition::NotRun,
        _ => return None,
    })
}
fn parse_stderr_encoding(s: &str) -> Option<StderrEncoding> {
    Some(match s {
        "utf8" => StderrEncoding::Utf8,
        "locale_dependent" => StderrEncoding::LocaleDependent,
        "bytes_only" => StderrEncoding::BytesOnly,
        "unknown" => StderrEncoding::Unknown,
        _ => return None,
    })
}
fn parse_hash_scope(s: &str) -> Option<ReceiptHashScope> {
    Some(match s {
        "stable_verdict_only" => ReceiptHashScope::StableVerdictOnly,
        "full_receipt_including_run_metadata" => ReceiptHashScope::FullReceiptIncludingRunMetadata,
        "unknown" => ReceiptHashScope::Unknown,
        _ => return None,
    })
}
fn parse_invocation(s: &str) -> Option<OracleInvocationIdentity> {
    Some(match s {
        "argv_exact" => OracleInvocationIdentity::ArgvExact,
        "template_only" => OracleInvocationIdentity::TemplateOnly,
        "shell_expanded" => OracleInvocationIdentity::ShellExpanded,
        "unknown" => OracleInvocationIdentity::Unknown,
        _ => return None,
    })
}
fn parse_class_location_verdict(s: &str) -> Option<ClassLocationVerdict> {
    Some(match s {
        "class_location_match" => ClassLocationVerdict::ClassLocationMatch,
        "class_match_location_diff" => ClassLocationVerdict::ClassMatchLocationDiff,
        "divergence" => ClassLocationVerdict::Divergence,
        "reference_only" => ClassLocationVerdict::ReferenceOnly,
        "candidate_only" => ClassLocationVerdict::CandidateOnly,
        "not_compared" => ClassLocationVerdict::NotCompared,
        _ => return None,
    })
}
