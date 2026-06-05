//! Orchestrating a whole compile run: selection → compile each zone → write files → links.
//!
//! This is the library entry point the CLI calls. Keeping it here (rather than in `cli`)
//! means the full compile pipeline is usable as an API, per the project's library-first
//! design. It returns a deterministic [`CompileReport`] and writes only under the
//! configured output root.

use crate::compile::{compile_zone, compile_zone_styled};
use crate::diagnostics::Diagnostic;
use crate::error::{Error, Result};
use crate::fs::output_tree;
use crate::model::{Database, ZoneEra, ZoneRules};
use crate::tzif::write_bytes;
use crate::{
    CompileConfig, CompileReport, LinkReport, UnsupportedPolicy, ZoneReport, ZoneSelection,
};

/// One zone successfully compiled in the staging phase, held in memory until *all* selected zones
/// are known good — so a later fatal failure never leaves a half-written output tree (T9.3).
struct Staged {
    name: String,
    bytes: Vec<u8>,
    version: u8,
    transition_count: usize,
}

/// The portable abbreviation-length range. tzfile(5) recommends **3–6 ASCII characters** for POSIX
/// compatibility; reference `zic` warns (always-on, non-fatal) for anything shorter or longer —
/// pinned empirically against tzcode 2026b (length 3–6 silent; ≤ 2 → "fewer than 3", ≥ 7 → "too
/// many"). Counted in **characters**, matching `zic`'s `strlen` over the (ASCII) abbreviation.
const MIN_PORTABLE_ABBR_LEN: usize = 3;
const MAX_PORTABLE_ABBR_LEN: usize = 6;

/// Whether `c` is in the POSIX-portable abbreviation character set: ASCII alphanumeric (any case) or
/// `+`/`-`. Reference `zic` warns for anything else ("differs from POSIX standard"); case is fine.
fn is_posix_abbr_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '+' || c == '-'
}

/// T13.3/T13.4 — append non-fatal abbreviation **warnings** for each *distinct* abbreviation a compiled
/// zone emits, located at the zone's source origin (mirroring reference `zic`, which reports them at
/// the `Zone` line). This is a **direct port of `zic.c::checkabbr`** so the class matches reference
/// exactly — in particular its three subtleties (verified against tzcode 2026b):
///
/// 1. **Exactly one warning per abbreviation**, by precedence — `zic` walks the leading run of
///    POSIX-portable characters and assigns a single `mp` through sequential `if`s, so the *last*
///    matching condition wins: **non-POSIX char > too-many > fewer-than-3**. (So `"A_"` warns only
///    `NotPosix`, never *also* "fewer than 3" — an earlier independent-checks version got this wrong.)
/// 2. Length is measured over the **conforming prefix** (`cp - string`), not the whole string; for a
///    fully-conforming abbreviation that equals its length, and for a non-conforming one the POSIX
///    rule wins anyway.
/// 3. **"fewer than 3" is `noise`(`-v`)-gated in `zic`**; "too many" and "differs from POSIX" are
///    always-on. zic-rs has no verbosity tiers — it always collects all three (its diagnostic set
///    corresponds to `zic -v`), a documented verbosity divergence, never a class/location one.
///
/// A **pure observation over the finished `TzifData`** — never mutates `data`, fails the compile, or
/// changes the exit status, so CORE.1 output is untouched (the canonical zones have only 3–6-char
/// alnum/`+`/`-` abbreviations, so nothing fires on the real sweep). Distinct abbreviations are
/// reported once each, in first-seen type order.
fn collect_abbreviation_warnings(
    db: &Database,
    name: &str,
    data: &crate::tzif::TzifData,
    out: &mut Vec<Diagnostic>,
) {
    use crate::diagnostics::{Diagnostic, DiagnosticCode};
    // Source location for the warning: the zone's defining line. (A link/alias resolves to its
    // canonical zone before we get here, so `name` is always a real zone in `db`.)
    let origin = match db.zones.iter().find(|z| z.name == name) {
        Some(z) => &z.origin,
        None => return, // defensive: nothing to locate against (should not happen)
    };
    let mut seen: Vec<&str> = Vec::new();
    for ty in &data.types {
        let abbr = ty.abbr.as_str();
        if abbr.is_empty() || seen.contains(&abbr) {
            continue; // empty designations are an internal concern, not a user abbreviation
        }
        seen.push(abbr);

        // `cp - string`: the length of the leading POSIX-conforming run. Equal to the full length iff
        // every character conforms; shorter iff a non-conforming character appears.
        let conforming_prefix = abbr.chars().take_while(|&c| is_posix_abbr_char(c)).count();
        let total = abbr.chars().count();

        // Single warning, `checkabbr` precedence (last-assigned wins): POSIX > too-many > fewer-than-3.
        // The `verbose` flag mirrors `zic`'s `noise` gating (T13.5): only "fewer than 3" is `-v`-gated.
        let (code, msg, verbose) = if conforming_prefix < total {
            (
                DiagnosticCode::AbbreviationNotPosix,
                format!("time zone abbreviation {abbr:?} differs from the POSIX standard (non-alphanumeric, non-+/- character)"),
                false,
            )
        } else if conforming_prefix > MAX_PORTABLE_ABBR_LEN {
            (
                DiagnosticCode::AbbreviationPolicyViolation,
                format!("time zone abbreviation {abbr:?} has too many characters ({total} > {MAX_PORTABLE_ABBR_LEN})"),
                false,
            )
        } else if conforming_prefix < MIN_PORTABLE_ABBR_LEN {
            (
                DiagnosticCode::AbbreviationPolicyViolation,
                format!("time zone abbreviation {abbr:?} has fewer than {MIN_PORTABLE_ABBR_LEN} characters ({total})"),
                true, // reference `zic` gates this one behind `-v` (noise) — T13.5 verbosity model
            )
        } else {
            continue; // conforming, 3–6 chars → no warning
        };
        let mut d = Diagnostic::warning(code, msg, &origin.file, origin.line);
        if verbose {
            d = d.verbose_only();
        }
        out.push(d);
    }
}

/// Reference `zic`'s portable component-length cap (`componentcheck`'s `component_len_max`).
const ZONE_NAME_COMPONENT_LEN_MAX: usize = 14;

/// A byte in reference `zic`'s portable "benign" name set *within a component*: ASCII letters, `-`, `_`
/// (`/` is the separator, handled by the caller). Bytes outside this set — digits, `+`, `.`, space,
/// punctuation, control/high bytes — are non-portable (reference `-v`-warns, `namecheck`).
fn is_benign_name_byte(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'-' || b == b'_'
}

/// T14.5 — non-fatal, **verbose-only** zone/link *name* portability warnings, mirroring reference
/// `zic`'s `-v`-gated `namecheck`/`componentcheck`: `ZIC024` for a byte outside the benign set,
/// `ZIC025` for a component longer than 14 bytes. **Pure observation over the name string** — it never
/// alters the compiled bytes or the exit status (the *fatal* structural path rules — absolute · `..` ·
/// `//` · empty · trailing `/` — live in `output_tree::safe_relative_path` as `ZIC008`). One warning of
/// each kind per name (the first offender), bounded to avoid per-byte spam. Verbose-only: zic-rs has no
/// quiet mode, so the report always collects these but the CLI prints them only under `--verbose`.
fn collect_zone_name_warnings(
    name: &str,
    file: &std::path::Path,
    line: usize,
    out: &mut Vec<Diagnostic>,
) {
    use crate::diagnostics::{Diagnostic, DiagnosticCode};
    if let Some(&b) = name
        .as_bytes()
        .iter()
        .find(|&&b| b != b'/' && !is_benign_name_byte(b))
    {
        let shown = if b.is_ascii_graphic() {
            format!("'{}'", b as char)
        } else {
            format!("byte \\{b:03o}")
        };
        out.push(
            Diagnostic::warning(
                DiagnosticCode::ZoneNameNonPortableByte,
                format!(
                    "zone/link name {name:?} contains non-portable {shown} (portable set: ASCII letters, '-', '_')"
                ),
                file,
                line,
            )
            .verbose_only(),
        );
    }
    if let Some(comp) = name
        .split('/')
        .find(|c| c.len() > ZONE_NAME_COMPONENT_LEN_MAX)
    {
        out.push(
            Diagnostic::warning(
                DiagnosticCode::ZoneNameOverlengthComponent,
                format!(
                    "zone/link name {name:?} has an overlength component {comp:?} (> {ZONE_NAME_COMPONENT_LEN_MAX} bytes)"
                ),
                file,
                line,
            )
            .verbose_only(),
        );
    }
}

/// The legacy-client transition-count threshold, pinned from `zic.c` (`if (1200 < timecnt)` →
/// "pre-2014 clients may mishandle more than 1200 transition times"). `noise`/`-v`-gated in reference,
/// non-fatal.
const TRANSITION_WARN_THRESHOLD: usize = 1200;

/// T13.6 — append a non-fatal, **verbose-only** [`DiagnosticCode::TooManyTransitionsForLegacyClient`]
/// (`ZIC020`) warning when a zone's **emitted** transition count exceeds the legacy-client threshold,
/// mirroring reference `zic`'s `-v` warning. **Read-only over `data`** — it observes
/// `data.transitions.len()` (the count for the active emit style) and never mutates bytes/types/exit,
/// so CORE.1 output is untouched. *Style note (honest):* the count is the emitted count, so the
/// default fat style can cross the threshold where reference's slim default would not; the warning is
/// `VerboseOnly` (quiet by default) and is about the *emitted* stream, so this is consistent.
fn collect_transition_count_warning(
    db: &Database,
    name: &str,
    data: &crate::tzif::TzifData,
    out: &mut Vec<Diagnostic>,
) {
    use crate::diagnostics::{Diagnostic, DiagnosticCode};
    let timecnt = data.transitions.len();
    if timecnt <= TRANSITION_WARN_THRESHOLD {
        return;
    }
    let origin = match db.zones.iter().find(|z| z.name == name) {
        Some(z) => &z.origin,
        None => return,
    };
    out.push(
        Diagnostic::warning(
            DiagnosticCode::TooManyTransitionsForLegacyClient,
            format!(
                "{timecnt} transition times — more than {TRANSITION_WARN_THRESHOLD}; pre-2014 clients may mishandle this zone"
            ),
            &origin.file,
            origin.line,
        )
        .verbose_only(),
    );
}

/// 24 hours in seconds. Reference `zic`'s `gethms` warns when a value's magnitude is **strictly greater**
/// than this (exactly `24:00:00` is allowed); the check is on the unsigned magnitude.
const TWENTY_FOUR_HOURS_SECS: i64 = 24 * 60 * 60;

/// True when a parsed time value's magnitude exceeds 24:00:00 (reference `gethms`:
/// `hh > HOURSPERDAY || (hh == HOURSPERDAY && (mm||ss))`). `unsigned_abs()` because the sign is stripped
/// before the magnitude test, exactly as reference handles the leading `-`.
fn value_over_24h(seconds: i32) -> bool {
    i64::from(seconds).unsigned_abs() > TWENTY_FOUR_HOURS_SECS as u64
}

/// T15.5-remainder — non-fatal, **verbose-only** `ZIC026` warnings for any `STDOFF` / `SAVE` / `AT` /
/// `UNTIL` time value whose magnitude exceeds 24:00:00, mirroring reference `zic`'s `noise`/`-v`-gated
/// `gethms` "values over 24 hours not handled by pre-2007 versions of zic" (closes the T14.4 ledger
/// residual). **Pure observation over the parsed `Database`** — it never alters compiled bytes or exit
/// status (canonical `tzdata.zi` has no such value → 0 on the sweep, byte output unchanged).
///
/// *Scope note (honest):* like reference's parse-time emission, this is **file-wide** (every rule and
/// every zone era in the database), not scoped to the selected-zone set — so under a selective
/// `--zone X` compile it may still warn about other entries, exactly as reference (which always parses
/// the whole file) would. One warning per offending field, at that field's source line.
fn collect_over_24h_warnings(db: &Database, out: &mut Vec<Diagnostic>) {
    use crate::diagnostics::{Diagnostic, DiagnosticCode};
    let mut warn = |seconds: i32, kind: &str, file: &std::path::Path, line: usize| {
        if value_over_24h(seconds) {
            out.push(
                Diagnostic::warning(
                    DiagnosticCode::ValueOverTwentyFourHours,
                    format!(
                        "{kind} value {seconds}s exceeds 24:00:00 — not handled by pre-2007 versions of zic"
                    ),
                    file,
                    line,
                )
                .verbose_only(),
            );
        }
    };
    // Rule lines: the `AT` time of day and the `SAVE` amount (reference calls `gethms` for each).
    for recs in db.rules.values() {
        for r in recs {
            warn(r.at.seconds, "rule AT", &r.origin.file, r.origin.line);
            warn(r.save.seconds, "rule SAVE", &r.origin.file, r.origin.line);
        }
    }
    // Zone eras: the `STDOFF`, an inline `SAVE` (when `RULES` is a bare offset), and the `UNTIL` time.
    for z in &db.zones {
        for era in &z.eras {
            warn(era.stdoff.0, "STDOFF", &era.origin.file, era.origin.line);
            if let crate::model::ZoneRules::Save(s) = &era.rules {
                warn(s.seconds, "inline SAVE", &era.origin.file, era.origin.line);
            }
            if let Some(until) = &era.until {
                warn(
                    until.time.seconds,
                    "UNTIL",
                    &era.origin.file,
                    era.origin.line,
                );
            }
        }
    }
}

/// Run a full compilation according to `config`, writing output and returning a report.
///
/// **No partial install after a fatal (T9.3, intentional safer divergence vs reference `zic`).** The
/// run has two phases: (1) compile **every** selected zone to memory — serialising its bytes and
/// pre-validating its output path — so any fatal (parse error, unsupported construct under the
/// default `error` policy, oversized output, path-traversal name) aborts *before a single file is
/// written*; (2) materialise the staged bytes + links. A failure in phase 1 leaves the output tree
/// untouched (under `--no-create-dirs` it is never created at all). The hard output-safety boundary
/// (`ZIC008`, atomic no-clobber) is unchanged; this only moves the failure point earlier.
pub fn run(db: &Database, config: &CompileConfig) -> Result<CompileReport> {
    let mut report = CompileReport::default();

    // T15.5-remainder — file-wide `ZIC026` "values over 24 hours" warnings (verbose-only, read-only),
    // emitted once at parse time over the whole database, mirroring reference `zic`'s `gethms` check.
    collect_over_24h_warnings(db, &mut report.diagnostics);

    // A user may select a zone by a `Link` alias (e.g. `--zone UTC`, where `UTC` is an
    // alias of `Etc/UTC`). Resolve each requested name to its canonical zone before
    // compiling, de-duplicating so two aliases of one zone compile it once. The alias files
    // themselves are written by the link loop below.
    let requested = select_zones(db, &config.zones);
    let mut canonical_zones: Vec<String> = Vec::new();
    for name in &requested {
        let canonical = canonical_zone_name(db, name);
        if !canonical_zones.contains(&canonical) {
            canonical_zones.push(canonical);
        }
    }

    // T9.4 — validate the optional `localtime` install-policy target *before* any write, so a bad
    // request (a zone that was not selected, or an unsafe link name) is a phase-1 fatal and never a
    // partial install. The link itself is materialised in phase 2, after its target zone is on disk.
    // The link name is constrained to a safe relative name under `--out` — reference `zic` would
    // write to an arbitrary/absolute path (e.g. `/etc/localtime`); zic-rs deliberately will not
    // (intentional safer divergence, bucket 3).
    let localtime_link: Option<(String, String)> = match &config.localtime {
        None => None,
        Some(zone) => {
            let canonical = canonical_zone_name(db, zone);
            if !canonical_zones.contains(&canonical) {
                return Err(Error::config(format!(
                    "--localtime {zone:?} resolves to {canonical:?}, which is not among the \
                     selected zones; also select it (e.g. --zone {canonical}, or --all-supported)"
                )));
            }
            let name = config.localtime_name.as_deref().unwrap_or("localtime");
            output_tree::safe_relative_path(name)?; // reject absolute / traversal / unsafe up front
            Some((name.to_string(), canonical))
        }
    };

    // T9.5 — `--mode` (file permission bits) is **Unix-only** install metadata. Reject it up front on
    // other platforms so nothing is written; it never touches the compiled bytes (CORE.1). The octal
    // value itself was validated at the CLI boundary.
    if config.file_mode.is_some() && !cfg!(unix) {
        return Err(Error::config(
            "--mode (file permission bits) is only supported on Unix platforms",
        ));
    }

    // --- Phase 1: compile every selected zone to memory (no filesystem writes yet). ---
    let mut staged: Vec<Staged> = Vec::new();
    for name in &canonical_zones {
        let emit = crate::EmitOptions {
            style: config.emit_style,
            redundant_until: config.redundant_until,
            range: config.range,
            allow_empty_footer_on_legacy_nonposix_recurrence: config
                .allow_empty_footer_on_legacy_nonposix_recurrence,
        };
        match compile_zone_styled(db, name, emit) {
            Ok(mut data) => {
                // T11.6 — `right/` build profile: when an explicit leap-source was given
                // (`--leapseconds`), apply its leap table to **every** compiled zone (reference
                // `zic -L`). This is opt-in and never the default; a fatal here (e.g. `Rolling` +
                // `-r`) is a phase-1 abort, so nothing is written. Leaps are orthogonal — only the
                // leap table (and `version` for `Expires`) change, not the zone's transitions/types.
                if let Some(table) = &config.leaps {
                    crate::compile::apply_leaps(&mut data, table, config.range)?;
                }
                // T13.3 — non-fatal **warning** collection: flag any abbreviation longer than the
                // portable maximum (> 6 chars), matching reference `zic`'s always-on "too many
                // characters" warning. This is **read-only over `data`** — it never alters the
                // compiled bytes, the type table, or the exit status, so CORE.1 output is untouched;
                // it only appends to `report.diagnostics` (which the CLI prints to stderr like any
                // other diagnostic). Warnings are structured artifacts, never bolted-on stderr text.
                collect_abbreviation_warnings(db, name, &data, &mut report.diagnostics);
                // T13.6 — transition-count client-compat warning (`ZIC020`, verbose-only), read-only.
                collect_transition_count_warning(db, name, &data, &mut report.diagnostics);
                // T14.5 — zone *name* portability warnings (`ZIC024`/`ZIC025`, verbose-only), located at
                // the zone's defining line. The fatal structural path rules are enforced separately by
                // `safe_relative_path` below; these are the softer `zic -v` `namecheck` portability axis.
                if let Some(z) = db.zones.iter().find(|z| z.name == *name) {
                    collect_zone_name_warnings(
                        name,
                        &z.origin.file,
                        z.origin.line,
                        &mut report.diagnostics,
                    );
                }

                // Serialise now so a writer error (e.g. an over-255-byte abbreviation table) is a
                // phase-1 fatal — never a partial write — and pre-validate the output path so a
                // traversal/unsafe zone name (`ZIC008`) also aborts before anything is written.
                let bytes = write_bytes(&data)?;
                output_tree::safe_relative_path(name)?;
                staged.push(Staged {
                    name: name.clone(),
                    bytes,
                    version: data.version,
                    transition_count: data.transitions.len(),
                });
            }
            Err(e) => match (config.unsupported_policy, e.diagnostic()) {
                // Fail-closed is the default; warn-and-skip downgrades to a diagnostic so a
                // bulk run can make progress while still reporting exactly what it skipped.
                (UnsupportedPolicy::WarnAndSkipZone, Some(d)) => {
                    let mut warn = d.clone();
                    warn.severity = crate::Severity::Warning;
                    report.diagnostics.push(warn);
                }
                _ => return Err(e),
            },
        }
    }

    // --- Phase 2: materialise. Everything below is known-good, so output is all-or-nothing. ---
    // The output directory is always explicit (never a system default — there is no implicit
    // `/usr/share/zoneinfo`). `--no-create-dirs` (`-D`) requires it to pre-exist instead.
    if config.no_create_dirs {
        if !config.output_dir.is_dir() {
            return Err(Error::config(format!(
                "output directory {} does not exist (and --no-create-dirs was given)",
                config.output_dir.display()
            )));
        }
    } else {
        std::fs::create_dir_all(&config.output_dir)
            .map_err(|e| Error::io(&config.output_dir, e))?;
    }

    let mut compiled: Vec<String> = Vec::new();
    // ZIC-MATRIX.1.D — `--no-create-dirs` (`-D`) matches reference `zic -D`: a zone whose parent
    // subdirectory does not already exist is *not* materialised (no directory is ever created), the
    // remaining zones are still written, and the run fails (exit 1) if any zone was skipped this way.
    let mut skipped_no_dir: Vec<String> = Vec::new();
    for z in &staged {
        if config.no_create_dirs && parent_dir_missing(&config.output_dir, &z.name)? {
            skipped_no_dir.push(z.name.clone());
            continue;
        }
        // `durable = true`: the install path requests content-fsync + atomic-publish + parent-dir-fsync
        // (T17.4). A crash after a successful run leaves each written file durably published. Under `-D`
        // the parent already exists (checked above), so `write_zone_file`'s `create_dir_all` is a no-op.
        let path = output_tree::write_zone_file(
            &config.output_dir,
            &z.name,
            &z.bytes,
            config.overwrite,
            true,
        )?;
        if let Some(mode) = config.file_mode {
            output_tree::set_file_mode(&path, mode)?; // T9.5: `-m` on the created TZif file
        }
        report.zones_compiled.push(ZoneReport {
            name: z.name.clone(),
            output_path: path,
            tzif_version: z.version,
            transition_count: z.transition_count,
        });
        compiled.push(z.name.clone());
    }

    // Links: write any link whose ultimate target was compiled in this run. We materialise
    // each link against its *resolved canonical zone* — not the immediate `target` — so a
    // multi-hop chain (`Link A B; Link B C`) is order-independent and always points at a
    // real, freshly-written zone file rather than a possibly-not-yet-written intermediate.
    for link in &db.links {
        let canonical = match crate::resolve_link_target(db, &link.link_name) {
            Ok(t) => t.to_string(),
            Err(_) => continue, // chain doesn't terminate at a real zone; skip quietly
        };
        if !compiled.contains(&canonical) {
            continue;
        }
        if config.no_create_dirs && parent_dir_missing(&config.output_dir, &link.link_name)? {
            skipped_no_dir.push(link.link_name.clone());
            continue;
        }
        let link_path = output_tree::write_link(
            &config.output_dir,
            &link.link_name,
            &canonical,
            config.link_mode,
            config.overwrite,
            true, // durable: install path (T17.4)
        )?;
        apply_link_mode(&link_path, config)?; // T9.5: `-m` on a *copied* link (never a symlink)

        // T14.5 — the link *name* is a new output path: warn on its name portability too (verbose-only).
        collect_zone_name_warnings(
            &link.link_name,
            &link.origin.file,
            link.origin.line,
            &mut report.diagnostics,
        );
        report.links_written.push(LinkReport {
            link_name: link.link_name.clone(),
            target: canonical,
            mode: config.link_mode,
        });
    }

    // T9.4 — the optional `localtime` install-policy link (`-l`/`-t`), materialised last so its
    // target zone is already on disk. Validated in phase 1, so this is the only place it can fail
    // is a genuine write error. If the target was *selected but skipped* (only possible under
    // `--unsupported skip`), we emit a diagnostic rather than write a dangling link.
    if let Some((name, canonical)) = localtime_link {
        if config.no_create_dirs && parent_dir_missing(&config.output_dir, &name)? {
            skipped_no_dir.push(name.clone());
        } else if compiled.contains(&canonical) {
            let link_path = output_tree::write_link(
                &config.output_dir,
                &name,
                &canonical,
                config.link_mode,
                config.overwrite,
                true, // durable: install path (T17.4)
            )?;
            apply_link_mode(&link_path, config)?;
            report.links_written.push(LinkReport {
                link_name: name,
                target: canonical,
                mode: config.link_mode,
            });
        } else {
            report.diagnostics.push(Diagnostic::warning(
                crate::DiagnosticCode::UnsupportedDirective,
                format!(
                    "localtime target {canonical:?} was skipped (unsupported); no {name:?} link written"
                ),
                std::path::Path::new("<output>"),
                0,
            ));
        }
    }

    // ZIC-MATRIX.1.D — match reference `zic -D`: if any zone/link was skipped because its parent
    // directory was absent (and `-D` forbade creating it), the written zones remain but the run
    // fails (exit 1), naming what was not materialised. With no skips (`-D` over a complete tree)
    // this is unreached and the run exits 0, exactly like reference `zic -D`.
    if config.no_create_dirs && !skipped_no_dir.is_empty() {
        return Err(Error::config(format!(
            "{} output(s) not materialised under --no-create-dirs (-D): parent directory does not exist: {}",
            skipped_no_dir.len(),
            skipped_no_dir.join(", ")
        )));
    }

    Ok(report)
}

/// Under `--no-create-dirs` (`-D`): does the output path for `name` have a parent directory that
/// does **not** already exist? (A missing parent means reference `zic -D` would refuse to create it.)
/// The output root itself is checked separately; this is the per-zone/-link subdirectory check.
fn parent_dir_missing(root: &std::path::Path, name: &str) -> Result<bool> {
    let target = root.join(crate::fs::safe_relative_path(name)?);
    Ok(target.parent().map(|p| !p.is_dir()).unwrap_or(false))
}

/// Apply `--mode` (T9.5) to a just-written link entry, but **only when it is a copied regular file**.
/// A `symlink`-mode entry is left untouched — chmod follows the link and would alter the *target*'s
/// permissions, which is never what `-m` means. No-op when `--mode` is unset.
fn apply_link_mode(link_path: &std::path::Path, config: &CompileConfig) -> Result<()> {
    match (config.file_mode, config.link_mode) {
        (Some(mode), crate::LinkMode::Copy) => output_tree::set_file_mode(link_path, mode),
        _ => Ok(()),
    }
}

/// Map a requested name to the canonical zone to compile: the name itself if it is a real
/// zone, otherwise the zone its link chain resolves to (falling back to the name unchanged
/// so an unknown name surfaces a clear "no such zone" error at compile time).
fn canonical_zone_name(db: &Database, name: &str) -> String {
    if db.zone(name).is_some() {
        return name.to_string();
    }
    crate::resolve_link_target(db, name)
        .map(|t| t.to_string())
        .unwrap_or_else(|_| name.to_string())
}

/// Resolve a [`ZoneSelection`] into a concrete, deterministic list of zone names.
///
/// Public so the provenance manifest can record exactly which identifiers were *requested*
/// (e.g. `--all-supported` expands to the full source-order zone list).
pub fn select_zones(db: &Database, sel: &ZoneSelection) -> Vec<String> {
    match sel {
        ZoneSelection::One(z) => vec![z.clone()],
        ZoneSelection::Many(zs) => zs.clone(),
        ZoneSelection::AllSupported => {
            // Deterministic order: source order of zones.
            db.zones.iter().map(|z| z.name.clone()).collect()
        }
    }
}

/// Produce a human-readable **evidence trace** for one zone (or link alias): the decoded
/// source eras, the compiled local-time-types, the transitions (with UT instants decoded to
/// readable timestamps and the type each switches to), the footer, and notes that surface the
/// subtle bits (e.g. an equal-`utoff` boundary that is still a real type change). For an
/// unsupported zone it returns the located diagnostic instead.
///
/// This turns otherwise-invisible `zic` semantics into something a reviewer can read and
/// check by eye — pairing with the `compare` oracle and the provenance manifests.
pub fn explain(db: &Database, zone: &str) -> std::result::Result<String, Diagnostic> {
    let mut out = String::new();

    // Resolve a link alias to its canonical zone, and report that up front. If the name is
    // neither a real zone nor a resolvable link, surface the precise reason (a link **cycle**
    // or a **missing target**) as a diagnostic rather than a generic "no such zone".
    let canonical = if db.zone(zone).is_some() {
        zone.to_string()
    } else {
        match crate::resolve_link_target(db, zone) {
            Ok(target) => {
                out.push_str(&format!("{zone}: link alias -> canonical zone {target}\n"));
                target.to_string()
            }
            Err(e) => {
                return Err(Diagnostic::error(
                    crate::DiagnosticCode::InvalidValue,
                    e.to_string(),
                    std::path::Path::new("<input>"),
                    0,
                ))
            }
        }
    };

    // Decode the source eras (parsed fields) before showing the compiled result.
    if let Some(zrec) = db.zone(&canonical) {
        out.push_str(&format!(
            "{canonical}: {} era(s) [{}:{}]\n",
            zrec.eras.len(),
            zrec.origin.file.display(),
            zrec.origin.line
        ));
        for (i, era) in zrec.eras.iter().enumerate() {
            out.push_str(&format!("  era {}: {}\n", i + 1, era_summary(era)));
        }
        // Aliases that point at this zone (materialised as links on compile).
        let aliases: Vec<&str> = db
            .links
            .iter()
            .filter(|l| {
                crate::resolve_link_target(db, &l.link_name).ok() == Some(canonical.as_str())
            })
            .map(|l| l.link_name.as_str())
            .collect();
        if !aliases.is_empty() {
            out.push_str(&format!("  aliases (links): {}\n", aliases.join(", ")));
        }
    }

    let data = match compile_zone(db, &canonical) {
        Ok(d) => d,
        Err(Error::Diagnostic(d)) => return Err(*d),
        Err(e) => {
            return Err(Diagnostic::error(
                crate::DiagnosticCode::InvalidValue,
                e.to_string(),
                std::path::Path::new("<input>"),
                0,
            ))
        }
    };

    out.push_str(&format!(
        "  compiled: TZif v{} — {} local-time-type(s), {} transition(s); footer {:?}\n",
        data.version as char,
        data.types.len(),
        data.transitions.len(),
        data.footer
    ));
    out.push_str("  types:\n");
    for (i, t) in data.types.iter().enumerate() {
        out.push_str(&format!(
            "    [{i}] utoff={} is_dst={} abbr={:?}\n",
            fmt_offset(t.utoff),
            t.is_dst,
            t.abbr
        ));
    }

    // Transitions, decoded. Cap the listing for recurring zones (dozens) but always show the
    // count and the first/last few so the trace stays readable yet complete at the edges.
    if data.transitions.is_empty() {
        out.push_str("  transitions: none (fixed offset / footer-only)\n");
    } else {
        out.push_str("  transitions (UT instant -> type):\n");
        let n = data.transitions.len();
        let show: Vec<usize> = if n <= 8 {
            (0..n).collect()
        } else {
            (0..4).chain(n - 2..n).collect()
        };
        let mut last_shown: Option<usize> = None;
        for idx in show {
            if let Some(prev) = last_shown {
                if idx != prev + 1 {
                    out.push_str(&format!("    … ({} more)\n", idx - prev - 1));
                }
            }
            let tr = &data.transitions[idx];
            let ty = &data.types[tr.type_index as usize];
            out.push_str(&format!(
                "    {} -> [{}] {} {}\n",
                fmt_instant(tr.at),
                tr.type_index,
                fmt_offset(ty.utoff),
                ty.abbr
            ));
            last_shown = Some(idx);
        }
    }

    // Surface the equal-utoff-but-distinct-type trap: any UT offset shared by >1 type.
    let mut by_off: std::collections::BTreeMap<i32, Vec<usize>> = std::collections::BTreeMap::new();
    for (i, t) in data.types.iter().enumerate() {
        by_off.entry(t.utoff).or_default().push(i);
    }
    for (off, idxs) in by_off.iter().filter(|(_, v)| v.len() > 1) {
        let desc: Vec<String> = idxs
            .iter()
            .map(|&i| format!("[{i}] {} dst={}", data.types[i].abbr, data.types[i].is_dst))
            .collect();
        out.push_str(&format!(
            "  note: utoff {} is shared by distinct types ({}) — kept as separate types \
             (offset equal, DST/abbreviation differ)\n",
            fmt_offset(*off),
            desc.join(", ")
        ));
    }

    Ok(out)
}

/// One-line decoded summary of a zone era for the explain trace.
fn era_summary(era: &ZoneEra) -> String {
    let rules = match &era.rules {
        ZoneRules::None => "-".to_string(),
        ZoneRules::Named(n) => format!("rules {n}"),
        ZoneRules::Save(s) => format!("inline save {}", fmt_offset(s.seconds)),
    };
    let until = match &era.until {
        Some(u) => {
            let suffix = match u.time.reference {
                crate::model::TimeRef::Wall => "w",
                crate::model::TimeRef::Standard => "s",
                crate::model::TimeRef::Universal => "u",
            };
            format!(
                " UNTIL {}-{:02}-{} {}{}",
                u.year,
                u.month,
                fmt_on_day(u.day),
                fmt_signed(u.time.seconds),
                suffix
            )
        }
        None => " (final era)".to_string(),
    };
    format!(
        "STDOFF {} | RULES {} | FORMAT {:?}{}",
        fmt_offset(era.stdoff.0),
        rules,
        era.format,
        until
    )
}

/// Render an `ON` day spec back into its tzdata-ish surface form for the trace.
fn fmt_on_day(on: crate::model::calendar::OnDay) -> String {
    use crate::model::calendar::OnDay;
    let wd = |w: crate::model::calendar::Weekday| format!("{w:?}");
    match on {
        OnDay::Day(d) => d.to_string(),
        OnDay::Last(w) => format!("last{}", wd(w)),
        OnDay::OnAfter(w, n) => format!("{}>={n}", wd(w)),
        OnDay::OnBefore(w, n) => format!("{}<={n}", wd(w)),
    }
}

/// Format a signed seconds count as `±h`, `±h:mm`, or `±h:mm:ss` (compact). Used for both UT
/// offsets and `AT`/`UNTIL` times in the trace.
fn fmt_offset(secs: i32) -> String {
    fmt_signed(secs)
}

fn fmt_signed(secs: i32) -> String {
    let sign = if secs < 0 { "-" } else { "" };
    let a = secs.unsigned_abs();
    let (h, m, s) = (a / 3600, (a % 3600) / 60, a % 60);
    if s != 0 {
        format!("{sign}{h}:{m:02}:{s:02}")
    } else if m != 0 {
        format!("{sign}{h}:{m:02}")
    } else {
        format!("{sign}{h}")
    }
}

/// Decode a Unix-second UT instant to `YYYY-MM-DD HH:MM:SSZ` for the trace.
fn fmt_instant(secs: i64) -> String {
    let (y, mo, d) = crate::model::calendar::civil_from_days(secs.div_euclid(86400));
    let tod = secs.rem_euclid(86400);
    let (h, mi, s) = (tod / 3600, (tod % 3600) / 60, tod % 60);
    format!("{y:04}-{mo:02}-{d:02} {h:02}:{mi:02}:{s:02}Z")
}
