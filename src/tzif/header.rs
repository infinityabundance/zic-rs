//! The 44-octet TZif header (RFC 9636 §3.1) and big-endian write helpers.
//!
//! Header layout, all integers big-endian ("network byte order"):
//!
//! | bytes | field      | meaning                                   |
//! |-------|------------|-------------------------------------------|
//! | 0..4  | magic      | ASCII `TZif`                              |
//! | 4     | version    | `\0`, `'2'`, `'3'`, or `'4'`              |
//! | 5..20 | reserved   | 15 NUL bytes                              |
//! | 20..24| isutcnt    | count of UT/local indicators              |
//! | 24..28| isstdcnt   | count of standard/wall indicators         |
//! | 28..32| leapcnt    | count of leap-second records              |
//! | 32..36| timecnt    | count of transition times                 |
//! | 36..40| typecnt    | count of local-time-type records (>= 1)   |
//! | 40..44| charcnt    | total bytes of abbreviation string table  |

/// The four magic bytes that begin every TZif file.
pub const MAGIC: [u8; 4] = *b"TZif";

/// The six count fields of a TZif header, in their on-disk order.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    pub isutcnt: u32,
    pub isstdcnt: u32,
    pub leapcnt: u32,
    pub timecnt: u32,
    pub typecnt: u32,
    pub charcnt: u32,
}

/// Append a 44-byte header for `version` with the given `counts` to `out`.
pub fn write_header(out: &mut Vec<u8>, version: u8, counts: &Counts) {
    out.extend_from_slice(&MAGIC);
    out.push(version);
    out.extend_from_slice(&[0u8; 15]); // reserved
    out.extend_from_slice(&counts.isutcnt.to_be_bytes());
    out.extend_from_slice(&counts.isstdcnt.to_be_bytes());
    out.extend_from_slice(&counts.leapcnt.to_be_bytes());
    out.extend_from_slice(&counts.timecnt.to_be_bytes());
    out.extend_from_slice(&counts.typecnt.to_be_bytes());
    out.extend_from_slice(&counts.charcnt.to_be_bytes());
}
