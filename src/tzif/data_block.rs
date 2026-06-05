//! Building a TZif **data block** (RFC 9636 §3.2).
//!
//! A data block is seven variable-length arrays, in this fixed order:
//!
//! 1. transition times      — `timecnt` × `time_size` (4 or 8 bytes), big-endian, ascending
//! 2. transition type idxs   — `timecnt` × 1 byte (index into the type array)
//! 3. local-time-type records— `typecnt` × 6 bytes (`utoff:i32`, `isdst:u8`, `desigidx:u8`)
//! 4. designation table      — `charcnt` bytes (NUL-separated, possibly shared, abbrevs)
//! 5. leap-second records    — `leapcnt` × (`time_size` + 4 bytes)  [none in T1]
//! 6. standard/wall indicators— `isstdcnt` bytes                    [none in T1]
//! 7. UT/local indicators    — `isutcnt` bytes                      [none in T1]
//!
//! `time_size` is 4 for the v1 block and 8 for the v2+ block; everything else is identical,
//! which is why one function serves both.

use super::header::{write_header, Counts};
use super::{LeapRecord, LocalTimeType, Transition};
use crate::error::{Error, Result};

/// Build the abbreviation designation table and the per-type index into it, using the same
/// **suffix-sharing** packer as reference `zic` (`zic.c::addabbr`).
///
/// `zic` packs abbreviations so that when one is a *suffix* of another they share a single
/// NUL-terminated run of bytes: e.g. `HST` reuses the tail of `AHST`, and `LMT` the tail of
/// `PLMT`, with the shorter one's index pointing partway into the longer one. This is purely a
/// layout optimisation — every `tt_desigidx` still resolves to the correct NUL-terminated string,
/// so behaviour (`zdump`) is unaffected — but matching it is what `charcnt` structural parity needs
/// (campaign T8-abbrev). Plain exact-match dedup (our previous behaviour) is just the `alen == clen`
/// special case of suffix sharing.
///
/// Algorithm — verbatim from `zic.c` (`addabbr` + the two-pass loop in `writezone`):
///   1. **build** — add each type's abbreviation in order; [`add_abbr`] mutates the table per its
///      three cases (new abbr is a suffix of an entry → no growth; an entry is a suffix of the new
///      abbr → splice the missing prefix in; otherwise append). Offsets are *not* recorded here
///      because a later splice can shift earlier entries.
///   2. **index** — with the table now final, look each abbreviation up again ([`abbr_offset`]);
///      every lookup now hits the suffix case and returns a stable offset without growing the table.
///
/// The total byte count is independent of add-order (suffix sharing is confluent on size), so this
/// matches `zic`'s `charcnt` regardless of our type-intern order. Returns `(table_bytes,
/// desig_indices)`. The designation index (`tt_desigidx`) is a single byte, so each offset must fit
/// in `u8`; rather than panic on pathological input we return an error.
fn build_designations(types: &[LocalTimeType]) -> Result<(Vec<u8>, Vec<u8>)> {
    // Pass 1: build the shared table (offsets resolved afterwards).
    let mut table: Vec<u8> = Vec::new();
    for t in types {
        add_abbr(&mut table, t.abbr.as_bytes());
    }
    // Pass 2: resolve each type's stable offset. The table is complete, so every lookup succeeds
    // and never grows it; the single-byte `tt_desigidx` bounds each offset to 0..=255.
    let mut indices = Vec::with_capacity(types.len());
    for t in types {
        let off = abbr_offset(&table, t.abbr.as_bytes())
            .expect("every abbreviation is present after the build pass");
        let off = u8::try_from(off).map_err(|_| {
            Error::message(
                "abbreviation designation table exceeds 255 bytes \
                 (too many/long timezone abbreviations)",
            )
        })?;
        indices.push(off);
    }
    Ok((table, indices))
}

/// Length of the NUL-terminated abbreviation stored at byte offset `i` (excluding the NUL).
fn cstr_len(table: &[u8], i: usize) -> usize {
    table[i..]
        .iter()
        .position(|&b| b == 0)
        .expect("designation table entries are NUL-terminated")
}

/// Add one abbreviation to the shared designation `table`, matching `zic.c::addabbr`. The three
/// cases (in `zic`'s search order):
///   1. **`abbr` is a suffix of an existing entry** (incl. exact match) — already representable;
///      nothing is added.
///   2. **an existing entry is a suffix of the (longer) `abbr`** — splice `abbr`'s missing prefix
///      in *before* that entry, so `abbr` now spans `[i, i+alen)` and the old entry remains its
///      NUL-terminated tail (its bytes shift right but stay valid).
///   3. **no overlap** — append `abbr` and a NUL.
fn add_abbr(table: &mut Vec<u8>, abbr: &[u8]) {
    let alen = abbr.len();
    let mut i = 0;
    while i < table.len() {
        let clen = cstr_len(table, i);
        if alen <= clen {
            // Case 1: is `abbr` the last `alen` bytes of the entry at `i`? (`alen == 0` — an empty
            // abbreviation — trivially matches any entry's terminating position.)
            let isuff = i + (clen - alen);
            if table[isuff..isuff + alen] == *abbr {
                return;
            }
        } else if table[i..i + clen] == abbr[alen - clen..] {
            // Case 2: the entry at `i` is a suffix of `abbr`. Insert only the prefix bytes
            // `abbr[..alen-clen]` before it (Vec::splice shifts the tail right, like zic's memmove).
            table.splice(i..i, abbr[..alen - clen].iter().copied());
            return;
        }
        i += clen + 1;
    }
    // Case 3: append `abbr` followed by its NUL terminator.
    table.extend_from_slice(abbr);
    table.push(0);
}

/// Find the offset of `abbr` in a fully-built designation `table` (the pass-2 lookup): the first
/// entry of which `abbr` is a NUL-terminated suffix, returning that suffix's offset. Mirrors
/// `zic.c::addabbr`'s case-1 search; `None` only if `abbr` was never added.
fn abbr_offset(table: &[u8], abbr: &[u8]) -> Option<usize> {
    let alen = abbr.len();
    let mut i = 0;
    while i < table.len() {
        let clen = cstr_len(table, i);
        if alen <= clen {
            let isuff = i + (clen - alen);
            if table[isuff..isuff + alen] == *abbr {
                return Some(isuff);
            }
        }
        i += clen + 1;
    }
    None
}

/// Serialise one complete block (header + data) for the given `version`/`time_size`.
pub fn write_block(
    out: &mut Vec<u8>,
    version: u8,
    time_size: usize,
    types: &[LocalTimeType],
    transitions: &[Transition],
    leaps: &[LeapRecord],
) -> Result<()> {
    let (designations, desig_idx) = build_designations(types)?;

    let counts = Counts {
        isutcnt: 0,
        isstdcnt: 0,
        leapcnt: leaps.len() as u32,
        timecnt: transitions.len() as u32,
        typecnt: types.len() as u32,
        charcnt: designations.len() as u32,
    };
    write_header(out, version, &counts);

    // 1. transition times.
    for tr in transitions {
        write_time(out, tr.at, time_size);
    }
    // 2. transition type indices.
    for tr in transitions {
        out.push(tr.type_index);
    }
    // 3. local-time-type records (6 bytes each).
    for (i, t) in types.iter().enumerate() {
        out.extend_from_slice(&t.utoff.to_be_bytes());
        out.push(t.is_dst as u8);
        out.push(desig_idx[i]);
    }
    // 4. designation string table.
    out.extend_from_slice(&designations);
    // 5. leap-second records — each is the (already-adjusted) occurrence time + cumulative
    //    correction (T11.3). Leaps are orthogonal to the transition/type arrays above; ordinary
    //    zones pass an empty slice, so `leapcnt` stays 0 and the output is byte-unchanged.
    for lp in leaps {
        write_time(out, lp.trans, time_size);
        out.extend_from_slice(&lp.corr.to_be_bytes());
    }
    // 6/7. standard/wall + UT/local indicators: none emitted (counts are zero).
    Ok(())
}

/// Write a transition time as a big-endian signed integer of `time_size` bytes (4 or 8).
fn write_time(out: &mut Vec<u8>, at: i64, time_size: usize) {
    match time_size {
        4 => {
            // The v1 block only holds 32-bit times; callers must not pass out-of-range
            // values here. In T1 the v1 block carries no transitions, so this is unused.
            let v = i32::try_from(at).expect("32-bit transition time out of range");
            out.extend_from_slice(&v.to_be_bytes());
        }
        8 => out.extend_from_slice(&at.to_be_bytes()),
        other => unreachable!("unsupported TZif time size {other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ty(abbr: &str) -> LocalTimeType {
        LocalTimeType {
            utoff: 0,
            is_dst: false,
            abbr: abbr.to_string(),
        }
    }

    /// Decode the NUL-terminated string at `off` — what a reader resolves a `tt_desigidx` to.
    fn at(table: &[u8], off: u8) -> String {
        let s = off as usize;
        let end = s + cstr_len(table, s);
        String::from_utf8(table[s..end].to_vec()).unwrap()
    }

    /// Every type's index must resolve back to its exact abbreviation, whatever the packing.
    fn assert_indices_resolve(types: &[LocalTimeType]) -> Vec<u8> {
        let (table, idx) = build_designations(types).unwrap();
        for (t, &off) in types.iter().zip(&idx) {
            assert_eq!(
                at(&table, off),
                t.abbr,
                "index for {:?} must resolve",
                t.abbr
            );
        }
        table
    }

    #[test]
    fn suffix_is_shared_regardless_of_order() {
        // `HST` is a suffix of `AHST`; the table should hold one shared run (`AHST\0`, 5 bytes),
        // not two (`AHST\0HST\0`, 9). This is the America/Adak saving. Order must not matter.
        let longer_first = assert_indices_resolve(&[ty("AHST"), ty("HST")]);
        let shorter_first = assert_indices_resolve(&[ty("HST"), ty("AHST")]);
        assert_eq!(longer_first.len(), 5, "AHST\\0 only");
        assert_eq!(shorter_first.len(), 5, "splice yields the same size");
        // Behaviour-equivalent layout: both contain a single `AHST\0`.
        assert_eq!(longer_first, b"AHST\0");
        assert_eq!(shorter_first, b"AHST\0");
    }

    #[test]
    fn plmt_lmt_share_like_ho_chi_minh() {
        // `LMT` ⊂ `PLMT`: one shared run. (Asia/Ho_Chi_Minh: ours was 4 bytes over reference.)
        let table = assert_indices_resolve(&[ty("PLMT"), ty("LMT"), ty("+07")]);
        assert_eq!(table, b"PLMT\0+07\0"); // 9 bytes, not 13
    }

    #[test]
    fn exact_duplicates_dedup() {
        // The old behaviour (exact-match dedup) is the `alen == clen` suffix case — still holds.
        let table = assert_indices_resolve(&[ty("EST"), ty("EDT"), ty("EST")]);
        assert_eq!(table, b"EST\0EDT\0");
    }

    #[test]
    fn non_suffix_abbrs_are_appended() {
        let table = assert_indices_resolve(&[ty("GMT"), ty("BST")]);
        assert_eq!(table, b"GMT\0BST\0"); // no shared suffix
    }

    #[test]
    fn empty_abbr_reuses_a_terminator() {
        // An empty abbreviation resolves to any entry's terminating position — no extra bytes
        // beyond the first append.
        let table = assert_indices_resolve(&[ty("UTC"), ty("")]);
        assert_eq!(table, b"UTC\0");
    }
}
