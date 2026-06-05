//! The result of an oracle comparison for one zone, plus honest rendering.

use super::semantic::Difference;

/// Which comparison produced this result — so summaries never overclaim what was checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComparisonKind {
    /// Decoded both TZif files and diffed the model (footer + transitions + types).
    DecodedTzif,
    /// Diffed `zdump -v -c lo,hi` behaviour over an inclusive year horizon.
    ZdumpBehaviour { lo: i32, hi: i32 },
}

/// Outcome of comparing our output against reference `zic` for a single zone.
#[derive(Debug, Clone)]
pub struct ZoneComparison {
    pub zone: String,
    pub kind: ComparisonKind,
    /// Differences found; empty means the two agree under this comparison kind.
    pub differences: Vec<Difference>,
    /// Whether the two files were byte-identical (a stricter, informational signal).
    pub byte_identical: bool,
}

impl ZoneComparison {
    /// Whether the two agree under the comparison that was run. Note this is scoped to the
    /// `kind`: a `DecodedTzif` match is *not* a claim about `zdump` behaviour, and a
    /// `ZdumpBehaviour` match is scoped to its declared horizon.
    pub fn is_match(&self) -> bool {
        self.differences.is_empty()
    }

    /// A short, deterministic, honest human summary that names the comparison performed.
    pub fn summary(&self) -> String {
        let label = match self.kind {
            ComparisonKind::DecodedTzif => "decoded TZif match".to_string(),
            ComparisonKind::ZdumpBehaviour { lo, hi } => {
                format!("zdump behaviour match over {lo}..{hi}")
            }
        };
        let mut s = String::new();
        if self.is_match() {
            s.push_str(&format!(
                "{}: {label}{}",
                self.zone,
                if self.byte_identical {
                    " (byte-identical)"
                } else {
                    ""
                }
            ));
        } else {
            let what = match self.kind {
                ComparisonKind::DecodedTzif => "decoded-TZif",
                ComparisonKind::ZdumpBehaviour { .. } => "zdump-behaviour",
            };
            s.push_str(&format!(
                "{}: {} {what} difference(s):",
                self.zone,
                self.differences.len()
            ));
            for d in &self.differences {
                s.push_str(&format!(
                    "\n  - {}: ours={:?} theirs={:?}",
                    d.what, d.ours, d.theirs
                ));
            }
        }
        s
    }
}
