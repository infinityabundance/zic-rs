//! Reading TZif back into semantics.
//!
//! This is the inverse of the writer, and it pulls double duty:
//!
//! * the writer's tests round-trip their own output through [`parse`] to prove the bytes
//!   decode to what went in; and
//! * the oracle (`compare::semantic`) decodes **reference `zic`** output with the very same
//!   code, so a semantic diff compares like with like.
//!
//! We decode the v2+ block (the authoritative data in modern files) plus the footer. The
//! v1 block is parsed only enough to know how many bytes to skip — in slim files it is the
//! placeholder stub described in `tzif/mod.rs`, so its contents are not meaningful.

use super::header::{Counts, MAGIC};
use super::{LeapRecord, LocalTimeType, Transition};
use crate::error::{Error, Result};

/// The semantically-meaningful contents decoded from a TZif file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTzif {
    pub version: u8,
    pub types: Vec<LocalTimeType>,
    pub transitions: Vec<Transition>,
    /// Decoded leap-second records of the authoritative block (T11.3): (occurrence, cumulative
    /// correction). Empty for ordinary zones.
    pub leaps: Vec<LeapRecord>,
    pub footer: String,
    /// The raw count fields of the *authoritative* block's header (the v2+ block for a v2+
    /// file; the sole block for a v1-only file). Exposed so the structural-parity inventory
    /// (T8) can compare `timecnt`/`typecnt`/`charcnt`/`isutcnt`/`isstdcnt`/`leapcnt` against
    /// reference `zic` without re-decoding the header.
    pub counts: Counts,
    /// Raw structural bytes the RFC-9636 validator needs but which `parse` otherwise collapses or skips
    /// (T23.kani.3f.enforce): per-type `isdst`/`desigidx` octets, the designation table, and the std/UT
    /// indicator arrays. `parse` stays memory-safe-*lenient* (it does not reject on these); the strict RFC
    /// byte-validity rules are applied by `rfc9636::validate` over this. (`utoff` lives in [`types`].)
    pub raw: RawStructural,
}

/// Raw structural fields of the authoritative block, captured for the RFC-9636 validator. *Memory-safe ≠
/// format-valid:* `parse` does not reject on these byte values; `rfc9636::validate` applies the strict
/// RFC 9636 §3.2 rules (`isdst ∈ {0,1}`, designation-index validity, indicator byte values + `isut⇒isstd`
/// pairing — the predicates proven by T23.kani.3f.1–.4). Indexed parallel to `ParsedTzif::types`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RawStructural {
    pub isdst: Vec<u8>,
    pub desigidx: Vec<u8>,
    pub designation: Vec<u8>,
    pub std_indicators: Vec<u8>,
    pub ut_indicators: Vec<u8>,
}

/// A tiny cursor over the input with bounds-checked reads.
struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Cursor { buf, pos: 0 }
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self
            .pos
            .checked_add(n)
            .ok_or_else(|| Error::message("TZif length overflow"))?;
        if end > self.buf.len() {
            return Err(Error::message("TZif truncated"));
        }
        let s = &self.buf[self.pos..end];
        self.pos = end;
        Ok(s)
    }

    fn u32(&mut self) -> Result<u32> {
        let b = self.take(4)?;
        Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// Bytes not yet consumed. Used (T17.5) to bound an untrusted declared count against the data that
    /// is *physically present* before allocating for it.
    fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.pos)
    }

    /// Advance past `n` bytes (T17.5: checked — a count-driven skip, e.g. the v1 block, must not
    /// overflow `pos` nor land past the buffer end; either is a typed `Err`, never a wrap or a silent
    /// out-of-range cursor).
    fn skip(&mut self, n: usize) -> Result<()> {
        let end = self
            .pos
            .checked_add(n)
            .ok_or_else(|| Error::message("TZif length overflow"))?;
        if end > self.buf.len() {
            return Err(Error::message(
                "TZif truncated (a counted block extends past the input)",
            ));
        }
        self.pos = end;
        Ok(())
    }
}

/// Read a 44-byte header, validating the magic, and return its version byte and counts.
fn read_header(c: &mut Cursor<'_>) -> Result<(u8, Counts)> {
    let magic = c.take(4)?;
    if magic != MAGIC {
        return Err(Error::message("bad TZif magic"));
    }
    let version = c.take(1)?[0];
    let _reserved = c.take(15)?;
    let counts = Counts {
        isutcnt: c.u32()?,
        isstdcnt: c.u32()?,
        leapcnt: c.u32()?,
        timecnt: c.u32()?,
        typecnt: c.u32()?,
        charcnt: c.u32()?,
    };
    Ok((version, counts))
}

/// Number of bytes a data block occupies for the given counts and `time_size`.
///
/// **T17.5 (CountArithmeticVerdict):** every term is `count × element-size`, where the counts are
/// **untrusted `u32`s read straight from the header**. The arithmetic is fully **checked** — a header
/// claiming `timecnt = u32::MAX` (× 8 + the other terms) overflows `usize` on a 32-bit target and is a
/// large value on 64-bit; either way an overflow is a typed `Err`, never a wrap (which on 32-bit would
/// under-compute the length and mis-slice) and never a panic (which `overflow-checks` would otherwise
/// raise). *Counts from input are hostile until range-checked.*
fn checked_block_len(counts: &Counts, time_size: usize) -> Result<usize> {
    let oflow =
        || Error::message("TZif count arithmetic overflow (declared counts are implausibly large)");
    // Accumulate `count * size` terms with checked math.
    let term = |count: u32, size: usize| count_mul(count, size).ok_or_else(oflow);
    let mut total: usize = 0;
    for t in [
        term(counts.timecnt, time_size)?,     // transition times
        term(counts.timecnt, 1)?,             // transition type indices
        term(counts.typecnt, 6)?,             // ttinfo records
        term(counts.charcnt, 1)?,             // designation table
        term(counts.leapcnt, time_size + 4)?, // leap records
        term(counts.isstdcnt, 1)?,            // std/wall indicators
        term(counts.isutcnt, 1)?,             // ut/local indicators
    ] {
        total = total.checked_add(t).ok_or_else(oflow)?;
    }
    Ok(total)
}

/// `count (u32) * size (usize)` with overflow → `None` (T17.5).
fn count_mul(count: u32, size: usize) -> Option<usize> {
    (count as usize).checked_mul(size)
}

/// The position of the first transition type-index that is **out of range** for a `typecnt`-length
/// local-time-type table, or `None` if every index is in range (RFC 9636 §3.2).
///
/// Pure, allocation-free, and format-free **by design** so it can be bounded-model-checked in isolation
/// (`audits/kani`, T23.kani.3a) — the proof shows `None` ⟺ every index is `< typecnt`, which is exactly the
/// precondition that makes the downstream `types[type_index]` indexing safe. The caller ([`read_block`])
/// turns a `Some(pos)` into the typed error (the `format!` stays out of the proven helper).
fn first_oob_type_index(type_indices: &[u8], typecnt: usize) -> Option<usize> {
    type_indices.iter().position(|&i| (i as usize) >= typecnt)
}

/// Whether a designation (abbreviation) index is a **slice-safe** start offset into a `table_len`-byte
/// abbreviation table: `idx <= table_len`, so `&table[idx..]` is an in-bounds (possibly empty) slice and
/// [`read_cstr`] cannot panic on a hostile `ttinfo` designation index. Pure + alloc-free → BMC-checkable
/// (`audits/kani`, T23.kani.3b proves exactly this).
///
/// **Precision boundary (memory-safe ≠ format-valid):** this is *Rust slice safety*, **not** RFC 9636
/// designation-index *validity*. RFC 9636 §3.2 is stricter — a conformant `desigidx` must be `< charcnt`
/// (not merely `<= charcnt`), with a NUL octet at or after it, and `charcnt` itself must be non-zero. The
/// reader is intentionally memory-safe-*lenient* here (an out-of-range-but-in-bounds index yields an empty
/// or unterminated abbreviation, never a panic); enforcing the stricter RFC structural rule is the RFC-9636
/// validator's job (`tzif/rfc9636.rs`) and is currently a tracked gap (see `audits/claim-boundary-map.md`).
fn abbr_index_slice_safe(idx: usize, table_len: usize) -> bool {
    idx <= table_len
}

/// RFC 9636 §3.2 designation-index **validity** (stricter than the slice-safety of `abbr_index_slice_safe`):
/// a conformant `ttinfo` designation index must be `< charcnt` (not merely `<= charcnt`), the designation
/// table must be non-empty (`charcnt != 0`), and a NUL terminator must exist at or after `idx`. The reader is
/// intentionally memory-safe-*lenient* here; this is the proven basis for the stricter RFC check (enforcement
/// in `rfc9636::validate` — now **enforced** there, T23.kani.3f.enforce). Pure → BMC-checkable (T23.kani.3f.1).
pub(crate) fn rfc_designation_index_valid(
    idx: usize,
    charcnt: usize,
    has_nul_at_or_after: bool,
) -> bool {
    charcnt != 0 && idx < charcnt && has_nul_at_or_after
}

/// RFC 9636 §3.2: a `ttinfo` `isdst` octet is structurally valid iff it is 0 or 1. `parse` collapses it to a
/// `bool` via `!= 0`, so the raw byte value is unchecked today — this is the proven basis for the stricter
/// check (T23.kani.3f.2). Pure.
pub(crate) fn isdst_byte_valid(isdst: u8) -> bool {
    isdst <= 1
}

/// RFC 9636 §3.2 indicator-byte pairing: the std/wall (`isstd`) and UT/local (`isut`) octets must each be 0 or
/// 1, and a UT indicator is only meaningful with a standard indicator: `isut == 1 ⇒ isstd == 1`. `parse` reads
/// only the indicator *counts*, not these byte values — proven basis for the stricter check (T23.kani.3f.3).
pub(crate) fn indicator_pair_valid(isut: u8, isstd: u8) -> bool {
    isut <= 1 && isstd <= 1 && (isut == 0 || isstd == 1)
}

/// RFC 9636 §3.2: a `ttinfo` `utoff` (UT offset, seconds) must not be `-2^31` (`i32::MIN`) — the one value
/// whose signed negation overflows, which would break offset arithmetic. Proven basis for the stricter check
/// (T23.kani.3f.4). Pure.
pub(crate) fn utoff_structural_valid(utoff: i32) -> bool {
    utoff != i32::MIN
}

/// Decode a full data block (transitions + types) given its counts and time size.
// The 4-tuple is a private decode result threaded straight into `parse`'s two call sites; a named struct
// would be ceremony for one internal path, so the complexity lint is allowed here with this note.
#[allow(clippy::type_complexity)]
fn read_block(
    c: &mut Cursor<'_>,
    counts: &Counts,
    time_size: usize,
) -> Result<(
    Vec<Transition>,
    Vec<LocalTimeType>,
    Vec<LeapRecord>,
    RawStructural,
)> {
    // T17.5 — bound the *untrusted declared counts* against the data actually present BEFORE allocating
    // for them. The block's full byte length (checked arithmetic) must fit in the remaining input; a
    // header claiming `timecnt = 4e9` with a 100-byte file is rejected here, so `Vec::with_capacity(tc)`
    // below can never be asked to reserve gigabytes from a tiny file (a real DoS otherwise — the
    // allocation happens up front, before the per-element `take()` bounds checks would ever fire).
    let needed = checked_block_len(counts, time_size)?;
    if needed > c.remaining() {
        return Err(Error::message(
            "TZif truncated: declared counts require more bytes than the input contains",
        ));
    }

    let tc = counts.timecnt as usize;
    let ty = counts.typecnt as usize;

    // 1. transition times. `with_capacity(tc)` is now safe: the check above proved the whole block —
    // including `tc * time_size` bytes — fits in the remaining input, so `tc` is bounded by the file size.
    let mut times = Vec::with_capacity(tc);
    for _ in 0..tc {
        let b = c.take(time_size)?;
        let at = match time_size {
            4 => i32::from_be_bytes([b[0], b[1], b[2], b[3]]) as i64,
            8 => i64::from_be_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]),
            _ => return Err(Error::message("bad time size")),
        };
        times.push(at);
    }
    // 2. transition type indices.
    let idxs = c.take(tc)?.to_vec();
    // 3. ttinfo records.
    let mut raw_types = Vec::with_capacity(ty);
    let mut raw_isdst = Vec::with_capacity(ty);
    let mut raw_desigidx = Vec::with_capacity(ty);
    for _ in 0..ty {
        let b = c.take(6)?;
        let utoff = i32::from_be_bytes([b[0], b[1], b[2], b[3]]);
        let is_dst = b[4] != 0;
        let desigidx = b[5] as usize;
        // Capture the raw `isdst`/`desigidx` octets for the RFC-9636 validator (parse stays lenient:
        // `is_dst` is `b[4] != 0`, so `isdst == 2` would be silently `true` without this).
        raw_isdst.push(b[4]);
        raw_desigidx.push(b[5]);
        raw_types.push((utoff, is_dst, desigidx));
    }
    // 4. designation table.
    let table = c.take(counts.charcnt as usize)?;
    let designation = table.to_vec();
    // 5. leap-second records — decoded (T11.3): (occurrence, cumulative correction).
    let mut leaps = Vec::with_capacity(counts.leapcnt as usize);
    for _ in 0..counts.leapcnt as usize {
        let tb = c.take(time_size)?;
        let trans = match time_size {
            4 => i32::from_be_bytes([tb[0], tb[1], tb[2], tb[3]]) as i64,
            8 => i64::from_be_bytes([tb[0], tb[1], tb[2], tb[3], tb[4], tb[5], tb[6], tb[7]]),
            _ => return Err(Error::message("bad time size")),
        };
        let cb = c.take(4)?;
        let corr = i32::from_be_bytes([cb[0], cb[1], cb[2], cb[3]]);
        leaps.push(LeapRecord { trans, corr });
    }
    // 6/7. standard/wall + UT/local indicators — captured for the RFC-9636 validator (byte values + the
    // `isut ⇒ isstd` pairing; `parse` itself does not enforce them — memory-safe ≠ format-valid).
    let std_indicators = c.take(counts.isstdcnt as usize)?.to_vec();
    let ut_indicators = c.take(counts.isutcnt as usize)?.to_vec();

    // Resolve designation indices to strings (NUL-terminated from the index).
    let types = raw_types
        .into_iter()
        .map(|(utoff, is_dst, idx)| {
            let abbr = read_cstr(table, idx)?;
            Ok(LocalTimeType {
                utoff,
                is_dst,
                abbr,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    // T17.1 bounds-guard: every transition's type index must point at a real local-time type.
    // `parse` is the single choke point before any consumer (`compare::semantic::diff`,
    // `compile::leap`) indexes `types[transition.type_index as usize]`, so enforcing the RFC 9636
    // §3.2 invariant (transition type indices < `typecnt`) HERE turns a malformed/hostile TZif into
    // a typed rejection instead of a latent out-of-bounds panic downstream. (`typecnt == 0` with any
    // transition is caught by the same check, since no index can be `< 0`.) The decision is extracted
    // into the pure [`first_oob_type_index`] so it can be bounded-model-checked alone (`audits/kani`,
    // T23.kani.3a) without dragging the allocating parse path through the solver; the `format!` for the
    // error stays here (out of the proven helper). Checked on the raw `idxs` before they are zipped.
    if let Some(pos) = first_oob_type_index(&idxs, types.len()) {
        return Err(Error::message(format!(
            "TZif transition type index {} out of range (typecnt {})",
            idxs[pos],
            types.len()
        )));
    }

    let transitions: Vec<Transition> = times
        .into_iter()
        .zip(idxs)
        .map(|(at, type_index)| Transition { at, type_index })
        .collect();

    Ok((
        transitions,
        types,
        leaps,
        RawStructural {
            isdst: raw_isdst,
            desigidx: raw_desigidx,
            designation,
            std_indicators,
            ut_indicators,
        },
    ))
}

/// Read a NUL-terminated abbreviation starting at `idx` within the designation table.
fn read_cstr(table: &[u8], idx: usize) -> Result<String> {
    if !abbr_index_slice_safe(idx, table.len()) {
        return Err(Error::message("designation index out of range"));
    }
    let rest = &table[idx..];
    let end = rest.iter().position(|&b| b == 0).unwrap_or(rest.len());
    String::from_utf8(rest[..end].to_vec()).map_err(|_| Error::message("non-UTF-8 abbreviation"))
}

/// Parse a complete TZif file into its semantically-meaningful contents.
pub fn parse(bytes: &[u8]) -> Result<ParsedTzif> {
    let mut c = Cursor::new(bytes);
    let (version, v1_counts) = read_header(&mut c)?;

    if version == 0 {
        // A v1-only file: the v1 block *is* the data; there is no v2 block or footer.
        let (transitions, types, leaps, raw) = read_block(&mut c, &v1_counts, 4)?;
        return Ok(ParsedTzif {
            version,
            types,
            transitions,
            leaps,
            footer: String::new(),
            counts: v1_counts,
            raw,
        });
    }

    // Skip the v1 stub block, then decode the authoritative v2+ block. T17.5: the skip distance is a
    // checked count×size sum, and `skip` verifies it stays within the buffer — a header with implausible
    // v1 counts is a typed `Err`, never a wrapped/overrun cursor.
    c.skip(checked_block_len(&v1_counts, 4)?)?;
    let (v2_version, v2_counts) = read_header(&mut c)?;
    let (transitions, types, leaps, raw) = read_block(&mut c, &v2_counts, 8)?;

    // Footer: `\n` <TZ> `\n` occupying the rest of the file.
    let tail = &c.buf[c.pos..];
    let footer = parse_footer(tail)?;

    Ok(ParsedTzif {
        version: v2_version,
        types,
        transitions,
        leaps,
        footer,
        counts: v2_counts,
        raw,
    })
}

/// Extract the POSIX TZ string from the trailing `\n<TZ>\n` footer.
fn parse_footer(tail: &[u8]) -> Result<String> {
    if tail.is_empty() {
        return Ok(String::new());
    }
    // The footer is a `\n<TZ>\n` envelope. Strip both ends rather than indexing `[1..len-1]`:
    // a region lacking BOTH a leading and trailing newline (e.g. a lone `\n`, len 1, which the old
    // `tail[0]==b'\n' && tail[last]==b'\n'` guard accepted because the one byte is both first and
    // last) would otherwise slice `[1..0]` and panic. Such a region is a malformed footer → typed Err.
    let inner = tail
        .strip_prefix(b"\n")
        .and_then(|t| t.strip_suffix(b"\n"))
        .ok_or_else(|| Error::message("malformed TZif footer"))?;
    String::from_utf8(inner.to_vec()).map_err(|_| Error::message("non-UTF-8 footer"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tzif::{write_bytes, TzifData};

    #[test]
    fn single_newline_footer_is_typed_err_not_panic() {
        // T23.cargo-fuzz.2 / F1 regression: a lone `\n` footer region satisfied the old
        // `tail[0]==b'\n' && tail[last]==b'\n'` guard (one byte is both ends) then sliced `[1..0]`
        // and panicked. It is a malformed footer → typed Err, never a panic.
        assert!(parse_footer(b"\n").is_err());
        // Sanity: the well-formed envelope shapes still classify correctly.
        assert_eq!(parse_footer(b"").unwrap(), "");
        assert_eq!(parse_footer(b"\n\n").unwrap(), "");
        assert_eq!(parse_footer(b"\nEST5\n").unwrap(), "EST5");
        assert!(parse_footer(b"EST5").is_err());
    }

    #[test]
    fn round_trip_fixed_offset() {
        let data = TzifData::fixed(-18000, "EST", "EST5");
        let bytes = write_bytes(&data).unwrap();
        let parsed = parse(&bytes).unwrap();
        assert_eq!(parsed.version, b'2');
        assert_eq!(parsed.transitions.len(), 0);
        assert_eq!(parsed.types.len(), 1);
        assert_eq!(parsed.types[0].utoff, -18000);
        assert!(!parsed.types[0].is_dst);
        assert_eq!(parsed.types[0].abbr, "EST");
        assert_eq!(parsed.footer, "EST5");
    }

    #[test]
    fn round_trip_utc() {
        let data = TzifData::fixed(0, "UTC", "UTC0");
        let parsed = parse(&write_bytes(&data).unwrap()).unwrap();
        assert_eq!(parsed.types[0].abbr, "UTC");
        assert_eq!(parsed.footer, "UTC0");
    }

    /// Hand-build a minimal v1-only TZif (`version` byte `0x00` → the v1-block-is-data path) with
    /// exactly one transition and one local-time type, where the transition's type index is
    /// `trans_type_index`. With `typecnt == 1`, index `0` is the only valid value; anything `>= 1`
    /// is out of range. Lets the regression test drive the exact byte shape that previously fed an
    /// out-of-bounds index downstream.
    fn v1_tzif_one_transition(trans_type_index: u8) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(&MAGIC); // "TZif"
        b.push(0); // version 0 → v1-only path
        b.extend_from_slice(&[0u8; 15]); // reserved
                                         // counts: isutcnt, isstdcnt, leapcnt, timecnt, typecnt, charcnt
        for v in [0u32, 0, 0, 1, 1, 4] {
            b.extend_from_slice(&v.to_be_bytes());
        }
        b.extend_from_slice(&0i32.to_be_bytes()); // 1 transition time (4 bytes, v1)
        b.push(trans_type_index); // 1 transition type index
        b.extend_from_slice(&0i32.to_be_bytes()); // ttinfo[0].utoff
        b.push(0); // ttinfo[0].is_dst
        b.push(0); // ttinfo[0].desigidx
        b.extend_from_slice(b"UTC\0"); // designation table (charcnt = 4)
        b
    }

    #[test]
    fn valid_transition_type_index_parses() {
        // Index 0 is in range (typecnt == 1) → parses cleanly.
        let parsed = parse(&v1_tzif_one_transition(0)).unwrap();
        assert_eq!(parsed.transitions.len(), 1);
        assert_eq!(parsed.transitions[0].type_index, 0);
        assert_eq!(parsed.types.len(), 1);
    }

    /// Hand-build a v1-only TZif header with arbitrary counts and a chosen body length (T17.5). Lets a
    /// test declare an implausibly-large count with a tiny body.
    fn v1_header_with_counts(
        isutcnt: u32,
        isstdcnt: u32,
        leapcnt: u32,
        timecnt: u32,
        typecnt: u32,
        charcnt: u32,
        body_len: usize,
    ) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(&MAGIC);
        b.push(0); // version 0 → v1-only path
        b.extend_from_slice(&[0u8; 15]);
        for v in [isutcnt, isstdcnt, leapcnt, timecnt, typecnt, charcnt] {
            b.extend_from_slice(&v.to_be_bytes());
        }
        b.extend(std::iter::repeat(0u8).take(body_len));
        b
    }

    #[test]
    fn implausibly_large_declared_count_is_rejected_not_ooming() {
        // T17.5: a header claiming a billion transitions with only a few body bytes must be a typed
        // `Err` *before* any `Vec::with_capacity(timecnt)` — never a multi-gigabyte allocation/abort.
        let bytes = v1_header_with_counts(0, 0, 0, 1_000_000_000, 1, 4, 8);
        let err = parse(&bytes).unwrap_err();
        assert!(
            err.to_string()
                .contains("more bytes than the input contains")
                || err.to_string().contains("truncated"),
            "expected a counts-exceed-input rejection, got: {err}"
        );
    }

    #[test]
    fn count_block_len_is_checked_arithmetic() {
        // The block-length arithmetic is checked: a maximal `timecnt` does not wrap; on 64-bit it is a
        // large-but-finite value that simply exceeds any real input (→ rejected), never a wrap that
        // under-computes the length and mis-slices.
        let needed = checked_block_len(
            &Counts {
                isutcnt: 0,
                isstdcnt: 0,
                leapcnt: 0,
                timecnt: u32::MAX,
                typecnt: 1,
                charcnt: 4,
            },
            8,
        )
        .unwrap();
        // u32::MAX transitions × (8 bytes time + 1 byte index) is ~38.6e9 — far past any real file.
        assert!(needed > 30_000_000_000);
    }

    #[test]
    fn out_of_range_transition_type_index_is_rejected_not_panic() {
        // T17.1 regression: a transition referencing type index 1 when only type 0 exists must be a
        // typed Err, never a panic. This is the exact shape that would otherwise reach
        // `types[type_index]` in `compare::semantic::diff` / `compile::leap`.
        let err = parse(&v1_tzif_one_transition(1)).unwrap_err();
        assert!(
            err.to_string().contains("type index 1 out of range"),
            "expected an out-of-range rejection, got: {err}"
        );
        // A wildly out-of-range index (max u8) is equally rejected, not indexed.
        assert!(parse(&v1_tzif_one_transition(255)).is_err());
    }

    #[test]
    fn rfc_designation_index_validity_rules() {
        assert!(rfc_designation_index_valid(0, 4, true));
        assert!(!rfc_designation_index_valid(4, 4, true)); // idx == charcnt: slice-safe but NOT RFC-valid
        assert!(!rfc_designation_index_valid(0, 0, true)); // empty designation table
        assert!(!rfc_designation_index_valid(0, 4, false)); // no NUL terminator at/after idx
        assert!(abbr_index_slice_safe(3, 4) && rfc_designation_index_valid(3, 4, true));
        // RFC-valid ⇒ slice-safe
    }

    #[test]
    fn isdst_byte_validity_rules() {
        assert!(isdst_byte_valid(0) && isdst_byte_valid(1));
        assert!(!isdst_byte_valid(2) && !isdst_byte_valid(255));
    }

    #[test]
    fn indicator_pair_validity_rules() {
        assert!(
            indicator_pair_valid(0, 0) && indicator_pair_valid(0, 1) && indicator_pair_valid(1, 1)
        );
        assert!(!indicator_pair_valid(1, 0)); // UT indicator without its standard indicator
        assert!(!indicator_pair_valid(2, 0) && !indicator_pair_valid(0, 2)); // byte not in {0,1}
    }

    #[test]
    fn utoff_structural_validity_rules() {
        assert!(
            utoff_structural_valid(0)
                && utoff_structural_valid(-18000)
                && utoff_structural_valid(i32::MAX)
        );
        assert!(!utoff_structural_valid(i32::MIN));
    }
}

/// T23.kani.3 — bounded proof that the TZif parser never panics (`audits/kani`).
///
/// Compiled **only** under `--cfg kani` (set by the Kani compiler), so it has zero effect on the normal
/// `cargo build`/`test`/`clippy` gate — it is `cfg`-gated out everywhere else; `cargo fmt` still formats it.
/// Over an arbitrary byte array up to a declared bound, [`parse`] must return `Ok`/`Err` **without panicking
/// or exhibiting UB**. This is the *proving* complement to `audits/panic-analysis` (static census) and
/// `audits/miri` (concrete-test execution): a bounded symbolic proof over the invariant itself.
///
/// **Bounded proof — what it does NOT establish:** semantic correctness, full RFC-9636 validity,
/// reference-`zic` parity, inputs longer than the bound, resource-exhaustion freedom, or filesystem safety.
#[cfg(kani)]
mod kani_harness {
    use super::{
        abbr_index_slice_safe, checked_block_len, first_oob_type_index, indicator_pair_valid,
        isdst_byte_valid, rfc_designation_index_valid, utoff_structural_valid, Counts, Cursor,
    };

    /// **T23.kani.3b — the abbreviation-index guard is SLICE-SAFE (memory safety, NOT RFC validity).** For
    /// an arbitrary table and `idx`, when `abbr_index_slice_safe` accepts the index, `&table[idx..]` is an
    /// in-bounds (possibly empty) slice — so `read_cstr` never panics on a hostile `ttinfo` designation
    /// index; when it rejects, `idx > table.len()`. Alloc-free, loop-free → converges. **This proves Rust
    /// slice safety only — NOT RFC 9636 §3.2 designation-index validity** (`idx < charcnt` + NUL at/after +
    /// `charcnt != 0`), which is a separate, tracked gap (`audits/claim-boundary-map.md`).
    #[kani::proof]
    fn abbr_index_guard_prevents_oob_slice() {
        let table: [u8; 8] = kani::any();
        let idx: usize = kani::any();
        if abbr_index_slice_safe(idx, table.len()) {
            let _slice = &table[idx..]; // CBMC proves this slice index is in-bounds under the guard
            assert!(idx <= table.len());
        } else {
            assert!(idx > table.len());
        }
    }

    /// **T23.kani.3a — the transition type-index guard is sound (RFC 9636 §3.2 / the T17.1a OOB-index
    /// guard, `RISK.COUNT.1`).** For an arbitrary index slice and any `typecnt`, `first_oob_type_index`
    /// returns `None` **iff** every index is `< typecnt` — i.e. a `None` is exactly the precondition that
    /// makes the downstream `types[type_index]` indexing in-bounds — and on `Some(p)` the reported index is
    /// genuinely the first out-of-range one. Alloc-free + format-free → converges. This is the formal form
    /// of what `audits/miri` only checks on concrete fixtures and what `parse` enforces at the choke point.
    #[kani::proof]
    #[kani::unwind(5)]
    fn type_index_guard_is_sound() {
        let idxs: [u8; 4] = kani::any();
        let typecnt: usize = kani::any();
        match first_oob_type_index(&idxs, typecnt) {
            None => {
                // soundness: no out-of-range index ⇒ every index is a valid offset into a
                // `typecnt`-length type table, so the downstream `types[idx]` can never be OOB.
                for &i in idxs.iter() {
                    assert!((i as usize) < typecnt);
                }
            }
            Some(p) => {
                assert!(p < idxs.len()); // a real position
                assert!((idxs[p] as usize) >= typecnt); // and it is genuinely out of range
            }
        }
    }

    /// **T23.kani.2 — `Cursor::take` never panics and preserves the bounds invariant.** For an arbitrary
    /// buffer, start `pos ≤ len`, and requested `n`: `take` returns `Ok` only with `pos' = pos + n ≤ len`
    /// (no wrap — `checked_add`) and never indexes out of bounds; on `Err` it leaves `pos` unchanged.
    /// Alloc-free, loop-free → CBMC converges. (Risk: section-offset accumulation / truncation, T17.5.)
    #[kani::proof]
    fn take_never_panics_and_preserves_bounds() {
        let buf: [u8; 8] = kani::any();
        let pos: usize = kani::any();
        kani::assume(pos <= buf.len());
        let mut c = Cursor { buf: &buf, pos };
        let n: usize = kani::any();
        match c.take(n) {
            Ok(s) => {
                assert!(s.len() == n);
                assert!(c.pos == pos + n);
                assert!(c.pos <= buf.len());
            }
            Err(_) => assert!(c.pos == pos),
        }
    }

    /// **T23.kani.2 — `Cursor::skip` never panics and never advances past the end** (the count-driven v1-block
    /// skip, T17.5): `Ok` ⇒ `pos' = pos + n ≤ len` (no wrap); `Err` ⇒ `pos` unchanged.
    #[kani::proof]
    fn skip_never_panics_and_preserves_bounds() {
        let buf: [u8; 8] = kani::any();
        let pos: usize = kani::any();
        kani::assume(pos <= buf.len());
        let mut c = Cursor { buf: &buf, pos };
        let n: usize = kani::any();
        match c.skip(n) {
            Ok(()) => assert!(c.pos == pos + n && c.pos <= buf.len()),
            Err(_) => assert!(c.pos == pos),
        }
    }

    /// **T23.kani.2 — the pre-allocation bound is sound.** `remaining()` is exactly the bytes left, and any
    /// block whose length is `≤ remaining()` can always be skipped without truncation. This is the link
    /// the T17.5 guard relies on: *`count × size ≤ remaining()` ⇒ those bytes are physically present*, so a
    /// passing bound check never lets a later read run off the end.
    #[kani::proof]
    fn skip_within_remaining_cannot_truncate() {
        let buf: [u8; 8] = kani::any();
        let pos: usize = kani::any();
        kani::assume(pos <= buf.len());
        let mut c = Cursor { buf: &buf, pos };
        let r = c.remaining();
        assert!(pos + r == buf.len()); // `remaining` is exactly what is left (no panic, no wrap)
        let n: usize = kani::any();
        kani::assume(n <= r);
        assert!(c.skip(n).is_ok()); // anything within `remaining` is skippable
        assert!(c.pos <= buf.len());
    }

    /// **T23.kani.1 (converges) — the T17.5 count-arithmetic guard never panics/overflow-wraps.** For *all*
    /// symbolic header counts and `time_size ∈ {4, 8}`, [`checked_block_len`] returns `Ok(total)` or a typed
    /// overflow `Err` — never a panic, never a wraparound (the `overflow-checks` hazard), never a giant
    /// allocation (it computes a length, it does not allocate). This is the *tightest* form of the danger
    /// surface `reports/t17-count-arithmetic-verdict.md` describes, and it is alloc-free + format-light so
    /// CBMC converges. **Does not prove:** the full parser, semantic correctness, or anything about reads.
    #[kani::proof]
    fn checked_block_len_never_panics() {
        let counts = Counts {
            isutcnt: kani::any(),
            isstdcnt: kani::any(),
            leapcnt: kani::any(),
            timecnt: kani::any(),
            typecnt: kani::any(),
            charcnt: kani::any(),
        };
        let time_size: usize = if kani::any() { 4 } else { 8 };
        let _ = checked_block_len(&counts, time_size);
    }

    /// **T23.kani.3f.1 — RFC designation-index validity is exact and strictly implies slice-safety.** For an
    /// arbitrary `(idx, charcnt, has_nul)`, `rfc_designation_index_valid` accepts iff `charcnt != 0 && idx <
    /// charcnt && has_nul`; and whenever it accepts, the index is also `abbr_index_slice_safe` — so RFC
    /// validity is strictly stronger than the T23.kani.3b slice-safety (no RFC-valid index is ever OOB).
    #[kani::proof]
    fn rfc_designation_index_valid_is_exact_and_implies_slice_safe() {
        let idx: usize = kani::any();
        let charcnt: usize = kani::any();
        let has_nul: bool = kani::any();
        let got = rfc_designation_index_valid(idx, charcnt, has_nul);
        assert_eq!(got, charcnt != 0 && idx < charcnt && has_nul);
        if got {
            assert!(abbr_index_slice_safe(idx, charcnt)); // RFC-valid ⇒ slice-safe (idx < charcnt ⇒ idx ≤ charcnt)
        }
    }

    /// **T23.kani.3f.2 — `isdst` byte structural validity (RFC: octet ∈ {0,1}).** Total over all `u8`; an
    /// accepted byte is exactly a Rust `bool` (0 or 1), the only values for which the lenient `!= 0` reading
    /// loses no information.
    #[kani::proof]
    fn isdst_byte_valid_is_exact() {
        let b: u8 = kani::any();
        let got = isdst_byte_valid(b);
        assert_eq!(got, b <= 1);
        if got {
            assert!(b == 0 || b == 1);
        }
    }

    /// **T23.kani.3f.3 — indicator-byte pairing (RFC: each ∈ {0,1} and `isut == 1 ⇒ isstd == 1`).** Total over
    /// all `(u8, u8)`; an accepted pair never carries a UT indicator without its standard indicator.
    #[kani::proof]
    fn indicator_pair_valid_is_exact() {
        let isut: u8 = kani::any();
        let isstd: u8 = kani::any();
        let got = indicator_pair_valid(isut, isstd);
        assert_eq!(got, isut <= 1 && isstd <= 1 && !(isut == 1 && isstd == 0));
        if got && isut == 1 {
            assert!(isstd == 1);
        }
    }

    /// **T23.kani.3f.4 — `utoff` structural validity (RFC forbids `i32::MIN`).** Total over all `i32`; an
    /// accepted offset can always be negated without overflow — the concrete safety reason the RFC excludes
    /// `-2^31` (its signed negation is the one value that wraps).
    #[kani::proof]
    fn utoff_structural_valid_excludes_unnegatable_min() {
        let u: i32 = kani::any();
        let got = utoff_structural_valid(u);
        assert_eq!(got, u != i32::MIN);
        if got {
            assert!(u.checked_neg().is_some()); // valid ⇒ negation cannot overflow
        }
    }
}
