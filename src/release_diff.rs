//! `release-diff` (T16.6a) — diff two IANA tzdb releases per identifier.
//!
//! Given two `tzdata.zi` source releases (OLD, NEW), this compiles each identifier in **both** and
//! classifies how it changed. It keeps two axes strictly separate — exactly as the rest of the project
//! does (CORE.1 behaviour vs T8 structural parity are never collapsed):
//!
//! * **structural** — always available: decode each side's TZif and compare the [`Shape`] (version /
//!   `timecnt` / `typecnt` / `charcnt` / `isutcnt` / `isstdcnt` / `leapcnt` / footer), reusing the T8
//!   taxonomy ([`ParityClass`]).
//! * **behavioural** — only when a `zdump` oracle is available: dump each side over the declared horizon
//!   and report whether behaviour changed in the **past** window (years `[lo, split-1]`), the **future**
//!   window (years `[split, hi]`), or both. `split` is an **exclusive seam** (T17.3): the split year
//!   belongs to the future window, so a change in the split year is `behavior_future`, never
//!   double-counted as both. The split is a **declared year**, never host-`now` (determinism: the same
//!   inputs always yield the same diff, independent of when or where it runs). Oracle absence is
//!   surfaced (`oracle_mode = unavailable` + reason), never silently treated as "no change".
//!
//! Non-claims: a diff is scoped to the **declared horizon + split**, not all-time; the behaviour axis
//! requires the oracle; and it says nothing about *why* a zone changed (that is IANA's NEWS, not ours).
//! Identifiers zic-rs cannot compile (out of its declared subset) are reported as errors, never guessed.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::error::{Error, Result};
use crate::manifest::OracleMode;
use crate::model::Database;
use crate::structural::{classify, differing_dims, ParityClass, Shape};

/// How one identifier changed between the two releases. Exactly one kind per identifier (precedence is
/// resolved in [`build_release_diff`]: presence → link → byte-identity → leap → behaviour).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseChangeKind {
    /// Byte-for-byte identical compiled output (or an unchanged link).
    Unchanged,
    /// Present in NEW but not OLD.
    Added,
    /// Present in OLD but not NEW.
    Removed,
    /// The identifier's link status or target changed (zone↔link flip, or retargeted link).
    LinkChanged,
    /// Only the leap-second table (`leapcnt`) differs.
    LeapOnly,
    /// Bytes differ but `zdump` behaviour is **identical** over the whole horizon (footer / version /
    /// encoding-only). Requires the oracle to assert.
    MetadataOnly,
    /// Behaviour differs only in the past window (years `[lo, split-1]`).
    BehaviorPast,
    /// Behaviour differs only in the future window (years `[split, hi]`; the split year is future).
    BehaviorFuture,
    /// Behaviour differs in both the past and future windows.
    BehaviorPastAndFuture,
    /// Bytes differ but **no `zdump` oracle** was available, so behaviour was not assessed (the
    /// structural delta is still recorded). An honest "we did not check," never "no change."
    BehaviourUnassessed,
}

impl ReleaseChangeKind {
    /// Stable snake_case label used in both text and JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            ReleaseChangeKind::Unchanged => "unchanged",
            ReleaseChangeKind::Added => "added",
            ReleaseChangeKind::Removed => "removed",
            ReleaseChangeKind::LinkChanged => "link_changed",
            ReleaseChangeKind::LeapOnly => "leap_only",
            ReleaseChangeKind::MetadataOnly => "metadata_only",
            ReleaseChangeKind::BehaviorPast => "behavior_past",
            ReleaseChangeKind::BehaviorFuture => "behavior_future",
            ReleaseChangeKind::BehaviorPastAndFuture => "behavior_past_and_future",
            ReleaseChangeKind::BehaviourUnassessed => "behaviour_unassessed",
        }
    }

    /// Every variant, in stable order — for totality tests and summary tabulation.
    pub const ALL: [ReleaseChangeKind; 10] = [
        ReleaseChangeKind::Unchanged,
        ReleaseChangeKind::Added,
        ReleaseChangeKind::Removed,
        ReleaseChangeKind::LinkChanged,
        ReleaseChangeKind::LeapOnly,
        ReleaseChangeKind::MetadataOnly,
        ReleaseChangeKind::BehaviorPast,
        ReleaseChangeKind::BehaviorFuture,
        ReleaseChangeKind::BehaviorPastAndFuture,
        ReleaseChangeKind::BehaviourUnassessed,
    ];
}

/// The per-window behavioural delta (counts of differing `zdump` lines), present only when the oracle ran.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BehaviourDelta {
    pub past_diffs: usize,
    pub future_diffs: usize,
}

/// Why the behaviour axis could not assess something (T17.3 — was conflated into one "flip the whole
/// run unavailable" path). The distinction is load-bearing: a tool that cannot be resolved at all is a
/// **global** outage (the whole axis is unavailable), but a failure assessing *one* identifier (a bad
/// path/data for that zone) is **row-scoped** and must **not** poison the rest of the run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OracleFailureScope {
    /// The `zdump` tool itself could not be resolved/run — the entire behaviour axis is unavailable.
    GlobalToolUnavailable,
    /// The tool resolved, but assessing this one identifier failed; other identifiers stay assessable.
    RowOrIdentifierFailure,
}

impl OracleFailureScope {
    /// Stable snake_case label.
    pub fn as_str(self) -> &'static str {
        match self {
            OracleFailureScope::GlobalToolUnavailable => "global_tool_unavailable",
            OracleFailureScope::RowOrIdentifierFailure => "row_or_identifier_failure",
        }
    }
}

/// One identifier's release-to-release comparison.
#[derive(Debug, Clone)]
pub struct DiffRow {
    pub name: String,
    pub change_kind: ReleaseChangeKind,
    /// Structural classification (present when both sides compiled to a zone and bytes differ).
    pub parity_class: Option<ParityClass>,
    /// The differing structural dimensions (stable order).
    pub diffs: Vec<&'static str>,
    /// Behavioural delta (present only when the `zdump` oracle ran for this row).
    pub behaviour: Option<BehaviourDelta>,
    /// Link target change `(old_target, new_target)` when this identifier is/was a link.
    pub link_change: Option<(Option<String>, Option<String>)>,
    /// T17.3: when the oracle resolved but failed *for this identifier* (an
    /// [`OracleFailureScope::RowOrIdentifierFailure`]), the reason — so a row-scoped behaviour failure is
    /// recorded on the row that hit it, while the rest of the run stays assessed. `None` otherwise.
    /// Additive to `zic-rs-release-diff-v1` (only emitted when present) — no schema bump.
    pub behaviour_error: Option<String>,
}

/// An identifier that could not be compared (one side failed to compile within zic-rs's subset).
#[derive(Debug, Clone)]
pub struct DiffError {
    pub name: String,
    pub reason: String,
}

/// Options for a release diff.
#[derive(Debug, Clone)]
pub struct ReleaseDiffOptions {
    /// Behaviour horizon in **years** `(lo, hi)`.
    pub horizon: (i32, i32),
    /// The past/future split **year** (declared, deterministic — never host-`now`).
    pub split: i32,
    /// Restrict to a single identifier.
    pub zone_filter: Option<String>,
    /// The `zdump` program for the behaviour axis. `None` ⇒ behaviour not assessed.
    pub zdump_program: Option<String>,
}

/// The complete release diff.
#[derive(Debug)]
pub struct ReleaseDiffReport {
    pub oracle_mode: OracleMode,
    pub horizon: (i32, i32),
    pub split: i32,
    pub rows: Vec<DiffRow>,
    pub errors: Vec<DiffError>,
}

impl ReleaseDiffReport {
    /// Count of rows per change kind, in the enum's canonical order.
    pub fn kind_counts(&self) -> BTreeMap<&'static str, usize> {
        let mut m: BTreeMap<&'static str, usize> = BTreeMap::new();
        for k in ReleaseChangeKind::ALL {
            m.insert(k.as_str(), 0);
        }
        for r in &self.rows {
            *m.entry(r.change_kind.as_str()).or_default() += 1;
        }
        m
    }
}

/// Build the link-name → target map for a database (last-wins, matching `zic`'s `make_links` dedup).
fn link_map(db: &Database) -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    for l in &db.links {
        m.insert(l.link_name.clone(), l.target.clone());
    }
    m
}

/// Run `zdump` over `[lo, hi]` on a freshly-written copy of `bytes`, returning normalised lines.
fn dump(
    program: &str,
    root: &std::path::Path,
    name: &str,
    bytes: &[u8],
    lo: i32,
    hi: i32,
) -> Result<Vec<String>> {
    let path = crate::fs::output_tree::write_zone_file(root, name, bytes, true, false)?;
    crate::compare::zdump::run(program, &path, lo, hi)
}

/// Compute the behaviour delta for one zone across the two releases. Returns `Err` if the oracle could
/// not run (so the caller can flip to "unavailable" and stop attempting it).
fn behaviour_delta(
    program: &str,
    name: &str,
    old_bytes: &[u8],
    new_bytes: &[u8],
    opts: &ReleaseDiffOptions,
    work: &std::path::Path,
) -> Result<BehaviourDelta> {
    let (lo, hi) = opts.horizon;
    let split = opts.split;
    let old_root = work.join("old");
    let new_root = work.join("new");
    // T17.3 — `split` is an **exclusive seam**: past = years `[lo, split-1]`, future = years
    // `[split, hi]`, so a change in the split year is attributed to the **future** window and **never
    // double-counted** into both. `zdump -c` year bounds are inclusive, hence `split - 1` for the past
    // upper bound. Each window is skipped (no diff) when it is empty (`split <= lo` ⇒ no past window;
    // `split > hi` ⇒ no future window), so a degenerate split never asks zdump for an inverted range.
    let past = if split > lo {
        crate::compare::zdump::diff(
            &dump(program, &old_root, name, old_bytes, lo, split - 1)?,
            &dump(program, &new_root, name, new_bytes, lo, split - 1)?,
        )
    } else {
        Vec::new()
    };
    let future = if split <= hi {
        crate::compare::zdump::diff(
            &dump(program, &old_root, name, old_bytes, split, hi)?,
            &dump(program, &new_root, name, new_bytes, split, hi)?,
        )
    } else {
        Vec::new()
    };
    Ok(BehaviourDelta {
        past_diffs: past.len(),
        future_diffs: future.len(),
    })
}

/// Diff two parsed releases into a [`ReleaseDiffReport`].
pub fn build_release_diff(
    old_db: &Database,
    new_db: &Database,
    opts: &ReleaseDiffOptions,
) -> Result<ReleaseDiffReport> {
    let old_zones: BTreeSet<&str> = old_db.zones.iter().map(|z| z.name.as_str()).collect();
    let new_zones: BTreeSet<&str> = new_db.zones.iter().map(|z| z.name.as_str()).collect();
    let old_links = link_map(old_db);
    let new_links = link_map(new_db);

    // The identifier universe: every zone + link name on either side (sorted, deterministic).
    let mut names: BTreeSet<String> = BTreeSet::new();
    for z in &old_db.zones {
        names.insert(z.name.clone());
    }
    for z in &new_db.zones {
        names.insert(z.name.clone());
    }
    for k in old_links.keys().chain(new_links.keys()) {
        names.insert(k.clone());
    }
    if let Some(only) = &opts.zone_filter {
        names.retain(|n| n == only);
    }

    // Scratch tree for the behaviour axis (absolute, auto-cleaned). Created even if the oracle is off
    // (cheap); only written to when zdump actually runs.
    let work = tempfile::Builder::new()
        .prefix("zic-rs-release-diff-")
        .tempdir()
        .map_err(|e| Error::io(PathBuf::from("<tempdir>"), e))?;

    let mut rows = Vec::new();
    let mut errors = Vec::new();
    // T17.3 — distinguish global tool outage from per-identifier failure (OracleFailureScope). Probe the
    // tool ONCE up front: if it cannot be resolved, the behaviour axis is GlobalToolUnavailable for the
    // whole run; if it resolves, a later per-row failure is a RowOrIdentifierFailure recorded on *that*
    // row only — it must not poison the rest. `oracle_unavailable` is now set only by a *global* outage.
    let oracle_unavailable: Option<String> = match &opts.zdump_program {
        None => Some("behaviour axis not requested (pass --reference-zdump to assess)".into()),
        Some(prog) => {
            if crate::doctor::resolve(prog).is_some() {
                None
            } else {
                Some(format!(
                    "{}: zdump program {prog:?} could not be resolved on PATH or as an explicit path",
                    OracleFailureScope::GlobalToolUnavailable.as_str()
                ))
            }
        }
    };

    for name in &names {
        let in_old = old_zones.contains(name.as_str()) || old_links.contains_key(name);
        let in_new = new_zones.contains(name.as_str()) || new_links.contains_key(name);
        let old_link = old_links.get(name);
        let new_link = new_links.get(name);
        let old_zone = old_zones.contains(name.as_str());
        let new_zone = new_zones.contains(name.as_str());

        // 1. presence
        if !in_old && in_new {
            rows.push(simple_row(name, ReleaseChangeKind::Added));
            continue;
        }
        if in_old && !in_new {
            rows.push(simple_row(name, ReleaseChangeKind::Removed));
            continue;
        }
        // 2. link involvement (a link on either side, or a zone↔link flip)
        if old_link.is_some() || new_link.is_some() {
            let unchanged_link = old_link.is_some()
                && new_link.is_some()
                && old_link == new_link
                && !old_zone
                && !new_zone;
            let kind = if unchanged_link {
                ReleaseChangeKind::Unchanged
            } else {
                ReleaseChangeKind::LinkChanged
            };
            let mut row = simple_row(name, kind);
            row.link_change = Some((old_link.cloned(), new_link.cloned()));
            rows.push(row);
            continue;
        }
        // 3. both are zones → compile both and diff
        debug_assert!(old_zone && new_zone);
        let old_bytes = match crate::compile_zone_to_bytes(old_db, name) {
            Ok(b) => b,
            Err(e) => {
                errors.push(DiffError {
                    name: name.clone(),
                    reason: format!("OLD: {e}"),
                });
                continue;
            }
        };
        let new_bytes = match crate::compile_zone_to_bytes(new_db, name) {
            Ok(b) => b,
            Err(e) => {
                errors.push(DiffError {
                    name: name.clone(),
                    reason: format!("NEW: {e}"),
                });
                continue;
            }
        };
        if old_bytes == new_bytes {
            rows.push(simple_row(name, ReleaseChangeKind::Unchanged));
            continue;
        }
        // bytes differ → structural classification
        let (op, np) = match (
            crate::tzif::validate::parse(&old_bytes),
            crate::tzif::validate::parse(&new_bytes),
        ) {
            (Ok(o), Ok(n)) => (o, n),
            _ => {
                errors.push(DiffError {
                    name: name.clone(),
                    reason: "could not decode compiled TZif on one side".into(),
                });
                continue;
            }
        };
        let dims = differing_dims(&Shape::of(&op), &Shape::of(&np));
        let parity = classify(false, &dims);

        // leap-only short-circuits (independent of behaviour).
        if dims.as_slice() == ["leapcnt"] {
            let mut row = simple_row(name, ReleaseChangeKind::LeapOnly);
            row.parity_class = Some(parity);
            row.diffs = dims;
            rows.push(row);
            continue;
        }

        // behaviour axis. The oracle was probed up front, so a failure HERE is row-scoped
        // (OracleFailureScope::RowOrIdentifierFailure): record it on this row and keep assessing the
        // rest — never flip the whole run unavailable (that is reserved for the global probe above).
        let mut behaviour = None;
        let mut behaviour_error = None;
        let kind = if let (Some(program), None) = (&opts.zdump_program, &oracle_unavailable) {
            match behaviour_delta(program, name, &old_bytes, &new_bytes, opts, work.path()) {
                Ok(d) => {
                    behaviour = Some(d);
                    match (d.past_diffs > 0, d.future_diffs > 0) {
                        (true, true) => ReleaseChangeKind::BehaviorPastAndFuture,
                        (true, false) => ReleaseChangeKind::BehaviorPast,
                        (false, true) => ReleaseChangeKind::BehaviorFuture,
                        (false, false) => ReleaseChangeKind::MetadataOnly,
                    }
                }
                Err(e) => {
                    behaviour_error = Some(format!(
                        "{}: {e}",
                        OracleFailureScope::RowOrIdentifierFailure.as_str()
                    ));
                    ReleaseChangeKind::BehaviourUnassessed
                }
            }
        } else {
            ReleaseChangeKind::BehaviourUnassessed
        };
        let mut row = simple_row(name, kind);
        row.parity_class = Some(parity);
        row.diffs = dims;
        row.behaviour = behaviour;
        row.behaviour_error = behaviour_error;
        rows.push(row);
    }

    let oracle_mode = match (&opts.zdump_program, oracle_unavailable) {
        (Some(_), None) => OracleMode::ReferenceZdump,
        (_, Some(reason)) => OracleMode::Unavailable(reason),
        (None, None) => OracleMode::Unavailable("behaviour axis not requested".into()),
    };

    Ok(ReleaseDiffReport {
        oracle_mode,
        horizon: opts.horizon,
        split: opts.split,
        rows,
        errors,
    })
}

fn simple_row(name: &str, kind: ReleaseChangeKind) -> DiffRow {
    DiffRow {
        name: name.to_string(),
        change_kind: kind,
        parity_class: None,
        diffs: Vec::new(),
        behaviour: None,
        link_change: None,
        behaviour_error: None,
    }
}

/// The schema id (versioned, immutable).
pub const SCHEMA: &str = "zic-rs-release-diff-v1";

impl ReleaseDiffReport {
    /// Render the report as deterministic JSON (`zic-rs-release-diff-v1`).
    pub fn to_json(&self) -> String {
        use crate::json::escape;
        let mut s = String::new();
        s.push_str("{\n");
        s.push_str(&format!("  \"schema\": {},\n", escape(SCHEMA)));
        s.push_str(&crate::manifest::provenance_block_json());
        s.push_str(&format!(
            "  \"oracle_mode\": {},\n",
            self.oracle_mode.to_json_field()
        ));
        s.push_str(&format!(
            "  \"horizon\": {{ \"lo\": {}, \"hi\": {} }},\n",
            self.horizon.0, self.horizon.1
        ));
        s.push_str(&format!("  \"split\": {},\n", self.split));
        s.push_str(
            "  \"non_claim\": \"a release-diff is scoped to the declared horizon + split, not all-time; \
             the behaviour axis requires a zdump oracle (absence ⇒ behaviour_unassessed, never 'no change'); \
             it does not state WHY a zone changed (that is IANA NEWS); identifiers outside zic-rs's compile \
             subset are reported as errors, never guessed\",\n",
        );
        // summary counts
        let counts = self.kind_counts();
        s.push_str("  \"summary\": {");
        let mut first = true;
        for (k, v) in &counts {
            s.push_str(if first { "\n" } else { ",\n" });
            first = false;
            s.push_str(&format!("    {}: {}", escape(k), v));
        }
        s.push_str("\n  },\n");
        // rows
        s.push_str("  \"identifiers\": [");
        for (i, r) in self.rows.iter().enumerate() {
            s.push_str(if i == 0 { "\n" } else { ",\n" });
            s.push_str(&row_json(r));
        }
        s.push_str(if self.rows.is_empty() {
            "],\n"
        } else {
            "\n  ],\n"
        });
        // errors
        s.push_str("  \"errors\": [");
        for (i, e) in self.errors.iter().enumerate() {
            s.push_str(if i == 0 { "\n" } else { ",\n" });
            s.push_str(&format!(
                "    {{ \"name\": {}, \"reason\": {} }}",
                escape(&e.name),
                escape(&e.reason)
            ));
        }
        s.push_str(if self.errors.is_empty() {
            "]\n"
        } else {
            "\n  ]\n"
        });
        s.push_str("}\n");
        s
    }
}

fn row_json(r: &DiffRow) -> String {
    use crate::json::escape;
    let mut s = String::new();
    s.push_str(&format!(
        "    {{ \"name\": {}, \"change_kind\": {}",
        escape(&r.name),
        escape(r.change_kind.as_str())
    ));
    if let Some(p) = r.parity_class {
        let dims: Vec<String> = r.diffs.iter().map(|d| escape(d)).collect();
        s.push_str(&format!(
            ", \"structural\": {{ \"parity_class\": {}, \"differing\": [{}] }}",
            escape(p.label()),
            dims.join(", ")
        ));
    }
    if let Some(b) = r.behaviour {
        s.push_str(&format!(
            ", \"behaviour\": {{ \"past_diffs\": {}, \"future_diffs\": {} }}",
            b.past_diffs, b.future_diffs
        ));
    }
    if let Some((old_t, new_t)) = &r.link_change {
        let f = |o: &Option<String>| {
            o.as_ref()
                .map(|t| escape(t))
                .unwrap_or_else(|| "null".into())
        };
        s.push_str(&format!(
            ", \"link\": {{ \"old_target\": {}, \"new_target\": {} }}",
            f(old_t),
            f(new_t)
        ));
    }
    // T17.3: a row-scoped behaviour-oracle failure, emitted only when present (additive).
    if let Some(reason) = &r.behaviour_error {
        s.push_str(&format!(", \"behaviour_error\": {}", escape(reason)));
    }
    s.push_str(" }");
    s
}
