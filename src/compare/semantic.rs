//! Semantic comparison of two TZif files.
//!
//! Byte-equality is too strict a contract in general: `zic`'s slim heuristics, type
//! ordering, and de-duplication can differ from ours while the *meaning* is identical. So
//! the binding contract is semantic — do the two files describe the same local time for
//! every instant? We approximate that faithfully by comparing:
//!
//! * the **footer** POSIX `TZ` string (governs the open-ended future); and
//! * the **transition timeline**: the same number of transitions, at the same UT instants,
//!   each switching to a local-time-type with the same `(utoff, is_dst, abbreviation)`;
//! * the **initial** local-time-type effective before the first transition.
//!
//! To compare initial types consistently we apply the standard reader convention to *both*
//! files: with no transitions, use type 0; otherwise use the first non-DST type (falling
//! back to type 0). Applying one rule to both sides makes the comparison fair.

use crate::tzif::{LocalTimeType, ParsedTzif};

/// A single semantic difference, rendered for humans.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Difference {
    pub what: String,
    pub ours: String,
    pub theirs: String,
}

/// The effective behaviour of a local-time-type, abstracted away from its storage index.
type Eff = (i32, bool, String);

fn eff(t: &LocalTimeType) -> Eff {
    (t.utoff, t.is_dst, t.abbr.clone())
}

/// The local-time-type effective before the first transition, per reader convention.
fn initial_type(p: &ParsedTzif) -> Eff {
    if p.transitions.is_empty() {
        return eff(&p.types[0]);
    }
    let idx = p.types.iter().position(|t| !t.is_dst).unwrap_or(0);
    eff(&p.types[idx])
}

/// The effective `(utoff, is_dst, abbreviation)` **at UT instant `t`**, by the standard TZif reader
/// rule: the type of the latest transition with `at <= t`, or the `initial_type` before the first
/// transition. (The open-ended future beyond the last explicit transition is governed by the POSIX
/// footer, which this does *not* evaluate — probe instants for semantic witnesses are chosen within the
/// explicit range, or the caller treats post-footer instants as `NotApplicable`.) Reused by T15.3
/// semantic witnesses so both zic-rs and the reference are read by one identical rule.
pub fn effective_at(p: &ParsedTzif, t: i64) -> (i32, bool, String) {
    match p.transitions.iter().rev().find(|tr| tr.at <= t) {
        Some(tr) => eff(&p.types[tr.type_index as usize]),
        None => initial_type(p),
    }
}

/// Compare two parsed TZif files; an empty result means "semantically identical".
pub fn diff(ours: &ParsedTzif, theirs: &ParsedTzif) -> Vec<Difference> {
    let mut diffs = Vec::new();

    if ours.footer != theirs.footer {
        diffs.push(Difference {
            what: "footer (POSIX TZ)".into(),
            ours: ours.footer.clone(),
            theirs: theirs.footer.clone(),
        });
    }

    let oi = initial_type(ours);
    let ti = initial_type(theirs);
    if oi != ti {
        diffs.push(Difference {
            what: "initial local-time-type".into(),
            ours: format!("{oi:?}"),
            theirs: format!("{ti:?}"),
        });
    }

    if ours.transitions.len() != theirs.transitions.len() {
        diffs.push(Difference {
            what: "transition count".into(),
            ours: ours.transitions.len().to_string(),
            theirs: theirs.transitions.len().to_string(),
        });
        // Counts differ: comparing element-wise below would be noise.
        return diffs;
    }

    for (i, (a, b)) in ours.transitions.iter().zip(&theirs.transitions).enumerate() {
        if a.at != b.at {
            diffs.push(Difference {
                what: format!("transition[{i}] instant"),
                ours: a.at.to_string(),
                theirs: b.at.to_string(),
            });
        }
        let ea = eff(&ours.types[a.type_index as usize]);
        let eb = eff(&theirs.types[b.type_index as usize]);
        if ea != eb {
            diffs.push(Difference {
                what: format!("transition[{i}] target type"),
                ours: format!("{ea:?}"),
                theirs: format!("{eb:?}"),
            });
        }
    }

    diffs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tzif::{write_bytes, TzifData};

    fn parsed(d: &TzifData) -> ParsedTzif {
        crate::tzif::parse(&write_bytes(d).unwrap()).unwrap()
    }

    #[test]
    fn identical_fixed_zones_match() {
        let a = parsed(&TzifData::fixed(-18000, "EST", "EST5"));
        let b = parsed(&TzifData::fixed(-18000, "EST", "EST5"));
        assert!(diff(&a, &b).is_empty());
    }

    #[test]
    fn offset_mismatch_is_reported() {
        let a = parsed(&TzifData::fixed(-18000, "EST", "EST5"));
        let b = parsed(&TzifData::fixed(-14400, "EDT", "EDT4"));
        assert!(!diff(&a, &b).is_empty());
    }
}
