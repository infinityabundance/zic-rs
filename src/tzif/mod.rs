//! TZif output: the in-memory [`TzifData`] model and the writer that serialises it to the
//! on-disk format described by [RFC 9636] / `tzfile(5)`.
//!
//! # File shape (RFC 9636 §3)
//!
//! A version-2+ TZif file is three parts back to back:
//!
//! 1. a **version-1 block** — a 44-byte header followed by a data block that uses 32-bit
//!    transition times;
//! 2. a **version-2+ block** — an identical-shape header (same version byte) followed by a
//!    data block that uses 64-bit transition times;
//! 3. a **footer** — `\n`, a POSIX `TZ` string describing behaviour after the last
//!    transition, then `\n`.
//!
//! Modern `zic` (the "slim" default) emits the v1 block as a near-empty **stub**: zero
//! transitions and a single placeholder local-time-type with offset 0 and an empty
//! abbreviation. All real information lives in the v2+ block and the footer. We reproduce
//! that exactly (it is what we must byte-match), and we document it loudly because it is
//! surprising: a strictly v1-only reader gets UT for every zone. That is `zic`'s own
//! behaviour in slim mode, not ours.
//!
//! [RFC 9636]: https://www.rfc-editor.org/rfc/rfc9636

pub mod data_block;
pub mod header;
pub mod rfc9636;
pub mod validate;
pub mod writer;

pub use header::Counts;
pub use validate::{parse, ParsedTzif};
pub use writer::write_bytes;

/// One local-time-type (`ttinfo`): a UT offset, a DST flag, and an abbreviation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalTimeType {
    /// Seconds east of UT (the `tt_utoff` field).
    pub utoff: i32,
    /// Whether this type is daylight saving time.
    pub is_dst: bool,
    /// The timezone abbreviation (e.g. `EST`). May be empty (the v1 stub uses `""`).
    pub abbr: String,
}

/// One transition: the UT instant at which `type_index` begins to apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Transition {
    /// Seconds since the Unix epoch, UT.
    pub at: i64,
    /// Index into [`TzifData::types`].
    pub type_index: u8,
}

/// The fully-resolved, ready-to-serialise contents of one zone's TZif file.
///
/// Invariants the writer relies on (and [`validate`] re-checks):
/// * `types` is non-empty;
/// * every `transitions[i].type_index` is a valid index into `types`;
/// * `transitions` is strictly increasing in `at`.
#[derive(Debug, Clone)]
pub struct TzifData {
    pub types: Vec<LocalTimeType>,
    pub transitions: Vec<Transition>,
    /// POSIX `TZ` string for the footer, **without** the surrounding newlines. Empty is
    /// permitted (means "no proleptic rule"), though we always synthesise one in T1.
    pub footer: String,
    /// The TZif version byte to stamp (`b'2'`, `b'3'`, ...). Content-driven; see the
    /// module docs and `docs/tzif-notes.md`. T1 always emits `b'2'`.
    pub version: u8,
    /// Leap-second correction records for the **authoritative (v2+) block** (T11.3). Each entry is
    /// **already adjusted** (cumulative `corr`; `trans` includes prior corrections — `zic`'s
    /// `adjleap`). **Empty for every ordinary zone** (leaps come only from an explicit leap-source via
    /// [`crate::compile::apply_leaps`]), so ordinary output is byte-unchanged. Orthogonal to
    /// `transitions`/`types`: a leap is **not** a local-time-type transition.
    pub leaps: Vec<LeapRecord>,
}

/// One emitted TZif leap-second record (RFC 9636 §3.2 array 5): the (already-adjusted) occurrence
/// instant and the cumulative correction at/after it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeapRecord {
    /// Occurrence, seconds since epoch, in the file's timescale (cumulative-adjusted by `adjleap`).
    pub trans: i64,
    /// Cumulative correction (e.g. `+1, +2, +3, …`).
    pub corr: i32,
}

impl TzifData {
    /// Convenience constructor for a single fixed-offset type with no transitions.
    pub fn fixed(utoff: i32, abbr: impl Into<String>, footer: impl Into<String>) -> Self {
        TzifData {
            types: vec![LocalTimeType {
                utoff,
                is_dst: false,
                abbr: abbr.into(),
            }],
            transitions: Vec::new(),
            footer: footer.into(),
            version: b'2',
            leaps: Vec::new(),
        }
    }
}
