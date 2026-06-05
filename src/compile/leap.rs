//! Compiling a parsed leap table into a zone's TZif leap-correction records (T11.3 Stationary,
//! T11.4 Rolling, T11.5 Expires/v4).
//!
//! Applies `zic`'s `adjleap` (cumulative corrections; each occurrence shifted by the corrections that
//! precede it) and writes the result into [`TzifData::leaps`]. **Stationary** leaps are UT-as-written;
//! **Rolling** leaps are local-wall and stored as `adjusted − utoff` (the offset in effect at the leap
//! instant). An **`Expires`** line appends a *no-op* terminal marker and forces **TZif v4**
//! (content-triggered). It sets `leaps` (and, for `Expires`, the `version`) and — for the `right/`
//! profile — **leap-adjusts the zone's transition instants** (T23.reader-compat.2): a `right/` TZif
//! stores each transition at `posix + cumulative leap correction`, matching reference `zic`'s TAI-based
//! encoding (a leap second is still not itself a local-time-type transition). It is invoked **only** when
//! a leap source is given (`-L`), so the default/POSIX profile is byte-unchanged (CORE.1 safe).
//! **`Rolling` + `-r` is a hard error**; leap compilation under `-r` (table truncation) is otherwise
//! deferred and fails closed here.

use crate::error::{Error, Result};
use crate::model::LeapTable;
use crate::tzif::{LeapRecord, LocalTimeType, Transition, TzifData};
use crate::RangeSpec;

const SECS_PER_DAY: i64 = 86_400;

/// Apply a leap table to `data`, replacing [`TzifData::leaps`] with the adjusted (cumulative) TZif
/// leap-correction records. Matches reference `zic`'s `adjleap` + emission:
/// * `corr` **accumulates** (`+1, +2, …`); each occurrence is the raw instant **plus** the
///   corrections that precede it.
/// * **Stationary** (T11.3) leaps are UT-as-written → the occurrence is stored verbatim.
/// * **Rolling** (T11.4) leaps are *local-wall* → the occurrence is `adjusted − utoff`, where `utoff`
///   is the UT offset of the local-time-type **in effect at the leap instant** in `data`'s transition
///   stream (before the first transition: the first non-DST type, else type 0) — exactly `zic`'s
///   `todo = leap.trans - utoffs[j]`.
///
/// An **`Expires`** line (T11.5) appends a no-op terminal marker at the (correction-shifted) expiry
/// instant and forces `version = b'4'`. **`Rolling` + `-r` is a hard error** (`range.is_some()` ⇒
/// refuse, matching `zic`'s `leapadd`); leap compilation under `-r` is otherwise deferred. Leaps
/// closer than 28 days are rejected. **`right/` transition adjustment (T23.reader-compat.2):** each
/// transition instant is shifted by the cumulative leap correction effective there (POSIX `types`/`footer`
/// untouched). For a transition-free zone (e.g. UTC) this is a no-op; the default/POSIX profile never calls
/// this function, so CORE.1 output is byte-unchanged.
pub fn apply_leaps(data: &mut TzifData, table: &LeapTable, range: Option<RangeSpec>) -> Result<()> {
    // Range truncation of a leap table (leap-table truncation v4 trigger + expiry clamping to `hi`)
    // is a deeper `-r`×leap composition, deferred. The only pinned `-r`×leap rule is the Rolling
    // hard error below; otherwise we only support leap compilation without `-r`.
    if range.is_some() && (table.expires.is_some() || !table.entries.is_empty()) {
        if table.entries.iter().any(|e| e.rolling) {
            // Part B (T11.4): Rolling + `-r` is a hard error (`zic`'s `leapadd`).
            return Err(Error::config(
                "Rolling leap seconds are not supported with -r (range truncation)",
            ));
        }
        return Err(Error::message(
            "leap-second compilation with -r (range truncation) is not yet implemented",
        ));
    }

    let mut out = Vec::with_capacity(table.entries.len());
    let mut last: i32 = 0; // cumulative correction so far
    let mut prevtrans: i64 = 0; // previous *raw* occurrence (adjleap starts at 0)
    for e in &table.entries {
        if e.trans - prevtrans < 28 * SECS_PER_DAY {
            return Err(Error::message("Leap seconds too close together"));
        }
        prevtrans = e.trans;
        let adjusted = e.trans + last as i64; // occurrence shifted by prior corrections
        let corr = last + e.correction; // cumulative correction at/after this leap
                                        // Part A (T11.4): Rolling = local-wall → subtract the offset in effect at the leap.
        let trans = if e.rolling {
            adjusted - utoff_in_effect(&data.transitions, &data.types, adjusted) as i64
        } else {
            adjusted
        };
        out.push(LeapRecord { trans, corr });
        last += e.correction;
    }

    // T11.5 — `Expires`: append a **no-op** terminal record marking the leap-table expiry, and force
    // **TZif v4** (content-triggered, not "newer is better"). The expiry instant is itself shifted by
    // the total cumulative correction (`zic`'s `adjleap`: `leapexpires += last`); the record's `corr`
    // is the *last cumulative* correction (no ±1 — it is a marker, not a real leap second). The last
    // real leap must precede the expiry.
    if let Some(raw_expires) = table.expires {
        let adj_expires = raw_expires + last as i64;
        if let Some(prev) = out.last() {
            if prev.trans >= adj_expires {
                return Err(Error::message(
                    "last Leap time does not precede Expires time",
                ));
            }
        }
        out.push(LeapRecord {
            trans: adj_expires,
            corr: last, // no-op: correction unchanged from the last real leap (0 if none)
        });
        data.version = b'4';
    }

    // T23.reader-compat.2 FIX — leap-adjust the zone's TRANSITION instants for the `right/` profile.
    // A `right/` (leap-aware, TAI-based) TZif stores each transition at `posix_time + cumulative leap
    // correction effective at that instant`; reference `zic` does this, and a leap-aware reader (`zdump`)
    // subtracts the corrections to recover civil time. zic-rs previously left transitions at their POSIX
    // values (the old "orthogonal to transitions" design), so `right/<zone-with-transitions>` drifted
    // behind reference by the running leap count (found on `right/America/New_York`; `right/Etc/UTC` has no
    // transitions, which is why T11's UTC-only check missed it). The correction at a transition is the sum
    // of the leap-second corrections whose raw occurrence is at/before the transition. No-op for
    // transition-free zones; the default/POSIX profile never calls `apply_leaps`, so CORE.1 is unaffected.
    if !data.transitions.is_empty() && !table.entries.is_empty() {
        for tr in &mut data.transitions {
            let corr: i32 = table
                .entries
                .iter()
                .filter(|e| e.trans <= tr.at)
                .map(|e| e.correction)
                .sum();
            tr.at += corr as i64;
        }
    }

    data.leaps = out;
    Ok(())
}

/// The UT offset of the local-time-type in effect at instant `t` in `data`'s transition stream —
/// `zic`'s leap-emission `utoffs[j]` selection. Before the first transition (or with none), the
/// first non-DST type, else type 0.
fn utoff_in_effect(transitions: &[Transition], types: &[LocalTimeType], t: i64) -> i32 {
    if let Some(tr) = transitions.iter().rev().find(|tr| tr.at <= t) {
        types[tr.type_index as usize].utoff
    } else {
        types
            .iter()
            .find(|ty| !ty.is_dst)
            .or_else(|| types.first())
            .map(|ty| ty.utoff)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{LeapSecond, LeapTable};
    use crate::tzif::TzifData;

    fn stationary(trans: i64) -> LeapSecond {
        LeapSecond {
            trans,
            correction: 1,
            rolling: false,
        }
    }

    #[test]
    fn cumulative_corrections_and_shifted_trans() {
        // Three +1 Stationary leaps, ~yearly apart.
        let t = LeapTable {
            entries: vec![
                stationary(78_796_800),  // 1972-07-01
                stationary(94_694_400),  // 1973-01-01 (raw)
                stationary(126_230_400), // 1974-01-01 (raw)
            ],
            expires: None,
        };
        let mut d = TzifData::fixed(0, "UTC", "");
        apply_leaps(&mut d, &t, None).unwrap();
        assert_eq!(d.leaps.len(), 3);
        assert_eq!(d.leaps[0].corr, 1);
        assert_eq!(d.leaps[1].corr, 2);
        assert_eq!(d.leaps[2].corr, 3);
        // trans shifted by the corrections that precede each leap.
        assert_eq!(d.leaps[0].trans, 78_796_800);
        assert_eq!(d.leaps[1].trans, 94_694_400 + 1);
        assert_eq!(d.leaps[2].trans, 126_230_400 + 2);
        // A transition-FREE zone has nothing to leap-adjust (the transition shift below is a no-op here).
        assert!(d.transitions.is_empty());
        assert_eq!(d.version, b'2');
    }

    /// T23.reader-compat.2 regression — a `right/` zone **with transitions** must have each transition
    /// instant shifted by the cumulative leap correction in effect there (reference `zic`'s TAI-based
    /// `right/` encoding). A transition-free zone (the test above) cannot catch this — which is exactly
    /// why T11's `right/UTC`-only witness missed the divergence the reader gauntlet later found on
    /// `right/America/New_York`.
    #[test]
    fn right_profile_shifts_transitions_by_cumulative_leap_correction() {
        let leaps = LeapTable {
            entries: vec![stationary(3_000_000), stationary(6_000_000)], // ≥28 days apart, +1 each
            expires: None,
        };
        let mut d = TzifData::fixed(0, "X", ""); // 1 local-time type (index 0), no transitions yet
        d.transitions = vec![
            crate::tzif::Transition {
                at: 1_000_000,
                type_index: 0,
            }, // before any leap   → +0
            crate::tzif::Transition {
                at: 4_000_000,
                type_index: 0,
            }, // after leap1 (3e6) → +1
            crate::tzif::Transition {
                at: 9_000_000,
                type_index: 0,
            }, // after both leaps  → +2
        ];
        apply_leaps(&mut d, &leaps, None).unwrap();
        assert_eq!(
            d.transitions[0].at, 1_000_000,
            "pre-leap transition is unshifted"
        );
        assert_eq!(d.transitions[1].at, 4_000_000 + 1, "one leap precedes → +1");
        assert_eq!(d.transitions[2].at, 9_000_000 + 2, "two leaps precede → +2");
        assert_eq!(d.leaps.len(), 2); // leap table itself still emitted
    }

    /// T11.4 Part A — a Rolling leap on a +5h fixed-offset zone is stored as `adjusted − 18000`
    /// (local-wall → UT), while a Stationary leap is UT-as-written; cumulative `corr` is identical.
    #[test]
    fn rolling_subtracts_offset_in_effect() {
        let raw = 1_483_228_800; // 2017-01-01 00:00:00 UT
        let mut roll = TzifData::fixed(18_000, "E5T", ""); // +5h, single non-DST type
        apply_leaps(
            &mut roll,
            &LeapTable {
                entries: vec![LeapSecond {
                    trans: raw,
                    correction: 1,
                    rolling: true,
                }],
                expires: None,
            },
            None,
        )
        .unwrap();
        assert_eq!(roll.leaps[0].trans, raw - 18_000, "Rolling = local-wall");
        assert_eq!(roll.leaps[0].corr, 1);

        let mut stat = TzifData::fixed(18_000, "E5T", "");
        apply_leaps(
            &mut stat,
            &LeapTable {
                entries: vec![stationary(raw)],
                expires: None,
            },
            None,
        )
        .unwrap();
        assert_eq!(stat.leaps[0].trans, raw, "Stationary = UT-as-written");
        // The two differ by exactly the zone offset.
        assert_eq!(stat.leaps[0].trans - roll.leaps[0].trans, 18_000);
    }

    /// T11.4 Part B — a Rolling leap with `-r` (range) is a hard error, before any output.
    #[test]
    fn rolling_with_range_fails_closed() {
        let mut d = TzifData::fixed(0, "UTC", "");
        let table = LeapTable {
            entries: vec![LeapSecond {
                trans: 78_796_800,
                correction: 1,
                rolling: true,
            }],
            expires: None,
        };
        // Without -r, Rolling now succeeds; with -r it must fail.
        assert!(apply_leaps(&mut d.clone(), &table, None).is_ok());
        let range = Some(RangeSpec {
            lo: Some(0),
            hi: None,
        });
        assert!(
            apply_leaps(&mut d, &table, range).is_err(),
            "Rolling + -r is a hard error"
        );
    }

    /// T11.5 — `Expires` appends a **no-op** terminal record (corr unchanged) at the expiry instant
    /// (itself shifted by the total cumulative correction) and forces **TZif v4**.
    #[test]
    fn expires_appends_noop_and_sets_v4() {
        let mut d = TzifData::fixed(0, "UTC", "");
        let raw_leap = 78_796_800; // 1972-07-01
        let raw_exp = 1_700_000_000;
        apply_leaps(
            &mut d,
            &LeapTable {
                entries: vec![stationary(raw_leap)],
                expires: Some(raw_exp),
            },
            None,
        )
        .unwrap();
        assert_eq!(d.version, b'4', "Expires is content-triggered v4");
        assert_eq!(d.leaps.len(), 2, "one real leap + the no-op expiry marker");
        // The marker: occurrence shifted by the total correction (+1), correction unchanged (no ±1).
        assert_eq!(d.leaps[1].trans, raw_exp + 1);
        assert_eq!(d.leaps[1].corr, d.leaps[0].corr, "no-op: corr unchanged");
        assert_eq!(d.leaps[1].corr, 1);
    }

    /// The last real leap must precede the expiry.
    #[test]
    fn expires_before_last_leap_rejected() {
        let mut d = TzifData::fixed(0, "UTC", "");
        let t = LeapTable {
            entries: vec![stationary(1_500_000_000)],
            expires: Some(1_400_000_000), // before the leap
        };
        assert!(apply_leaps(&mut d, &t, None).is_err());
    }

    #[test]
    fn too_close_rejected() {
        let t = LeapTable {
            entries: vec![stationary(78_796_800), stationary(78_796_800 + 86_400)],
            expires: None,
        };
        let mut d = TzifData::fixed(0, "UTC", "");
        assert!(apply_leaps(&mut d, &t, None).is_err());
    }
}
