//! `structural-report` — a measured **structural-parity inventory** of zic-rs TZif output
//! against reference `zic` (campaign T8).
//!
//! This is a *separate axis* from the behaviour oracle. CORE.1 already establishes that every
//! canonical zone **behaviour-matches** reference `zic`/`zdump` over `1900..2040` (the binding
//! contract). This report answers a different question: *how do the emitted TZif bytes differ
//! structurally* — which is exactly what a "drop-in file replacement" claim would need, and is
//! deliberately **not** claimed by CORE.1.
//!
//! For each canonical zone we compile both ways (zic-rs in memory, reference `zic` into a temp
//! tree), decode both with the shared [`crate::tzif::parse`], and classify the difference into a
//! fixed taxonomy ([`ParityClass`]). The point is to *measure* the distance, not to chase byte
//! parity: behaviour parity stays the contract, structural parity is reported honestly, and byte
//! parity is only ever claimed where a reference blob is pinned (`fixtures/expected/`).
//!
//! Empirically (tzdata.zi 2026b vs tzcode 2026b): `isutcnt`/`isstdcnt`/`leapcnt`/`typecnt`/`version`
//! /`footer` are at full parity across all 341 zones (version+footer 341/341 after T8-v3 pinned
//! `zic.c`'s `compat >= 2013` version rule — see `compile::posix_footer::recurring`). The only
//! remaining differences are `timecnt` (the documented slim/fat explicit-transition window) and two
//! `charcnt` zones (zic shares abbreviation *suffixes* in the designation table; zic-rs stores them
//! separately). All are `zdump`-equivalent.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::compare::reference_zic;
use crate::error::{Error, Result};
use crate::json::escape;
use crate::model::Database;
use crate::tzif::{self, ParsedTzif};

/// Schema identifier for the JSON form.
const SCHEMA: &str = "zic-rs-structural-report-v3";

/// How many example zones to show per class in the text report (the JSON form lists all).
const TEXT_EXAMPLES: usize = 8;

/// A structural snapshot of one compiled TZif file's authoritative (v2+) block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shape {
    pub version: u8,
    pub timecnt: u32,
    pub typecnt: u32,
    pub charcnt: u32,
    pub isutcnt: u32,
    pub isstdcnt: u32,
    pub leapcnt: u32,
    pub footer: String,
}

impl Shape {
    /// Decode a [`Shape`] from a parsed TZif. `pub(crate)` so `release_diff` (T16.6) reuses the
    /// exact same structural snapshot the structural-parity inventory (T8) uses — one source of truth.
    pub(crate) fn of(p: &ParsedTzif) -> Self {
        Shape {
            version: p.version,
            timecnt: p.counts.timecnt,
            typecnt: p.counts.typecnt,
            charcnt: p.counts.charcnt,
            isutcnt: p.counts.isutcnt,
            isstdcnt: p.counts.isstdcnt,
            leapcnt: p.counts.leapcnt,
            footer: p.footer.clone(),
        }
    }
}

/// The taxonomy a zone's structural difference falls into. A zone lands in exactly one class:
/// the single differing dimension when there is exactly one, else a coarser catch-all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParityClass {
    /// Byte-for-byte identical output.
    ByteIdentical,
    /// Bytes differ but every decoded count + version + footer matches (type/abbreviation
    /// *ordering* or designation *packing* differs but is invisible at the count level).
    StructurallyEquivalent,
    /// Only `timecnt` differs — the documented slim/fat explicit-transition window.
    SlimFatTimecnt,
    /// Only `typecnt` differs — local-time-type table count.
    TypeCount,
    /// Only `charcnt` differs — designation-table packing (zic suffix-sharing).
    AbbrevTable,
    /// Only the TZif version byte differs.
    Version,
    /// Only the POSIX `TZ` footer string differs.
    Footer,
    /// Only `isutcnt`/`isstdcnt` differ — `ttisut`/`ttisstd` indicator policy.
    TtisStdUt,
    /// Only `leapcnt` differs — leap-record count.
    Leap,
    /// More than one dimension differs — flagged for investigation.
    Mixed,
}

impl ParityClass {
    /// A stable label used in both the text report and the JSON keys.
    pub fn label(self) -> &'static str {
        match self {
            ParityClass::ByteIdentical => "byte-identical",
            ParityClass::StructurallyEquivalent => "structurally-equivalent",
            ParityClass::SlimFatTimecnt => "slim/fat-timecnt",
            ParityClass::TypeCount => "type-count",
            ParityClass::AbbrevTable => "abbreviation-table",
            ParityClass::Version => "version-byte",
            ParityClass::Footer => "footer",
            ParityClass::TtisStdUt => "ttisstd/ttisut",
            ParityClass::Leap => "leap-count",
            ParityClass::Mixed => "mixed/unexpected",
        }
    }
}

/// The list of dimensions that differ between two shapes (stable order, for diagnostics).
/// `pub(crate)` so `release_diff` (T16.6) classifies a two-release structural delta with the same
/// dimension vocabulary the T8 inventory uses.
pub(crate) fn differing_dims(a: &Shape, b: &Shape) -> Vec<&'static str> {
    let mut d = Vec::new();
    if a.version != b.version {
        d.push("version");
    }
    if a.timecnt != b.timecnt {
        d.push("timecnt");
    }
    if a.typecnt != b.typecnt {
        d.push("typecnt");
    }
    if a.charcnt != b.charcnt {
        d.push("charcnt");
    }
    if a.isutcnt != b.isutcnt {
        d.push("isutcnt");
    }
    if a.isstdcnt != b.isstdcnt {
        d.push("isstdcnt");
    }
    if a.leapcnt != b.leapcnt {
        d.push("leapcnt");
    }
    if a.footer != b.footer {
        d.push("footer");
    }
    d
}

/// Classify a single zone given its byte-identity and the differing dimensions.
/// `pub(crate)` so `release_diff` (T16.6) reuses the identical single-class taxonomy.
pub(crate) fn classify(byte_identical: bool, dims: &[&str]) -> ParityClass {
    if byte_identical {
        return ParityClass::ByteIdentical;
    }
    match dims {
        [] => ParityClass::StructurallyEquivalent,
        ["timecnt"] => ParityClass::SlimFatTimecnt,
        ["typecnt"] => ParityClass::TypeCount,
        ["charcnt"] => ParityClass::AbbrevTable,
        ["version"] => ParityClass::Version,
        ["footer"] => ParityClass::Footer,
        ["isutcnt"] | ["isstdcnt"] | ["isutcnt", "isstdcnt"] => ParityClass::TtisStdUt,
        ["leapcnt"] => ParityClass::Leap,
        _ => ParityClass::Mixed,
    }
}

/// One canonical zone's structural comparison.
#[derive(Debug, Clone)]
pub struct ZoneShape {
    pub name: String,
    pub class: ParityClass,
    /// The differing dimensions (empty when byte-identical or structurally-equivalent).
    pub diffs: Vec<&'static str>,
    pub ours: Shape,
    pub theirs: Shape,
}

/// A zone that could not be structurally compared (one side failed to compile or decode).
#[derive(Debug, Clone)]
pub struct ZoneError {
    pub name: String,
    pub reason: String,
}

/// The complete structural-parity inventory for one source file.
#[derive(Debug)]
pub struct StructuralReport {
    pub tzdb_version: Option<String>,
    pub reference_zic: String,
    /// Per-zone comparisons, sorted by name.
    pub zones: Vec<ZoneShape>,
    /// Zones skipped because one side errored, sorted by name.
    pub errors: Vec<ZoneError>,
    /// Net extra explicit transitions zic-rs emits over reference `zic` (Σ over slim/fat zones;
    /// negative would mean reference is fatter). A pure measure of the slim/fat gap.
    pub timecnt_delta_total: i64,
}

impl StructuralReport {
    /// Count of zones in each class, in the taxonomy's canonical order.
    pub fn class_counts(&self) -> BTreeMap<ParityClass, usize> {
        let mut m: BTreeMap<ParityClass, usize> = BTreeMap::new();
        for z in &self.zones {
            *m.entry(z.class).or_default() += 1;
        }
        m
    }

    pub fn zones_compared(&self) -> usize {
        self.zones.len()
    }

    /// Zones whose behaviour-relevant emitted structure — the TZif **version byte** and the POSIX
    /// **footer** — matches reference `zic` exactly. Computed directly (not by class) so a zone
    /// that is `mixed` only on benign dimensions (slim/fat `timecnt` + `charcnt` packing) still
    /// counts as version+footer-clean. This is the honest "structurally drop-in modulo slim/fat +
    /// abbreviation packing" count.
    pub fn version_footer_match(&self) -> usize {
        self.zones
            .iter()
            .filter(|z| z.ours.version == z.theirs.version && z.ours.footer == z.theirs.footer)
            .count()
    }

    pub fn to_text(&self) -> String {
        let mut s = String::new();
        s.push_str("zic-rs structural-parity inventory (campaign T8)\n");
        s.push_str(
            "  axis: TZif *structure* vs reference `zic` — SEPARATE from behaviour parity.\n",
        );
        s.push_str(
            "  behaviour parity (CORE.1: 341/341 zdump-match over 1900..2040) is the contract;\n",
        );
        s.push_str("  byte parity is claimed only where a reference blob is pinned.\n\n");
        if let Some(v) = &self.tzdb_version {
            s.push_str(&format!("  tzdb release      : {v}\n"));
        }
        s.push_str(&format!("  reference zic     : {}\n", self.reference_zic));
        s.push_str(&format!(
            "  zones compared    : {}\n",
            self.zones_compared()
        ));
        s.push_str(&format!(
            "  version+footer ok : {} (structure drop-in modulo slim/fat + packing)\n",
            self.version_footer_match()
        ));
        s.push_str(&format!(
            "  net extra transitions (ours − ref, slim/fat): {}\n\n",
            self.timecnt_delta_total
        ));

        s.push_str("  parity classes:\n");
        let counts = self.class_counts();
        for (class, n) in &counts {
            s.push_str(&format!("    {:<24} {}\n", class.label(), n));
            // Show examples for the classes a maintainer cares about (anything not the bulk
            // byte-identical / structurally-equivalent / slim-fat rows).
            if matches!(
                class,
                ParityClass::Version
                    | ParityClass::Footer
                    | ParityClass::AbbrevTable
                    | ParityClass::TypeCount
                    | ParityClass::TtisStdUt
                    | ParityClass::Leap
                    | ParityClass::Mixed
            ) {
                for (shown, z) in self.zones.iter().filter(|z| z.class == *class).enumerate() {
                    if shown == TEXT_EXAMPLES {
                        s.push_str(&format!(
                            "        (+{} more)\n",
                            counts[class] - TEXT_EXAMPLES
                        ));
                        break;
                    }
                    s.push_str(&format!(
                        "        {}: [{}]  ours v{} tc={} cc={}  ref v{} tc={} cc={}\n",
                        z.name,
                        z.diffs.join(","),
                        z.ours.version as char,
                        z.ours.timecnt,
                        z.ours.charcnt,
                        z.theirs.version as char,
                        z.theirs.timecnt,
                        z.theirs.charcnt,
                    ));
                }
            }
        }

        if !self.errors.is_empty() {
            s.push_str(&format!("\n  not compared ({}):\n", self.errors.len()));
            for e in self.errors.iter().take(TEXT_EXAMPLES) {
                s.push_str(&format!("    {}: {}\n", e.name, e.reason));
            }
            if self.errors.len() > TEXT_EXAMPLES {
                s.push_str(&format!(
                    "    (+{} more)\n",
                    self.errors.len() - TEXT_EXAMPLES
                ));
            }
        }
        // T12.6 — static provenance/capability statement (manifest schema + source-variant gate).
        s.push_str(&crate::manifest::provenance_block_text());
        s
    }

    pub fn to_json(&self) -> String {
        let mut s = String::new();
        s.push_str("{\n");
        s.push_str(&format!("  \"schema\": {},\n", escape(SCHEMA)));
        // T12.6 — static provenance/capability block (schema + source-variant pin-gate state +
        // T15.2 `negative_capabilities`).
        s.push_str(&crate::manifest::provenance_block_json());
        // T15.2 — oracle availability, typed. `structural-report` compares against reference `zic`'s
        // emitted bytes, so the oracle that backs its verdicts is `reference_zic`. Visible, never silent.
        s.push_str(&format!(
            "  \"oracle_mode\": {},\n",
            crate::manifest::OracleMode::ReferenceZic.to_json_field()
        ));
        match &self.tzdb_version {
            Some(v) => s.push_str(&format!("  \"tzdb_version\": {},\n", escape(v))),
            None => s.push_str("  \"tzdb_version\": null,\n"),
        }
        s.push_str(&format!(
            "  \"reference_zic\": {},\n",
            escape(&self.reference_zic)
        ));
        s.push_str(&format!(
            "  \"zones_compared\": {},\n",
            self.zones_compared()
        ));
        s.push_str(&format!(
            "  \"version_footer_match\": {},\n",
            self.version_footer_match()
        ));
        s.push_str(&format!(
            "  \"timecnt_delta_total\": {},\n",
            self.timecnt_delta_total
        ));
        s.push_str("  \"class_counts\": {");
        let mut first = true;
        for (class, n) in &self.class_counts() {
            s.push_str(if first { "\n" } else { ",\n" });
            first = false;
            s.push_str(&format!("    {}: {}", escape(class.label()), n));
        }
        s.push_str(if first { "}," } else { "\n  },\n" });
        s.push('\n');
        // Per-zone rows only for the classes worth auditing (omit the byte-identical /
        // structurally-equivalent / slim-fat bulk to keep the JSON focused on differences).
        s.push_str("  \"differences\": [");
        let mut first = true;
        for z in self.zones.iter().filter(|z| {
            !matches!(
                z.class,
                ParityClass::ByteIdentical
                    | ParityClass::StructurallyEquivalent
                    | ParityClass::SlimFatTimecnt
            )
        }) {
            s.push_str(if first { "\n" } else { ",\n" });
            first = false;
            let dims: Vec<String> = z.diffs.iter().map(|d| escape(d)).collect();
            // The version byte is the ASCII digit ('2'/'3'); emit it as a quoted digit string.
            s.push_str(&format!(
                "    {{ \"zone\": {}, \"class\": {}, \"dims\": [{}], \"ours_version\": {}, \"ref_version\": {}, \"ours_timecnt\": {}, \"ref_timecnt\": {}, \"ours_charcnt\": {}, \"ref_charcnt\": {} }}",
                escape(&z.name),
                escape(z.class.label()),
                dims.join(", "),
                escape(&(z.ours.version as char).to_string()),
                escape(&(z.theirs.version as char).to_string()),
                z.ours.timecnt,
                z.theirs.timecnt,
                z.ours.charcnt,
                z.theirs.charcnt,
            ));
        }
        s.push_str(if first { "],\n" } else { "\n  ],\n" });
        s.push_str(&format!("  \"errors\": {}\n", self.errors.len()));
        s.push_str("}\n");
        s
    }
}

/// Build the structural-parity inventory for `db`.
///
/// `inputs` are the source files (passed to reference `zic`, which takes files not dirs);
/// `reference_zic` is the reference compiler program; `work_dir` is a caller-controlled
/// (absolute) scratch directory (typically a tempdir). `only` restricts to a single zone.
pub fn build_structural_report(
    db: &Database,
    inputs: &[PathBuf],
    reference_zic: &str,
    work_dir: &Path,
    only: Option<&str>,
    tzdb_version: Option<String>,
    emit_style: crate::EmitStyle,
) -> Result<StructuralReport> {
    // Compile every zone with reference `zic` once, into work_dir/ref.
    let ref_root = work_dir.join("ref");
    std::fs::create_dir_all(&ref_root).map_err(|e| Error::io(&ref_root, e))?;
    reference_zic::compile_with_reference(reference_zic, inputs, &ref_root)?;

    let names: Vec<String> = match only {
        Some(z) => vec![z.to_string()],
        None => {
            let mut v: Vec<String> = db.zones.iter().map(|z| z.name.clone()).collect();
            v.sort();
            v
        }
    };

    let mut zones = Vec::new();
    let mut errors = Vec::new();
    let mut timecnt_delta_total: i64 = 0;

    for name in names {
        let ours_bytes = match crate::compile_zone_to_bytes_styled(db, &name, emit_style) {
            Ok(b) => b,
            Err(e) => {
                errors.push(ZoneError {
                    name,
                    reason: format!("ours: {e}"),
                });
                continue;
            }
        };
        let ref_path = reference_zic::compiled_path(&ref_root, &name);
        let theirs_bytes = match std::fs::read(&ref_path) {
            Ok(b) => b,
            Err(e) => {
                errors.push(ZoneError {
                    name,
                    reason: format!("reference: {e}"),
                });
                continue;
            }
        };
        let byte_identical = ours_bytes == theirs_bytes;
        let ours = match tzif::parse(&ours_bytes) {
            Ok(p) => Shape::of(&p),
            Err(e) => {
                errors.push(ZoneError {
                    name,
                    reason: format!("decode ours: {e}"),
                });
                continue;
            }
        };
        let theirs = match tzif::parse(&theirs_bytes) {
            Ok(p) => Shape::of(&p),
            Err(e) => {
                errors.push(ZoneError {
                    name,
                    reason: format!("decode reference: {e}"),
                });
                continue;
            }
        };
        let diffs = differing_dims(&ours, &theirs);
        let class = classify(byte_identical, &diffs);
        timecnt_delta_total += ours.timecnt as i64 - theirs.timecnt as i64;
        zones.push(ZoneShape {
            name,
            class,
            diffs,
            ours,
            theirs,
        });
    }

    Ok(StructuralReport {
        tzdb_version,
        reference_zic: reference_zic.to_string(),
        zones,
        errors,
        timecnt_delta_total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shape(version: u8, timecnt: u32, charcnt: u32, footer: &str) -> Shape {
        Shape {
            version,
            timecnt,
            typecnt: 3,
            charcnt,
            isutcnt: 0,
            isstdcnt: 0,
            leapcnt: 0,
            footer: footer.to_string(),
        }
    }

    #[test]
    fn byte_identical_dominates() {
        let a = shape(b'2', 100, 20, "EST5");
        let b = shape(b'2', 100, 20, "EST5");
        assert_eq!(
            classify(true, &differing_dims(&a, &b)),
            ParityClass::ByteIdentical
        );
    }

    #[test]
    fn equal_decode_but_byte_diff_is_structurally_equivalent() {
        // Same counts/version/footer but bytes differ (e.g. type ordering) → equivalent.
        let a = shape(b'2', 100, 20, "EST5");
        let b = shape(b'2', 100, 20, "EST5");
        assert_eq!(
            classify(false, &differing_dims(&a, &b)),
            ParityClass::StructurallyEquivalent
        );
    }

    #[test]
    fn lone_timecnt_diff_is_slim_fat() {
        let a = shape(b'2', 236, 20, "EST5EDT,M3.2.0,M11.1.0");
        let b = shape(b'2', 175, 20, "EST5EDT,M3.2.0,M11.1.0");
        let d = differing_dims(&a, &b);
        assert_eq!(d, ["timecnt"]);
        assert_eq!(classify(false, &d), ParityClass::SlimFatTimecnt);
    }

    #[test]
    fn lone_charcnt_diff_is_abbrev_table() {
        // Adak/Ho_Chi_Minh shape: ours 4 bytes larger, everything else equal.
        let a = shape(b'2', 145, 37, "HST10");
        let b = shape(b'2', 145, 33, "HST10");
        assert_eq!(
            classify(false, &differing_dims(&a, &b)),
            ParityClass::AbbrevTable
        );
    }

    #[test]
    fn lone_version_diff_is_version() {
        // Santiago shape: ref v3, ours v2, identical footer.
        let a = shape(b'2', 100, 12, "<-04>4<-03>,M9.1.6/24,M4.1.6/24");
        let b = shape(b'3', 100, 12, "<-04>4<-03>,M9.1.6/24,M4.1.6/24");
        assert_eq!(
            classify(false, &differing_dims(&a, &b)),
            ParityClass::Version
        );
    }

    #[test]
    fn multiple_dims_is_mixed() {
        let a = shape(b'2', 100, 20, "EST5");
        let b = shape(b'3', 90, 18, "EST5EDT,M3.2.0,M11.1.0");
        assert_eq!(classify(false, &differing_dims(&a, &b)), ParityClass::Mixed);
    }
}
