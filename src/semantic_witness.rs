//! T15.3 — **semantic witnesses**: typed, `zdump`-backed behaviour evidence for the public conformance
//! engine. A semantic witness answers *"at this instant, do zic-rs and the reference agree on the local
//! `(utoff, is_dst, abbreviation)`?"* — read through the project's footer-aware behaviour oracle,
//! reference `zdump`. It is deliberately **not** structural validation (RFC 9636 byte-format correctness
//! is T15.4) and **not** a diagnostic: a witness proves *selected behaviour under an oracle*, nothing more.
//!
//! **Oracle absence is visible.** If reference `zic`/`zdump` are not on `PATH`, the report renders
//! [`OracleMode::Unavailable`] with a `skipped_with_reason` and every witness verdict is
//! [`SkippedOracleUnavailable`](SemanticWitnessVerdict::SkippedOracleUnavailable) — never silence.
//!
//! **Footer-aware by construction.** `zdump -v -c LO,HI` evaluates the POSIX footer to project future
//! transitions, so far-future probes are sound even though zic-rs (fat) and reference (slim) keep
//! different *explicit* transition sets — exactly why the oracle is `zdump`, not raw TZif decoding.
//!
//! Compact by design (by design: *make the mechanism public, do not sweep every zone*): a curated
//! zone set, a handful of representative probe instants drawn from the oracle's own reported timeline.

use std::path::{Path, PathBuf};

use crate::compare::{reference_zic, zdump};
use crate::error::Result;
use crate::json::escape;
use crate::manifest::{ArtifactCategory, OracleMode};
use crate::model::Database;

/// Schema id for the semantic-witness report JSON.
const SCHEMA: &str = "zic-rs-semantic-report-v1";

/// The named fixture set this report's witnesses are drawn from — so future expansion is auditable and a
/// reader can tell *which* curated set produced these rows (a small seed in T15.3).
const FIXTURE_SET: &str = "semantic-witness-seed-v1";

/// **How a single `ReferenceBuildProfile` axis is known** (T16.2) — the typed-honesty core of T16:
/// *ecology evidence, not ecology vibes*. An axis we have **not** actually captured or measured renders an
/// explicit disposition; it is **never** silently inferred from the host. The whole point: a reviewer can
/// tell "we measured this" from "we don't know" from "we refuse to guess this from the host."
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildAxisEvidence {
    /// Captured/measured directly — carries the observed value (e.g. the resolved binary's sha256).
    Known(String),
    /// The axis exists for this reference but we have **not** captured/measured it. Honest "don't know".
    UnknownUnmeasured,
    /// The axis cannot be observed on this host (e.g. the reference binary is not on `PATH`).
    UnavailableOnThisHost,
    /// The axis does not apply to this reference / release era.
    NotApplicable,
    /// We have documented behaviour but no captured value (e.g. a vendor caveat).
    DocumentationOnly,
    /// We **deliberately refuse to infer** this from the host because doing so would be a category error
    /// (e.g. the host's `time_t` model is *not* the reference `zic`'s; reading it would be a vibe, not
    /// evidence). The non-inference is itself the typed claim.
    InferredForbidden,
}

impl BuildAxisEvidence {
    fn disposition(&self) -> &'static str {
        match self {
            BuildAxisEvidence::Known(_) => "known",
            BuildAxisEvidence::UnknownUnmeasured => "unknown_unmeasured",
            BuildAxisEvidence::UnavailableOnThisHost => "unavailable_on_this_host",
            BuildAxisEvidence::NotApplicable => "not_applicable",
            BuildAxisEvidence::DocumentationOnly => "documentation_only",
            BuildAxisEvidence::InferredForbidden => "inferred_forbidden",
        }
    }
    /// Render as `{ "disposition": <token>, "value": <string|null> }` — the value is non-null *only* for
    /// `Known`, so a reader can never mistake an honest "don't know" for a captured fact.
    fn to_json(&self) -> String {
        let value = match self {
            BuildAxisEvidence::Known(v) => escape(v),
            _ => "null".to_string(),
        };
        format!(
            "{{ \"disposition\": {}, \"value\": {} }}",
            escape(self.disposition()),
            value
        )
    }
}

/// **`ReferenceBuildProfile`** (T16.2) — the answer to the T16 north star *"matches **which** reference
/// `zic`?"*. Two `zic` binaries with the same `--version` can differ by compile-time flags, `time_t`
/// model, runtime-leap support, and data-path policy — all of which change observable output/diagnostics.
/// This captures those axes **with explicit dispositions**: today most are honestly `UnknownUnmeasured`
/// or `InferredForbidden` (we do not read compile flags or the host `time_t` and pretend they are the
/// reference's). The genuinely-known axes — binary identity, the data-path policy, the platform we ran on,
/// the captured `--version` — are `Known`. Richer capture (build flags, measured `time_t`, runtime-leap)
/// is the T16.5 QEMU vendor-lab job; this is the honest first cut, not a fake-precision surface.
#[derive(Debug, Clone)]
pub struct ReferenceBuildProfile {
    /// The reference's release identity, from its captured `--version` (not the *admitted* release — that
    /// is the pinned 2026b archive; this is what the *live* oracle binary reports).
    pub source_release: BuildAxisEvidence,
    /// The resolved binary's content hash — the strongest identity anchor (`Known` iff resolvable).
    pub binary_sha256: BuildAxisEvidence,
    /// The platform the oracle ran on (finite vocabulary via `std::env::consts::OS`).
    pub reference_platform: BuildAxisEvidence,
    /// Compile-time build flags (`ZIC_BLOAT_DEFAULT`/`TZNAME_MAXIMUM`/`ZIC_MAX_ABBR_LEN_WO_WARN`/…) —
    /// `InferredForbidden`: they are **not** derivable from the version string (T16.5 captures them).
    pub build_flags: BuildAxisEvidence,
    /// The reference's `time_t` model — `InferredForbidden`: the host's model is not the reference's.
    pub time_t_model: BuildAxisEvidence,
    /// Whether the reference supports runtime leap seconds — `UnknownUnmeasured` until actually tested.
    pub runtime_leap_support: BuildAxisEvidence,
    /// The data-path policy zic-rs uses when querying the oracle — genuinely `Known`: we always hand
    /// `zdump` an explicit compiled-file path, never a zone name resolved against system zoneinfo.
    pub tzdir_resolution_policy: BuildAxisEvidence,
    /// The ambient `LC_ALL` at capture (`Known` iff set; else `UnknownUnmeasured`).
    pub locale: BuildAxisEvidence,
    /// The reference's `-v` warning thresholds — `UnknownUnmeasured` until captured.
    pub warning_thresholds: BuildAxisEvidence,
}

impl ReferenceBuildProfile {
    /// Capture the profile honestly from what we can actually observe of the oracle today.
    fn capture(
        zic_version: &Option<String>,
        zic_binary_sha256: &Option<String>,
        env_lc_all: &Option<String>,
    ) -> Self {
        let from_opt = |o: &Option<String>| match o {
            Some(v) => BuildAxisEvidence::Known(v.clone()),
            None => BuildAxisEvidence::UnknownUnmeasured,
        };
        ReferenceBuildProfile {
            source_release: from_opt(zic_version),
            // Binary present-but-unresolved is a *host* limitation, not "don't know it exists".
            binary_sha256: match zic_binary_sha256 {
                Some(v) => BuildAxisEvidence::Known(v.clone()),
                None => BuildAxisEvidence::UnavailableOnThisHost,
            },
            reference_platform: BuildAxisEvidence::Known(std::env::consts::OS.to_string()),
            // Compile flags + time_t model are NOT inferred from the host/version — that would be a vibe.
            build_flags: BuildAxisEvidence::InferredForbidden,
            time_t_model: BuildAxisEvidence::InferredForbidden,
            runtime_leap_support: BuildAxisEvidence::UnknownUnmeasured,
            // Genuinely known: zic-rs always queries via an explicit `-c <file>` path.
            tzdir_resolution_policy: BuildAxisEvidence::Known(
                "explicit_tzif_path_argument".to_string(),
            ),
            locale: from_opt(env_lc_all),
            warning_thresholds: BuildAxisEvidence::UnknownUnmeasured,
        }
    }

    fn to_json(&self) -> String {
        format!(
            "{{ \"source_release\": {}, \"binary_sha256\": {}, \"reference_platform\": {}, \
             \"build_flags\": {}, \"time_t_model\": {}, \"runtime_leap_support\": {}, \
             \"tzdir_resolution_policy\": {}, \"locale\": {}, \"warning_thresholds\": {} }}",
            self.source_release.to_json(),
            self.binary_sha256.to_json(),
            self.reference_platform.to_json(),
            self.build_flags.to_json(),
            self.time_t_model.to_json(),
            self.runtime_leap_support.to_json(),
            self.tzdir_resolution_policy.to_json(),
            self.locale.to_json(),
            self.warning_thresholds.to_json(),
        )
    }
}

/// **Oracle identity** (T15.3 shape; T15.5-remainder enrichment; T16.2 `reference_build_profile`):
/// "which `zdump`, from which build, with which data path?". `OracleMode` says *whether* a kind of oracle
/// ran; this says *which*,
/// and now *how it was invoked* and *that it read the intended bytes*:
/// - `zic_binary_sha256` / `zdump_binary_sha256` — the resolved tool binary's content hash (best-effort;
///   `None` if the tool can't be located on `PATH`), so a divergence can be pinned to an exact build.
/// - `zdump_command_line` — the exact `zdump` invocation *template* (`-v -c LO,HI <tzif>`), so a reviewer
///   can reproduce the query.
/// - `zoneinfo_resolution` — proof that the oracle was pointed at the **compiled file path** (an explicit
///   `-c` argument), *not* a zone name resolved against system zoneinfo (the "did `zdump` read the
///   intended tree?" hazard). Always `explicit_tzif_path_argument` here by construction.
/// - `env_tz` / `env_lc_all` — the ambient `TZ`/`LC_ALL` at capture (host-time-independence is a non-claim
///   to *witness*, not assume; recorded so a non-`C`/non-`UTC` host is visible, never silent).
#[derive(Debug, Clone)]
pub struct OracleIdentity {
    pub zic_program: String,
    pub zdump_program: String,
    pub zic_version: Option<String>,
    pub zdump_version: Option<String>,
    pub zic_binary_sha256: Option<String>,
    pub zdump_binary_sha256: Option<String>,
    pub zdump_command_line: String,
    pub zoneinfo_resolution: &'static str,
    pub env_tz: Option<String>,
    pub env_lc_all: Option<String>,
    pub reference_platform: &'static str,
    /// T16.2 — the typed "which *build* of the reference?" profile (build flags / `time_t` / runtime-leap
    /// / data-path policy / locale / warning thresholds), each axis with an explicit evidence disposition.
    pub build_profile: ReferenceBuildProfile,
}

/// First line of `program --version` (best-effort; `None` if the tool is absent or silent).
fn tool_version(program: &str) -> Option<String> {
    let out = std::process::Command::new(program)
        .arg("--version")
        .output()
        .ok()?;
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
}

/// Resolve `program` to a filesystem path: used as-is if it contains a path separator, else searched on
/// `PATH`. Returns the first existing candidate (best-effort; `None` if not found).
fn resolve_program(program: &str) -> Option<std::path::PathBuf> {
    if program.contains('/') {
        let p = std::path::PathBuf::from(program);
        return p.exists().then_some(p);
    }
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(program))
        .find(|cand| cand.is_file())
}

/// SHA-256 of the resolved tool binary (best-effort; `None` if it can't be located or read), so an oracle
/// divergence can be pinned to an exact binary build, not just a version string that vendors can reuse.
fn binary_sha256(program: &str) -> Option<String> {
    let path = resolve_program(program)?;
    let bytes = std::fs::read(path).ok()?;
    Some(crate::hash::sha256_hex(&bytes))
}

impl OracleIdentity {
    fn capture(zic: &str, zdump: &str) -> Self {
        OracleIdentity {
            zic_program: zic.to_string(),
            zdump_program: zdump.to_string(),
            zic_version: tool_version(zic),
            zdump_version: tool_version(zdump),
            zic_binary_sha256: binary_sha256(zic),
            zdump_binary_sha256: binary_sha256(zdump),
            // The exact query template — witnesses always pass the compiled file path to `-c`.
            zdump_command_line: format!("{zdump} -v -c {HORIZON_LO},{HORIZON_HI} <tzif>"),
            // We never let `zdump` resolve a zone *name* against system zoneinfo; it reads the file we
            // hand it. This is the proof that the oracle observed *our* bytes, not the host's tree.
            zoneinfo_resolution: "explicit_tzif_path_argument",
            env_tz: std::env::var("TZ").ok(),
            env_lc_all: std::env::var("LC_ALL").ok(),
            reference_platform: std::env::consts::OS,
            build_profile: ReferenceBuildProfile::capture(
                &tool_version(zic),
                &binary_sha256(zic),
                &std::env::var("LC_ALL").ok(),
            ),
        }
    }

    fn to_json(&self) -> String {
        let opt = |o: &Option<String>| match o {
            Some(v) => escape(v),
            None => "null".to_string(),
        };
        format!(
            "{{ \"zic\": {}, \"zdump\": {}, \"zic_version\": {}, \"zdump_version\": {}, \
             \"zic_binary_sha256\": {}, \"zdump_binary_sha256\": {}, \"zdump_command_line\": {}, \
             \"zoneinfo_resolution\": {}, \"env_tz\": {}, \"env_lc_all\": {}, \
             \"reference_platform\": {}, \"reference_build_profile\": {}, \
             \"reference_admission\": {{ \"live_oracle\": {}, \"sealed_reference\": {} }} }}",
            escape(&self.zic_program),
            escape(&self.zdump_program),
            opt(&self.zic_version),
            opt(&self.zdump_version),
            opt(&self.zic_binary_sha256),
            opt(&self.zdump_binary_sha256),
            escape(&self.zdump_command_line),
            escape(self.zoneinfo_resolution),
            opt(&self.env_tz),
            opt(&self.env_lc_all),
            escape(self.reference_platform),
            self.build_profile.to_json(),
            // T16.3 — the *live* oracle a report runs against is whatever is on `PATH`: an unverified
            // `LiveCurrentDirectory` binary, exploration-grade, NOT sealed-claim-backing. The project's
            // *sealed* reference is the separate T12.5a.2 versioned + fingerprint-anchored 2026b archive.
            crate::manifest::ReferenceAdmission {
                locator: crate::manifest::ReferenceLocatorKind::LiveCurrentDirectory,
                trust: crate::manifest::SignatureTrustModel::Unknown,
            }
            .to_json(),
            crate::manifest::ADMITTED_2026B_REFERENCE.to_json(),
        )
    }
}

/// How wide a `zdump` horizon to read. Spans well past the 32-bit boundary so `post_2038`/`far_future`
/// probes land on footer-projected transitions the oracle computes.
const HORIZON_LO: i32 = 1900;
const HORIZON_HI: i32 = 2200;

/// The verdict for a single semantic-witness row — a **finite** vocabulary (CONTRACT.TYPING).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticWitnessVerdict {
    /// zic-rs and the reference agree on `(utoff, is_dst, abbreviation)` at this instant.
    Match,
    /// They disagree (a real behaviour divergence at this instant).
    Mismatch,
    /// The oracle (reference `zic`/`zdump`) was unavailable, so no comparison was made.
    SkippedOracleUnavailable,
    /// The probe falls outside the comparable range for this zone (e.g. no observation there).
    NotApplicable,
    /// The probe instant lies beyond the dumped oracle horizon (`-c LO,HI`), so no observation exists.
    OutOfHorizon,
    /// A *documented* divergence recorded as a typed verdict rather than prose — e.g. the T14 multi-era
    /// "two rules for same instant" residual, where zic-rs accepts (valid output) but reference errors.
    KnownDivergence,
}

impl SemanticWitnessVerdict {
    pub fn as_str(self) -> &'static str {
        match self {
            SemanticWitnessVerdict::Match => "match",
            SemanticWitnessVerdict::Mismatch => "mismatch",
            SemanticWitnessVerdict::SkippedOracleUnavailable => "skipped_oracle_unavailable",
            SemanticWitnessVerdict::NotApplicable => "not_applicable",
            SemanticWitnessVerdict::OutOfHorizon => "out_of_horizon",
            SemanticWitnessVerdict::KnownDivergence => "known_divergence",
        }
    }
}

/// One observed local-time behaviour at an instant: the conformance-relevant `(offset, is_dst, abbr)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticObservation {
    pub utc: String,
    pub offset_seconds: i32,
    pub is_dst: bool,
    pub abbreviation: String,
}

/// One witness row (the public table shape): zone · timestamp · reference obs · zic-rs obs · verdict ·
/// oracle mode · artifact category.
#[derive(Debug, Clone)]
pub struct SemanticWitness {
    pub zone: String,
    pub timestamp: String,
    pub reference: Option<SemanticObservation>,
    pub zic_rs: Option<SemanticObservation>,
    pub verdict: SemanticWitnessVerdict,
}

/// The semantic-witness report.
#[derive(Debug, Clone)]
pub struct SemanticWitnessReport {
    pub oracle_mode: OracleMode,
    pub oracle_identity: OracleIdentity,
    /// The declared oracle horizon `(start, end)` years — witnesses are scoped to it, so a reader cannot
    /// overgeneralize the set into "semantic parity". A witness is *matched for the declared set*, no more.
    pub horizon: (i32, i32),
    pub witnesses: Vec<SemanticWitness>,
}

/// Parse one **normalised** `zdump -v` line (`<UTC> UT = <local> <abbr> isdst=N gmtoff=N`) into a typed
/// observation. Returns `None` for out-of-range sentinel lines (`(localtime failed)` etc.).
fn parse_obs(line: &str) -> Option<SemanticObservation> {
    let (utc, rest) = line.split_once(" UT = ")?;
    let toks: Vec<&str> = rest.split_whitespace().collect();
    let isdst_pos = toks.iter().position(|t| t.starts_with("isdst="))?;
    let gmtoff = toks.iter().find_map(|t| t.strip_prefix("gmtoff="))?;
    let isdst = toks[isdst_pos].strip_prefix("isdst=")?;
    let abbr = toks.get(isdst_pos.checked_sub(1)?)?;
    Some(SemanticObservation {
        utc: utc.trim().to_string(),
        offset_seconds: gmtoff.parse().ok()?,
        is_dst: isdst == "1",
        abbreviation: (*abbr).to_string(),
    })
}

/// Pick a compact, representative subset of indices from `n` observations (first few + last few), so the
/// witness spans early history through footer-projected far-future without sweeping every transition.
fn representative_indices(n: usize) -> Vec<usize> {
    if n == 0 {
        return Vec::new();
    }
    let mut idx: Vec<usize> = Vec::new();
    for i in [0usize, 1, 2] {
        if i < n {
            idx.push(i);
        }
    }
    for i in [n.saturating_sub(2), n.saturating_sub(1)] {
        if i >= 3 && !idx.contains(&i) {
            idx.push(i);
        }
    }
    idx
}

impl SemanticWitnessReport {
    /// Deterministic JSON (hand-rolled, shared escaper — no serde). Every claim-bearing artifact carries
    /// its [`ArtifactCategory`]; the report's evidence is `semantic_witness_artifact` (explicitly **not**
    /// `structural_validation_artifact` — that is T15.4).
    pub fn to_json(&self) -> String {
        let mut s = String::new();
        s.push_str("{\n");
        s.push_str(&format!("  \"schema\": {},\n", escape(SCHEMA)));
        s.push_str(&format!(
            "  \"artifact_category\": {},\n",
            escape(ArtifactCategory::SemanticWitnessArtifact.as_str())
        ));
        s.push_str(&format!("  \"fixture_set\": {},\n", escape(FIXTURE_SET)));
        s.push_str("  \"witness_scope\": \"small_seed\",\n");
        s.push_str(&format!(
            "  \"oracle_mode\": {},\n",
            self.oracle_mode.to_json_field()
        ));
        s.push_str(&format!(
            "  \"oracle_identity\": {},\n",
            self.oracle_identity.to_json()
        ));
        s.push_str(&format!(
            "  \"witness_horizon\": {{ \"start\": {}, \"end\": {}, \"reason\": {} }},\n",
            self.horizon.0,
            self.horizon.1,
            escape(
                "witnesses are scoped to this oracle horizon; a match is 'matched for the declared \
                 witness set', NOT a claim of universal semantic parity"
            )
        ));
        s.push_str(&format!(
            "  \"note\": {},\n",
            escape(
                "a semantic witness proves selected (offset, is_dst, abbreviation) behaviour under the \
                 zdump oracle; it is NOT a claim of RFC 9636 structural validity (that is the structural \
                 validator, T15.4)."
            )
        ));
        s.push_str("  \"witnesses\": [");
        for (i, w) in self.witnesses.iter().enumerate() {
            s.push_str(if i == 0 { "\n" } else { ",\n" });
            let obs = |o: &Option<SemanticObservation>| match o {
                Some(o) => format!(
                    "{{ \"offset_seconds\": {}, \"is_dst\": {}, \"abbreviation\": {} }}",
                    o.offset_seconds,
                    o.is_dst,
                    escape(&o.abbreviation)
                ),
                None => "null".to_string(),
            };
            s.push_str(&format!(
                "    {{ \"zone\": {}, \"timestamp\": {}, \"reference\": {}, \"zic_rs\": {}, \
                 \"verdict\": {}, \"artifact_category\": {} }}",
                escape(&w.zone),
                escape(&w.timestamp),
                obs(&w.reference),
                obs(&w.zic_rs),
                escape(w.verdict.as_str()),
                escape(ArtifactCategory::SemanticWitnessArtifact.as_str()),
            ));
        }
        s.push_str(if self.witnesses.is_empty() {
            "]\n"
        } else {
            "\n  ]\n"
        });
        s.push_str("}\n");
        s
    }
}

/// Build the semantic-witness report for a curated `zones` set, reading behaviour through reference
/// `zic` + `zdump`. If either oracle tool is unavailable the report degrades **visibly**:
/// `oracle_mode = Unavailable(reason)` and each zone gets one `SkippedOracleUnavailable` witness.
pub fn build_semantic_witness_report(
    db: &Database,
    zones: &[String],
    reference_zic: &str,
    zdump_program: &str,
    inputs: &[PathBuf],
    work_dir: &Path,
) -> Result<SemanticWitnessReport> {
    // Oracle availability is part of the report, never silent.
    if !reference_zic::is_available(reference_zic) || !zdump::is_available(zdump_program) {
        let reason = format!(
            "reference oracle unavailable (need `{reference_zic}` and `{zdump_program}` on PATH)"
        );
        let witnesses = zones
            .iter()
            .map(|z| SemanticWitness {
                zone: z.clone(),
                timestamp: "-".into(),
                reference: None,
                zic_rs: None,
                verdict: SemanticWitnessVerdict::SkippedOracleUnavailable,
            })
            .collect();
        return Ok(SemanticWitnessReport {
            oracle_mode: OracleMode::Unavailable(reason),
            oracle_identity: OracleIdentity::capture(reference_zic, zdump_program),
            horizon: (HORIZON_LO, HORIZON_HI),
            witnesses,
        });
    }

    // Reference side: compile all inputs once with reference `zic`.
    let ref_root = work_dir.join("ref");
    reference_zic::compile_with_reference(reference_zic, inputs, &ref_root)?;
    let ours_root = work_dir.join("ours");

    let mut witnesses = Vec::new();
    for zone in zones {
        // zic-rs side: compile to bytes, write to an absolute temp path for `zdump`.
        let ours_bytes = match crate::compile_zone_to_bytes(db, zone) {
            Ok(b) => b,
            Err(_) => {
                witnesses.push(SemanticWitness {
                    zone: zone.clone(),
                    timestamp: "-".into(),
                    reference: None,
                    zic_rs: None,
                    verdict: SemanticWitnessVerdict::NotApplicable,
                });
                continue;
            }
        };
        let ours_path = ours_root.join(zone);
        if let Some(parent) = ours_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&ours_path, &ours_bytes).ok();
        let ref_path = reference_zic::compiled_path(&ref_root, zone);

        let ours_dump = zdump::run(zdump_program, &ours_path, HORIZON_LO, HORIZON_HI)?;
        let ref_dump = zdump::run(zdump_program, &ref_path, HORIZON_LO, HORIZON_HI)?;
        let ours_obs: Vec<SemanticObservation> =
            ours_dump.iter().filter_map(|l| parse_obs(l)).collect();
        let ref_obs: Vec<SemanticObservation> =
            ref_dump.iter().filter_map(|l| parse_obs(l)).collect();

        // Pair representative observations by index (the oracle dumps both files over the same horizon).
        let n = ours_obs.len().max(ref_obs.len());
        for i in representative_indices(n) {
            let zr = ours_obs.get(i).cloned();
            let rf = ref_obs.get(i).cloned();
            let verdict = match (&zr, &rf) {
                (Some(a), Some(b))
                    if a.offset_seconds == b.offset_seconds
                        && a.is_dst == b.is_dst
                        && a.abbreviation == b.abbreviation =>
                {
                    SemanticWitnessVerdict::Match
                }
                (Some(_), Some(_)) => SemanticWitnessVerdict::Mismatch,
                _ => SemanticWitnessVerdict::NotApplicable,
            };
            let timestamp = zr
                .as_ref()
                .or(rf.as_ref())
                .map(|o| o.utc.clone())
                .unwrap_or_else(|| "-".into());
            witnesses.push(SemanticWitness {
                zone: zone.clone(),
                timestamp,
                reference: rf,
                zic_rs: zr,
                verdict,
            });
        }
    }

    Ok(SemanticWitnessReport {
        // The verdicts were read through reference `zic`'s bytes, dumped by `zdump`.
        oracle_mode: OracleMode::ReferenceZdump,
        oracle_identity: OracleIdentity::capture(reference_zic, zdump_program),
        horizon: (HORIZON_LO, HORIZON_HI),
        witnesses,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_valid_zdump_line() {
        let o = parse_obs(
            "Sun Mar 14 06:59:59 2021 UT = Sun Mar 14 01:59:59 2021 EST isdst=0 gmtoff=-18000",
        )
        .unwrap();
        assert_eq!(o.offset_seconds, -18000);
        assert!(!o.is_dst);
        assert_eq!(o.abbreviation, "EST");
    }

    #[test]
    fn rejects_out_of_range_sentinel_line() {
        assert!(parse_obs(
            "Thu Jan  1 00:00:00 -2147481748 UT = -67768040609740800 (localtime failed)"
        )
        .is_none());
    }

    #[test]
    fn verdict_vocab_is_finite_and_stable() {
        for (v, s) in [
            (SemanticWitnessVerdict::Match, "match"),
            (SemanticWitnessVerdict::Mismatch, "mismatch"),
            (
                SemanticWitnessVerdict::SkippedOracleUnavailable,
                "skipped_oracle_unavailable",
            ),
            (SemanticWitnessVerdict::NotApplicable, "not_applicable"),
            (SemanticWitnessVerdict::OutOfHorizon, "out_of_horizon"),
            (SemanticWitnessVerdict::KnownDivergence, "known_divergence"),
        ] {
            assert_eq!(v.as_str(), s);
        }
    }
}
