//! `tzcompile` — a memory-safe Rust implementation of (a declared subset of) the IANA
//! timezone compiler `zic`.
//!
//! # What this crate is
//!
//! It compiles IANA *tzdata source text* (`Zone` / `Rule` / `Link` lines) into binary
//! **TZif** files as specified by [RFC 9636] / `tzfile(5)`. It is library-first: the CLI
//! in the `zic-rs` binary is a thin shell over the API exposed here.
//!
//! # Compiler pipeline
//!
//! ```text
//! source text  ->  lexer  ->  parser  ->  Database  ->  compile  ->  TzifData  ->  TZif bytes
//!                 (source::)          (model records)  (compile::)   (tzif::)
//! ```
//!
//! The companion [`compare`] module is an *oracle*: it can run the reference C `zic` and
//! diff its output against ours, semantically (the contract) and byte-for-byte (a stretch
//! goal for the simplest zones). It is the only place that ever executes an external
//! `zic`; the compile path never shells out.
//!
//! # Scope (current declared subset)
//!
//! `zic-rs` grows strictly by fixture class, so the supported surface is exactly what the
//! checked-in fixtures (and their reference-`zic`/`zdump` oracle results) prove. At present
//! that is: fixed-offset zones and `Link`s; finite named-`Rule` DST sets
//! (`lastSun`/`Sun>=N`/`Sun<=N`, `AT` suffixes `w`/`s`/`u`, `FORMAT` `%s`/slash/`%z`);
//! recurring (`TO=maximum`) rule sets summarised by an exact POSIX-TZ footer; and multi-era
//! `UNTIL` continuations (including the final-recurring-era anchor). Everything outside the
//! proven subset — e.g. inline-saving eras with a `%s`/slash `FORMAT` or a negative save,
//! `24:00`/negative compiled times, leap seconds — is **rejected with an explicit diagnostic**
//! rather than approximated (the fail-closed doctrine). (Accepted breadth: inline-save eras with
//! a literal/`%z` `FORMAT`; both mixed finite+recurring final-era shapes — effectively
//! recurring-only as in Europe/London, and genuinely mixed-in-era as in America/New_York; and
//! the obsolete `FROM=minimum`, coerced to 1900 like reference `zic`.) The authoritative,
//! always-current lists live in `docs/supported-syntax.md` and `docs/unsupported-syntax.md`;
//! the milestone history is in `docs/roadmap.md`.
//!
//! [RFC 9636]: https://www.rfc-editor.org/rfc/rfc9636

#![forbid(unsafe_code)]
#![warn(missing_debug_implementations)]

pub mod aux_tables;
pub mod cli;
pub mod compare;
pub mod compile;
pub mod diagnostics;
pub mod doctor;
pub mod error;
pub mod fs;
pub mod hash;
pub(crate) mod json;
pub mod limits;
pub mod manifest;
pub mod model;
pub mod release_diff;
pub mod report;
pub mod semantic_witness;
pub mod size_report;
pub mod source;
pub mod structural;
pub mod tzif;
pub mod vendor_oracle;

pub use diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticLayer, DiagnosticSpanPrecision, DiagnosticVerbosity,
    Severity,
};
pub use error::{Error, Result};

use std::path::{Path, PathBuf};

/// How `Link` records are materialised in the output tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinkMode {
    /// Copy the target file's bytes to the link name (portable; the default).
    #[default]
    Copy,
    /// Create a relative symbolic link to the target (smaller; Unix-only semantics).
    Symlink,
}

impl LinkMode {
    /// The boundary rendering used in the alias-map JSON (`"copy"` / `"symlink"`). Owning the literal
    /// here (CONTRACT.TYPING) means the alias-map's `materialised` field is a typed `LinkMode`, not a
    /// hand-emitted string that could drift from the actual materialisation policy.
    pub fn as_str(self) -> &'static str {
        match self {
            LinkMode::Copy => "copy",
            LinkMode::Symlink => "symlink",
        }
    }
}

/// Explicit-transition **emission style** (campaign T8-slim). This is an *emission policy* — it
/// never changes behaviour (every mode is `zdump`-equivalent: the POSIX footer governs the tail
/// identically); it only changes *how many explicit transitions* are written before the footer
/// takes over.
///
/// * [`EmitStyle::Default`] — zic-rs's behaviour-matched output: explicit transitions expanded
///   through `RECUR_HI`. This is the CORE.1-gated default and is **never** altered by this enum.
/// * [`EmitStyle::ZicSlim`] — reproduce reference `zic`'s slim (`-b slim`) output: drop the
///   footer-governed recurring tail, keeping explicit transitions only up to the first
///   footer-governed (recurring) transition (`zic.c`'s `keep_at_max = TZstarttime`). Byte parity is
///   claimed only in this mode, never for the default.
/// * [`EmitStyle::ZicFat`] — reference `zic`'s fat (`-b fat`) policy. zic-rs's default emission is
///   already fat-style, so this currently aliases [`EmitStyle::Default`]; kept distinct so the CLI
///   surface matches `zic` and a future redundant-tail refinement has a home.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmitStyle {
    /// Behaviour-matched default (fat-ish, CORE.1-gated). Never altered.
    #[default]
    Default,
    /// Reference `zic` slim: truncate the footer-governed recurring tail.
    ZicSlim,
    /// Reference `zic` fat (currently == [`EmitStyle::Default`]).
    ZicFat,
}

/// Full emission options: an [`EmitStyle`] plus the optional `-R` redundant-tail bound (T10.3).
///
/// `redundant_until` is reference `zic`'s `-R @hi`. It is meaningful **only with
/// [`EmitStyle::ZicSlim`]**: it keeps the otherwise-droppable, footer-governed explicit transitions
/// out to this UT instant (inclusive), so old readers that ignore the POSIX footer still see correct
/// timestamps up to that point. It is **not** fat mode (`-b`) and **not** range truncation (`-r`).
/// With the fat-style default it is a no-op (everything is kept already). [`EmitStyle`] converts into
/// `EmitOptions` with `redundant_until: None`, so callers that don't use `-R` are unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmitOptions {
    pub style: EmitStyle,
    pub redundant_until: Option<i64>,
    /// `-r '[@lo][/@hi]'` range truncation (T10.4d). A **distinct** input from
    /// [`Self::redundant_until`] — they both inform what is emitted but are never merged: `-R` widens
    /// what slim keeps, `-r` reshapes the representable interval (and introduces the `-00`
    /// "local time unspecified" boundary type(s)). `None` = no truncation.
    pub range: Option<RangeSpec>,
    /// **PERPETUAL-EXPANSION.1** (`--legacy-empty-footer`). Off by default. **Footer-emission policy
    /// only — it does not change transition generation.** When a zone's final recurring era **cannot**
    /// be reduced to one POSIX DST/standard footer pair (the live case: historical **perpetual
    /// year-parity** rules like `1990 max even/odd`, which a POSIX `TZ` string cannot encode), the
    /// default **fails closed** (`ZIC001`). When this is set, and *only* on that synthesis-failure
    /// path, zic-rs falls back to the explicit transitions it already expands through the replay
    /// horizon (`RECUR_HI` = 2037) plus an **empty footer** — frozen beyond the last explicit
    /// transition — matching the admitted historical `zic` oracle's *behaviour* over the declared
    /// `[1980, 2037]` window. It does **not** reproduce any oracle's private expansion constant
    /// byte-for-byte, and beyond the last explicit transition is intentionally **not** claimed. Only
    /// the failure path is affected, so footer-synthesising zones (all of CORE.1) are byte-unchanged.
    /// The field name is deliberately explicit to prevent it being mistaken for a general relaxation.
    pub allow_empty_footer_on_legacy_nonposix_recurrence: bool,
}

impl From<EmitStyle> for EmitOptions {
    fn from(style: EmitStyle) -> Self {
        EmitOptions {
            style,
            redundant_until: None,
            range: None,
            allow_empty_footer_on_legacy_nonposix_recurrence: false,
        }
    }
}

impl Default for EmitOptions {
    fn default() -> Self {
        EmitStyle::Default.into()
    }
}

/// Reference `zic`'s `-r '[@lo][/@hi]'` range-truncation bounds (T10.4), as **raw parsed** Unix-second
/// instants — `None` on a side means that side is unbounded (`zic`'s `min_time`/`max_time` default).
///
/// **T10.4b (parse-only):** this is currently parsed and validated but **not yet applied** — a value
/// fails closed rather than silently producing un-truncated output (range truncation lands in
/// T10.4d). When applied, truncation introduces a leading (and, if the end is cut, trailing)
/// **`-00` "local time unspecified"** type — it is *not* a plain transition filter, and `-r` is a
/// *different* input to the clamp than `-R` ([`CompileConfig::redundant_until`]); they do not merge.
/// The `hi -= 1` / `limitrange` resolution is deferred to T10.4d (the stored `hi` is the raw `@hi`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RangeSpec {
    pub lo: Option<i64>,
    pub hi: Option<i64>,
}

/// Policy for source constructs this version does not implement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UnsupportedPolicy {
    /// Abort the whole compilation (fail closed). The safe default.
    #[default]
    Error,
    /// Emit a diagnostic, skip the offending zone, and keep going.
    WarnAndSkipZone,
}

/// Selection of which zones (and their links) to compile.
#[derive(Debug, Clone)]
pub enum ZoneSelection {
    /// A single explicit zone name.
    One(String),
    /// An explicit list of zone names.
    Many(Vec<String>),
    /// Every zone the compiler currently supports; unsupported zones are reported.
    AllSupported,
}

/// Fully-resolved configuration for a compile run.
///
/// Built by the CLI but usable directly from the library.
#[derive(Debug, Clone)]
pub struct CompileConfig {
    /// Source files and/or directories to read tzdata from.
    pub input_paths: Vec<PathBuf>,
    /// Root of the output tree. Required — there is no implicit system default.
    pub output_dir: PathBuf,
    /// Which zones to compile.
    pub zones: ZoneSelection,
    /// How to write `Link`s.
    pub link_mode: LinkMode,
    /// Overwrite existing output files.
    pub overwrite: bool,
    /// What to do when a zone uses unsupported syntax.
    pub unsupported_policy: UnsupportedPolicy,
    /// Hard cap on transitions emitted per zone (defence against pathological input).
    pub transition_limit: usize,
    /// Explicit-transition emission style (T8-slim). [`EmitStyle::Default`] is behaviour-matched
    /// and CORE.1-gated; `zic-slim`/`zic-fat` are opt-in structural-parity modes.
    pub emit_style: EmitStyle,
    /// Do **not** create the `--out` tree (reference `zic`'s `-D`): if set, the output directory must
    /// already exist. Default `false` — zic-rs creates the (explicit, never system-default) `--out`
    /// root. The output-safety boundary is unchanged either way (T9.3).
    pub no_create_dirs: bool,
    /// Optional `localtime` install-policy target (reference `zic`'s `-l <zone>`): after a successful
    /// compile, create a `localtime` entry in the output tree linking the named (already-compiled)
    /// zone. `None` (default) writes no such link. Opt-in install policy — it **never** affects the
    /// canonical-zone behaviour sweep (CORE.1) and is materialised in phase 2 with the same
    /// no-partial-install guarantee as every other output file (T9.4).
    pub localtime: Option<String>,
    /// Name of the `localtime` link within the output tree (reference `zic`'s `-t`, default
    /// `"localtime"`). Constrained to a **safe relative name under `--out`** — unlike reference `zic`,
    /// zic-rs will not write the link to an arbitrary/absolute/system path (e.g. `/etc/localtime`);
    /// that is an *intentional safer divergence* (bucket 3). Ignored when [`Self::localtime`] is
    /// `None`.
    pub localtime_name: Option<String>,
    /// Optional file permission bits (reference `zic`'s `-m <mode>`), as parsed **octal** (e.g. `644`
    /// → `0o644`). Applied to the created regular files (compiled TZif zones and **copied** link
    /// files) after they are written; **not** applied to symlink entries (it would follow the link).
    /// `None` (default) leaves the process umask in effect. **Unix-only** install metadata: a value on
    /// a non-Unix platform is a config error caught *before any write*. Never affects the compiled
    /// bytes or the canonical-zone behaviour sweep (T9.5). Note: zic-rs accepts the **octal** subset of
    /// `zic`'s `-m`; symbolic chmod expressions (`u+rwx`) are not parsed.
    pub file_mode: Option<u32>,
    /// Optional `-R @hi` redundant-tail bound (T10.3): under [`EmitStyle::ZicSlim`], keep the
    /// otherwise-droppable footer-governed explicit transitions out to this UT instant (inclusive),
    /// for readers that ignore the POSIX footer. `None` (default) = pure slim. A no-op with the
    /// fat-style default (everything is already kept). Independent of [`Self::emit_style`]'s slim/fat
    /// choice — it only *widens* what slim keeps; it never changes behaviour (the kept transitions are
    /// redundant with the footer) and never the TZif version.
    pub redundant_until: Option<i64>,
    /// Optional `-r '[@lo][/@hi]'` range-truncation bounds (T10.4). **Parse-only for now** — a value
    /// is validated but **not yet applied**; the compile fails closed (rather than silently emitting
    /// un-truncated output) until T10.4d lands the clamp + the `-00` placeholder semantics.
    pub range: Option<RangeSpec>,
    /// Optional parsed leap-second table (reference `zic`'s `-L leapseconds`) — the **`right/` build
    /// profile** (T11.6). When `Some`, the leap table is applied to **every** compiled zone via
    /// [`compile::apply_leaps`]. `None` (default) is the ordinary/`posix` profile: no leap table, and
    /// ordinary canonical-zone conformance is unchanged. Opt-in; never the default.
    pub leaps: Option<model::LeapTable>,
    /// **PERPETUAL-EXPANSION.1** (`--legacy-empty-footer`). Off by default. A footer-emission fallback
    /// for historical zones whose final recurring era cannot be represented as one POSIX footer pair
    /// (perpetual year-parity). See [`EmitOptions::allow_empty_footer_on_legacy_nonposix_recurrence`].
    /// Never the default; never affects footer-synthesising zones (CORE.1).
    pub allow_empty_footer_on_legacy_nonposix_recurrence: bool,
}

/// Sensible, safe defaults (no default output dir — that is always explicit).
pub const DEFAULT_TRANSITION_LIMIT: usize = 100_000;

/// Per-zone outcome record for the compile report.
#[derive(Debug, Clone)]
pub struct ZoneReport {
    pub name: String,
    pub output_path: PathBuf,
    pub tzif_version: u8,
    pub transition_count: usize,
}

/// Per-link outcome record for the compile report.
#[derive(Debug, Clone)]
pub struct LinkReport {
    pub link_name: String,
    pub target: String,
    pub mode: LinkMode,
}

/// Structured, deterministic summary of a compile run.
#[derive(Debug, Default, Clone)]
pub struct CompileReport {
    pub zones_compiled: Vec<ZoneReport>,
    pub links_written: Vec<LinkReport>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Read every tzdata source file referenced by `paths` and parse it into a
/// [`Database`](model::Database).
///
/// Directories are read non-recursively for files ending in `.zi`/`.tab`-free tzdata
/// (we accept any regular file when given explicitly). Order is deterministic: paths in
/// the order given, files within a directory sorted by name.
pub fn load_database(paths: &[PathBuf]) -> Result<model::Database> {
    load_database_with(paths, source::LegacySource::default())
}

/// Like [`load_database`], but `legacy` opts into **bounded** historical-source replay modes
/// ([`source::LegacySource`]): Latin-1 comments (LEGACY-SOURCE.1) and/or the `yearistype` Rule `TYPE`
/// ecology (YEARISTYPE.1), for admitted historical tzdb releases. The modern source contract is
/// unchanged with the default (all-off) bundle.
pub fn load_database_with(
    paths: &[PathBuf],
    legacy: source::LegacySource,
) -> Result<model::Database> {
    // T17.1b: bound input-driven resource use (bucket-3 safer divergence — reference `zic` caps none
    // of these; the defaults sit far above any real tzdb, so legitimate input is never rejected).
    let limits = limits::ResourceLimits::default();
    let files = collect_source_files(paths)?;
    let mut db = model::Database::default();
    for f in &files {
        let bytes = std::fs::read(f).map_err(|e| Error::io(f, e))?;
        limits.check_source_bytes(bytes.len(), f)?;
        source::parse_into_with(&bytes, f, &mut db, legacy)?;
    }
    limits.enforce(&db)?;
    Ok(db)
}

/// Expand input paths into a deterministic, flat list of source files.
///
/// Directories are read non-recursively and their regular files included in name order;
/// explicitly-named files are taken as given. This is shared by [`load_database`] and the
/// `compare` oracle, so our parser and the reference `zic` are fed exactly the same files.
pub fn collect_source_files(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = Vec::new();
    for p in paths {
        if p.is_dir() {
            let mut entries: Vec<PathBuf> = std::fs::read_dir(p)
                .map_err(|e| Error::io(p, e))?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.is_file())
                .collect();
            entries.sort();
            files.extend(entries);
        } else {
            files.push(p.clone());
        }
    }
    Ok(files)
}

/// Compile one zone (by name) from a parsed database into in-memory TZif data.
///
/// This is the semantic heart of the compiler; it does not touch the filesystem.
pub fn compile_zone(db: &model::Database, name: &str) -> Result<tzif::TzifData> {
    compile::compile_zone(db, name)
}

/// Compile one zone with an explicit emission policy. Accepts a bare [`EmitStyle`] *or* a full
/// [`EmitOptions`] (style + `-R` redundant-tail bound). [`compile_zone`] is exactly this with
/// [`EmitStyle::Default`]; the default output is byte-for-byte unchanged from before T8-slim.
pub fn compile_zone_styled(
    db: &model::Database,
    name: &str,
    opts: impl Into<EmitOptions>,
) -> Result<tzif::TzifData> {
    compile::compile_zone_styled(db, name, opts.into())
}

/// Convenience: compile a zone straight to TZif bytes (default emission style).
pub fn compile_zone_to_bytes(db: &model::Database, name: &str) -> Result<Vec<u8>> {
    let data = compile_zone(db, name)?;
    tzif::write_bytes(&data)
}

/// Convenience: compile a zone straight to TZif bytes with an explicit emission policy
/// ([`EmitStyle`] or [`EmitOptions`]).
pub fn compile_zone_to_bytes_styled(
    db: &model::Database,
    name: &str,
    opts: impl Into<EmitOptions>,
) -> Result<Vec<u8>> {
    let data = compile_zone_styled(db, name, opts.into())?;
    tzif::write_bytes(&data)
}

/// Resolve a link target name to the underlying canonical zone name, following link chains.
///
/// Distinguishes the two failure modes a downstream consumer cares about: a **cycle**
/// (`A → B → A`, which would otherwise loop forever) versus a **missing target** (the chain
/// ends at a name that is neither a zone nor a link). Both return a clear message; the cycle
/// case reports the path it detected so the offending `Link` lines are easy to find.
pub fn resolve_link_target<'a>(db: &'a model::Database, name: &'a str) -> Result<&'a str> {
    let mut current = name;
    // Track visited link names to detect a cycle precisely (rather than bounding hops).
    let mut visited: Vec<&'a str> = Vec::new();
    loop {
        if db.zones.iter().any(|z| z.name == current) {
            return Ok(current);
        }
        // T17.1b: bound a pathological *acyclic* chain (the cycle check below catches loops, but a
        // straight chain of millions of links would still run — and cost O(n²) via `visited.contains`).
        if visited.len() >= limits::DEFAULT_LINK_CHAIN_DEPTH_MAX {
            return Err(Error::config(format!(
                "link chain for {name:?} exceeds the zic-rs depth limit of {} hops",
                limits::DEFAULT_LINK_CHAIN_DEPTH_MAX
            )));
        }
        if visited.contains(&current) {
            visited.push(current);
            return Err(Error::message(format!(
                "link chain for {name:?} forms a cycle: {}",
                visited.join(" -> ")
            )));
        }
        match db.links.iter().find(|l| l.link_name == current) {
            Some(l) => {
                visited.push(current);
                current = &l.target;
            }
            None => {
                return Err(Error::message(format!(
                    "link target {current:?} does not name a zone or link (resolving {name:?})"
                )))
            }
        }
    }
}

/// True when `path` is strictly contained within `root` after normalising `.`/`..`.
///
/// Used by the output tree; exposed here so tests can exercise it directly.
pub fn is_contained(root: &Path, candidate: &Path) -> bool {
    fs::output_tree::is_contained(root, candidate)
}
