//! Command-line interface — a thin shell over the `tzcompile` library.
//!
//! The CLI parses arguments, builds a [`CompileConfig`], and calls the library. It holds no
//! compiler logic of its own; everything substantive lives in the library so it can be
//! reused and tested directly. Subcommands:
//!
//! * `compile` — compile selected zones to a TZif tree under `--out`;
//! * `compare` — compile a zone and diff it against reference `zic` (the oracle);
//! * `explain` — say what a zone would compile to, or why it is unsupported;
//! * `supported-syntax` — print the subset this version implements;
//! * `support-report` — map a whole source file: which identifiers compile, and why the rest don't.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::compile::plan;
use crate::error::{Error, Result};
use crate::{CompileConfig, LinkMode, UnsupportedPolicy, ZoneSelection, DEFAULT_TRANSITION_LIMIT};

/// Top-level CLI definition.
#[derive(Debug, Parser)]
#[command(
    name = "zic-rs",
    about = "A memory-safe Rust timezone compiler for IANA tzdata (declared subset).",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

// `Compile(CompileArgs)` is much larger than the unit/small variants because `CompileArgs` carries
// the full flag surface (T12.5c added the `--packratlist*` provenance flags). Boxing the variant
// would break clap's `Subcommand` derive (it expects the bare `Args` type), and a `Command` value is
// constructed exactly once per process from argv — the size difference is operationally irrelevant —
// so we accept it deliberately rather than distort the CLI type.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Compile zones from tzdata source into a TZif output tree.
    Compile(CompileArgs),
    /// Compile a zone and compare it against reference `zic`.
    Compare(CompareArgs),
    /// Describe what a zone would compile to (or why it is unsupported).
    Explain(ExplainArgs),
    /// Print the syntax subset supported by this version.
    SupportedSyntax,
    /// Report which identifiers in a source file compile, and bucket the rest by reason.
    SupportReport(SupportReportArgs),
    /// Inventory how zic-rs TZif output differs *structurally* from reference `zic` (T8).
    StructuralReport(StructuralReportArgs),
    /// Emit typed `zdump`-backed **semantic witnesses** (offset/is_dst/abbreviation at probe instants)
    /// for a compact zone set — the behaviour axis of the conformance engine (T15.3). Degrades visibly
    /// (`oracle_mode: unavailable`) when reference `zic`/`zdump` are absent; never errors on that.
    SemanticReport(SemanticReportArgs),
    /// Validate emitted TZif against **RFC 9636** structural invariants (T15.4) — a *separate* axis from
    /// semantic behaviour. Emits typed structural/footer/reader-compat/leap-expiry/version verdicts;
    /// validates zic-rs output **and** reference `zic` output (so the validator respects the real
    /// producer profile, not only zic-rs's assumptions).
    TzifValidate(TzifValidateArgs),
    /// Validate auxiliary tables (`zone.tab`/`zone1970.tab`/`zonenow.tab`/`iso3166.tab`) for **table
    /// structural admissibility only** (T16.4) — a *separate* surface from compile/semantic/structural.
    /// These are policy/index/reference artifacts, **not** compile inputs; a conformant row proves the
    /// row is well-formed, never that the named zone was compiled or is historically equivalent.
    AuxTableValidate(AuxTableValidateArgs),
    /// Emit the canonical **`vendor-oracle-receipt-v1`** sample (T16.5) — the schema example an external
    /// vendor/platform lab fills in. The core repo *admits* receipts (verifies the contract + rules); it
    /// **does not** run QEMU/VMs (`does_not_ship_or_operate_vendor_qemu_labs_in_core_repo`).
    VendorOracleSample,
    /// **Ingest + admit** an externally-produced `vendor-oracle-receipt-v1` JSON file (T16.5b) — the
    /// convergence point: an external lab generates a receipt, the core repo *admits or rejects* it by
    /// typed reason. Fail-closed parsing (a parse error is distinct from an inadmissible receipt); the
    /// core runs no VMs.
    VendorOracleAdmit(VendorOracleAdmitArgs),
    /// **Diff two tzdb releases** (T16.6a). Compiles each identifier in `--old` and `--new` and
    /// classifies the change: structural (always) + behavioural past/future (with a `zdump` oracle).
    /// Read-only; never installs. JSON `zic-rs-release-diff-v1`.
    ReleaseDiff(ReleaseDiffArgs),
    /// **Read-only environment probe** (T16.6b) — reference `zic`/`zdump` present? optional `tzdata.zi`
    /// version+hash? It admits/validates nothing and always exits 0 (a diagnosis, not a gate).
    Doctor(DoctorArgs),
    /// **Read-only bundle footprint** (T21.2) — measure a produced `--out` tree (TZif/link/other counts,
    /// bytes, version histogram, largest file) + a deterministic `bundle_hash`. For container/embedded
    /// image builders (`docs/container-embedded-builder.md`). Never compiles/writes/admits.
    SizeReport(SizeReportArgs),
}

#[derive(Debug, Args)]
pub struct CompileArgs {
    /// Source file(s) or directory(ies) of tzdata.
    #[arg(long = "input", required = true, num_args = 1..)]
    pub input: Vec<PathBuf>,
    /// Output directory root. Required — there is no system default.
    #[arg(long = "out")]
    pub out: Option<PathBuf>,
    /// Compile a single named zone.
    #[arg(long = "zone")]
    pub zone: Option<String>,
    /// Compile the zones listed (one per line) in this file.
    #[arg(long = "zones")]
    pub zones: Option<PathBuf>,
    /// Compile every zone this version supports (others are reported and skipped).
    #[arg(long = "all-supported", default_value_t = false)]
    pub all_supported: bool,
    /// How to materialise links: `copy` (default) or `symlink`.
    #[arg(long = "link-mode", default_value = "copy")]
    pub link_mode: LinkModeArg,
    /// Overwrite existing output files.
    #[arg(long = "force", default_value_t = false)]
    pub force: bool,
    /// On unsupported syntax: `error` (default, fail closed) or `skip`.
    #[arg(long = "unsupported", default_value = "error")]
    pub unsupported: UnsupportedArg,
    /// Maximum transitions emitted per zone.
    #[arg(long = "transition-limit", default_value_t = DEFAULT_TRANSITION_LIMIT)]
    pub transition_limit: usize,
    /// Explicit-transition emission style (T8-slim). `default` = behaviour-matched (CORE.1-gated);
    /// `zic-slim` reproduces reference `zic`'s slim explicit-transition set; `zic-fat` == default.
    /// Never changes behaviour — only how many explicit transitions precede the POSIX footer.
    #[arg(long = "emit-style", value_enum, default_value_t = EmitStyleArg::Default)]
    pub emit_style: EmitStyleArg,
    /// Emission bloat (reference `zic`'s `-b {slim|fat}`): a thin alias onto `--emit-style`
    /// (`slim` → `zic-slim`, `fat` → `zic-fat`). Not the TZif version, not the `-R` redundant tail.
    /// If both `-b` and a conflicting `--emit-style` are given, that is an error.
    #[arg(long = "bloat", short = 'b', value_enum)]
    pub bloat: Option<BloatArg>,
    /// Redundant-tail bound (reference `zic`'s `-R @hi`): under slim emission, keep otherwise-droppable
    /// footer-governed transitions out to this instant (`@<unix-seconds>`), for readers that ignore the
    /// POSIX footer. Only affects `--emit-style zic-slim`/`-b slim` (zic-rs's default is already fat).
    /// Not bloat (`-b`) and not range truncation (`-r`); never changes behaviour or the TZif version.
    #[arg(long = "redundant-until", short = 'R')]
    pub redundant_until: Option<String>,
    /// Range truncation (reference `zic`'s `-r '[@lo][/@hi]'`): restrict emitted timestamps to the
    /// `@`-prefixed Unix-second window. **Parse-only for now (T10.4b)** — a well-formed value is
    /// recognised but not yet applied (the compile fails closed rather than emit un-truncated output);
    /// truncation + the `-00` unspecified-local-time placeholder land in T10.4d.
    #[arg(long = "range", short = 'r')]
    pub range: Option<String>,
    /// Leap-seconds source (reference `zic`'s `-L leapseconds`): the **`right/` build profile**. When
    /// given, the parsed leap table is applied to **every** compiled zone. Opt-in; **never** the
    /// default — without it, ordinary canonical-zone output is unchanged (no leap table).
    #[arg(long = "leapseconds", short = 'L')]
    pub leapseconds: Option<PathBuf>,
    /// Do not create the `--out` tree (reference `zic`'s `-D`); require it to already exist.
    #[arg(long = "no-create-dirs", short = 'D', default_value_t = false)]
    pub no_create_dirs: bool,
    /// Install policy (reference `zic`'s `-l <zone>`): also create a `localtime` link in `--out`
    /// pointing at this (also-selected) zone. Opt-in; never affects canonical-zone behaviour.
    #[arg(long = "localtime", short = 'l')]
    pub localtime: Option<String>,
    /// Name of the `localtime` link (reference `zic`'s `-t`, default `localtime`). Constrained to a
    /// safe relative name under `--out` — zic-rs will not write to an arbitrary/system path.
    #[arg(long = "localtime-name", short = 't')]
    pub localtime_name: Option<String>,
    /// File permission bits for created files (reference `zic`'s `-m`), as **octal** (e.g. `644`).
    /// Unix-only; applies to compiled TZif files and copied links (not symlinks). Symbolic chmod
    /// expressions are not supported.
    #[arg(long = "mode", short = 'm')]
    pub mode: Option<String>,
    /// Also write an alias/canonical manifest (zones vs links + hashes) to this path.
    #[arg(long = "alias-map")]
    pub alias_map: Option<PathBuf>,
    /// Also write a compile-provenance manifest (source hash, build-profile identity, oracle) here.
    #[arg(long = "manifest")]
    pub manifest: Option<PathBuf>,
    /// Optional **claimed** tzdb release (e.g. `2026b`) recorded in the manifest and reconciled
    /// against the version *detected* from the source — surfaces a `detected_differs_from_claim`
    /// rather than silently stamping a release.
    #[arg(long = "tzdb-version")]
    pub tzdb_version: Option<String>,
    /// Optional **claimed** `backward` (legacy-alias) source-set membership, recorded in the
    /// manifest's `source_profile` as a bare claim (`included`/`excluded`) — never trusted as
    /// detection. **Provenance only:** does not change what is compiled or linked.
    #[arg(long = "backward", value_parser = ["included", "excluded"])]
    pub backward: Option<String>,
    /// Optional file the caller asserts is the `backward` source. The manifest hash-checks whether
    /// its *bytes* participated in this build (→ `detected: present|absent`); it does **not** assert
    /// the file is genuinely the IANA `backward`, and it does **not** affect compilation.
    #[arg(long = "backward-source")]
    pub backward_source: Option<PathBuf>,
    /// Optional **claimed** `backzone` (`PACKRATDATA`) source-set membership, recorded in the manifest's
    /// `source_profile.backzone_evidence` as a bare claim (`included`/`excluded`) — never trusted as
    /// detection. Detection is hash-anchored to the pinned reference release (T12.5b). **Provenance
    /// only:** does not change what is compiled or linked.
    #[arg(long = "backzone", value_parser = ["included", "excluded"])]
    pub backzone: Option<String>,
    /// Optional **claimed** `backzone` *scope* (`PACKRATLIST`): `full` (all backzone) / `subset`
    /// (filtered, e.g. `PACKRATLIST=zone.tab`) / `none` (no backzone). Recorded in
    /// `source_profile.packratlist_evidence` as a bare claim — never trusted as detection (T12.5c).
    /// **Provenance only.**
    #[arg(long = "packratlist", value_parser = ["full", "subset", "none"])]
    pub packratlist: Option<String>,
    /// Optional file the caller **explicitly admits** as the `PACKRATLIST` subset source (e.g.
    /// `zone.tab`). Detection confirms its *bytes* participated alongside `backzone` (→ `subset`); mere
    /// presence of `zone.tab` among inputs is **not** admission. **Provenance only** (T12.5c).
    #[arg(long = "packratlist-source")]
    pub packratlist_source: Option<PathBuf>,
    /// Optional **claimed** `DATAFORM` *encoding* form: `main` / `vanguard` / `rearguard`. Recorded in
    /// `source_profile.dataform_evidence` as a bare claim — never trusted as detection. Detection is
    /// hash-backed against the pinned 2026b `.zi` artifacts via the compiled inputs (there is no
    /// `--dataform-source`: the `.zi` artifacts *are* compile sources). **Provenance only** (T12.5d).
    #[arg(long = "dataform", value_parser = ["main", "vanguard", "rearguard"])]
    pub dataform: Option<String>,
    /// Verbose diagnostics (reference `zic`'s `-v`): also surface `VerboseOnly` warnings — e.g. the
    /// "fewer than 3 characters" abbreviation warning and the transition-count client-compat warning.
    /// Default (quiet) prints only always-on diagnostics, matching `zic` without `-v` (T13.6). Never
    /// affects compiled output, exit status, or which warnings are *collected* — only what is printed.
    #[arg(long = "verbose", short = 'v', default_value_t = false)]
    pub verbose: bool,

    /// **Bounded** Latin-1 historical-source replay (LEGACY-SOURCE.1). Off by default: the modern
    /// source contract is UTF-8, and invalid UTF-8 is a hard `ZIC012`. When set, ISO-8859-1 (Latin-1)
    /// is admitted **only** for non-UTF-8 bytes that lie inside a `#` comment — the case in pre-2013
    /// tzdb releases (accented author/place names in comments). A non-UTF-8 byte in a semantics-bearing
    /// field is still refused. The compiled output is unaffected (comment bytes are stripped). Use this
    /// only to replay admitted historical tzdb releases, never to relax the modern contract.
    #[arg(long = "legacy-latin1", default_value_t = false)]
    pub legacy_latin1: bool,

    /// **Bounded** historical Rule `TYPE` / `yearistype` replay (YEARISTYPE.1). Off by default: the
    /// modern source contract has no year-type predicates, and any non-`-` TYPE is a hard
    /// `ZIC027` (reference `zic` removed the `-y`/`yearistype` ecology in tzcode 2020a). When set, the
    /// four historical predicates `even`/`odd`/`uspres`/`nonpres` are admitted (anchored to
    /// `yearistype.sh` v7.4), so pre-2000f releases (the `AS` rules for Australia/Adelaide &
    /// Broken_Hill, 1990–1994) compile. An unknown ("wild") type is still refused. This adds
    /// historical-source replay support; it does **not** claim current reference `zic` still supports
    /// `-y`. zic-rs never executes the historical shell script — the predicates are internal,
    /// deterministic functions.
    #[arg(long = "legacy-yearistype", default_value_t = false)]
    pub legacy_yearistype: bool,

    /// **Bounded** empty-footer fallback for non-POSIX final recurrence (PERPETUAL-EXPANSION.1). Off by
    /// default — a final recurring era that cannot be reduced to one POSIX DST/standard footer pair
    /// (the live case: historical **perpetual year-parity** rules like `1990 max even/odd`, which a
    /// POSIX `TZ` string cannot encode) **fails closed** (`ZIC001`). When set, *only* on that
    /// synthesis-failure path, zic-rs emits the explicit transitions it already expands through the
    /// replay horizon (2037) plus an **empty footer** (frozen beyond), matching the historical `zic`
    /// oracle's behaviour over the declared window. This is a **footer-emission policy** — it does not
    /// change transition generation — and is distinct from `--legacy-yearistype` (Rule TYPE admission);
    /// the historical releases that need it want both. Output beyond the last explicit transition is
    /// intentionally not claimed.
    #[arg(long = "legacy-empty-footer", default_value_t = false)]
    pub legacy_empty_footer: bool,
}

#[derive(Debug, Args)]
pub struct CompareArgs {
    #[arg(long = "input", required = true, num_args = 1..)]
    pub input: Vec<PathBuf>,
    #[arg(long = "zone")]
    pub zone: String,
    /// Program name/path of the reference `zic`.
    #[arg(long = "reference-zic", default_value = "zic")]
    pub reference_zic: String,
    /// Comparison mode: `zdump` (behaviour over a horizon; the default and the real
    /// correctness oracle) or `structural` (decoded-TZif model diff; fixed-offset/debug).
    #[arg(long = "mode", default_value = "zdump")]
    pub mode: CompareModeArg,
    /// Year horizon `LO,HI` for `zdump` mode (inclusive). Behaviour is only compared within
    /// this declared window.
    #[arg(long = "horizon", default_value = "1900,2100")]
    pub horizon: String,
    /// Program name/path of `zdump` (zdump mode only).
    #[arg(long = "zdump", default_value = "zdump")]
    pub zdump: String,
}

/// `--mode` value.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum CompareModeArg {
    Zdump,
    Structural,
}

#[derive(Debug, Args)]
pub struct ExplainArgs {
    #[arg(long = "input", required = true, num_args = 1..)]
    pub input: Vec<PathBuf>,
    #[arg(long = "zone")]
    pub zone: String,
}

#[derive(Debug, Args)]
pub struct SupportReportArgs {
    /// Source file(s) of tzdata — typically the installed `/usr/share/zoneinfo/tzdata.zi`.
    #[arg(long = "input", required = true, num_args = 1..)]
    pub input: Vec<PathBuf>,
    /// Output format.
    #[arg(long = "format", value_enum, default_value_t = ReportFormatArg::Text)]
    pub format: ReportFormatArg,
    /// Annotate each unsupported/fail-closed bucket with the deep `zic` semantic law it represents
    /// (text mode). The JSON form always carries a `deep_semantic` field. See
    /// `docs/zic-deep-semantics.md`.
    #[arg(long = "explain-buckets", default_value_t = false)]
    pub explain_buckets: bool,
}

/// `--format` value for `support-report`.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ReportFormatArg {
    Text,
    Json,
}

#[derive(Debug, Args)]
pub struct StructuralReportArgs {
    /// Source file(s) of tzdata — typically the installed `/usr/share/zoneinfo/tzdata.zi`.
    #[arg(long = "input", required = true, num_args = 1..)]
    pub input: Vec<PathBuf>,
    /// Program name/path of the reference `zic` to compare against (required — this is a
    /// comparison, not a self-report).
    #[arg(long = "reference-zic", default_value = "zic")]
    pub reference_zic: String,
    /// Restrict the inventory to a single canonical zone (default: every canonical zone).
    #[arg(long = "zone")]
    pub zone: Option<String>,
    /// Emission style for *our* side of the comparison (T8-slim). `default` keeps the
    /// behaviour-matched output; `zic-slim` reproduces reference `zic`'s slim explicit-transition
    /// set, collapsing the `slim/fat-timecnt` class.
    #[arg(long = "emit-style", value_enum, default_value_t = EmitStyleArg::Default)]
    pub emit_style: EmitStyleArg,
    /// Output format.
    #[arg(long = "format", value_enum, default_value_t = ReportFormatArg::Text)]
    pub format: ReportFormatArg,
}

/// `semantic-report` (T15.3): typed `zdump`-backed semantic witnesses for a compact zone set.
#[derive(Debug, clap::Args)]
pub struct SemanticReportArgs {
    /// Source file(s) of tzdata.
    #[arg(long = "input", required = true, num_args = 1..)]
    pub input: Vec<PathBuf>,
    /// Reference `zic` program (compiles the oracle side).
    #[arg(long = "reference-zic", default_value = "zic")]
    pub reference_zic: String,
    /// Reference `zdump` program (the footer-aware behaviour oracle).
    #[arg(long = "zdump", default_value = "zdump")]
    pub zdump: String,
    /// Zones to witness (repeatable). Default: a small curated set (those present in the input).
    #[arg(long = "zone")]
    pub zone: Vec<String>,
    /// Output format (JSON is the machine-readable conformance surface).
    #[arg(long = "format", value_enum, default_value_t = ReportFormatArg::Json)]
    pub format: ReportFormatArg,
}

/// `tzif-validate` (T15.4): RFC 9636 structural validation of zic-rs + reference TZif output.
#[derive(Debug, clap::Args)]
pub struct TzifValidateArgs {
    /// Source file(s) of tzdata.
    #[arg(long = "input", required = true, num_args = 1..)]
    pub input: Vec<PathBuf>,
    /// Reference `zic` program — its output is validated too (the producer-profile guard).
    #[arg(long = "reference-zic", default_value = "zic")]
    pub reference_zic: String,
    /// Zones to validate (repeatable). Default: a small curated set (those present in the input).
    #[arg(long = "zone")]
    pub zone: Vec<String>,
    /// Output format.
    #[arg(long = "format", value_enum, default_value_t = ReportFormatArg::Json)]
    pub format: ReportFormatArg,
}

/// `aux-table-validate` (T16.4): structural validation of auxiliary `.tab` files. Each flag points at a
/// table file directly — they are **not** `--input` compile sources (the category law).
#[derive(Debug, clap::Args)]
pub struct AuxTableValidateArgs {
    /// Path to `zone.tab`.
    #[arg(long = "zone-tab")]
    pub zone_tab: Option<PathBuf>,
    /// Path to `zone1970.tab` (cross-validated against `iso3166.tab` when both are given).
    #[arg(long = "zone1970-tab")]
    pub zone1970_tab: Option<PathBuf>,
    /// Path to `zonenow.tab` (now/future-agreement table — `XX` codes allowed).
    #[arg(long = "zonenow-tab")]
    pub zonenow_tab: Option<PathBuf>,
    /// Path to `iso3166.tab`.
    #[arg(long = "iso3166-tab")]
    pub iso3166_tab: Option<PathBuf>,
    /// Output format.
    #[arg(long = "format", value_enum, default_value_t = ReportFormatArg::Json)]
    pub format: ReportFormatArg,
}

/// `vendor-oracle-admit` (T16.5b): ingest + admit an external receipt JSON file.
#[derive(Debug, clap::Args)]
pub struct VendorOracleAdmitArgs {
    /// Path to an external `vendor-oracle-receipt-v1` JSON file.
    #[arg(long = "receipt", required = true)]
    pub receipt: PathBuf,
}

/// `release-diff` (T16.6a): diff two tzdb releases per identifier.
#[derive(Debug, clap::Args)]
pub struct ReleaseDiffArgs {
    /// The OLD release source (`tzdata.zi` file or a source directory).
    #[arg(long = "old", required = true)]
    pub old: Vec<PathBuf>,
    /// The NEW release source (`tzdata.zi` file or a source directory).
    #[arg(long = "new", required = true)]
    pub new: Vec<PathBuf>,
    /// Restrict to a single identifier.
    #[arg(long = "zone")]
    pub zone: Option<String>,
    /// Behaviour horizon in years (default `1900,2040`, CORE.1's).
    #[arg(long = "horizon", default_value = "1900,2040")]
    pub horizon: String,
    /// Past/future split **year** (deterministic; default `2025`). Never host-`now`.
    #[arg(long = "split", default_value_t = 2025)]
    pub split: i32,
    /// Reference `zdump` for the behaviour axis. Omit ⇒ behaviour not assessed (structural still runs).
    #[arg(long = "reference-zdump")]
    pub reference_zdump: Option<String>,
    /// Output format.
    #[arg(long = "format", value_enum, default_value_t = ReportFormatArg::Json)]
    pub format: ReportFormatArg,
}

/// `doctor` (T16.6b): read-only environment probe.
#[derive(Debug, clap::Args)]
pub struct DoctorArgs {
    /// Reference `zic` program to probe (default `zic`).
    #[arg(long = "reference-zic", default_value = "zic")]
    pub reference_zic: String,
    /// Reference `zdump` program to probe (default `zdump`).
    #[arg(long = "reference-zdump", default_value = "zdump")]
    pub reference_zdump: String,
    /// Optional explicit `tzdata.zi` to probe for version + hash (never read implicitly).
    #[arg(long = "tzdata")]
    pub tzdata: Option<PathBuf>,
    /// Output format.
    #[arg(long = "format", value_enum, default_value_t = ReportFormatArg::Text)]
    pub format: ReportFormatArg,
}

#[derive(Debug, Args)]
pub struct SizeReportArgs {
    /// The produced output tree to measure (an existing directory written by `compile --out`).
    #[arg(long = "out")]
    pub out: PathBuf,
    /// Output format.
    #[arg(long = "format", value_enum, default_value_t = ReportFormatArg::Text)]
    pub format: ReportFormatArg,
}

/// `--emit-style` value (T8-slim).
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum EmitStyleArg {
    /// Behaviour-matched default (CORE.1-gated; fat-ish).
    Default,
    /// Reference `zic` slim: truncate the footer-governed recurring tail.
    ZicSlim,
    /// Reference `zic` fat (currently == default).
    ZicFat,
}

impl From<EmitStyleArg> for crate::EmitStyle {
    fn from(a: EmitStyleArg) -> Self {
        match a {
            EmitStyleArg::Default => crate::EmitStyle::Default,
            EmitStyleArg::ZicSlim => crate::EmitStyle::ZicSlim,
            EmitStyleArg::ZicFat => crate::EmitStyle::ZicFat,
        }
    }
}

/// Reference `zic`'s `-b {slim|fat}` *bloat* value (T10.2). This is the **emission policy** knob —
/// whether otherwise-redundant transitions are kept (`fat`) or dropped (`slim`) — and is a thin
/// alias onto [`EmitStyleArg`]. It is **not** the TZif version (content-driven) and **not** the
/// `-R` redundant-tail range. `zic`'s own default is `slim`; zic-rs keeps a behaviour-matched
/// fat-style default and only changes emission when asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum BloatArg {
    Slim,
    Fat,
}

impl From<BloatArg> for EmitStyleArg {
    fn from(b: BloatArg) -> Self {
        match b {
            BloatArg::Slim => EmitStyleArg::ZicSlim,
            BloatArg::Fat => EmitStyleArg::ZicFat,
        }
    }
}

/// `--link-mode` value.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum LinkModeArg {
    Copy,
    Symlink,
}

impl From<LinkModeArg> for LinkMode {
    fn from(a: LinkModeArg) -> Self {
        match a {
            LinkModeArg::Copy => LinkMode::Copy,
            LinkModeArg::Symlink => LinkMode::Symlink,
        }
    }
}

/// `--unsupported` value.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum UnsupportedArg {
    Error,
    Skip,
}

impl From<UnsupportedArg> for UnsupportedPolicy {
    fn from(a: UnsupportedArg) -> Self {
        match a {
            UnsupportedArg::Error => UnsupportedPolicy::Error,
            UnsupportedArg::Skip => UnsupportedPolicy::WarnAndSkipZone,
        }
    }
}

/// Run the CLI. Returns `Ok(())` on success; the caller maps `Err` to a process exit code.
pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Compile(args) => run_compile(args),
        Command::Compare(args) => run_compare(args),
        Command::Explain(args) => run_explain(args),
        Command::SupportedSyntax => {
            print!("{}", supported_syntax_text());
            Ok(())
        }
        Command::SupportReport(args) => run_support_report(args),
        Command::StructuralReport(args) => run_structural_report(args),
        Command::SemanticReport(args) => run_semantic_report(args),
        Command::TzifValidate(args) => run_tzif_validate(args),
        Command::AuxTableValidate(args) => run_aux_table_validate(args),
        Command::VendorOracleSample => {
            print!(
                "{}",
                crate::vendor_oracle::VendorOracleReceipt::minimal_sample().to_json()
            );
            Ok(())
        }
        Command::VendorOracleAdmit(args) => run_vendor_oracle_admit(args),
        Command::ReleaseDiff(args) => run_release_diff(args),
        Command::Doctor(args) => run_doctor(args),
        Command::SizeReport(args) => run_size_report(args),
    }
}

/// `support-report`: load the source, sniff the tzdb release, build the frontier map, print it.
fn run_support_report(args: SupportReportArgs) -> Result<()> {
    // Sniff the tzdb release from the *raw* bytes (comments are stripped during parsing). Use the
    // first input file's header, where `zic`'s single-file output records `# version <X>`.
    let tzdb_version = std::fs::read(&args.input[0])
        .ok()
        .and_then(|b| crate::report::sniff_tzdb_version(&b));
    let db = crate::load_database(&args.input)?;
    let report = crate::report::build_support_report(&db, tzdb_version);
    match args.format {
        ReportFormatArg::Text if args.explain_buckets => print!("{}", report.to_text_explained()),
        ReportFormatArg::Text => print!("{}", report.to_text()),
        ReportFormatArg::Json => print!("{}", report.to_json()),
    }
    Ok(())
}

/// `structural-report`: compile every canonical zone with zic-rs and reference `zic`, decode
/// both, and classify the structural differences (T8). This is a *separate axis* from the
/// behaviour oracle — see `src/structural.rs` and `docs/structural-parity.md`.
fn run_structural_report(args: StructuralReportArgs) -> Result<()> {
    if !crate::compare::reference_zic::is_available(&args.reference_zic) {
        return Err(Error::config(format!(
            "reference zic `{}` not found; structural-report compares against it",
            args.reference_zic
        )));
    }
    let tzdb_version = std::fs::read(&args.input[0])
        .ok()
        .and_then(|b| crate::report::sniff_tzdb_version(&b));
    let db = crate::load_database(&args.input)?;
    // Reference `zic` takes files, not directories — expand inputs the same way `compare` does.
    let files = crate::collect_source_files(&args.input)?;
    let work = tempfile::Builder::new()
        .prefix("zic-rs-structural-")
        .tempdir()
        .map_err(|e| Error::io(std::env::temp_dir(), e))?;
    let report = crate::structural::build_structural_report(
        &db,
        &files,
        &args.reference_zic,
        work.path(),
        args.zone.as_deref(),
        tzdb_version,
        args.emit_style.into(),
    )?;
    match args.format {
        ReportFormatArg::Text => print!("{}", report.to_text()),
        ReportFormatArg::Json => print!("{}", report.to_json()),
    }
    Ok(())
}

/// T15.3 — emit typed `zdump`-backed semantic witnesses for a compact zone set. Does **not** error when
/// the oracle is absent: the report renders `oracle_mode: unavailable` (visible, never silent).
fn run_semantic_report(args: SemanticReportArgs) -> Result<()> {
    let db = crate::load_database(&args.input)?;
    let files = crate::collect_source_files(&args.input)?;
    // Curated default: a small, meaningful spread (UTC + DST + multi-era + sub-hour), filtered to those
    // actually present in the input — *make the mechanism public, do not sweep every zone*.
    let zones: Vec<String> = if !args.zone.is_empty() {
        args.zone.clone()
    } else {
        const CURATED: &[&str] = &[
            "Etc/UTC",
            "America/New_York",
            "Europe/London",
            "Australia/Sydney",
            "Asia/Kolkata",
        ];
        let present: Vec<String> = CURATED
            .iter()
            .filter(|z| db.zone(z).is_some())
            .map(|z| z.to_string())
            .collect();
        if present.is_empty() {
            db.zones.iter().take(3).map(|z| z.name.clone()).collect()
        } else {
            present
        }
    };
    let work = tempfile::Builder::new()
        .prefix("zic-rs-semantic-")
        .tempdir()
        .map_err(|e| Error::io(std::env::temp_dir(), e))?;
    let report = crate::semantic_witness::build_semantic_witness_report(
        &db,
        &zones,
        &args.reference_zic,
        &args.zdump,
        &files,
        work.path(),
    )?;
    match args.format {
        ReportFormatArg::Json => print!("{}", report.to_json()),
        ReportFormatArg::Text => {
            println!(
                "semantic witnesses (oracle_mode: {})",
                report.oracle_mode.mode_str()
            );
            for w in &report.witnesses {
                println!("  {} @ {}: {}", w.zone, w.timestamp, w.verdict.as_str());
            }
        }
    }
    Ok(())
}

/// T15.4 — RFC 9636 TZif structural validation of zic-rs + reference output for a compact zone set.
fn run_tzif_validate(args: TzifValidateArgs) -> Result<()> {
    let db = crate::load_database(&args.input)?;
    let files = crate::collect_source_files(&args.input)?;
    let zones: Vec<String> = if !args.zone.is_empty() {
        args.zone.clone()
    } else {
        const CURATED: &[&str] = &["Etc/UTC", "America/New_York", "Europe/London", "Asia/Gaza"];
        let present: Vec<String> = CURATED
            .iter()
            .filter(|z| db.zone(z).is_some())
            .map(|z| z.to_string())
            .collect();
        if present.is_empty() {
            db.zones.iter().take(3).map(|z| z.name.clone()).collect()
        } else {
            present
        }
    };
    let work = tempfile::Builder::new()
        .prefix("zic-rs-tzif-validate-")
        .tempdir()
        .map_err(|e| Error::io(std::env::temp_dir(), e))?;
    let report = crate::tzif::rfc9636::build_validation_report(
        &db,
        &zones,
        &args.reference_zic,
        &files,
        work.path(),
    )?;
    match args.format {
        ReportFormatArg::Json => print!("{}", report.to_json()),
        ReportFormatArg::Text => {
            println!(
                "TZif structural validation (reference validated: {})",
                report.reference_validated
            );
            for row in &report.rows {
                println!(
                    "  [{}] {} : {}{}",
                    row.producer,
                    row.zone,
                    row.validation.structural.as_str(),
                    if row.validation.violations.is_empty() {
                        String::new()
                    } else {
                        format!(" — {}", row.validation.violations.join("; "))
                    }
                );
            }
        }
    }
    Ok(())
}

/// T16.4 — validate auxiliary tables for **structural admissibility only** (policy/index artifacts, not
/// compile inputs). Reads each table file directly; `zone1970.tab` is cross-validated against
/// `iso3166.tab` when both are supplied. A missing flag simply omits that table (no error).
fn run_aux_table_validate(args: AuxTableValidateArgs) -> Result<()> {
    use crate::aux_tables::{
        iso3166_codes, validate_zone_table, AuxTableValidationReport, InstallEcologyStatus,
        ZoneTableKind,
    };
    let read =
        |p: &PathBuf| -> Result<Vec<u8>> { std::fs::read(p).map_err(|e| Error::io(p.clone(), e)) };
    // Build the ISO-3166 code set first (also validated below), for zone1970 cross-validation.
    let iso_bytes = args.iso3166_tab.as_ref().map(read).transpose()?;
    let iso_codes = iso_bytes.as_ref().map(|b| iso3166_codes(b));

    let mut tables = Vec::new();
    if let Some(p) = &args.zone_tab {
        // Cross-validate zone.tab country codes against the same-release iso3166.tab when supplied.
        tables.push(validate_zone_table(
            ZoneTableKind::ZoneTab,
            &read(p)?,
            iso_codes.as_ref(),
        ));
    }
    if let Some(p) = &args.zone1970_tab {
        tables.push(validate_zone_table(
            ZoneTableKind::Zone1970Tab,
            &read(p)?,
            iso_codes.as_ref(),
        ));
    }
    if let Some(p) = &args.zonenow_tab {
        tables.push(validate_zone_table(
            ZoneTableKind::ZonenowTab,
            &read(p)?,
            None,
        ));
    }
    if let Some(b) = &iso_bytes {
        tables.push(validate_zone_table(ZoneTableKind::Iso3166Tab, b, None));
    }
    let report = AuxTableValidationReport {
        tables,
        install_ecology: InstallEcologyStatus::current(),
    };
    match args.format {
        ReportFormatArg::Json => print!("{}", report.to_json()),
        ReportFormatArg::Text => {
            println!("auxiliary-table validation (structural admissibility only)");
            for t in &report.tables {
                println!(
                    "  {} [{}] : {} ({} rows, {} findings)",
                    t.kind.as_str(),
                    t.kind.coverage(),
                    t.verdict.as_str(),
                    t.rows_checked,
                    t.findings.len()
                );
            }
        }
    }
    Ok(())
}

/// T16.5b — ingest + admit an external vendor-oracle receipt JSON file (the convergence point: external
/// lab generates the receipt, the core repo admits/rejects it). **Parse failure ≠ inadmissible** — a
/// malformed receipt is a parse error (exit 1, distinct message); a well-formed-but-failing receipt parses
/// and is reported non-admitted (exit 1, admission reason). An admitted receipt exits 0.
fn run_vendor_oracle_admit(args: VendorOracleAdmitArgs) -> Result<()> {
    let bytes = std::fs::read(&args.receipt).map_err(|e| Error::io(args.receipt.clone(), e))?;
    let text = String::from_utf8(bytes)
        .map_err(|_| Error::config("receipt is not valid UTF-8 (parse error)"))?;
    match crate::vendor_oracle::VendorOracleReceipt::from_json(&text) {
        // Distinct path 1: the bytes are not a well-formed v1 receipt.
        Err(e) => Err(Error::config(format!("receipt parse error: {e}"))),
        Ok(receipt) => {
            let verdict = receipt.admit();
            println!(
                "vendor-oracle receipt: platform={} fixture_set={} → admission={}",
                receipt.platform,
                receipt.fixture_set,
                verdict.as_str()
            );
            if verdict.is_admitted() {
                Ok(())
            } else {
                // Distinct path 2: parsed fine, but the evidence does not meet the admission rules.
                Err(Error::config(format!(
                    "receipt not admitted: {}",
                    verdict.as_str()
                )))
            }
        }
    }
}

// `release-diff` is a **witness/report, not a gate**: finding differences between OLD and NEW is the
// *output*, so it still exits 0. A non-zero exit means only an *operational* failure (a malformed
// `--horizon`, an unreadable source, an inverted horizon) — never "the releases differ." Identifiers
// outside zic-rs's compile subset are reported as `errors[]` rows, not a process failure. (Contrast
// `doctor`, also non-gating but always-0; and `compile`, which *is* a gate on the source.)
fn run_release_diff(args: ReleaseDiffArgs) -> Result<()> {
    use crate::release_diff::{build_release_diff, ReleaseDiffOptions};
    // Parse "LO,HI" years (an operational/config error if malformed — distinct from "no differences").
    let (lo, hi) = args
        .horizon
        .split_once(',')
        .and_then(|(a, b)| Some((a.trim().parse::<i32>().ok()?, b.trim().parse::<i32>().ok()?)))
        .ok_or_else(|| {
            Error::config(format!(
                "invalid --horizon {:?} (expected LO,HI)",
                args.horizon
            ))
        })?;
    if hi < lo {
        return Err(Error::config("--horizon HI must be >= LO"));
    }
    let old_db = crate::load_database(&args.old)?;
    let new_db = crate::load_database(&args.new)?;
    let opts = ReleaseDiffOptions {
        horizon: (lo, hi),
        split: args.split,
        zone_filter: args.zone,
        zdump_program: args.reference_zdump,
    };
    let report = build_release_diff(&old_db, &new_db, &opts)?;
    match args.format {
        ReportFormatArg::Json => print!("{}", report.to_json()),
        ReportFormatArg::Text => {
            println!(
                "release-diff (horizon {}..{}, split {}, oracle={})",
                report.horizon.0,
                report.horizon.1,
                report.split,
                report.oracle_mode.mode_str()
            );
            for (kind, n) in report.kind_counts() {
                if n > 0 {
                    println!("  {n:5}  {kind}");
                }
            }
            if !report.errors.is_empty() {
                println!(
                    "  {} identifier(s) not comparable (outside zic-rs subset):",
                    report.errors.len()
                );
                for e in &report.errors {
                    println!("    {} — {}", e.name, e.reason);
                }
            }
        }
    }
    Ok(())
}

fn run_doctor(args: DoctorArgs) -> Result<()> {
    use crate::doctor::{run_doctor as probe, DoctorOptions};
    let report = probe(&DoctorOptions {
        reference_zic: args.reference_zic,
        reference_zdump: args.reference_zdump,
        tzdata: args.tzdata,
    })?;
    match args.format {
        ReportFormatArg::Json => print!("{}", report.to_json()),
        ReportFormatArg::Text => print!("{}", report.to_text()),
    }
    // `doctor` is a diagnosis, not a gate: always exit 0 (absent tools are reported, not errors).
    Ok(())
}

fn run_size_report(args: SizeReportArgs) -> Result<()> {
    use crate::size_report::{run_size_report as measure, SizeReportOptions};
    let report = measure(&SizeReportOptions { out: args.out })?;
    match args.format {
        ReportFormatArg::Json => print!("{}", report.to_json()),
        ReportFormatArg::Text => print!("{}", report.to_text()),
    }
    Ok(())
}

/// Parse `--mode` (reference `zic`'s `-m`) as an **octal** permission string (e.g. `644`, `0644`,
/// `0o600`) into permission bits. A leading `0o` is tolerated; a leading `0` is fine (octal radix).
/// Rejects non-octal input and values above `0o7777` as a config error — caught *before* any write.
/// zic-rs accepts only this octal subset; symbolic chmod expressions (`u+rwx`) are not parsed.
/// Load + parse an explicit leap-seconds source (reference `zic`'s `-L`), the `right/` build profile
/// (T11.6). `None` → ordinary profile (no leap table). A missing/unreadable file or a malformed
/// leap-source is a config error (caught before any output).
fn load_leap_table(path: Option<&std::path::Path>) -> Result<Option<crate::model::LeapTable>> {
    let Some(p) = path else { return Ok(None) };
    let bytes = std::fs::read(p).map_err(|e| Error::io(p, e))?;
    Ok(Some(crate::source::parse_leap_source(&bytes, p)?))
}

fn parse_octal_mode(raw: Option<&str>) -> Result<Option<u32>> {
    let Some(s) = raw else { return Ok(None) };
    let digits = s.strip_prefix("0o").unwrap_or(s);
    let bits = u32::from_str_radix(digits, 8)
        .map_err(|_| Error::config(format!("--mode {s:?} is not a valid octal mode (e.g. 644)")))?;
    if bits > 0o7777 {
        return Err(Error::config(format!(
            "--mode {s:?} is out of range (max 7777)"
        )));
    }
    Ok(Some(bits))
}

/// Reconcile the two spellings of the emission-bloat knob (T10.2). `--emit-style` is the native
/// surface; `-b {slim|fat}` is the reference-`zic` alias. When only one is given it decides; when
/// both are given they must agree (else "incompatible options", mirroring `zic`'s own `-b` check).
/// `-b` against a left-at-`default` `--emit-style` simply wins.
fn reconcile_emit_style(style: EmitStyleArg, bloat: Option<BloatArg>) -> Result<crate::EmitStyle> {
    match bloat {
        None => Ok(style.into()),
        Some(b) => {
            let from_bloat: EmitStyleArg = b.into();
            if style != EmitStyleArg::Default && style != from_bloat {
                let bn = match b {
                    BloatArg::Slim => "slim",
                    BloatArg::Fat => "fat",
                };
                let sn = match style {
                    EmitStyleArg::ZicSlim => "zic-slim",
                    EmitStyleArg::ZicFat => "zic-fat",
                    EmitStyleArg::Default => "default",
                };
                return Err(Error::config(format!(
                    "-b {bn} conflicts with --emit-style {sn} (incompatible emission options)"
                )));
            }
            Ok(from_bloat.into())
        }
    }
}

/// Parse reference `zic`'s `-R @hi` redundant-tail bound: an `@`-prefixed Unix-seconds instant
/// (e.g. `@4102444800`). Matches `zic`'s `redundant_time_option`, which **requires** the `@`. A
/// missing `@` or non-integer body is a config error — caught before any write.
fn parse_redundant_until(raw: Option<&str>) -> Result<Option<i64>> {
    let Some(s) = raw else { return Ok(None) };
    let body = s.strip_prefix('@').ok_or_else(|| {
        Error::config(format!(
            "-R expects an @-prefixed Unix-seconds instant (e.g. @4102444800), got {s:?}"
        ))
    })?;
    let secs = body
        .parse::<i64>()
        .map_err(|_| Error::config(format!("-R {s:?} is not a valid @<seconds> instant")))?;
    Ok(Some(secs))
}

/// Parse reference `zic`'s `-r '[@lo][/@hi]'` range spec into [`crate::RangeSpec`] (T10.4b). Grammar:
/// an optional `@lo`, an optional `/@hi`, at least one present; each `@`-prefixed Unix seconds. This
/// is **parse + validation only** — the bounds are not yet applied (T10.4d); the `hi -= 1` /
/// `limitrange` resolution is deferred. Rejects empty input, a missing `@`, a non-integer body,
/// trailing junk, and `hi < lo`.
fn parse_range(raw: Option<&str>) -> Result<Option<crate::RangeSpec>> {
    let Some(s) = raw else { return Ok(None) };
    let parse_at = |part: &str, which: &str| -> Result<i64> {
        part.strip_prefix('@')
            .and_then(|b| b.parse::<i64>().ok())
            .ok_or_else(|| {
                Error::config(format!(
                    "-r {which} bound {part:?} must be an @-prefixed Unix-seconds instant (e.g. @0)"
                ))
            })
    };
    // Grammar `[@lo][/@hi]`: split on the first `/`. No `/` → the whole thing is `@lo`.
    let (lo, hi) = match s.split_once('/') {
        Some((lo_str, hi_str)) => {
            let lo = if lo_str.is_empty() {
                None
            } else {
                Some(parse_at(lo_str, "lo")?)
            };
            (lo, Some(parse_at(hi_str, "hi")?))
        }
        None => (Some(parse_at(s, "lo")?), None),
    };
    if lo.is_none() && hi.is_none() {
        return Err(Error::config(
            "-r requires @lo and/or /@hi (e.g. @0/@4102444800)",
        ));
    }
    if let (Some(l), Some(h)) = (lo, hi) {
        if h < l {
            return Err(Error::config(format!("-r hi (@{h}) is before lo (@{l})")));
        }
    }
    Ok(Some(crate::RangeSpec { lo, hi }))
}

fn run_compile(args: CompileArgs) -> Result<()> {
    // Resolve the (borrowing) selection before consuming any owned fields of `args`.
    let zones = resolve_selection(&args)?;
    let output_dir = args
        .out
        .ok_or_else(|| Error::config("--out is required; there is no default output directory"))?;
    let db = crate::load_database_with(
        &args.input,
        crate::source::LegacySource {
            latin1: args.legacy_latin1,
            yearistype: args.legacy_yearistype,
        },
    )?;

    let config = CompileConfig {
        input_paths: args.input.clone(),
        output_dir,
        zones,
        link_mode: args.link_mode.into(),
        overwrite: args.force,
        unsupported_policy: args.unsupported.into(),
        transition_limit: args.transition_limit,
        emit_style: reconcile_emit_style(args.emit_style, args.bloat)?,
        no_create_dirs: args.no_create_dirs,
        localtime: args.localtime.clone(),
        localtime_name: args.localtime_name.clone(),
        file_mode: parse_octal_mode(args.mode.as_deref())?,
        redundant_until: parse_redundant_until(args.redundant_until.as_deref())?,
        range: parse_range(args.range.as_deref())?,
        leaps: load_leap_table(args.leapseconds.as_deref())?,
        allow_empty_footer_on_legacy_nonposix_recurrence: args.legacy_empty_footer,
    };

    let report = plan::run(&db, &config)?;

    // Deterministic, human-readable summary on stdout; diagnostics on stderr.
    for z in &report.zones_compiled {
        println!(
            "compiled {} -> {} (TZif v{}, {} transitions)",
            z.name,
            z.output_path.display(),
            z.tzif_version as char, // stored as the raw version byte (e.g. b'2')
            z.transition_count
        );
    }
    for l in &report.links_written {
        println!("linked {} -> {} ({:?})", l.link_name, l.target, l.mode);
    }
    // T13.6 — verbosity filter (mirrors reference `zic`'s `noise`/`-v` gating): always surface
    // `AlwaysOn` diagnostics; surface `VerboseOnly` ones (e.g. the "fewer than 3 characters"
    // abbreviation warning, the transition-count warning) only under `--verbose`/`-v`. The report
    // still *collects* everything (programmatic consumers + the comparison harness see it all); this
    // only governs what the CLI prints, so default output matches `zic` and `-v` matches `zic -v`.
    for d in &report.diagnostics {
        if args.verbose || d.verbosity == crate::diagnostics::DiagnosticVerbosity::AlwaysOn {
            eprintln!("{d}");
        }
    }

    // Optional producer-side artifacts (T3.4b/c). These describe *this* invocation only.
    if let Some(path) = &args.alias_map {
        let map = crate::manifest::build(&report, &config.output_dir)?;
        map.write_to(path)?;
        println!("alias-map -> {}", path.display());
    }
    if let Some(path) = &args.manifest {
        // `requested` is the resolved identifier list (so `--all-supported` is recorded in
        // full); the oracle is `not-run` because `compile` does not invoke `compare`.
        let requested = plan::select_zones(&db, &config.zones);
        let source_files = crate::collect_source_files(&config.input_paths)?;
        // Source-variant evidence axes (T12.4d `backward`; T12.5b `backzone`) — provenance only, from
        // explicit flags. `--backward`/`--backzone` are bare claims; `--backward-source` is a file whose
        // bytes are hash-checked. None affect compilation; absence leaves each axis `unknown` (backzone
        // detection is hash-anchored to the pinned 2026b reference regardless of the claim flag).
        let variants = crate::manifest::SourceVariantArgs {
            backward_claim: args.backward.as_deref().map(|v| v == "included"),
            backward_source: args.backward_source.clone(),
            backzone_claim: args.backzone.as_deref().map(|v| v == "included"),
            packratlist_claim: args.packratlist.clone(),
            packratlist_source: args.packratlist_source.clone(),
            dataform_claim: args.dataform.clone(),
        };
        let manifest = crate::manifest::build_compile_manifest(
            &requested,
            &source_files,
            &report,
            &config,
            &db,
            args.tzdb_version.as_deref(),
            args.leapseconds.as_deref(),
            &variants,
        )?;
        manifest.write_to(path)?;
        println!("manifest -> {}", path.display());
    }
    Ok(())
}

/// Turn the three mutually-exclusive selection flags into a [`ZoneSelection`].
fn resolve_selection(args: &CompileArgs) -> Result<ZoneSelection> {
    match (&args.zone, &args.zones, args.all_supported) {
        (Some(z), None, false) => Ok(ZoneSelection::One(z.clone())),
        (None, Some(path), false) => {
            let text = std::fs::read_to_string(path).map_err(|e| Error::io(path, e))?;
            let names: Vec<String> = text
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .map(|l| l.to_string())
                .collect();
            Ok(ZoneSelection::Many(names))
        }
        (None, None, true) => Ok(ZoneSelection::AllSupported),
        _ => Err(Error::config(
            "specify exactly one of --zone, --zones, or --all-supported",
        )),
    }
}

fn run_compare(args: CompareArgs) -> Result<()> {
    let db = crate::load_database(&args.input)?;
    // Reference `zic` takes files, not directories — expand inputs to the same flat file
    // list our own parser used, so both compilers see identical source.
    let files = crate::collect_source_files(&args.input)?;

    // Resolve the comparison mode (default zdump — the real behaviour oracle).
    let mode = match args.mode {
        CompareModeArg::Structural => crate::compare::CompareMode::Structural,
        CompareModeArg::Zdump => {
            let (lo, hi) = parse_horizon(&args.horizon)?;
            crate::compare::CompareMode::Zdump {
                program: args.zdump.clone(),
                lo,
                hi,
            }
        }
    };

    // A unique, self-cleaning working directory for scratch output (absolute path, which
    // `zdump` requires).
    let work = tempfile::Builder::new()
        .prefix("zic-rs-compare-")
        .tempdir()
        .map_err(|e| Error::io(std::env::temp_dir(), e))?;
    let cmp = crate::compare::compare_zone(
        &db,
        &files,
        &args.zone,
        &args.reference_zic,
        work.path(),
        &mode,
    )?;
    println!("{}", cmp.summary());
    if cmp.is_match() {
        Ok(())
    } else {
        Err(Error::message(format!(
            "{}: output disagrees with reference zic",
            args.zone
        )))
    }
}

/// Parse a `LO,HI` year horizon.
fn parse_horizon(s: &str) -> Result<(i32, i32)> {
    let (lo, hi) = s
        .split_once(',')
        .ok_or_else(|| Error::config(format!("--horizon must be `LO,HI`, got {s:?}")))?;
    let lo: i32 = lo
        .trim()
        .parse()
        .map_err(|_| Error::config(format!("invalid horizon start {lo:?}")))?;
    let hi: i32 = hi
        .trim()
        .parse()
        .map_err(|_| Error::config(format!("invalid horizon end {hi:?}")))?;
    if lo > hi {
        return Err(Error::config(format!(
            "--horizon start {lo} exceeds end {hi}"
        )));
    }
    Ok((lo, hi))
}

fn run_explain(args: ExplainArgs) -> Result<()> {
    let db = crate::load_database(&args.input)?;
    match plan::explain(&db, &args.zone) {
        Ok(s) => {
            println!("{s}");
            Ok(())
        }
        Err(d) => {
            // An "unsupported" explanation is informational, not a crash: print it and
            // exit non-zero so scripts can detect it.
            println!("{d}");
            Err(Error::message(format!("{} is not supported", args.zone)))
        }
    }
}

/// The human-readable supported-syntax summary (also the source of truth for
/// `docs/supported-syntax.md`'s prose).
pub fn supported_syntax_text() -> String {
    "\
zic-rs supported syntax (current declared subset)

Records (keywords accept zic-style unambiguous prefixes, incl. zishrink R/Z/L):
  Zone NAME STDOFF RULES FORMAT [UNTIL...]   (single or multi-era via UNTIL continuations)
    RULES = '-'        -> fixed-offset era (any constant standard offset)
    RULES = <clock>    -> inline-save era: fixed type at STDOFF+SAVE, is_dst set, literal/%z FORMAT
    RULES = <name>     -> rule set: finite (FROM..TO years) or recurring (TO = maximum)
  Rule NAME FROM TO - IN ON AT SAVE LETTER
  Link TARGET LINK-NAME                (copy or symlink; chains resolved)
  The installed single-file tzdata.zi (R/Z/L record keys) is read directly.

Offsets / times:
  -, integer hours, h:mm, h:mm:ss, signed; fractional seconds rounded to nearest.
  AT suffixes: w (wall, default), s (standard), u/g/z (universal).
  SAVE suffixes: s (standard), d (daylight); sign honoured.

ON day forms:
  numeric day, lastSun..lastSat, Sun>=N, Sun<=N (with month spill).

FORMAT:
  literal, %s (LETTER substitution), STD/DST slash, %z (numeric offset).

Footer (POSIX TZ):
  fixed offset for finite rule tails; recurring std/dst rule (e.g. EST5EDT,M3.2.0,M11.1.0)
  for TO = maximum rule sets with POSIX-expressible (nth/last weekday) day forms.

Multi-era zones:
  Cross-era state is carried correctly (UNTIL in the ending era's context with the
  prevailing save; footer from the final era). A final era whose finite rules all end
  before the era starts is classified by its EFFECTIVE in-era activations (recurring-only),
  which admits real zones such as Europe/London (first pinned IANA slice).

Output:
  Valid TZif version 2/3 (content-driven: v1 stub block + v2/v3 block + POSIX TZ footer;
  v3 only when a recurring rule's day form requires it).

  FROM = minimum is accepted as an obsolete spelling, coerced to 1900 (as reference zic).

NOT yet supported (rejected with an explicit diagnostic, never approximated):
  inline save with a %s or STD/DST slash FORMAT (a negative inline save IS supported, law 7),
  24:00/negative compiled times, recurring rules whose ON day is a fixed numeric day-of-month
  (the Sun<=N/Sat<=N weekday forms ARE supported, law 10), and leap seconds (-L).
Deferred operational modes: ownership (-u, privileged/Unix-only) and the legacy posixrules
link (-p). File mode (-m, octal, Unix-only) IS supported. See docs/unsupported-syntax.md and
docs/roadmap.md.
"
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        parse_octal_mode, parse_redundant_until, reconcile_emit_style, BloatArg, EmitStyleArg,
    };
    use crate::EmitStyle;

    #[test]
    fn range_parses_all_three_forms() {
        use super::parse_range;
        use crate::RangeSpec;
        assert_eq!(parse_range(None).unwrap(), None);
        // @lo only
        assert_eq!(
            parse_range(Some("@0")).unwrap(),
            Some(RangeSpec {
                lo: Some(0),
                hi: None
            })
        );
        // @lo/@hi
        assert_eq!(
            parse_range(Some("@0/@4102444800")).unwrap(),
            Some(RangeSpec {
                lo: Some(0),
                hi: Some(4102444800)
            })
        );
        // /@hi only
        assert_eq!(
            parse_range(Some("/@100")).unwrap(),
            Some(RangeSpec {
                lo: None,
                hi: Some(100)
            })
        );
    }

    #[test]
    fn range_rejects_malformed() {
        use super::parse_range;
        assert!(parse_range(Some("")).is_err()); // empty
        assert!(parse_range(Some("0/@1")).is_err()); // lo missing @
        assert!(parse_range(Some("@0/1")).is_err()); // hi missing @
        assert!(parse_range(Some("@abc")).is_err()); // non-integer
        assert!(parse_range(Some("@0/")).is_err()); // trailing slash, no @hi
        assert!(parse_range(Some("@10/@5")).is_err()); // hi < lo
        assert!(parse_range(Some("@1/@2/@3")).is_err()); // trailing junk
    }

    #[test]
    fn redundant_until_requires_at_prefix() {
        assert_eq!(parse_redundant_until(None).unwrap(), None);
        assert_eq!(
            parse_redundant_until(Some("@946684800")).unwrap(),
            Some(946684800)
        );
        assert_eq!(parse_redundant_until(Some("@-1")).unwrap(), Some(-1));
        // Missing `@` and non-integer bodies are errors (zic requires the `@`).
        assert!(parse_redundant_until(Some("946684800")).is_err());
        assert!(parse_redundant_until(Some("@abc")).is_err());
        assert!(parse_redundant_until(Some("@")).is_err());
    }

    #[test]
    fn bloat_alias_maps_onto_emit_style() {
        // `-b` alone decides; `--emit-style` left at Default.
        assert_eq!(
            reconcile_emit_style(EmitStyleArg::Default, Some(BloatArg::Slim)).unwrap(),
            EmitStyle::ZicSlim
        );
        assert_eq!(
            reconcile_emit_style(EmitStyleArg::Default, Some(BloatArg::Fat)).unwrap(),
            EmitStyle::ZicFat
        );
        // No `-b`: `--emit-style` passes through.
        assert_eq!(
            reconcile_emit_style(EmitStyleArg::ZicSlim, None).unwrap(),
            EmitStyle::ZicSlim
        );
        // Both given and agreeing: fine.
        assert_eq!(
            reconcile_emit_style(EmitStyleArg::ZicSlim, Some(BloatArg::Slim)).unwrap(),
            EmitStyle::ZicSlim
        );
    }

    #[test]
    fn bloat_conflicting_with_emit_style_is_error() {
        assert!(reconcile_emit_style(EmitStyleArg::ZicSlim, Some(BloatArg::Fat)).is_err());
        assert!(reconcile_emit_style(EmitStyleArg::ZicFat, Some(BloatArg::Slim)).is_err());
    }

    #[test]
    fn parses_octal_with_and_without_leading_zero() {
        assert_eq!(parse_octal_mode(Some("644")).unwrap(), Some(0o644));
        assert_eq!(parse_octal_mode(Some("0644")).unwrap(), Some(0o644));
        assert_eq!(parse_octal_mode(Some("0o600")).unwrap(), Some(0o600));
        assert_eq!(parse_octal_mode(Some("755")).unwrap(), Some(0o755));
        assert_eq!(parse_octal_mode(None).unwrap(), None);
    }

    #[test]
    fn rejects_non_octal_and_out_of_range() {
        // 8 and 9 are not octal digits.
        assert!(parse_octal_mode(Some("999")).is_err());
        assert!(parse_octal_mode(Some("64a")).is_err());
        // Above 0o7777 (perm + setuid/setgid/sticky bits).
        assert!(parse_octal_mode(Some("10000")).is_err());
    }
}
