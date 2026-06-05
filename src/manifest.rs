//! The **alias/canonical manifest** (`alias-map.json`) — a producer-side artifact that
//! records, for a compile run, which identifiers are *canonical zones* and which are *links*
//! (aliases), with content hashes and an explicit account of where link materialisation
//! duplicates bytes.
//!
//! ## Why this exists
//!
//! Downstream consumers/bundlers of timezone data care about *how* a bundle was produced, not
//! only the file bytes. jiff#258 is the concrete motivation: it observed that concatenated
//! zoneinfo appears to **duplicate** data for aliases (e.g. a release with 597 identifiers but
//! only 339 non-alias zones) and asked for that to be documented. This manifest answers that
//! directly: every identifier is tagged `zone` or `link`, links carry their target + the
//! target's hash, and the summary reports `identifiers` / `canonical_zones` / `links` plus
//! `duplicated_byte_links` (links we materialised as byte copies). See `docs/rust-ecosystem.md`
//! and `docs/generated-data-contract.md`.
//!
//! ## Dependency-free by design
//!
//! Hashing uses the in-house [`crate::hash`] SHA-256; JSON is written by a tiny deterministic
//! serializer here (no `serde`). The schema is small and fixed, and identifier strings are
//! escaped properly, so this stays correct without a serialization framework — consistent
//! with the crate's minimal-dependency ethos.

use std::collections::BTreeMap;
use std::path::Path;

use crate::error::{Error, Result};
use crate::hash::sha256_hex;
use crate::{CompileReport, LinkMode};

/// Stable schema identifier embedded in the output so consumers can version-gate.
pub const SCHEMA: &str = "zic-rs-alias-map-v1";

/// One identifier's entry in the alias map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AliasEntry {
    /// A canonical zone: the SHA-256 of its compiled TZif file.
    Zone { sha256: String },
    /// A link (alias) to a canonical zone, with the target's hash and how it was materialised.
    Link {
        target: String,
        target_sha256: String,
        /// How the link was materialised — the typed [`LinkMode`] (`Copy` = bytes duplicated; `Symlink`
        /// = no duplication), rendered as `"copy"`/`"symlink"` only at the JSON boundary (CONTRACT.TYPING:
        /// a finite-vocabulary claim-bearing field is owned by an enum, not a hand-emitted string).
        materialised: LinkMode,
    },
}

impl AliasEntry {
    /// The alias-map `"kind"` vocabulary (`"zone"` / `"link"`). **T17.2 (CONTRACT.TYPING):** the `kind`
    /// field was previously a hand-emitted string literal inside the JSON format string; the variant is
    /// the finite claim ("is this identifier a canonical zone or an alias?"), so the literal is owned
    /// here and rendered through this accessor — a new `kind` value cannot enter the alias map as prose.
    pub fn kind_str(&self) -> &'static str {
        match self {
            AliasEntry::Zone { .. } => "zone",
            AliasEntry::Link { .. } => "link",
        }
    }
}

/// The whole manifest: a deterministic (sorted) map of identifiers plus summary counts.
#[derive(Debug, Clone)]
pub struct AliasMap {
    /// Identifier → entry, ordered by name for deterministic output.
    pub entries: BTreeMap<String, AliasEntry>,
    pub identifiers: usize,
    pub canonical_zones: usize,
    pub links: usize,
    /// Links materialised as byte copies (i.e. where the same TZif bytes exist under two
    /// names). This is the figure jiff#258 was asking to make visible.
    pub duplicated_byte_links: usize,
}

/// Build an [`AliasMap`] from a finished [`CompileReport`], hashing each compiled file under
/// `root`. Reads the just-written output files (cheap; they are small).
pub fn build(report: &CompileReport, _root: &Path) -> Result<AliasMap> {
    // Hash every canonical zone once, keyed by zone name, by reading its output file.
    let mut zone_hash: BTreeMap<String, String> = BTreeMap::new();
    for z in &report.zones_compiled {
        let bytes = std::fs::read(&z.output_path).map_err(|e| Error::io(&z.output_path, e))?;
        zone_hash.insert(z.name.clone(), sha256_hex(&bytes));
    }

    let mut entries: BTreeMap<String, AliasEntry> = BTreeMap::new();
    for z in &report.zones_compiled {
        entries.insert(
            z.name.clone(),
            AliasEntry::Zone {
                sha256: zone_hash.get(&z.name).cloned().unwrap_or_default(),
            },
        );
    }

    let mut duplicated_byte_links = 0;
    for l in &report.links_written {
        // The typed materialisation policy (CONTRACT.TYPING owns the literal via `LinkMode::as_str()`).
        let materialised = l.mode;
        if materialised == LinkMode::Copy {
            duplicated_byte_links += 1;
        }
        // `l.target` is the resolved canonical zone (see compile::plan), so its hash is known.
        let target_sha256 = zone_hash.get(&l.target).cloned().unwrap_or_default();
        entries.insert(
            l.link_name.clone(),
            AliasEntry::Link {
                target: l.target.clone(),
                target_sha256,
                materialised,
            },
        );
    }

    let map = AliasMap {
        identifiers: entries.len(),
        canonical_zones: report.zones_compiled.len(),
        links: report.links_written.len(),
        duplicated_byte_links,
        entries,
    };
    // T12.4c — fail closed if the produced map is not internally consistent (a link to a
    // non-compiled zone, a hash that doesn't match its target, a self-link, or summary counts that
    // disagree with the entries). In normal flow this always passes — `plan::run` only records a
    // link whose resolved canonical zone was compiled — so this is a defensive invariant that turns
    // any future regression into a hard error rather than a silently wrong artifact.
    map.validate()?;
    Ok(map)
}

impl AliasMap {
    /// Validate the alias map's **internal consistency** (T12.4c). Guarantees that every `Link`
    /// entry "corresponds to a materialised link/copy" of a real compiled zone:
    ///
    /// - every `Link`'s `target` is present in this map as a **`Zone`** entry (no dangling alias —
    ///   *"missing target fails"*; a link that resolved to another link, or to nothing, never gets
    ///   here because `build` records the *resolved canonical* zone);
    /// - the `Link`'s recorded `target_sha256` is non-empty and **equals** that zone's hash (the
    ///   alias genuinely names those exact bytes — the jiff#258 duplication is real, not asserted);
    /// - no entry is a **self-link** (`name == target`) — `resolve_link_target` already rejects
    ///   self-links/cycles upstream (they are *skipped* in `plan::run` and counted `failed` in the
    ///   link profile), so this is the alias-map-level guard that keeps that coverage honest;
    /// - the summary counts (`canonical_zones`/`links`/`identifiers`) agree with the entries.
    ///
    /// `build` calls this before returning, so any [`AliasMap`] handed out is already consistent.
    pub fn validate(&self) -> Result<()> {
        let (mut zones, mut links) = (0usize, 0usize);
        for (name, entry) in &self.entries {
            match entry {
                AliasEntry::Zone { sha256 } => {
                    zones += 1;
                    if sha256.len() != 64 {
                        return Err(Error::message(format!(
                            "alias-map: zone {name:?} has a malformed content hash"
                        )));
                    }
                }
                AliasEntry::Link {
                    target,
                    target_sha256,
                    ..
                } => {
                    links += 1;
                    if name == target {
                        return Err(Error::message(format!(
                            "alias-map: {name:?} is a self-link"
                        )));
                    }
                    match self.entries.get(target) {
                        Some(AliasEntry::Zone { sha256 }) => {
                            if target_sha256 != sha256 {
                                return Err(Error::message(format!(
                                    "alias-map: link {name:?} records a target hash that does not \
                                     match its zone {target:?}"
                                )));
                            }
                        }
                        Some(AliasEntry::Link { .. }) => {
                            return Err(Error::message(format!(
                                "alias-map: link {name:?} targets another link {target:?}, not a \
                                 canonical zone"
                            )))
                        }
                        None => {
                            return Err(Error::message(format!(
                                "alias-map: link {name:?} targets {target:?}, which is not a \
                                 compiled zone in this map"
                            )))
                        }
                    }
                }
            }
        }
        if zones != self.canonical_zones || links != self.links || self.identifiers != zones + links
        {
            return Err(Error::message(
                "alias-map: summary counts disagree with the entries".to_string(),
            ));
        }
        Ok(())
    }

    /// Render the manifest as deterministic, pretty-printed JSON (2-space indent, keys in a
    /// fixed order; entries sorted by identifier via the `BTreeMap`).
    pub fn to_json(&self) -> String {
        let mut s = String::new();
        s.push_str("{\n");
        s.push_str(&format!("  \"schema\": {},\n", json_str(SCHEMA)));
        s.push_str("  \"zones\": {");
        let mut first = true;
        for (name, entry) in &self.entries {
            s.push_str(if first { "\n" } else { ",\n" });
            first = false;
            // CONTRACT.TYPING (T17.2): the `kind` literal is owned by `AliasEntry::kind_str()`, not
            // hand-typed in the format string.
            let kind = entry.kind_str();
            match entry {
                AliasEntry::Zone { sha256 } => {
                    s.push_str(&format!(
                        "    {}: {{ \"kind\": {}, \"sha256\": {} }}",
                        json_str(name),
                        json_str(kind),
                        json_str(sha256)
                    ));
                }
                AliasEntry::Link {
                    target,
                    target_sha256,
                    materialised,
                } => {
                    s.push_str(&format!(
                        "    {}: {{ \"kind\": {}, \"target\": {}, \"target_sha256\": {}, \"materialised\": {} }}",
                        json_str(name),
                        json_str(kind),
                        json_str(target),
                        json_str(target_sha256),
                        json_str(materialised.as_str())
                    ));
                }
            }
        }
        s.push_str(if self.entries.is_empty() {
            "},\n"
        } else {
            "\n  },\n"
        });
        s.push_str("  \"summary\": {\n");
        s.push_str(&format!("    \"identifiers\": {},\n", self.identifiers));
        s.push_str(&format!(
            "    \"canonical_zones\": {},\n",
            self.canonical_zones
        ));
        s.push_str(&format!("    \"links\": {},\n", self.links));
        s.push_str(&format!(
            "    \"duplicated_byte_links\": {}\n",
            self.duplicated_byte_links
        ));
        s.push_str("  }\n");
        s.push_str("}\n");
        s
    }

    /// Write the manifest JSON to `path`.
    pub fn write_to(&self, path: &Path) -> Result<()> {
        std::fs::write(path, self.to_json()).map_err(|e| Error::io(path, e))
    }
}

// ===========================================================================================
// Compile-provenance manifest (`zic-rs-manifest.json`, schema `zic-rs-compile-manifest-v8`).
// ===========================================================================================
//
// Records *how this output tree was produced* — the **build identity**: source provenance (path +
// hash + kind), the tzdb version (detected vs claimed, reconciled, never silently stamped), the
// real `build_profile` (emit_style / range / redundant_until / link_mode / output_tree + leap
// source), the zones/links touched, and the oracle result. **It describes this invocation, never
// the repo's general test status nor a capability claim**: undetected source-set axes are honest
// `"unknown"` (never an aspirational `"supported"`/`"unsupported"`), and a plain `compile` run did
// not invoke `compare`, so the oracle block is `not-run` even though the fixtures are verified
// elsewhere by the test suite.

/// Stable schema identifier for the compile manifest. Schema changelog (newest first), each version
/// a *consumer-gating* marker — bumped only for a genuine block addition/removal, never for an
/// in-session correction to an unreleased version:
///
/// - **v8** (T12.5d): added the `source_profile.dataform_evidence` axis — the *encoding* form
///   (`main`/`vanguard`/`rearguard`) as detected/claimed/status, **hash-backed** against the pinned
///   2026b generated `.zi` artifacts via `source_inputs` membership (the `.zi` files are compilable
///   sources, so this is category-correct — cf. `backzone`), plus two generated-artifact provenance
///   fields: `recipe_hash` (binds archive · `Makefile` · `ziguard.awk` · command · toolchain, raw
///   bytes) and `generated_from`. Never inferred from syntax/output/names/`PACKRATLIST`/`backzone`.
/// - **v7** (T12.5c): added the `source_profile.packratlist_evidence` axis (backzone *scope*:
///   `detected: subset_from_policy_input` only via an admitted `PACKRATLIST` *generation-policy*
///   input hash-matching the pinned 2026b `zone.tab` plus a present `backzone`, else `unknown`; claim
///   `full|subset|none`). `PACKRATLIST`/`zone.tab` is a generation-policy selector, **not** a `zic`
///   compile source, so detection never consults `source_inputs`. Also removed the now-contradictory
///   `build_profile.backzone` `"unknown"` stub (backzone is the `source_profile` axis since T12.5b).
/// - **v6** (T12.5b): added the `source_profile.backzone_evidence` axis (detected/claimed/status +
///   `evidence_sha256`), hash-anchored to the pinned reference `backzone` ([`REF_2026B_BACKZONE_SHA256`],
///   admitted in T12.5a.2) — source *membership*, never inferred.
/// - **v5** (T12.4d; corrected in T12.5a): added the `source_profile` block recording the `backward`
///   **evidence axis** (detected/claimed/status + `evidence_sha256`) — admitted only from hash-backed
///   source evidence or an explicit claim, never inferred from the alias/link surface; an extension
///   seam for the later `backzone`/`rearguard`/`vanguard` axes. The T12.5a correction removed a
///   contradictory `build_profile.backward` `"unknown"` stub that v5 first shipped (it was briefly
///   double-listed; `backward` is authoritatively the `source_profile` axis) — fixed *in place*, no
///   version bump, as v5 was never released/consumed. `rearguard`/`vanguard` remain `build_profile`
///   `"unknown"` placeholders pending their own evidence axes (T12.5d, reference-first).
/// - **v4** (T12.4b): added the `link_profile` block recording link/alias identity — counts
///   (`zones_compiled`/`links_selected`/`links_materialized`/`links_omitted`/`links_failed`),
///   `link_policy`, and stable hashes (`alias_map_sha256` + `selected`/`omitted_links_sha256`) that
///   bind the build to its alias map.
/// - **v3** (T12.3): added the `source_inputs` block recording the deterministic, **order-preserving**
///   input identity (logical names + per-file hashes + `aggregate_hash`); the structural `source_kind`
///   moved there (and `individual_files` → `multi_file`); `build_profile.source_set` retired
///   (superseded).
/// - **v2** (T12.2): `tzdb.version` split into `detected_version`/`claimed_version` (+
///   `version_status`); the stub `generation_options` block replaced by a real `build_profile`
///   recording *what this run actually used*.
pub const COMPILE_SCHEMA: &str = "zic-rs-compile-manifest-v8";

/// Source provenance for the tzdata that was compiled. Records **detected facts vs user claims**
/// separately — the manifest never silently stamps a release. *Input identity* (which files, in
/// what order, with what hashes) lives in [`SourceInputs`]; this block is the version provenance.
#[derive(Debug, Clone)]
pub struct TzdbProvenance {
    /// tzdb release **sniffed** from the source's `# version …` comment, if present.
    pub detected_version: Option<String>,
    /// tzdb release the **user asserted** (`--tzdb-version`), if any.
    pub claimed_version: Option<String>,
    /// The input path(s), joined for display. **Environment context, not identity** — these are
    /// machine-local paths (possibly absolute). The portable identity is [`SourceInputs`] (logical
    /// names + content hashes).
    pub source_path: String,
    /// SHA-256 over the concatenation of all source files in **sorted (canonicalized) path
    /// order** — an *order-independent* content identity ("are these the same bytes, however they
    /// were ordered?"). The *order-sensitive* identity is [`SourceInputs::aggregate_hash`].
    pub source_sha256: String,
}

impl TzdbProvenance {
    /// Detected-vs-claimed reconciliation — prevents false conformance claims.
    pub fn version_status(&self) -> &'static str {
        match (&self.detected_version, &self.claimed_version) {
            (Some(d), Some(c)) if d == c => "detected_matches_claim",
            (Some(_), Some(_)) => "detected_differs_from_claim",
            (Some(_), None) => "detected_only",
            (None, Some(_)) => "claimed_only",
            (None, None) => "unknown",
        }
    }
}

/// One input source file in the deterministic input list (T12.3). The portable identity is the
/// `logical_name` (basename — never a machine-local absolute path) plus the content `sha256`;
/// `order_index` records the file's position in the **input order** (part of the build identity).
#[derive(Debug, Clone)]
pub struct SourceFile {
    /// The file's basename (e.g. `"northamerica"`, `"tzdata.zi"`) — a portable label, not the
    /// absolute machine-local path. The real identity is `sha256` + `order_index`.
    pub logical_name: String,
    /// SHA-256 of this file's bytes (order-independent per file).
    pub sha256: String,
    /// Byte length of this file.
    pub bytes: usize,
    /// 0-based position in the input order. **Source order is part of the build identity** — the
    /// manifest does not pretend two differently ordered inputs are the same.
    pub order_index: usize,
}

/// The deterministic input source-set of *this run* (T12.3) — **input identity, not source
/// semantics**. Records which files were used, in what order, with what hashes, under what
/// structural `kind`. It deliberately does **not** infer source-set *membership*: those are
/// reconciled as hash-backed/claim-only evidence axes in [`SourceProfile`] (`backward` T12.4d,
/// `backzone` T12.5b, `PACKRATLIST` scope T12.5c), and the remaining `DATAFORM` encoding axes
/// (`rearguard`/`vanguard`) are recorded as `"unknown"` in the build profile until a pinned,
/// deterministic detector exists (T12.5d).
#[derive(Debug, Clone)]
pub struct SourceInputs {
    /// Structural input *form* (not membership), typed (T17.2): [`SourceInputKind`]. Multi-file means
    /// *source form* only — it never implies `backward`/`backzone` inclusion.
    pub kind: SourceInputKind,
    /// The input files in **input order** (directories expanded in sorted order, then in the order
    /// the paths were supplied) — never re-sorted, so the order is faithfully recorded.
    pub files: Vec<SourceFile>,
    /// SHA-256 over the input-ordered sequence of per-file hashes — an **order-sensitive** identity
    /// that changes if the same files are supplied in a different order (cf. the order-independent
    /// [`TzdbProvenance::source_sha256`]).
    pub aggregate_hash: String,
}

/// The leap-second source used by *this run* (T12.2). Describes the run, never the project's
/// capabilities — `mode: "none"` for an ordinary compile, never `"unsupported"`.
#[derive(Debug, Clone)]
pub struct LeapSourceInfo {
    /// `None` (ordinary/`posix` profile) or `File` (the `right/` profile, `-L`) — typed (T17.2).
    pub mode: LeapSourceMode,
    /// SHA-256 of the leap-source file, when `mode == "file"` and the path was available.
    pub sha256: Option<String>,
    pub entry_count: usize,
    pub expires: bool,
    pub rolling_entries: usize,
}

/// The build-profile identity of *this run* (T12.2) — structured fields, never a vague label. Only
/// the `DATAFORM` encoding axes (`rearguard`/`vanguard`) are recorded here as `"unknown"` (no
/// deterministic detector yet — kept explicit rather than guessed; T12.5d). The source-*membership*
/// axes (`backward`, `backzone`, `PACKRATLIST` scope) moved to [`SourceProfile`] as reconciled
/// evidence axes (T12.4d/T12.5b/T12.5c) — they are never build_profile placeholders.
#[derive(Debug, Clone)]
pub struct BuildProfile {
    /// `Posix` (no leap table) or `Right` (leap table applied) — typed (T17.2).
    pub output_tree: OutputTree,
    pub leap_source: LeapSourceInfo,
    /// Semantic emission identity, the typed [`crate::EmitStyle`] (T17.2; was a re-stringified `String`).
    /// `--emit-style zic-slim` and `-b slim` map to the same value; rendered to its manifest literal at the
    /// JSON boundary by the module-private `emit_style_str` (the enum is the source of truth, not a copy).
    pub emit_style: crate::EmitStyle,
    /// `-r` range, as `(lo, hi)` raw `@`-instants (`None` = no truncation).
    pub range: Option<(Option<i64>, Option<i64>)>,
    /// `-R` redundant-tail bound (`@`-instant), if any.
    pub redundant_until: Option<i64>,
    /// Link materialisation policy of this run — the typed [`crate::LinkMode`] (T17.2; was a
    /// re-stringified `String`). Rendered `"copy"`/`"symlink"` at the boundary via [`crate::LinkMode::as_str`].
    pub link_mode: crate::LinkMode,
}

// ── T17.2 (CONTRACT.TYPING) — the manifest's remaining finite claim-bearing vocabularies, born typed. ──
//
// Each of these was a free `String`/`&'static str` whose value came from a *closed* set but was emitted
// as prose, so a future code path (or a careless edit) could leak an unintended value into the public
// `zic-rs-compile-manifest-v8` JSON. Per the standing rule — *prose is the weakest guarantee; an
// exhaustive `match` that won't compile if a variant is unclassified is the strongest* — they are now
// enums owning their JSON literal via `as_str()`. The emitted strings are **byte-identical** to the
// previous output (the manifest tests + the conformance golden pin them), so no schema bumps.

/// Output-tree profile of *this run* (T12.2; typed at T17.2 — was `&'static str`). `posix` = no leap
/// table applied; `right` = a leap table (`-L`) was applied. This is an *output-identity* claim a
/// reproducer/report reader relies on, so it is owned by the enum, not a literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputTree {
    /// No leap table (the default / `posix` profile).
    Posix,
    /// A leap table was applied (the `right/` profile).
    Right,
}

impl OutputTree {
    /// The manifest literal (`"posix"` / `"right"`).
    pub fn as_str(self) -> &'static str {
        match self {
            OutputTree::Posix => "posix",
            OutputTree::Right => "right",
        }
    }
}

/// Leap-source mode of *this run* (T12.2; typed at T17.2 — was `&'static str`). Describes the run, never
/// the project's capabilities — `None` for an ordinary compile, never an aspirational `"unsupported"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeapSourceMode {
    /// No leap source (ordinary / `posix`).
    None,
    /// A leap-seconds file was supplied (`-L`, the `right/` profile).
    File,
}

impl LeapSourceMode {
    /// The manifest literal (`"none"` / `"file"`).
    pub fn as_str(self) -> &'static str {
        match self {
            LeapSourceMode::None => "none",
            LeapSourceMode::File => "file",
        }
    }
}

/// Structural input *form* of *this run* (T12.3; typed at T17.2 — was `String`). **Form only** — it
/// never implies source-set *membership* (`backward`/`backzone` are reconciled evidence axes elsewhere).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceInputKind {
    /// Exactly one `.zi` file (e.g. the zishrunk `tzdata.zi`).
    TzdataZi,
    /// Two or more source files.
    MultiFile,
    /// Exactly one non-`.zi` file.
    SingleFile,
    /// No input files.
    Unknown,
}

impl SourceInputKind {
    /// The manifest literal.
    pub fn as_str(self) -> &'static str {
        match self {
            SourceInputKind::TzdataZi => "tzdata_zi",
            SourceInputKind::MultiFile => "multi_file",
            SourceInputKind::SingleFile => "single_file",
            SourceInputKind::Unknown => "unknown",
        }
    }

    /// Every variant, in stable order — for the totality test.
    pub const ALL: [SourceInputKind; 4] = [
        SourceInputKind::TzdataZi,
        SourceInputKind::MultiFile,
        SourceInputKind::SingleFile,
        SourceInputKind::Unknown,
    ];
}

/// The oracle **verdict** vocabulary of a manifest (T17.2 — was a free `String`). A bare `compile`
/// never runs the oracle, so the only value today is [`OracleVerdict::NotRun`] (rendered `"not-run"`,
/// preserving the legacy literal). Born typed so a future verdict (e.g. match / mismatch, if a manifest
/// path ever runs the oracle) cannot enter as an unconstrained string — distinct from the *mode* axis
/// ([`OracleMode`]), which says *which* oracle, not *what it concluded*.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OracleVerdict {
    /// The oracle was not run for this invocation (the honest default for `compile`).
    NotRun,
}

impl OracleVerdict {
    /// The manifest literal (`"not-run"` — hyphen preserved from the pre-T17.2 string).
    pub fn as_str(self) -> &'static str {
        match self {
            OracleVerdict::NotRun => "not-run",
        }
    }
}

/// Render the typed [`crate::EmitStyle`] to its manifest literal (T17.2). The manifest now stores the
/// enum directly (no re-stringified copy that could drift); this boundary fn owns the literal.
fn emit_style_str(s: crate::EmitStyle) -> &'static str {
    match s {
        crate::EmitStyle::Default => "default",
        crate::EmitStyle::ZicSlim => "zic-slim",
        crate::EmitStyle::ZicFat => "zic-fat",
    }
}

/// The link / alias identity of *this run* (T12.4b) — counts + stable hashes that bind the
/// build to its `alias-map.json`. **Links are output identifiers, not source-set evidence**: this
/// block never infers `backward`/`backzone` membership from the alias set (that is the build
/// profile's `"unknown"` axis until T12.4d gives it hash-backed evidence).
///
/// `selected` / `omitted` / `failed` are kept **distinct** (by design):
/// - **selected** — a parsed `Link` whose resolved canonical zone *is* in the compiled output set
///   (eligible and materialised). In zic-rs a selected link always materialises (a write failure
///   aborts the whole run), so `links_selected_count` and the db-link share of
///   `links_materialized_count` coincide — they differ only by install-policy links like
///   `localtime` (materialised but not a source `Link`).
/// - **omitted** — a *valid* parsed `Link` not materialised because selection/profile excluded its
///   target from the output set. A policy outcome, **not** an error.
/// - **failed** — a `Link` whose chain does not terminate at a real zone (missing target / cycle).
///   An error class, never folded into `omitted`. (A bare successful `compile` writes no manifest
///   if a link fatally fails, so this is normally 0; it is recorded for completeness.)
#[derive(Debug, Clone)]
pub struct LinkProfile {
    /// `"copy"` or `"symlink"` — how links were materialised this run.
    pub link_policy: String,
    /// Canonical zones actually compiled this run.
    pub zones_compiled_count: usize,
    /// Parsed `Link`s eligible & materialised (resolved target in the compiled set).
    pub links_selected_count: usize,
    /// Links actually written to the output tree (includes install-policy links like `localtime`).
    pub links_materialized_count: usize,
    /// Valid links excluded by selection/profile (target not compiled). Policy, not error.
    pub links_omitted_count: usize,
    /// Links whose chain does not resolve to a real zone (missing/cycle/self). Error, not omission.
    pub links_failed_count: usize,
    /// SHA-256 of the **deterministic** `alias-map.json` serialization (sorted by identifier, fixed
    /// field order, LF, no timestamps) — binds this manifest to a specific alias map.
    pub alias_map_sha256: String,
    /// SHA-256 over the sorted selected-link names (LF-joined) — order-independent set identity.
    pub selected_links_sha256: String,
    /// SHA-256 over the sorted omitted-link names (LF-joined).
    pub omitted_links_sha256: String,
}

/// What the *admitted source evidence* mechanically proves about whether the `backward` source
/// participated in this build (T12.4d). **Bounded to an admitted artifact** — never a universal
/// claim and never inferred from the alias/link surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackwardDetected {
    /// The admitted backward source's bytes are present among `source_inputs` (hash-backed).
    Present,
    /// The admitted backward source's bytes are **not** in the build (hash-backed). This is absence
    /// *of the admitted artifact*, not a proof that "no backward data exists anywhere".
    Absent,
    /// No backward source was admitted, so nothing is mechanically proven. The honest default.
    Unknown,
}

/// What the build/user/config **explicitly asserts** about `backward` membership (T12.4d) — a bare
/// claim, recorded separately from (and never trusted as) detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackwardClaim {
    Included,
    Excluded,
    /// No claim was made.
    None,
}

/// The `backward` **evidence axis** (T12.4d) — mirrors T12.2's detected-vs-claimed version
/// reconciliation. **The alias surface is output identity, not source provenance:** a build can
/// expose legacy-looking aliases without proving the tzdb `backward` source participated, and the
/// absence of such aliases does not prove `backward` was excluded. So `backward` is recorded as a
/// reconciled evidence axis, **never a boolean**, and stays `Unknown` unless admitted by hash-backed
/// evidence or an explicit claim. **Admission law — `backward` status may be admitted only from
/// (1) hash-backed source evidence or (2) an explicit claim; it must NOT be inferred from alias
/// count, alias names, output filenames, source filenames alone, link target names, selected/
/// omitted/failed link counts, or legacy-looking identifiers.**
#[derive(Debug, Clone)]
pub struct BackwardEvidence {
    pub detected: BackwardDetected,
    pub claimed: BackwardClaim,
    /// SHA-256 of the *admitted* backward source (present or absent), when one was admitted. This is
    /// the hash whose presence/absence in `source_inputs` produced `detected`.
    pub evidence_sha256: Option<String>,
}

impl BackwardEvidence {
    /// Build the evidence axis from the admitted inputs (T12.4d). **This is the entire admission
    /// law in code:** detection comes *only* from an admitted backward `source` file, hash-checked
    /// against the build's `source_inputs`:
    /// - admitted file's bytes are among `source_inputs` → `Present` (hash-backed);
    /// - admitted file's bytes are **not** among them → `Absent` (hash-backed; bounded to *this*
    ///   artifact — never a universal "no backward exists" claim);
    /// - no file admitted → `Unknown` (we refuse to infer from alias counts, names, or filenames).
    ///
    /// The bare `claim` is recorded independently and never promoted to detection. Nothing here reads
    /// the link/alias surface, so `backward` can never be inferred from it.
    pub fn reconcile(source_inputs: &SourceInputs, args: &SourceVariantArgs) -> Result<Self> {
        let claimed = match args.backward_claim {
            Some(true) => BackwardClaim::Included,
            Some(false) => BackwardClaim::Excluded,
            None => BackwardClaim::None,
        };
        let (detected, evidence_sha256) = match &args.backward_source {
            Some(path) => {
                let bytes = std::fs::read(path).map_err(|e| Error::io(path, e))?;
                let h = sha256_hex(&bytes);
                let present = source_inputs.files.iter().any(|f| f.sha256 == h);
                let d = if present {
                    BackwardDetected::Present
                } else {
                    BackwardDetected::Absent
                };
                (d, Some(h))
            }
            None => (BackwardDetected::Unknown, None),
        };
        Ok(BackwardEvidence {
            detected,
            claimed,
            evidence_sha256,
        })
    }

    /// Reconcile detected vs claimed into a single status string (cf. [`TzdbProvenance::version_status`]).
    /// Detection always outranks a bare claim for the agreement/conflict verdicts; an unverified claim
    /// is explicitly labelled `*_unverified` so it can never be mistaken for a detected fact.
    pub fn status(&self) -> &'static str {
        use BackwardClaim as C;
        use BackwardDetected as D;
        match (self.detected, self.claimed) {
            (D::Present, C::Included) | (D::Absent, C::Excluded) => "detected_matches_claim",
            (D::Present, C::Excluded) | (D::Absent, C::Included) => "detected_contradicts_claim",
            (D::Present, C::None) => "detected_present",
            (D::Absent, C::None) => "detected_absent",
            (D::Unknown, C::Included) => "claimed_present_unverified",
            (D::Unknown, C::Excluded) => "claimed_absent_unverified",
            (D::Unknown, C::None) => "unknown_no_evidence",
        }
    }

    fn detected_str(&self) -> &'static str {
        match self.detected {
            BackwardDetected::Present => "present",
            BackwardDetected::Absent => "absent",
            BackwardDetected::Unknown => "unknown",
        }
    }

    fn claimed_str(&self) -> &'static str {
        match self.claimed {
            BackwardClaim::Included => "included",
            BackwardClaim::Excluded => "excluded",
            BackwardClaim::None => "none",
        }
    }
}

/// SHA-256 of the **pristine IANA tzdb 2026b `backzone` file**, admitted + signature-verified +
/// pinned in T12.5a.2 (`reports/t12_5a2-reference-admission.md`). The `backzone` evidence detector
/// (T12.5b) checks whether *this exact file* participated in a build's `source_inputs` — hash-backed,
/// version-scoped to 2026b (a later release has a different hash and needs its own admission).
pub const REF_2026B_BACKZONE_SHA256: &str =
    "63fb39adae0b0d8b2179629725a9dfb694c7a386b99750b636a017d896d28dfa";

/// What the admitted source evidence proves about `backzone`/`PACKRATDATA` participation (T12.5b).
/// **Presence is hash-backed; absence is NOT asserted** — the canonical `backzone` file not appearing
/// among `source_inputs` does *not* prove backzone data is absent (it can be merged into a
/// concatenated `.zi`), so non-presence is `Unknown`, never a false "absent".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackzoneDetected {
    /// The pinned reference `backzone` file's bytes are present among `source_inputs` (hash-backed).
    Present,
    /// No hash-backed evidence either way (canonical file not seen; may be merged — cannot conclude).
    Unknown,
}

/// What the build/user explicitly asserts about `backzone` membership (T12.5b) — recorded separately
/// from detection, never promoted to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackzoneClaim {
    Included,
    Excluded,
    None,
}

/// The `backzone` / `PACKRATDATA` **source-membership evidence axis** (T12.5b) — mirrors
/// `BackwardEvidence`, but detection is anchored to the *pinned reference release's* `backzone` hash
/// ([`REF_2026B_BACKZONE_SHA256`]). **Source membership, hash-backed or claim-only, never inferred**
/// from aliases, zone names, link counts, output byte shape, pre-1970 differences, or `DATAFORM`.
/// **Scope:** whether `backzone` participated at all — the *subset-vs-all* (`PACKRATLIST`) distinction
/// is T12.5c, and `DATAFORM` is T12.5d.
#[derive(Debug, Clone)]
pub struct BackzoneEvidence {
    pub detected: BackzoneDetected,
    pub claimed: BackzoneClaim,
    /// The pinned reference `backzone` hash, when detected present.
    pub evidence_sha256: Option<String>,
}

impl BackzoneEvidence {
    /// Detect `backzone` participation by checking whether the pinned reference `backzone` hash
    /// (`reference_backzone_sha256`) appears among `source_inputs`. Hash-backed, version-scoped. The
    /// claim is recorded independently. **Does not read the link/alias surface — inference is
    /// impossible by construction.** (`reference_backzone_sha256` is injected so the detector is unit-
    /// testable without vendoring the large reference file; production passes [`REF_2026B_BACKZONE_SHA256`].)
    pub fn reconcile(
        source_inputs: &SourceInputs,
        claim: Option<bool>,
        reference_backzone_sha256: &str,
    ) -> Self {
        let claimed = match claim {
            Some(true) => BackzoneClaim::Included,
            Some(false) => BackzoneClaim::Excluded,
            None => BackzoneClaim::None,
        };
        let present = source_inputs
            .files
            .iter()
            .any(|f| f.sha256 == reference_backzone_sha256);
        let (detected, evidence_sha256) = if present {
            (
                BackzoneDetected::Present,
                Some(reference_backzone_sha256.to_string()),
            )
        } else {
            (BackzoneDetected::Unknown, None)
        };
        BackzoneEvidence {
            detected,
            claimed,
            evidence_sha256,
        }
    }

    /// Reconcile detected vs claimed (cf. [`BackwardEvidence::status`]). No `detected_absent` —
    /// absence is never asserted for `backzone` (see [`BackzoneDetected`]).
    pub fn status(&self) -> &'static str {
        use BackzoneClaim as C;
        use BackzoneDetected as D;
        match (self.detected, self.claimed) {
            (D::Present, C::Included) => "detected_matches_claim",
            (D::Present, C::Excluded) => "detected_contradicts_claim",
            (D::Present, C::None) => "detected_present",
            (D::Unknown, C::Included) => "claimed_present_unverified",
            (D::Unknown, C::Excluded) => "claimed_absent_unverified",
            (D::Unknown, C::None) => "unknown_no_evidence",
        }
    }

    fn detected_str(&self) -> &'static str {
        match self.detected {
            BackzoneDetected::Present => "present",
            BackzoneDetected::Unknown => "unknown",
        }
    }

    fn claimed_str(&self) -> &'static str {
        match self.claimed {
            BackzoneClaim::Included => "included",
            BackzoneClaim::Excluded => "excluded",
            BackzoneClaim::None => "none",
        }
    }
}

/// SHA-256 of the **pristine IANA tzdb 2026b `zone.tab`** (admitted + pinned in T12.5a.2). It is the
/// canonical `PACKRATLIST` subset-selector. **Pinned for the test fixture / documentation only** — the
/// detector does *not* trigger on `zone.tab` merely appearing among inputs (it is a normal selection
/// table whose presence proves nothing about `PACKRATLIST`); see [`PackratlistEvidence`].
pub const REF_2026B_ZONE_TAB_SHA256: &str =
    "4d8e389e5f4b0ec0466d5b14f42e5dfb0308c4376165fcf478339afd9ddcb00c";

/// What the admitted evidence proves about the **`backzone` *scope*** (`PACKRATLIST`) — T12.5c.
/// **Category boundary (the empirical finding):** `PACKRATLIST` is a *generation-policy* input, **not a
/// `zic` compile source** — its list (`zone.tab`) filters `backzone` at generation time and is baked
/// into the produced `.zi`. So scope is **not recoverable from `source_inputs` (compile inputs)**:
/// `zone.tab` appearing among inputs would be a category error to read as evidence, and absence proves
/// nothing. The only hash-backed detection is **`SubsetFromPolicyInput`** — an *explicitly admitted*
/// `PACKRATLIST` selector (`--packratlist-source`) hashed as a **policy input**, alongside a present
/// `backzone`. Everything else is **`Unknown`**; `full`/`none` are claim-only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackratlistDetected {
    /// A hash-backed `PACKRATLIST` **policy input** was admitted alongside a present `backzone`
    /// (bounded to that artifact — does **not** assert the generation step applied the filter).
    SubsetFromPolicyInput,
    /// No hash-backed scope evidence (no admitted policy input, or no backzone).
    Unknown,
}

/// What the build/user explicitly asserts about `backzone` scope (T12.5c). `--packratlist
/// {full|subset|none}`: `full` = all backzone (`PACKRATLIST` empty), `subset` = filtered
/// (`PACKRATLIST=zone.tab`), `none` = no backzone (`PACKRATDATA` empty).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackratlistClaim {
    Full,
    Subset,
    None,
    NotClaimed,
}

/// The `PACKRATLIST` **backzone-scope evidence axis** (T12.5c) — mirrors the other axes
/// (`detected`/`claimed`/`status`/`evidence_sha256`). **Subset is the only hash-backed detection**
/// (admitted subset-list participated + backzone present); never inferred from output zone counts,
/// alias counts, filenames, `zone.tab`/`zone1970.tab` presence alone, link counts, pre-1970
/// differences, or global-tz-like output shape.
#[derive(Debug, Clone)]
pub struct PackratlistEvidence {
    pub detected: PackratlistDetected,
    pub claimed: PackratlistClaim,
    /// The admitted `PACKRATLIST`-source hash, when a subset was hash-detected.
    pub evidence_sha256: Option<String>,
}

impl PackratlistEvidence {
    /// Build the scope axis from a **generation-policy input** (T12.5c). **Category boundary (the
    /// empirical finding):** `PACKRATLIST` is a *generation-time* selector (the Makefile filters
    /// `backzone` *before* `zic`), and its list (`zone.tab`) is **not a compilable `zic` source** — so
    /// it does **not** belong in `source_inputs` (the compile inputs) and detection must **never** be
    /// keyed off `source_inputs` membership (that would be a category error). Instead, detection is
    /// keyed off an **explicitly admitted policy input**: `admitted_policy_input_sha256` is the
    /// SHA-256 of a `--packratlist-source` selector the caller supplied (`None` = none admitted).
    /// `Subset` (from a policy input) only when such a selector is admitted **and** `backzone` is
    /// present (a subset list is meaningless without backzone data). Everything else is `Unknown`;
    /// `full`/`none` are claim-only. Nothing here reads compile inputs, the link/alias surface, or
    /// output shape. **Bounded meaning:** an admitted hash-backed selector is stronger than a bare
    /// claim, but does **not** prove the generation step actually applied the filter.
    pub fn reconcile(
        claim: Option<&str>,
        admitted_policy_input_sha256: Option<&str>,
        reference_zone_tab_sha256: &str,
        backzone_present: bool,
    ) -> Self {
        let claimed = match claim {
            Some("full") => PackratlistClaim::Full,
            Some("subset") => PackratlistClaim::Subset,
            Some("none") => PackratlistClaim::None,
            _ => PackratlistClaim::NotClaimed,
        };
        // Version-scoped + category-correct: `SubsetFromPolicyInput` only when the admitted policy
        // input **is the pinned reference `zone.tab`** (hash match) AND `backzone` is present. An
        // arbitrary admitted file, a different release's table, or no backzone → `Unknown`. We never
        // look at `source_inputs` (compile inputs) — `zone.tab` is a generation-policy input.
        let detected_subset =
            backzone_present && admitted_policy_input_sha256 == Some(reference_zone_tab_sha256);
        let (detected, evidence_sha256) = if detected_subset {
            (
                PackratlistDetected::SubsetFromPolicyInput,
                Some(reference_zone_tab_sha256.to_string()),
            )
        } else {
            (PackratlistDetected::Unknown, None)
        };
        PackratlistEvidence {
            detected,
            claimed,
            evidence_sha256,
        }
    }

    /// Reconcile detected vs claimed (cf. [`BackzoneEvidence::status`]). A bare claim with no admitted
    /// policy input is `claimed_*_not_hash_backed` — never promoted to detection.
    pub fn status(&self) -> &'static str {
        use PackratlistClaim as C;
        use PackratlistDetected as D;
        match (self.detected, &self.claimed) {
            (D::SubsetFromPolicyInput, C::Subset) => "detected_matches_claim",
            (D::SubsetFromPolicyInput, C::Full) | (D::SubsetFromPolicyInput, C::None) => {
                "detected_contradicts_claim"
            }
            (D::SubsetFromPolicyInput, C::NotClaimed) => "detected_subset_from_policy_input",
            (D::Unknown, C::Full) => "claimed_full_not_hash_backed",
            (D::Unknown, C::Subset) => "claimed_subset_not_hash_backed",
            (D::Unknown, C::None) => "claimed_none_not_hash_backed",
            (D::Unknown, C::NotClaimed) => "unknown_no_evidence",
        }
    }

    fn detected_str(&self) -> &'static str {
        match self.detected {
            PackratlistDetected::SubsetFromPolicyInput => "subset_from_policy_input",
            PackratlistDetected::Unknown => "unknown",
        }
    }

    fn claimed_str(&self) -> &'static str {
        match self.claimed {
            PackratlistClaim::Full => "full",
            PackratlistClaim::Subset => "subset",
            PackratlistClaim::None => "none",
            PackratlistClaim::NotClaimed => "not_claimed",
        }
    }
}

// ---------------------------------------------------------------------------------------------
// DATAFORM (`main`/`vanguard`/`rearguard`) — the *encoding* evidence axis (T12.5d).
//
// **Category (the clean mental model):** `backzone` = source-*membership* evidence; `zone.tab` =
// generation-*policy* evidence; `vanguard.zi`/`main.zi`/`rearguard.zi` = **generated-artifact**
// evidence; `DATAFORM` = the upstream *encoding-policy* those artifacts realise. Crucially, unlike
// `PACKRATLIST`'s `zone.tab` (a non-compilable policy table), the three `.zi` artifacts **are
// compilable `zic` sources** — so DATAFORM detection is *category-correct* from `source_inputs`
// membership (it mirrors `backzone`, not `packratlist`): if you compiled `vanguard.zi`, its bytes
// are a `source_input`, and that is the only honest hash-backed signal of the encoding form.
//
// **Central law:** DATAFORM is admitted **only** by a hash-backed match against the pinned 2026b
// generated artifacts, or by an explicit claim — **never** by inspecting source syntax (mainline
// 2026b already uses negative `SAVE`, so "negative SAVE ⇒ vanguard" is provably wrong), output
// shape, zone names, filenames, `PACKRATLIST`/`backzone`, or diagnostic behaviour. `ziguard.awk` is
// **not** treated as a general converter (it targets *current* tzdata, is neither idempotent nor
// reversible); the `.zi` witnesses are recorded as *generated reference artifacts* with a
// `recipe_hash`, not as something zic-rs can reproduce or transform.

/// SHA-256 of the pinned 2026b complete-distribution archive (`tzdb-2026b.tar.lz`), admitted +
/// signature-verified in T12.5a.2. A `recipe_hash` input — it transitively binds every shipped file
/// (`Makefile`, `ziguard.awk`, the region sources) that the DATAFORM generation consumed.
pub const REF_2026B_ARCHIVE_SHA256: &str =
    "ffad46a04c8d1624197056630af475a35f3556d0887f028ac1bd33b7d47dc653";

/// SHA-256 of the pinned 2026b `Makefile` (the `DATAFORM`/`ziguard.awk` generation rules). A
/// `recipe_hash` input.
pub const REF_2026B_MAKEFILE_SHA256: &str =
    "0b4588ea467c969b23fc48335e91eb63f403574b4aac69380b84a00373c7e81d";

/// SHA-256 of the pinned 2026b `ziguard.awk` (the DATAFORM transform). A `recipe_hash` input —
/// pinned explicitly even though it ships inside the archive, so the recipe binding is legible.
pub const REF_2026B_ZIGUARD_AWK_SHA256: &str =
    "e4600a2360b692242d6da76666411ece8ada76b61e6f8fb69cec79592b261785";

/// The exact `make` invocation that generated the pinned DATAFORM `.zi` artifacts. A `recipe_hash`
/// input — changing the command changes the recipe identity.
pub const REF_2026B_DATAFORM_COMMAND: &str = "make vanguard.zi main.zi rearguard.zi";

/// The toolchain that ran the DATAFORM generation (recorded because the `.zi` witnesses are
/// *derived* — a different awk could in principle differ). A `recipe_hash` input.
pub const REF_2026B_DATAFORM_TOOLCHAIN: &str = "GNU Make 4.4.1; GNU Awk 5.4.0";

/// A short, stable tag for the release the pinned DATAFORM artifacts were generated from. Stamped
/// into `generated_from` when a form is detected.
pub const REF_2026B_DATAFORM_GENERATED_FROM: &str = "tzdb-2026b";

/// SHA-256 of the pinned 2026b `main.zi` (the default `DATAFORM=main` generated artifact, T12.5a.2).
pub const REF_2026B_MAIN_ZI_SHA256: &str =
    "e0225823ae0c3a99a016a4afd7e3c48cfd948132b65fbaa596a47c53ae45e4e1";

/// SHA-256 of the pinned 2026b `vanguard.zi` (`DATAFORM=vanguard`).
pub const REF_2026B_VANGUARD_ZI_SHA256: &str =
    "49e16da4a6252a2e432fc1f68bf6daac9a6f73507dde3e3bdbcbbf78e86727ce";

/// SHA-256 of the pinned 2026b `rearguard.zi` (`DATAFORM=rearguard`).
pub const REF_2026B_REARGUARD_ZI_SHA256: &str =
    "91c4f362a6bb297efd3cd35bce6b62367a4c00a9721a773bae0cbb0d1bf9fe23";

/// Compute the **`recipe_hash`** that binds the *generation provenance* of the pinned DATAFORM `.zi`
/// artifacts (T12.5d). It is a SHA-256 over a deterministic, labelled, newline-joined record of the
/// recipe inputs — the archive hash, the `Makefile` hash, the `ziguard.awk` hash, the generation
/// command, and the toolchain — **hashed as raw UTF-8 bytes, never line-ending-normalized** (a
/// transformed copy with different newline bytes is a *different* artifact and must hash differently).
/// The produced artifact itself is bound separately via the evidence axis's `evidence_sha256`; this
/// value answers "by what recipe was that artifact generated", so a generated artifact is never just
/// "hash matched" — it is "hash matched, and here is the recorded, reproducible recipe".
pub fn dataform_recipe_hash(
    archive_sha256: &str,
    makefile_sha256: &str,
    ziguard_awk_sha256: &str,
    command: &str,
    toolchain: &str,
) -> String {
    let recipe = format!(
        "archive_sha256={archive_sha256}\nmakefile_sha256={makefile_sha256}\n\
         ziguard_awk_sha256={ziguard_awk_sha256}\ncommand={command}\ntoolchain={toolchain}\n"
    );
    crate::hash::sha256_hex(recipe.as_bytes())
}

/// The pinned-release DATAFORM reference, injected into [`DataformEvidence::reconcile`] so the
/// detector is unit-testable without vendoring the large `.zi` files. Production builds this from the
/// `REF_2026B_*` consts plus the computed [`dataform_recipe_hash`].
#[derive(Debug, Clone, Copy)]
pub struct DataformReference<'a> {
    pub main_sha256: &'a str,
    pub vanguard_sha256: &'a str,
    pub rearguard_sha256: &'a str,
    /// The shared generation recipe hash, stamped into the evidence when a form is detected.
    pub recipe_hash: &'a str,
    /// The release tag the artifacts were generated from (e.g. `"tzdb-2026b"`).
    pub generated_from: &'a str,
}

/// Which **encoding form** the admitted source bytes match (T12.5d) — hash-backed against the pinned
/// generated artifacts, **never** inferred from syntax/output/names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataformDetected {
    Main,
    Vanguard,
    Rearguard,
    /// No admitted source matched a pinned DATAFORM artifact hash (e.g. a zishrunk `tzdata.zi`, a
    /// concatenated build, or a different release) — the encoding form is not hash-recoverable.
    Unknown,
}

/// What the build/user explicitly asserts about the encoding form (`--dataform
/// {main|vanguard|rearguard}`), recorded separately from detection and never promoted to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataformClaim {
    Main,
    Vanguard,
    Rearguard,
    None,
}

/// The `DATAFORM` **encoding evidence axis** (T12.5d). Mirrors the other axes
/// (`detected`/`claimed`/`status`/`evidence_sha256`) and adds two provenance fields specific to a
/// *generated* artifact: `recipe_hash` (how it was produced) and `generated_from` (which release).
/// **Encoding evidence — hash-backed or claim-only, never inferred** from negative `SAVE` or any
/// other syntax resemblance, output shape, zone names, source filenames, `PACKRATLIST`, `backzone`,
/// or diagnostics.
#[derive(Debug, Clone)]
pub struct DataformEvidence {
    pub detected: DataformDetected,
    pub claimed: DataformClaim,
    /// The matched pinned artifact hash, when a form was detected.
    pub evidence_sha256: Option<String>,
    /// The generation `recipe_hash` of the matched artifact, when detected (see [`dataform_recipe_hash`]).
    pub recipe_hash: Option<String>,
    /// The release the matched artifact was generated from (e.g. `"tzdb-2026b"`), when detected.
    pub generated_from: Option<String>,
}

impl DataformEvidence {
    /// Detect the encoding form by checking whether any `source_input`'s hash equals one of the
    /// pinned DATAFORM artifact hashes (`reference`). Category-correct: the `.zi` artifacts are
    /// compilable `zic` sources, so `source_inputs` membership is the honest signal (cf. `backzone`).
    /// The claim is recorded independently. **Reads only file hashes — never source syntax, output
    /// shape, names, or the link/alias surface, so inference is impossible by construction.** When a
    /// form is detected, the shared `recipe_hash`/`generated_from` are stamped so the artifact carries
    /// its generation provenance, not merely a matched hash.
    pub fn reconcile(
        source_inputs: &SourceInputs,
        claim: Option<&str>,
        reference: &DataformReference,
    ) -> Self {
        let claimed = match claim {
            Some("main") => DataformClaim::Main,
            Some("vanguard") => DataformClaim::Vanguard,
            Some("rearguard") => DataformClaim::Rearguard,
            _ => DataformClaim::None,
        };
        // First admitted source whose hash matches a pinned artifact wins; the three reference hashes
        // are distinct, so at most one form can match a given file.
        let mut detected = DataformDetected::Unknown;
        let mut evidence_sha256 = None;
        for f in &source_inputs.files {
            if f.sha256 == reference.main_sha256 {
                detected = DataformDetected::Main;
            } else if f.sha256 == reference.vanguard_sha256 {
                detected = DataformDetected::Vanguard;
            } else if f.sha256 == reference.rearguard_sha256 {
                detected = DataformDetected::Rearguard;
            } else {
                continue;
            }
            evidence_sha256 = Some(f.sha256.clone());
            break;
        }
        let (recipe_hash, generated_from) = if evidence_sha256.is_some() {
            (
                Some(reference.recipe_hash.to_string()),
                Some(reference.generated_from.to_string()),
            )
        } else {
            (None, None)
        };
        DataformEvidence {
            detected,
            claimed,
            evidence_sha256,
            recipe_hash,
            generated_from,
        }
    }

    /// Reconcile detected vs claimed. A bare claim with no hash-backed detection is `claim_only` —
    /// never promoted to detection.
    pub fn status(&self) -> &'static str {
        use DataformClaim as C;
        use DataformDetected as D;
        let claim_form = match self.claimed {
            C::Main => Some(D::Main),
            C::Vanguard => Some(D::Vanguard),
            C::Rearguard => Some(D::Rearguard),
            C::None => None,
        };
        match (self.detected, claim_form) {
            (D::Unknown, None) => "unknown_no_evidence",
            (D::Unknown, Some(_)) => "claim_only",
            (_, None) => "detected_only",
            (d, Some(c)) if d == c => "detected_matches_claim",
            (_, Some(_)) => "detected_contradicts_claim",
        }
    }

    fn detected_str(&self) -> &'static str {
        match self.detected {
            DataformDetected::Main => "main",
            DataformDetected::Vanguard => "vanguard",
            DataformDetected::Rearguard => "rearguard",
            DataformDetected::Unknown => "unknown",
        }
    }

    fn claimed_str(&self) -> &'static str {
        match self.claimed {
            DataformClaim::Main => "main",
            DataformClaim::Vanguard => "vanguard",
            DataformClaim::Rearguard => "rearguard",
            DataformClaim::None => "none",
        }
    }
}

/// The **source profile** block (T12.4d; extended T12.5b/c/d) — a deliberate extension seam. It
/// carries the `backward` (T12.4d), `backzone` (T12.5b), `packratlist` backzone-scope (T12.5c), and
/// `dataform` encoding (T12.5d) evidence axes, each detected/claimed/status, hash-backed or
/// claim-only, never inferred.
#[derive(Debug, Clone)]
pub struct SourceProfile {
    pub backward: BackwardEvidence,
    pub backzone: BackzoneEvidence,
    pub packratlist: PackratlistEvidence,
    pub dataform: DataformEvidence,
}

/// Caller-supplied inputs for the source-variant evidence axes (T12.4d `backward`; T12.5b `backzone`;
/// T12.5c/d to come). **Provenance-only** — these never influence compilation, link materialisation,
/// or the alias map; they only feed the manifest's `source_profile`. The bare claims come from
/// `--backward`/`--backzone`; `--backward-source` admits a file whose bytes are hash-checked.
#[derive(Debug, Clone, Default)]
pub struct SourceVariantArgs {
    /// `backward` claim: `Some(true)` = claimed included, `Some(false)` = excluded, `None` = no claim.
    pub backward_claim: Option<bool>,
    /// A file the caller asserts is the `backward` source; detection verifies whether its *bytes*
    /// participated in this build (it does **not** assert semantic identity as the IANA `backward`).
    pub backward_source: Option<std::path::PathBuf>,
    /// `backzone` (`PACKRATDATA`) claim: `Some(true)` = claimed included, `Some(false)` = excluded,
    /// `None` = no claim. Detection is hash-anchored to the pinned reference release (T12.5b); this is
    /// the *claim* side only. (`PACKRATLIST` subset selection → T12.5c; `DATAFORM` → T12.5d.)
    pub backzone_claim: Option<bool>,
    /// `backzone` *scope* (`PACKRATLIST`) claim (T12.5c): `"full"` / `"subset"` / `"none"` (else no
    /// claim). The bare `--packratlist` assertion; never promoted to detection.
    pub packratlist_claim: Option<String>,
    /// A file the caller explicitly admits as the `PACKRATLIST` subset source (T12.5c); detection
    /// confirms its *bytes* participated alongside `backzone` (→ `Subset`). Mere `zone.tab` presence
    /// among inputs is **not** admission and never triggers `Subset`.
    pub packratlist_source: Option<std::path::PathBuf>,
    /// `DATAFORM` *encoding* claim (T12.5d): `"main"` / `"vanguard"` / `"rearguard"` (else no claim).
    /// The bare `--dataform` assertion; never promoted to detection. Detection is hash-backed against
    /// the pinned 2026b `.zi` artifacts via `source_inputs` membership — there is intentionally **no**
    /// `--dataform-source`: the `.zi` artifacts *are* compile inputs, so admitting one you did not
    /// compile would assert provenance for bytes the build never used.
    pub dataform_claim: Option<String>,
}

// ===========================================================================================
// Provenance capability statement (T12.6) — a STATIC, run-independent description of the manifest
// schema this build emits and the **source-variant reference-pin gate** state. Surfaced read-only in
// `support-report`/`structural-report` so an operator/packager sees the trust boundary without
// reading manifest internals. It is deliberately NOT a per-run profile: a report run is not a
// configured output compile, so it has no honest `build_profile`/`link_profile`/`backward_evidence`
// of its own — those live in `compile --manifest` and are pointed to, never fabricated here.
// ===========================================================================================

/// Status of the source-variant reference-pin gate (T12.5a.1 created it; T12.5a.2 lifted it).
/// `"lifted_for_2026b"` — the pristine IANA tzdb 2026b reference set was fetched, **signature-verified**,
/// and SHA-256-pinned (`reports/t12_5a2-reference-admission.md`), so T12.5b–d are unblocked **for that
/// pinned reference only**. Version-scoped: a later release re-opens the gate until its own admission.
/// Single source of truth for the reports' provenance block. *(Admission ≠ implementation — see
/// [`SOURCE_VARIANT_BEHAVIOR_IMPLEMENTED`].)*
pub const SOURCE_VARIANT_GATE_STATUS: &str = "lifted_for_2026b";

/// **Which oracle backed a report's verdicts** (T15.2 — CONTRACT.TYPING). The single owner type for the
/// oracle-mode vocabulary: as of **T15.2a**, [`OracleResult::mode`](OracleResult) is this enum too (no
/// claim-bearing path emits the vocabulary as a free string). Reports render [`mode_str`](Self::mode_str)
/// (canonical snake_case); the `zic-rs-compile-manifest-v8` `oracle.mode` field renders
/// [`manifest_str`](Self::manifest_str), a **boundary-only compatibility shim** that preserves the one
/// legacy value (`"not-run"`) the manifest has ever emitted — *removal plan:* canonicalize to `mode_str`
/// at the next manifest major bump (a drift test pins that the shim diverges for that one value only).
/// The rule it enforces: **oracle *absence* is visible** — a report renders `Unavailable(reason)` (→
/// `skipped_with_reason`), never silence, so a verdict can never *silently* weaken when reference tools
/// are missing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OracleMode {
    /// No oracle was consulted by design (e.g. `support-report` is compile-coverage, not behaviour).
    NotRun,
    /// Reference `zic`'s emitted bytes were the oracle (e.g. `structural-report`).
    ReferenceZic,
    /// Reference `zdump`'s decoded behaviour was the oracle (the `compare` zdump mode).
    ReferenceZdump,
    /// A decoded-TZif structural comparison (the `compare` structural mode).
    StructuralDecode,
    /// The required oracle tool was unavailable; the verdict was skipped, with this reason.
    Unavailable(String),
}

impl OracleMode {
    /// The stable snake_case discriminant.
    pub fn mode_str(&self) -> &'static str {
        match self {
            OracleMode::NotRun => "not_run",
            OracleMode::ReferenceZic => "reference_zic",
            OracleMode::ReferenceZdump => "reference_zdump",
            OracleMode::StructuralDecode => "structural_decode",
            OracleMode::Unavailable(_) => "unavailable",
        }
    }

    /// The reason an oracle was skipped, when (and only when) it was [`Unavailable`](Self::Unavailable).
    pub fn skipped_with_reason(&self) -> Option<&str> {
        match self {
            OracleMode::Unavailable(reason) => Some(reason.as_str()),
            _ => None,
        }
    }

    /// The **`zic-rs-compile-manifest-v8` boundary** rendering (T15.2a compatibility shim). The manifest
    /// path only ever holds [`NotRun`](Self::NotRun) and has historically emitted `"not-run"`
    /// (hyphenated); that one value is preserved here for back-compat. Every other variant has no legacy
    /// manifest form (they never appeared there), so this is identical to [`mode_str`](Self::mode_str) for
    /// them — i.e. the shim diverges for exactly one value, which a drift test pins. Removal plan:
    /// canonicalize to `mode_str` at the next manifest major bump.
    pub fn manifest_str(&self) -> &'static str {
        match self {
            OracleMode::NotRun => "not-run",
            other => other.mode_str(),
        }
    }

    /// Render as the report's `oracle_mode` object: `{ "mode": …, "skipped_with_reason": …|null }`.
    /// Absence is always visible — `skipped_with_reason` is non-null exactly when the oracle was missing.
    pub fn to_json_field(&self) -> String {
        let reason = match self.skipped_with_reason() {
            Some(r) => json_str(r),
            None => "null".to_string(),
        };
        format!(
            "{{ \"mode\": {}, \"skipped_with_reason\": {} }}",
            json_str(self.mode_str()),
            reason
        )
    }
}

/// A **non-claim**, made a first-class machine-visible contract (T15.2). Advertised restraint is
/// engineering, not decoration: each variant renders a stable snake_case string **and** names the
/// guard/test/receipt that *enforces* the boundary (`enforced_by`). "We don't claim X" is exactly where
/// infrastructure tools get sloppy — this makes each non-claim auditable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NegativeCapability {
    DoesNotClaimAllIanaReleasesWithoutAdmission,
    DoesNotClaimArbitraryTzifRoundtrip,
    DoesNotClaimFullToctouResistance,
    DoesNotClaimFutureCivilTimeAuthority,
    /// zic-rs emits **discrete** TZif leap-second records; it never implements leap *smearing*.
    DoesNotClaimLeapSmearSemantics,
    /// The interaction of range truncation (`-r`) with leap-expiry has **no semantic witness** and is
    /// not claimed (the `Rolling`-leap-under-`-r` case is a hard error, not a parity claim).
    DoesNotClaimRangeTruncationLeapExpiryInteractionParityWithoutWitness,
    DoesNotClaimReportAuthenticityWithoutSignatureOrReproducibleContext,
    DoesNotClaimTzifValidatorAsSecuritySandbox,
    DoesNotClaimUnadmittedVendorParity,
    DoesNotCurateTimeOrDefineDisplayNames,
    /// TZif is big-endian on disk; zic-rs writes it big-endian regardless of host endianness.
    DoesNotDependOnHostEndianness,
    DoesNotInferDataformFromContent,
    DoesNotInferSourceVariantFromOutputShape,
    DoesNotRequireManifestToReadTzif,
    /// The core repo admits vendor-oracle *receipts*; it does not run/ship QEMU/VM labs (T16.5).
    DoesNotShipOrOperateVendorQemuLabsInCoreRepo,
    DoesNotTreatManifestAsTzifSemantics,
}

impl NegativeCapability {
    /// The stable snake_case identifier (the report's `capability` field).
    pub fn as_str(self) -> &'static str {
        use NegativeCapability::*;
        match self {
            DoesNotClaimAllIanaReleasesWithoutAdmission => {
                "does_not_claim_all_iana_releases_without_admission"
            }
            DoesNotClaimArbitraryTzifRoundtrip => "does_not_claim_arbitrary_tzif_roundtrip",
            DoesNotClaimFullToctouResistance => "does_not_claim_full_toctou_resistance",
            DoesNotClaimFutureCivilTimeAuthority => "does_not_claim_future_civil_time_authority",
            DoesNotClaimLeapSmearSemantics => "does_not_claim_leap_smear_semantics",
            DoesNotClaimRangeTruncationLeapExpiryInteractionParityWithoutWitness => {
                "does_not_claim_range_truncation_leap_expiry_interaction_parity_without_witness"
            }
            DoesNotClaimReportAuthenticityWithoutSignatureOrReproducibleContext => {
                "does_not_claim_report_authenticity_without_signature_or_reproducible_context"
            }
            DoesNotClaimTzifValidatorAsSecuritySandbox => {
                "does_not_claim_tzif_validator_as_security_sandbox"
            }
            DoesNotClaimUnadmittedVendorParity => "does_not_claim_unadmitted_vendor_parity",
            DoesNotCurateTimeOrDefineDisplayNames => "does_not_curate_time_or_define_display_names",
            DoesNotDependOnHostEndianness => "does_not_depend_on_host_endianness",
            DoesNotInferDataformFromContent => "does_not_infer_dataform_from_content",
            DoesNotInferSourceVariantFromOutputShape => {
                "does_not_infer_source_variant_from_output_shape"
            }
            DoesNotRequireManifestToReadTzif => "does_not_require_manifest_to_read_tzif",
            DoesNotShipOrOperateVendorQemuLabsInCoreRepo => {
                "does_not_ship_or_operate_vendor_qemu_labs_in_core_repo"
            }
            DoesNotTreatManifestAsTzifSemantics => "does_not_treat_manifest_as_tzif_semantics",
        }
    }

    /// The guard/test/receipt that **enforces** this non-claim (never empty — a non-claim without an
    /// enforcing reference would be decorative, which T15.2 forbids).
    pub fn enforced_by(self) -> &'static str {
        use NegativeCapability::*;
        match self {
            DoesNotClaimAllIanaReleasesWithoutAdmission => {
                "T12.5a.3 release-admission matrix (only 2026b admitted)"
            }
            DoesNotClaimArbitraryTzifRoundtrip => {
                "T15.4 tzif/rfc9636 (a validator/reader is not a round-trip preservation claim)"
            }
            DoesNotClaimFullToctouResistance => {
                "T14.6 hostile-output-tree ledger (RequiresOpenatStyleHardening)"
            }
            DoesNotClaimFutureCivilTimeAuthority => {
                "docs/tzdb-governance.md + RFC 9557 (tzdb predicts; named-tz rules change; not a legal oracle)"
            }
            DoesNotClaimLeapSmearSemantics => {
                "T11 emits discrete TZif leap-second records (compile::apply_leaps / LeapRecord); no smearing path exists"
            }
            DoesNotClaimRangeTruncationLeapExpiryInteractionParityWithoutWitness => {
                "T11.4 — Rolling-leap-under-`-r` is a hard error (compile/leap.rs); the -r×leap-expiry interaction has no semantic witness"
            }
            DoesNotClaimReportAuthenticityWithoutSignatureOrReproducibleContext => {
                "T15.5 ConformanceStatus.report_provenance (default unsigned_local_report — not an attestation)"
            }
            DoesNotClaimTzifValidatorAsSecuritySandbox => {
                "T15.4 tzif/rfc9636 non-claim (bounds-safe, but not a hardened sandbox for hostile binaries)"
            }
            DoesNotClaimUnadmittedVendorParity => {
                "T13 reference-platform diagnostic matrix (only upstream_iana_2026b admitted)"
            }
            DoesNotCurateTimeOrDefineDisplayNames => {
                "docs/tzdb-governance.md (IANA/CLDR boundary; not zic-rs's role)"
            }
            DoesNotDependOnHostEndianness => {
                "tzif/header.rs + data writers emit big-endian fixed-width fields (to_be_bytes); byte-identical Etc/UTC fixture pins it"
            }
            DoesNotInferDataformFromContent => {
                "T12.5d test (negative-SAVE is not vanguard; hash-backed only)"
            }
            DoesNotInferSourceVariantFromOutputShape => {
                "T12.5 source_variants_not_inferred_* tests"
            }
            DoesNotRequireManifestToReadTzif => {
                "RFC 9636 (a TZif reader needs only the emitted bytes; manifest is a sidecar)"
            }
            DoesNotShipOrOperateVendorQemuLabsInCoreRepo => {
                "T16.5 vendor_oracle — core defines/admits receipts only; no VM images/QEMU orchestration vendored"
            }
            DoesNotTreatManifestAsTzifSemantics => {
                "reports/t12-close-receipt.md §5 (manifest is provenance, not TZif semantics)"
            }
        }
    }
}

/// The **evidence category** of an artifact — the T12 doctrine spine, made a typed report field (T15.3).
/// This is the typed guardrail the `zone.tab`-is-policy-not-compile error (T12.5c) earned: a claim-bearing
/// artifact must declare which category it belongs to, so input/policy/reference/generated/output kinds
/// can never be silently conflated. **The rule: no claim-bearing artifact enters a report without a
/// category owner.** (`semantic_witness` and `structural_validation` are distinct *output-evidence*
/// categories — a semantic witness proves selected behaviour under an oracle, NOT RFC 9636 structural
/// validity, which is `structural_validation` / T15.4.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactCategory {
    CompileInput,
    PolicyInput,
    ReferenceInput,
    GeneratedArtifact,
    OutputArtifact,
    DiagnosticArtifact,
    SemanticWitnessArtifact,
    StructuralValidationArtifact,
    /// Non-compiling prose that is *policy* evidence (e.g. `theory.html` / Makefile knobs / NEWS).
    PolicyProse,
    /// Release-note evidence (e.g. tzdb NEWS entries) consulted for release-delta review.
    ReleaseNoteEvidence,
}

impl ArtifactCategory {
    /// The stable snake_case identifier rendered in reports.
    pub fn as_str(self) -> &'static str {
        use ArtifactCategory::*;
        match self {
            CompileInput => "compile_input",
            PolicyInput => "policy_input",
            ReferenceInput => "reference_input",
            GeneratedArtifact => "generated_artifact",
            OutputArtifact => "output_artifact",
            DiagnosticArtifact => "diagnostic_artifact",
            SemanticWitnessArtifact => "semantic_witness_artifact",
            StructuralValidationArtifact => "structural_validation_artifact",
            PolicyProse => "policy_prose",
            ReleaseNoteEvidence => "release_note_evidence",
        }
    }
}

/// The **report kind** — so a reader never confuses a compile-coverage `support-report` with a
/// structural validation or a behaviour witness (each proves a different claim). (T15.5)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportKind {
    Support,
    Structural,
    Manifest,
    SemanticWitness,
    TzifValidation,
}

impl ReportKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ReportKind::Support => "support",
            ReportKind::Structural => "structural",
            ReportKind::Manifest => "manifest",
            ReportKind::SemanticWitness => "semantic_witness",
            ReportKind::TzifValidation => "tzif_validation",
        }
    }
}

/// A **bounded** conformance level (T15.5). It reflects *scope, not ambition* — there is deliberately no
/// `compatible` / `conformant: true`. A standalone `support-report` establishes compile-coverage over an
/// admitted release; the behaviour / structural / diagnostic axes are *separate surfaces* (this points to
/// them, it does not roll their results into a single global verdict).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConformanceLevel {
    NotEvaluated,
    /// What `support-report` alone establishes: the admitted release's zones compile; **no oracle ran here**.
    ReleaseAdmittedCompileCoverage,
    StructurallyValidatedOnly,
    SemanticWitnessedOnly,
    KnownDivergencePresent,
    OracleUnavailable,
}

impl ConformanceLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            ConformanceLevel::NotEvaluated => "not_evaluated",
            ConformanceLevel::ReleaseAdmittedCompileCoverage => "release_admitted_compile_coverage",
            ConformanceLevel::StructurallyValidatedOnly => "structurally_validated_only",
            ConformanceLevel::SemanticWitnessedOnly => "semantic_witnessed_only",
            ConformanceLevel::KnownDivergencePresent => "known_divergence_present",
            ConformanceLevel::OracleUnavailable => "oracle_unavailable",
        }
    }
}

/// Whether the workspace that produced the report was clean (T15.5). Honest by default: without a git
/// tree (this project ships from an archive, not a checked-out repo, and has **no `build.rs`** to capture
/// VCS state) this is `Unknown` — never fabricated as clean.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceProvenance {
    CleanGitTree,
    DirtyGitTree,
    SourceArchive,
    Unknown,
}

/// The authenticity status of the report artifact itself (T15.5 — *a public report is a claim surface,
/// not an unexamined trust root*). Default is an unsigned local report: useful, but not an attestation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportProvenance {
    UnsignedLocalReport,
    ReproducibleCiArtifact,
    SignedReleaseArtifact,
}

/// Which tool build produced the output (T15.5). `rustc`/`git_commit`/full target-triple are **honestly
/// `unknown`** here because the project deliberately has no `build.rs` to capture them — disclosed, not
/// faked. `zic_rs_version` is the crate version; `target` is an `arch-os` approximation; `profile` is
/// debug/release.
#[derive(Debug, Clone)]
pub struct CompilerIdentity {
    pub zic_rs_version: &'static str,
    pub rustc: Option<&'static str>,
    pub target: String,
    pub profile: &'static str,
    pub git_commit: Option<&'static str>,
}

impl CompilerIdentity {
    pub fn capture() -> Self {
        CompilerIdentity {
            zic_rs_version: env!("CARGO_PKG_VERSION"),
            // No `build.rs` → these are not captured at build time; honestly `None`, never invented.
            rustc: option_env!("ZIC_RS_RUSTC_VERSION"),
            target: format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
            profile: if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            },
            git_commit: option_env!("ZIC_RS_GIT_COMMIT"),
        }
    }
}

/// The release-admission pin gate as a **type** (T15.5-remainder) rather than the bare
/// `SOURCE_VARIANT_GATE_STATUS` string. It renders the *same* literal at the JSON boundary (so no schema
/// churn), but the vocabulary is now exhaustive and totality-tested — a drift test pins
/// `current().as_str() == SOURCE_VARIANT_GATE_STATUS`. `Open` = no release admitted; `LiftedFor2026b` =
/// the single 2026b release is admitted (signature-verified + hash-pinned, per T12.5a.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferencePinGate {
    Open,
    LiftedFor2026b,
}

impl ReferencePinGate {
    pub fn as_str(self) -> &'static str {
        match self {
            ReferencePinGate::Open => "open",
            ReferencePinGate::LiftedFor2026b => "lifted_for_2026b",
        }
    }
    /// The gate state as currently shipped — single-sourced against `SOURCE_VARIANT_GATE_STATUS`.
    pub fn current() -> Self {
        ReferencePinGate::LiftedFor2026b
    }
}

/// **Where an admitted reference came from** (T16.3) — the "*which* reference?" question's *location*
/// half. The central rule: **only a `VersionedArchive` (a release tarball you can re-fetch and re-pin)
/// can back a *sealed* release claim**; the others support exploration/diagnosis but not a sealed claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceLocatorKind {
    /// A pinned, re-fetchable release archive (e.g. the T12.5a.2 `tzdb-2026b.tar.lz`). Sealed-claim grade.
    VersionedArchive,
    /// Whatever `zic`/`zdump` is on `PATH` right now — moves under your feet; exploration only.
    LiveCurrentDirectory,
    /// A local cached copy of bytes (integrity depends on how it was pinned).
    LocalCachedCopy,
    /// A distribution's source package (a patch-stack over upstream; distinct provenance).
    DistroSourcePackage,
    /// Provenance not established.
    Unknown,
}

impl ReferenceLocatorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ReferenceLocatorKind::VersionedArchive => "versioned_archive",
            ReferenceLocatorKind::LiveCurrentDirectory => "live_current_directory",
            ReferenceLocatorKind::LocalCachedCopy => "local_cached_copy",
            ReferenceLocatorKind::DistroSourcePackage => "distro_source_package",
            ReferenceLocatorKind::Unknown => "unknown",
        }
    }
}

/// **How an admitted reference is trusted** (T16.3) — the "*which* reference?" question's *trust* half,
/// kept precise so a reader knows *what kind* of trust they are getting. Crucially `HashOnly` proves
/// **integrity** (the bytes are what we pinned) but **not authenticity** (who produced them); it is never
/// rendered as "signature verified". `FingerprintAnchored` (the T12.5a.2 model — an OpenPGP signature
/// verified against a published key *fingerprint*) is **not** the weaker `WebOfTrustValidated`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureTrustModel {
    /// OpenPGP signature verified against a published key fingerprint (authenticity + integrity).
    FingerprintAnchored,
    /// Trust via a web-of-trust path (weaker than fingerprint-anchored; not claimed unless real).
    WebOfTrustValidated,
    /// Trust via an OS/platform keyring.
    PlatformKeyring,
    /// A content hash only — **integrity, not authenticity**; never "signature verified".
    HashOnly,
    /// Explicitly unsigned.
    Unsigned,
    /// Trust model not established.
    Unknown,
}

impl SignatureTrustModel {
    pub fn as_str(self) -> &'static str {
        match self {
            SignatureTrustModel::FingerprintAnchored => "fingerprint_anchored",
            SignatureTrustModel::WebOfTrustValidated => "web_of_trust_validated",
            SignatureTrustModel::PlatformKeyring => "platform_keyring",
            SignatureTrustModel::HashOnly => "hash_only",
            SignatureTrustModel::Unsigned => "unsigned",
            SignatureTrustModel::Unknown => "unknown",
        }
    }
    /// Whether this trust model **pins integrity** (the bytes are what we expect). `HashOnly` qualifies
    /// (integrity without authenticity); `Unsigned`/`Unknown` do not. Authenticity is a *separate* axis —
    /// see `FingerprintAnchored`.
    pub fn pins_integrity(self) -> bool {
        matches!(
            self,
            SignatureTrustModel::FingerprintAnchored
                | SignatureTrustModel::WebOfTrustValidated
                | SignatureTrustModel::PlatformKeyring
                | SignatureTrustModel::HashOnly
        )
    }
}

/// A reference's admission evidence (T16.3): *where it came from* × *how it is trusted*. The sealed-claim
/// rule is enforced here, not in prose: a claim may be *sealed* (re-verifiable, release-grade) **only** if
/// the locator is a `VersionedArchive` **and** the trust model pins integrity.
#[derive(Debug, Clone, Copy)]
pub struct ReferenceAdmission {
    pub locator: ReferenceLocatorKind,
    pub trust: SignatureTrustModel,
}

impl ReferenceAdmission {
    /// Only a versioned archive with integrity-pinned trust can back a sealed release claim. A live
    /// PATH binary, a distro package, or any unsigned/unknown-trust material is exploration-grade only.
    pub fn supports_sealed_claim(&self) -> bool {
        matches!(self.locator, ReferenceLocatorKind::VersionedArchive)
            && self.trust.pins_integrity()
    }
    pub fn to_json(&self) -> String {
        format!(
            "{{ \"locator\": {}, \"signature_trust\": {}, \"supports_sealed_claim\": {} }}",
            json_str(self.locator.as_str()),
            json_str(self.trust.as_str()),
            self.supports_sealed_claim()
        )
    }
}

/// The T12.5a.2 admitted 2026b reference: a **versioned archive** (`tzdb-2026b.tar.lz`), OpenPGP signature
/// verified against the published tz key **fingerprint** + SHA-256 hash-pinned. The one reference today
/// that backs a *sealed* claim. (Distinct from the *live* PATH `zic` a report's oracle runs against.)
pub const ADMITTED_2026B_REFERENCE: ReferenceAdmission = ReferenceAdmission {
    locator: ReferenceLocatorKind::VersionedArchive,
    trust: SignatureTrustModel::FingerprintAnchored,
};

/// The dimension a claim is portable **along** (T15.5-remainder) — i.e. what it stays true *under*. A
/// claim is never "globally true": it is true *for* a declared release / oracle / platform / profile /
/// fixture set, or it is a general project policy. This makes "true where?" a typed field, not prose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimPortability {
    ReleaseSpecific,
    OracleSpecific,
    PlatformSpecific,
    ProfileSpecific,
    FixtureSpecific,
    GeneralProjectPolicy,
}

impl ClaimPortability {
    pub fn as_str(self) -> &'static str {
        match self {
            ClaimPortability::ReleaseSpecific => "release_specific",
            ClaimPortability::OracleSpecific => "oracle_specific",
            ClaimPortability::PlatformSpecific => "platform_specific",
            ClaimPortability::ProfileSpecific => "profile_specific",
            ClaimPortability::FixtureSpecific => "fixture_specific",
            ClaimPortability::GeneralProjectPolicy => "general_project_policy",
        }
    }
}

/// The **kind of authority** a claim's evidence carries (T15.5-remainder) — orthogonal to whether the
/// claim is true; it says *what backs it*, so a reviewer can tell a normative-spec citation from an
/// implementation observation from project doctrine. (`NormativeSpec` = RFC 9636 TZif format;
/// `PolicyGuidance` = BCP 175 / tzdb *process*, not format — the two never blur.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceAuthorityKind {
    NormativeSpec,
    ImplementationObservation,
    ManpageDocumentation,
    PolicyGuidance,
    ReleaseNote,
    EmpiricalFixture,
    ProjectDoctrine,
}

impl EvidenceAuthorityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            EvidenceAuthorityKind::NormativeSpec => "normative_spec",
            EvidenceAuthorityKind::ImplementationObservation => "implementation_observation",
            EvidenceAuthorityKind::ManpageDocumentation => "manpage_documentation",
            EvidenceAuthorityKind::PolicyGuidance => "policy_guidance",
            EvidenceAuthorityKind::ReleaseNote => "release_note",
            EvidenceAuthorityKind::EmpiricalFixture => "empirical_fixture",
            EvidenceAuthorityKind::ProjectDoctrine => "project_doctrine",
        }
    }
}

/// What a report's claim **proves**, does **not** prove, and **depends on** (T15.5-remainder) — the
/// compact, machine-readable form of the non-claim doctrine, attached to the rollup. Static for a given
/// report kind (the boundary is fixed by what the surface actually measures).
#[derive(Debug, Clone, Copy)]
pub struct ClaimBoundary {
    pub proves: &'static str,
    pub does_not_prove: &'static str,
    pub depends_on: &'static str,
}

/// The distinct senses of "valid" the conformance engine keeps **impossible to blur** (T15.5-remainder /
/// T15.close). Emitted as a report field so "valid" can never be read as a single global verdict: each
/// entry is `<sense>: <what it is> — NOT <what it is not>`. The live behaviour claim (CORE.1) is the last
/// entry and is deliberately separate from structural / reader / release-admission validity.
pub const VALID_DISAMBIGUATION: &[&str] = &[
    "structurally_valid: RFC 9636 byte-format integrity (tzif-validate) — NOT behaviour or semantics",
    "semantically_witness_matching: offset/is_dst/abbr match zdump for the declared witness set — NOT all instants",
    "modern_reader_compatible: no v4/legacy reader hazards — NOT semantic correctness",
    "future_projection_matching: the POSIX footer projects like reference — separate from footer parseability",
    "release_admitted: the source release is signature-verified + hash-pinned — NOT all IANA releases",
    "compile_covered: the admitted release's zones compile — NOT behaviour-matched",
    "behaviour_matched: CORE.1 341/341 vs reference zic/zdump over 1900..2040 — the live claim, separate from all above",
];

/// The one-line, **machine-readable** conformance rollup (T15.5) — the claim *envelope*, so a reviewer
/// gets the scope in one scan without reconstructing the whole ladder. It is a pointer-rich summary, not
/// a global pass/fail: it names the admitted release, the bounded level for *this* report kind, the
/// available proof surfaces, the report's own provenance, and a `declared_scope_hash` that changes
/// whenever any scope element changes. T15.5-remainder added the typed claim-shape axes
/// (`reference_pin_gate` · `claim_portability` · `evidence_authority` · `claim_boundary`) and the
/// `valid_disambiguation`, so the *shape* of the claim is as machine-readable as its result.
#[derive(Debug, Clone)]
pub struct ConformanceStatus {
    pub report_kind: ReportKind,
    pub level: ConformanceLevel,
    pub workspace: WorkspaceProvenance,
    pub report_provenance: ReportProvenance,
    pub compiler: CompilerIdentity,
    pub reference_pin_gate: ReferencePinGate,
    pub claim_portability: ClaimPortability,
    pub evidence_authority: EvidenceAuthorityKind,
    pub claim_boundary: ClaimBoundary,
}

impl ConformanceStatus {
    /// The rollup for a `support-report` invocation (compile-coverage over the admitted release).
    pub fn support() -> Self {
        ConformanceStatus {
            report_kind: ReportKind::Support,
            level: ConformanceLevel::ReleaseAdmittedCompileCoverage,
            // No git tree / no build.rs here → honest Unknown.
            workspace: WorkspaceProvenance::Unknown,
            report_provenance: ReportProvenance::UnsignedLocalReport,
            compiler: CompilerIdentity::capture(),
            reference_pin_gate: ReferencePinGate::current(),
            // support-report's claim is about the admitted *release*; it is an observation of zic-rs's own
            // compile, not a normative-spec or oracle claim.
            claim_portability: ClaimPortability::ReleaseSpecific,
            evidence_authority: EvidenceAuthorityKind::ImplementationObservation,
            claim_boundary: ClaimBoundary {
                proves: "the admitted release's zones compile (compile-coverage), each accounted in exactly one bucket",
                does_not_prove: "behaviour / structural / reader-compatibility parity — those are separate surfaces (semantic-report · structural-report · tzif-validate)",
                depends_on: "the signature-verified + hash-pinned 2026b reference set and this zic-rs build",
            },
        }
    }

    /// `declared_scope_hash` — a SHA-256 over the **claim envelope**: admitted-release gate · manifest +
    /// report schema versions · the sorted negative-capability ids · the CORE.1 claim string. If any
    /// scope element changes, the hash changes — a compact identifier reviewers can pin a claim to.
    pub fn declared_scope_hash(&self) -> String {
        let mut envelope = String::new();
        envelope.push_str(SOURCE_VARIANT_GATE_STATUS);
        envelope.push('|');
        envelope.push_str(COMPILE_SCHEMA);
        envelope.push_str("|zic-rs-support-report-v4|zic-rs-structural-report-v3");
        envelope.push_str("|zic-rs-semantic-report-v1|zic-rs-tzif-validation-v1|");
        for nc in NEGATIVE_CAPABILITIES {
            envelope.push_str(nc.as_str());
            envelope.push(',');
        }
        envelope.push_str("|CORE.1=341/341@1900..2040;0mismatch;0failclosed");
        crate::hash::sha256_hex(envelope.as_bytes())
    }

    /// Render the `conformance_status` block (a comma-terminated object for insertion into a report).
    pub fn to_json_block(&self) -> String {
        let opt = |o: Option<&str>| match o {
            Some(v) => json_str(v),
            None => "null".to_string(),
        };
        // The valid-disambiguation array, rendered from the static const so the senses stay single-sourced.
        let mut valid_disambig = String::from("[");
        for (i, sense) in VALID_DISAMBIGUATION.iter().enumerate() {
            if i > 0 {
                valid_disambig.push_str(", ");
            }
            valid_disambig.push_str(&json_str(sense));
        }
        valid_disambig.push(']');
        format!(
            "  \"conformance_status\": {{\n\
             \"report_kind\": {}, \"conformance_level\": {}, \"declared_scope_hash\": {}, \
             \"admitted_release_gate\": {}, \"workspace_provenance\": {}, \"report_provenance\": {}, \
             \"claim_portability\": {}, \"evidence_authority\": {}, \
             \"claim_boundary\": {{ \"proves\": {}, \"does_not_prove\": {}, \"depends_on\": {} }}, \
             \"valid_disambiguation\": {}, \
             \"core1_claim\": {}, \
             \"available_surfaces\": [\"support-report\", \"structural-report\", \"semantic-report\", \
             \"tzif-validation\", \"compile-manifest\"], \
             \"compiler_identity\": {{ \"zic_rs_version\": {}, \"rustc\": {}, \"target\": {}, \
             \"profile\": {}, \"git_commit\": {} }} }},\n",
            json_str(self.report_kind.as_str()),
            json_str(self.level.as_str()),
            json_str(&self.declared_scope_hash()),
            json_str(self.reference_pin_gate.as_str()),
            json_str(match self.workspace {
                WorkspaceProvenance::CleanGitTree => "clean_git_tree",
                WorkspaceProvenance::DirtyGitTree => "dirty_git_tree",
                WorkspaceProvenance::SourceArchive => "source_archive",
                WorkspaceProvenance::Unknown => "unknown",
            }),
            json_str(match self.report_provenance {
                ReportProvenance::UnsignedLocalReport => "unsigned_local_report",
                ReportProvenance::ReproducibleCiArtifact => "reproducible_ci_artifact",
                ReportProvenance::SignedReleaseArtifact => "signed_release_artifact",
            }),
            json_str(self.claim_portability.as_str()),
            json_str(self.evidence_authority.as_str()),
            json_str(self.claim_boundary.proves),
            json_str(self.claim_boundary.does_not_prove),
            json_str(self.claim_boundary.depends_on),
            valid_disambig,
            json_str(
                "341/341 canonical zones behaviour-match reference zic/zdump over 1900..2040 \
                 (0 mismatch, 0 fail-closed)"
            ),
            json_str(self.compiler.zic_rs_version),
            opt(self.compiler.rustc),
            json_str(&self.compiler.target),
            json_str(self.compiler.profile),
            opt(self.compiler.git_commit),
        )
    }
}

/// The canonical, **sorted-by-`as_str()`** non-claims list surfaced in every report's provenance block.
/// Sorted so the emitted JSON array is deterministic; the order is asserted by a test.
pub const NEGATIVE_CAPABILITIES: &[NegativeCapability] = &[
    NegativeCapability::DoesNotClaimAllIanaReleasesWithoutAdmission,
    NegativeCapability::DoesNotClaimArbitraryTzifRoundtrip,
    NegativeCapability::DoesNotClaimFullToctouResistance,
    NegativeCapability::DoesNotClaimFutureCivilTimeAuthority,
    NegativeCapability::DoesNotClaimLeapSmearSemantics,
    NegativeCapability::DoesNotClaimRangeTruncationLeapExpiryInteractionParityWithoutWitness,
    NegativeCapability::DoesNotClaimReportAuthenticityWithoutSignatureOrReproducibleContext,
    NegativeCapability::DoesNotClaimTzifValidatorAsSecuritySandbox,
    NegativeCapability::DoesNotClaimUnadmittedVendorParity,
    NegativeCapability::DoesNotCurateTimeOrDefineDisplayNames,
    NegativeCapability::DoesNotDependOnHostEndianness,
    NegativeCapability::DoesNotInferDataformFromContent,
    NegativeCapability::DoesNotInferSourceVariantFromOutputShape,
    NegativeCapability::DoesNotRequireManifestToReadTzif,
    NegativeCapability::DoesNotShipOrOperateVendorQemuLabsInCoreRepo,
    NegativeCapability::DoesNotTreatManifestAsTzifSemantics,
];

/// Whether any backzone/PACKRATLIST/DATAFORM/rearguard/vanguard *behaviour* is implemented. Still
/// **false** — the gate lift (T12.5a.2) only *admitted the reference*; implementation begins at T12.5b.
pub const SOURCE_VARIANT_BEHAVIOR_IMPLEMENTED: bool = false;

/// Substeps still blocked by the gate. Empty since T12.5a.2 lifted it for 2026b — T12.5b–d are
/// unblocked (but not yet implemented; see [`SOURCE_VARIANT_BEHAVIOR_IMPLEMENTED`]).
pub const SOURCE_VARIANT_BLOCKED_SUBSTEPS: &[&str] = &[];

/// Required upstream tzdb reference files still **unpinned**. Empty since T12.5a.2 admitted +
/// SHA-256-pinned the full 2026b set (hashes in `reports/t12_5a2-reference-admission.md`).
pub const SOURCE_VARIANT_UNPINNED_FILES: &[&str] = &[];

/// The provenance/capability statement as a deterministic JSON object block (key `"provenance"`),
/// 2-space-indented and **comma-terminated** for insertion right after a report's `"schema"` line.
/// Shared by both reports so the trust state is identical and single-sourced.
pub fn provenance_block_json() -> String {
    let arr = |items: &[&str]| -> String {
        let inner: Vec<String> = items.iter().map(|i| json_str(i)).collect();
        format!("[{}]", inner.join(", "))
    };
    let mut s = String::new();
    s.push_str("  \"provenance\": {\n");
    s.push_str(&format!(
        "    \"manifest_schema\": {},\n",
        json_str(COMPILE_SCHEMA)
    ));
    s.push_str(
        "    \"per_run_profile\": \"see `compile --manifest`: build_profile / source_inputs / \
         link_profile / source_profile.backward_evidence\",\n",
    );
    s.push_str(&format!(
        "    \"source_variant_reference_pin_gate\": {},\n",
        json_str(SOURCE_VARIANT_GATE_STATUS)
    ));
    s.push_str(&format!(
        "    \"blocked_substeps\": {},\n",
        arr(SOURCE_VARIANT_BLOCKED_SUBSTEPS)
    ));
    s.push_str(&format!(
        "    \"unpinned_required_files\": {},\n",
        arr(SOURCE_VARIANT_UNPINNED_FILES)
    ));
    s.push_str(&format!(
        "    \"source_variant_behavior_implemented\": {},\n",
        SOURCE_VARIANT_BEHAVIOR_IMPLEMENTED
    ));
    s.push_str(
        "    \"note\": \"tzdb 2026b reference set admitted + signature-verified + SHA-256-pinned \
         (reports/t12_5a2-reference-admission.md); T12.5b–d source-variant **evidence axes** are \
         implemented for that pinned reference, while source-variant **behaviour** remains not \
         implemented or claimed. No backzone/PACKRATLIST/DATAFORM/rearguard/vanguard behaviour is \
         claimed; never inferred from aliases, filenames, link counts, or output byte shape.\",\n",
    );
    // T15.2 — `negative_capabilities`: the project's non-claims as a first-class, machine-visible array,
    // each tied to the guard/test/receipt that enforces it (never decorative). Sorted + deterministic.
    s.push_str("    \"negative_capabilities\": [");
    for (i, nc) in NEGATIVE_CAPABILITIES.iter().enumerate() {
        s.push_str(if i == 0 { "\n" } else { ",\n" });
        s.push_str(&format!(
            "      {{ \"capability\": {}, \"enforced_by\": {} }}",
            json_str(nc.as_str()),
            json_str(nc.enforced_by())
        ));
    }
    s.push_str("\n    ]\n");
    s.push_str("  },\n");
    s
}

/// The provenance/capability statement as a human-readable text block, appended to a report's text
/// output. Mirrors [`provenance_block_json`].
pub fn provenance_block_text() -> String {
    let mut s = String::new();
    s.push_str("\nprovenance / capability:\n");
    s.push_str(&format!(
        "  manifest schema: {COMPILE_SCHEMA}  (per-run build/source/link/backward profile: see \
         `compile --manifest`)\n"
    ));
    s.push_str(&format!(
        "  source-variant reference-pin gate: {SOURCE_VARIANT_GATE_STATUS}  (tzdb 2026b admitted + \
         signature-verified + SHA-256-pinned — reports/t12_5a2-reference-admission.md)\n"
    ));
    s.push_str(&format!(
        "  source-variant behaviour: {} — T12.5b–d unblocked for the pinned reference but not yet \
         implemented; backzone/PACKRATLIST/DATAFORM/rearguard/vanguard never inferred from \
         aliases/filenames/link counts/output shape\n",
        if SOURCE_VARIANT_BEHAVIOR_IMPLEMENTED {
            "implemented"
        } else {
            "NOT implemented or claimed"
        }
    ));
    s.push_str("  negative capabilities (non-claims, each enforced):\n");
    for nc in NEGATIVE_CAPABILITIES {
        s.push_str(&format!("    - {}  ({})\n", nc.as_str(), nc.enforced_by()));
    }
    s
}

/// The oracle result for *this* invocation. A bare `compile` never runs the oracle, so it is
/// recorded as `not-run` — the manifest must not infer success from the repo's test suite.
#[derive(Debug, Clone)]
pub struct OracleResult {
    /// The oracle mode, **typed** (T15.2a — was a free `String`). Rendered at the manifest boundary via
    /// [`OracleMode::manifest_str`]. The companion `result` is the verdict vocabulary (a separate axis).
    pub mode: OracleMode,
    pub horizon: Option<String>,
    /// The oracle verdict, typed (T17.2 — was a free `String`). Distinct from `mode`: *what the oracle
    /// concluded*, not *which* oracle. Rendered via [`OracleVerdict::as_str`].
    pub result: OracleVerdict,
}

impl OracleResult {
    /// The honest default for a `compile` invocation: the oracle was not run.
    pub fn not_run() -> Self {
        OracleResult {
            mode: OracleMode::NotRun,
            horizon: None,
            result: OracleVerdict::NotRun,
        }
    }
}

/// The full compile-provenance manifest.
#[derive(Debug, Clone)]
pub struct CompileManifest {
    pub zic_rs_version: String,
    pub tzdb: TzdbProvenance,
    pub source_inputs: SourceInputs,
    pub build_profile: BuildProfile,
    pub link_profile: LinkProfile,
    pub source_profile: SourceProfile,
    pub zones_requested: Vec<String>,
    pub zones_compiled: Vec<String>,
    pub links_materialized: Vec<String>,
    pub unsupported_zones: Vec<String>,
    pub oracle: OracleResult,
}

/// Render the `build_profile` block — the structured output identity of *this run* (T12.2). Fields
/// describe what was actually used; only the `DATAFORM` encoding axes `rearguard`/`vanguard` are
/// `"unknown"` here (no deterministic detector yet — kept explicit, never guessed or claimed; T12.5d).
/// Source-membership (`backward`/`backzone`/`PACKRATLIST`) lives in the `source_profile` evidence
/// axes, not here.
fn build_profile_json(p: &BuildProfile) -> String {
    let opt_at = |v: Option<i64>| match v {
        Some(n) => format!("\"@{n}\""),
        None => "null".to_string(),
    };

    let mut s = String::new();
    s.push_str("  \"build_profile\": {\n");
    s.push_str(&format!(
        "    \"output_tree\": {},\n",
        json_str(p.output_tree.as_str())
    ));
    // leap_source: describes the run, never capabilities.
    s.push_str("    \"leap_source\": {\n");
    s.push_str(&format!(
        "      \"mode\": {},\n",
        json_str(p.leap_source.mode.as_str())
    ));
    match &p.leap_source.sha256 {
        Some(h) => s.push_str(&format!("      \"sha256\": {},\n", json_str(h))),
        None => s.push_str("      \"sha256\": null,\n"),
    }
    s.push_str(&format!(
        "      \"entry_count\": {},\n",
        p.leap_source.entry_count
    ));
    s.push_str(&format!("      \"expires\": {},\n", p.leap_source.expires));
    s.push_str(&format!(
        "      \"rolling_entries\": {}\n",
        p.leap_source.rolling_entries
    ));
    s.push_str("    },\n");
    s.push_str(&format!(
        "    \"emit_style\": {},\n",
        json_str(emit_style_str(p.emit_style))
    ));
    match p.range {
        Some((lo, hi)) => s.push_str(&format!(
            "    \"range\": {{ \"lo\": {}, \"hi\": {} }},\n",
            opt_at(lo),
            opt_at(hi)
        )),
        None => s.push_str("    \"range\": null,\n"),
    }
    s.push_str(&format!(
        "    \"redundant_until\": {},\n",
        opt_at(p.redundant_until)
    ));
    // `link_mode` is the last `build_profile` field: as of T12.5d there are **no** source-variant
    // placeholders here. Every source-variant axis (`backward` T12.4d, `backzone` T12.5b, `PACKRATLIST`
    // T12.5c, `DATAFORM`=`main`/`vanguard`/`rearguard` T12.5d) is an authoritative `source_profile`
    // evidence axis; carrying an `"unknown"` copy here too would be a contradiction (`"unknown"` vs a
    // real detected/claimed status). The arc that removed the `backward` and `backzone` stubs ends here
    // by removing the last `rearguard`/`vanguard` stubs — `build_profile` now describes only *how this
    // run emitted* (tree/leap/emit/range/links), not source-set membership or encoding.
    s.push_str(&format!(
        "    \"link_mode\": {}\n",
        json_str(p.link_mode.as_str())
    ));
    s.push_str("  },\n");
    s
}

/// Render the `source_inputs` block — the deterministic *input identity* of this run (T12.3): the
/// structural `kind`, the **input-ordered** file list (logical name + content hash + size +
/// `order_index`), and the order-sensitive `aggregate_hash`. Portable: logical names, never
/// machine-local absolute paths.
fn source_inputs_json(si: &SourceInputs) -> String {
    let mut s = String::new();
    s.push_str("  \"source_inputs\": {\n");
    s.push_str(&format!("    \"kind\": {},\n", json_str(si.kind.as_str())));
    s.push_str("    \"files\": [");
    for (i, f) in si.files.iter().enumerate() {
        s.push_str(if i == 0 { "\n" } else { ",\n" });
        s.push_str(&format!(
            "      {{ \"order_index\": {}, \"logical_name\": {}, \"sha256\": {}, \"bytes\": {} }}",
            f.order_index,
            json_str(&f.logical_name),
            json_str(&f.sha256),
            f.bytes
        ));
    }
    s.push_str(if si.files.is_empty() {
        "],\n"
    } else {
        "\n    ],\n"
    });
    s.push_str(&format!(
        "    \"aggregate_hash\": {}\n",
        json_str(&si.aggregate_hash)
    ));
    s.push_str("  },\n");
    s
}

/// Render the `link_profile` block — link/alias identity (T12.4b): counts, policy, and the stable
/// hashes that bind the build to its `alias-map.json`. Never asserts source-set membership.
fn link_profile_json(lp: &LinkProfile) -> String {
    let mut s = String::new();
    s.push_str("  \"link_profile\": {\n");
    s.push_str(&format!(
        "    \"link_policy\": {},\n",
        json_str(&lp.link_policy)
    ));
    s.push_str(&format!(
        "    \"zones_compiled_count\": {},\n",
        lp.zones_compiled_count
    ));
    s.push_str(&format!(
        "    \"links_selected_count\": {},\n",
        lp.links_selected_count
    ));
    s.push_str(&format!(
        "    \"links_materialized_count\": {},\n",
        lp.links_materialized_count
    ));
    s.push_str(&format!(
        "    \"links_omitted_count\": {},\n",
        lp.links_omitted_count
    ));
    s.push_str(&format!(
        "    \"links_failed_count\": {},\n",
        lp.links_failed_count
    ));
    s.push_str(&format!(
        "    \"alias_map_sha256\": {},\n",
        json_str(&lp.alias_map_sha256)
    ));
    s.push_str(&format!(
        "    \"selected_links_sha256\": {},\n",
        json_str(&lp.selected_links_sha256)
    ));
    s.push_str(&format!(
        "    \"omitted_links_sha256\": {}\n",
        json_str(&lp.omitted_links_sha256)
    ));
    s.push_str("  },\n");
    s
}

/// Render the `source_profile` block — the source-evidence axes (T12.4d `backward`, T12.5b `backzone`,
/// T12.5c `packratlist` backzone-scope); an extension seam for `DATAFORM` later. Records detected vs
/// claimed vs reconciled `status` + the admitted `evidence_sha256` — never a boolean, never inferred.
fn source_profile_json(sp: &SourceProfile) -> String {
    // Both axes share the {detected, claimed, status, evidence_sha256} shape; render with one helper.
    let axis =
        |key: &str, detected: &str, claimed: &str, status: &str, ev: &Option<String>| -> String {
            let mut a = String::new();
            a.push_str(&format!("    {}: {{\n", json_str(key)));
            a.push_str(&format!("      \"detected\": {},\n", json_str(detected)));
            a.push_str(&format!("      \"claimed\": {},\n", json_str(claimed)));
            a.push_str(&format!("      \"status\": {},\n", json_str(status)));
            match ev {
                Some(h) => a.push_str(&format!("      \"evidence_sha256\": {}\n", json_str(h))),
                None => a.push_str("      \"evidence_sha256\": null\n"),
            }
            a.push_str("    }");
            a
        };
    let b = &sp.backward;
    let z = &sp.backzone;
    let mut s = String::new();
    s.push_str("  \"source_profile\": {\n");
    s.push_str(&axis(
        "backward_evidence",
        b.detected_str(),
        b.claimed_str(),
        b.status(),
        &b.evidence_sha256,
    ));
    s.push_str(",\n");
    s.push_str(&axis(
        "backzone_evidence",
        z.detected_str(),
        z.claimed_str(),
        z.status(),
        &z.evidence_sha256,
    ));
    s.push_str(",\n");
    let pl = &sp.packratlist;
    s.push_str(&axis(
        "packratlist_evidence",
        pl.detected_str(),
        pl.claimed_str(),
        pl.status(),
        &pl.evidence_sha256,
    ));
    s.push_str(",\n");
    // `dataform_evidence` shares the 4 standard fields but adds two generated-artifact provenance
    // fields (`recipe_hash`, `generated_from`), so it is rendered directly rather than via `axis`.
    let df = &sp.dataform;
    let opt = |v: &Option<String>| match v {
        Some(h) => json_str(h),
        None => "null".to_string(),
    };
    s.push_str("    \"dataform_evidence\": {\n");
    s.push_str(&format!(
        "      \"detected\": {},\n",
        json_str(df.detected_str())
    ));
    s.push_str(&format!(
        "      \"claimed\": {},\n",
        json_str(df.claimed_str())
    ));
    s.push_str(&format!("      \"status\": {},\n", json_str(df.status())));
    s.push_str(&format!(
        "      \"evidence_sha256\": {},\n",
        opt(&df.evidence_sha256)
    ));
    s.push_str(&format!(
        "      \"recipe_hash\": {},\n",
        opt(&df.recipe_hash)
    ));
    s.push_str(&format!(
        "      \"generated_from\": {}\n",
        opt(&df.generated_from)
    ));
    s.push_str("    }\n");
    s.push_str("  },\n");
    s
}

impl CompileManifest {
    /// Render deterministic, pretty-printed JSON.
    pub fn to_json(&self) -> String {
        let arr = |items: &[String]| -> String {
            if items.is_empty() {
                "[]".to_string()
            } else {
                let inner: Vec<String> = items.iter().map(|i| json_str(i)).collect();
                format!("[{}]", inner.join(", "))
            }
        };

        let mut s = String::new();
        s.push_str("{\n");
        s.push_str(&format!("  \"schema\": {},\n", json_str(COMPILE_SCHEMA)));
        s.push_str(&format!(
            "  \"zic_rs_version\": {},\n",
            json_str(&self.zic_rs_version)
        ));
        let opt_str = |v: &Option<String>| match v {
            Some(x) => json_str(x),
            None => "null".to_string(),
        };
        s.push_str("  \"tzdb\": {\n");
        s.push_str(&format!(
            "    \"detected_version\": {},\n",
            opt_str(&self.tzdb.detected_version)
        ));
        s.push_str(&format!(
            "    \"claimed_version\": {},\n",
            opt_str(&self.tzdb.claimed_version)
        ));
        s.push_str(&format!(
            "    \"version_status\": {},\n",
            json_str(self.tzdb.version_status())
        ));
        s.push_str(&format!(
            "    \"source_path\": {},\n",
            json_str(&self.tzdb.source_path)
        ));
        s.push_str(&format!(
            "    \"source_sha256\": {}\n",
            json_str(&self.tzdb.source_sha256)
        ));
        s.push_str("  },\n");
        s.push_str(&source_inputs_json(&self.source_inputs));
        s.push_str(&build_profile_json(&self.build_profile));
        s.push_str(&link_profile_json(&self.link_profile));
        s.push_str(&source_profile_json(&self.source_profile));
        s.push_str("  \"compile\": {\n");
        s.push_str(&format!(
            "    \"zones_requested\": {},\n",
            arr(&self.zones_requested)
        ));
        s.push_str(&format!(
            "    \"zones_compiled\": {},\n",
            arr(&self.zones_compiled)
        ));
        s.push_str(&format!(
            "    \"links_materialized\": {},\n",
            arr(&self.links_materialized)
        ));
        s.push_str(&format!(
            "    \"unsupported_zones\": {}\n",
            arr(&self.unsupported_zones)
        ));
        s.push_str("  },\n");
        s.push_str("  \"oracle\": {\n");
        s.push_str(&format!(
            "    \"mode\": {},\n",
            json_str(self.oracle.mode.manifest_str())
        ));
        match &self.oracle.horizon {
            Some(h) => s.push_str(&format!("    \"horizon\": {},\n", json_str(h))),
            None => s.push_str("    \"horizon\": null,\n"),
        }
        s.push_str(&format!(
            "    \"result\": {}\n",
            json_str(self.oracle.result.as_str())
        ));
        s.push_str("  }\n");
        s.push_str("}\n");
        s
    }

    /// Write the manifest JSON to `path`.
    pub fn write_to(&self, path: &Path) -> Result<()> {
        std::fs::write(path, self.to_json()).map_err(|e| Error::io(path, e))
    }
}

/// Build a [`CompileManifest`] from the run's inputs and report.
///
/// `requested` is the resolved list of identifiers the user asked for; `source_files` are the
/// expanded input files **in input order** (directories already expanded sorted by the caller).
/// The oracle is recorded as `not-run` because `compile` does not invoke `compare` — see the
/// module note.
///
/// Two complementary hashes are computed: `tzdb.source_sha256` over the source bytes in *sorted*
/// (canonicalized) order — an order-independent content identity — and
/// `source_inputs.aggregate_hash` over the *input-ordered* per-file hashes — an order-sensitive
/// identity. **Input order is part of the build identity**; the manifest records it faithfully.
///
/// This assembles the build identity from eight genuinely distinct provenance inputs (the requested
/// selection, the input file set, the compile report, the run config, the link database, the claimed
/// tzdb version, the leap-source path, and the source-variant claims). They do not naturally collapse
/// into a meaningful sub-struct — bundling would relocate the count, not reduce the complexity — so we
/// keep them explicit and silence the arity lint.
#[allow(clippy::too_many_arguments)]
pub fn build_compile_manifest(
    requested: &[String],
    source_files: &[std::path::PathBuf],
    report: &CompileReport,
    config: &crate::CompileConfig,
    db: &crate::model::Database,
    claimed_version: Option<&str>,
    leap_path: Option<&std::path::Path>,
    variants: &SourceVariantArgs,
) -> Result<CompileManifest> {
    // Per-file identity in INPUT ORDER (never re-sorted) — the order is part of the build identity.
    let mut input_files: Vec<SourceFile> = Vec::with_capacity(source_files.len());
    for (order_index, f) in source_files.iter().enumerate() {
        let bytes = std::fs::read(f).map_err(|e| Error::io(f, e))?;
        input_files.push(SourceFile {
            // Logical name = basename: a portable label, never the machine-local absolute path.
            logical_name: f
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| f.display().to_string()),
            sha256: sha256_hex(&bytes),
            bytes: bytes.len(),
            order_index,
        });
    }
    // Order-sensitive aggregate identity: hash the input-ordered sequence of per-file hashes (a
    // newline separator so reordering two files always changes the digest).
    let aggregate_seed = input_files
        .iter()
        .map(|f| f.sha256.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let aggregate_hash = sha256_hex(aggregate_seed.as_bytes());

    // Structural input *form* only — never a guess at source-set membership (backward/backzone).
    let kind = match source_files.len() {
        0 => SourceInputKind::Unknown,
        1 if source_files[0].extension().and_then(|e| e.to_str()) == Some("zi") => {
            SourceInputKind::TzdataZi
        }
        1 => SourceInputKind::SingleFile,
        _ => SourceInputKind::MultiFile,
    };

    // Order-independent content hash + display path + version detection: read in SORTED path order
    // so this digest is invariant to argument ordering (the explicitly-canonicalized companion to
    // `aggregate_hash`).
    let mut sorted: Vec<&std::path::PathBuf> = source_files.iter().collect();
    sorted.sort();
    let mut all = Vec::new();
    for f in &sorted {
        all.extend(std::fs::read(f).map_err(|e| Error::io(f, e))?);
    }
    let source_sha256 = sha256_hex(&all);
    let source_path = sorted
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let detected_version = crate::report::sniff_tzdb_version(&all);

    let source_inputs = SourceInputs {
        kind,
        files: input_files,
        aggregate_hash,
    };

    // Build-profile identity — what this run actually used (semantic, not argv).
    let leap_source = match &config.leaps {
        None => LeapSourceInfo {
            mode: LeapSourceMode::None,
            sha256: None,
            entry_count: 0,
            expires: false,
            rolling_entries: 0,
        },
        Some(table) => LeapSourceInfo {
            mode: LeapSourceMode::File,
            sha256: match leap_path {
                Some(p) => Some(sha256_hex(&std::fs::read(p).map_err(|e| Error::io(p, e))?)),
                None => None,
            },
            entry_count: table.entries.len(),
            expires: table.expires.is_some(),
            rolling_entries: table.entries.iter().filter(|e| e.rolling).count(),
        },
    };
    let build_profile = BuildProfile {
        output_tree: if config.leaps.is_some() {
            OutputTree::Right
        } else {
            OutputTree::Posix
        },
        leap_source,
        // T17.2: store the typed enums directly (the source of truth), rendered at the JSON boundary —
        // no re-stringified copy that could drift from `config`.
        emit_style: config.emit_style,
        range: config.range.map(|r| (r.lo, r.hi)),
        redundant_until: config.redundant_until,
        link_mode: config.link_mode,
    };

    let zones_compiled: Vec<String> = report
        .zones_compiled
        .iter()
        .map(|z| z.name.clone())
        .collect();
    let links_materialized: Vec<String> = report
        .links_written
        .iter()
        .map(|l| l.link_name.clone())
        .collect();

    // A requested identifier is "satisfied" if it was compiled as a canonical zone or written
    // as a link; anything else requested is reported as unsupported/skipped — honestly.
    let unsupported_zones: Vec<String> = requested
        .iter()
        .filter(|r| !zones_compiled.contains(r) && !links_materialized.contains(r))
        .cloned()
        .collect();

    // Link / alias identity (T12.4b). We classify against the *full parsed* link set (`db.links`),
    // NOT `report.links_written` — the report only knows what was *materialised*, but the manifest
    // also wants what was *omitted* (and *failed*), which only the db's complete link list reveals.
    // For each link we resolve its chain exactly as the compile path does (`plan::run` → the link
    // loop), then bucket by whether the resolved canonical zone landed in the compiled output set:
    //   - selected: target compiled (eligible & — since a write failure aborts the whole run —
    //     materialised);
    //   - omitted: target NOT compiled, i.e. excluded by the zone *selection* (a policy outcome);
    //   - failed: the chain does not resolve to a real zone (missing target / cycle / self-link) —
    //     an *error* class, deliberately never folded into "omitted".
    // **No source-set membership (`backward`/`backzone`) is inferred from these links** — they are
    // output identifiers, not evidence of which source file produced them (see T12.4a inventory).
    let mut selected_links: Vec<String> = Vec::new();
    let mut omitted_links: Vec<String> = Vec::new();
    let mut links_failed_count = 0usize;
    for link in &db.links {
        match crate::resolve_link_target(db, &link.link_name) {
            Ok(canonical) if zones_compiled.iter().any(|z| z == canonical) => {
                selected_links.push(link.link_name.clone())
            }
            Ok(_) => omitted_links.push(link.link_name.clone()),
            Err(_) => links_failed_count += 1, // dangling / cycle — never folded into "omitted"
        }
    }
    // Sort for a stable set hash; `dedup` because `zic` allows two `Link` lines with the same name
    // (last wins — see `make_links`), so the parsed db may legitimately carry a duplicate name we
    // must not double-count. (Sort-then-dedup removes only *adjacent* equals, hence the sort first.)
    selected_links.sort();
    selected_links.dedup();
    omitted_links.sort();
    omitted_links.dedup();
    // Order-independent *set* identity: names are already sorted+deduped, LF-joined then hashed.
    // The empty set hashes to `sha256("")` (a fixed, well-known digest) — that is intentional and
    // stable; do not special-case it to "" or a sentinel, or two empty-set runs would stop matching.
    let hash_names = |names: &[String]| sha256_hex(names.join("\n").as_bytes());
    // `alias-map.json` is serialized deterministically (sorted by identifier, fixed field order,
    // LF, no timestamps — see `AliasMap::to_json`), so hashing its bytes is a stable cross-machine
    // identity that binds this manifest to a specific alias map. `build` re-reads the just-written
    // output files to hash them; at manifest time (a successful compile) they are all on disk.
    let alias_map_sha256 = sha256_hex(build(report, &config.output_dir)?.to_json().as_bytes());
    let link_profile = LinkProfile {
        link_policy: match config.link_mode {
            crate::LinkMode::Copy => "copy",
            crate::LinkMode::Symlink => "symlink",
        }
        .to_string(),
        zones_compiled_count: zones_compiled.len(),
        links_selected_count: selected_links.len(),
        links_materialized_count: report.links_written.len(),
        links_omitted_count: omitted_links.len(),
        links_failed_count,
        alias_map_sha256,
        selected_links_sha256: hash_names(&selected_links),
        omitted_links_sha256: hash_names(&omitted_links),
    };

    // Source-evidence axes — reconciled against the *admitted* source inputs only (hash-backed
    // detection or explicit claim), never inferred from the link profile above. `backward` (T12.4d)
    // verifies an admitted file's participation; `backzone` (T12.5b) checks whether the pinned
    // reference `backzone` (T12.5a.2) participated, anchored to its release hash; `packratlist`
    // (T12.5c) `packratlist` is a **generation-policy** axis — detection comes ONLY from an admitted
    // policy input (`--packratlist-source`) whose hash equals the pinned 2026b `zone.tab`, never from
    // `source_inputs` (compile inputs); `zone.tab` is not a compilable `zic` source.
    let backzone = BackzoneEvidence::reconcile(
        &source_inputs,
        variants.backzone_claim,
        REF_2026B_BACKZONE_SHA256,
    );
    let backzone_present = backzone.detected == BackzoneDetected::Present;
    let packratlist_policy_sha = match &variants.packratlist_source {
        Some(p) => Some(sha256_hex(&std::fs::read(p).map_err(|e| Error::io(p, e))?)),
        None => None,
    };
    // `dataform` (T12.5d) — the *encoding* axis. Detection is hash-backed against the pinned 2026b
    // `.zi` artifacts via `source_inputs` membership (category-correct: the `.zi` files are compile
    // sources). The `recipe_hash` binds the generation provenance of those pinned artifacts.
    let dataform_recipe = dataform_recipe_hash(
        REF_2026B_ARCHIVE_SHA256,
        REF_2026B_MAKEFILE_SHA256,
        REF_2026B_ZIGUARD_AWK_SHA256,
        REF_2026B_DATAFORM_COMMAND,
        REF_2026B_DATAFORM_TOOLCHAIN,
    );
    let dataform_reference = DataformReference {
        main_sha256: REF_2026B_MAIN_ZI_SHA256,
        vanguard_sha256: REF_2026B_VANGUARD_ZI_SHA256,
        rearguard_sha256: REF_2026B_REARGUARD_ZI_SHA256,
        recipe_hash: &dataform_recipe,
        generated_from: REF_2026B_DATAFORM_GENERATED_FROM,
    };
    let source_profile = SourceProfile {
        backward: BackwardEvidence::reconcile(&source_inputs, variants)?,
        packratlist: PackratlistEvidence::reconcile(
            variants.packratlist_claim.as_deref(),
            packratlist_policy_sha.as_deref(),
            REF_2026B_ZONE_TAB_SHA256,
            backzone_present,
        ),
        dataform: DataformEvidence::reconcile(
            &source_inputs,
            variants.dataform_claim.as_deref(),
            &dataform_reference,
        ),
        backzone,
    };

    Ok(CompileManifest {
        zic_rs_version: env!("CARGO_PKG_VERSION").to_string(),
        tzdb: TzdbProvenance {
            detected_version,
            claimed_version: claimed_version.map(str::to_string),
            source_path,
            source_sha256,
        },
        source_inputs,
        build_profile,
        link_profile,
        source_profile,
        zones_requested: requested.to_vec(),
        zones_compiled,
        links_materialized,
        unsupported_zones,
        oracle: OracleResult::not_run(),
    })
}

// JSON string escaping is shared with the other deterministic emitters (`report`); the single
// implementation lives in `crate::json`. Identifier strings are already restricted by the
// output-tree validator, but we escape defensively — a name can contain a backslash, which JSON
// requires escaped. The `json_str` alias keeps this module's call sites unchanged.
use crate::json::escape as json_str;

#[cfg(test)]
mod tests {
    use super::*;

    // ── T17.2 CONTRACT.TYPING: totality of the newly-typed manifest vocabularies ──
    // Each enum owns its JSON literal via `as_str()`; these assert the exact literals are preserved
    // (so the `zic-rs-compile-manifest-v8` schema does not bump) and that the vocabularies stay closed.

    #[test]
    fn source_input_kind_totality_and_literals() {
        use std::collections::BTreeSet;
        let labels: Vec<&str> = SourceInputKind::ALL.iter().map(|k| k.as_str()).collect();
        // exact literals (pinned)
        assert_eq!(
            labels,
            ["tzdata_zi", "multi_file", "single_file", "unknown"]
        );
        // unique + non-empty (totality)
        let set: BTreeSet<&str> = labels.iter().copied().collect();
        assert_eq!(set.len(), SourceInputKind::ALL.len());
        assert!(labels.iter().all(|l| !l.is_empty()));
    }

    #[test]
    fn output_tree_leap_mode_oracle_verdict_literals() {
        assert_eq!(OutputTree::Posix.as_str(), "posix");
        assert_eq!(OutputTree::Right.as_str(), "right");
        assert_eq!(LeapSourceMode::None.as_str(), "none");
        assert_eq!(LeapSourceMode::File.as_str(), "file");
        // the hyphen is preserved from the pre-T17.2 free-string literal
        assert_eq!(OracleVerdict::NotRun.as_str(), "not-run");
    }

    #[test]
    fn emit_style_boundary_literals_unchanged() {
        // The manifest stores the typed `EmitStyle`; `emit_style_str` owns the literal at the boundary.
        assert_eq!(emit_style_str(crate::EmitStyle::Default), "default");
        assert_eq!(emit_style_str(crate::EmitStyle::ZicSlim), "zic-slim");
        assert_eq!(emit_style_str(crate::EmitStyle::ZicFat), "zic-fat");
    }

    #[test]
    fn alias_entry_kind_str() {
        let z = AliasEntry::Zone { sha256: "x".into() };
        let l = AliasEntry::Link {
            target: "t".into(),
            target_sha256: "y".into(),
            materialised: crate::LinkMode::Copy,
        };
        assert_eq!(z.kind_str(), "zone");
        assert_eq!(l.kind_str(), "link");
    }

    #[test]
    fn json_escaping() {
        assert_eq!(json_str("Europe/London"), "\"Europe/London\"");
        assert_eq!(json_str("a\\b"), "\"a\\\\b\"");
        assert_eq!(json_str("a\"b"), "\"a\\\"b\"");
    }

    #[test]
    fn empty_map_is_valid_json_shape() {
        let m = AliasMap {
            entries: BTreeMap::new(),
            identifiers: 0,
            canonical_zones: 0,
            links: 0,
            duplicated_byte_links: 0,
        };
        let j = m.to_json();
        assert!(j.contains("\"schema\": \"zic-rs-alias-map-v1\""));
        assert!(j.contains("\"zones\": {}"));
        assert!(j.contains("\"identifiers\": 0"));
    }
}
