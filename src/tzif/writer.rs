//! Orchestration of a complete TZif file: v1 stub block + v2+ block + footer.
//!
//! See the module docs in `tzif/mod.rs` for the three-part file shape and why the v1 block
//! is a stub in slim output. This file is intentionally tiny: all the per-block work lives
//! in [`data_block`](super::data_block); here we just stitch the three parts together in
//! the right order with the right time sizes.

use super::data_block::write_block;
use super::{LeapRecord, LocalTimeType, Transition, TzifData};
use crate::error::Result;

/// Serialise `data` to a complete TZif byte stream.
///
/// Fallible: serialising a zone whose abbreviation table would exceed the format's 255-byte
/// limit returns an error rather than panicking (see `data_block::build_designations`).
pub fn write_bytes(data: &TzifData) -> Result<Vec<u8>> {
    let mut out = Vec::new();

    // --- Part 1: the version-1 block (32-bit times). ---
    //
    // Slim `zic` writes a placeholder here: no transitions and a single local-time-type
    // with offset 0 and an empty abbreviation. We match that byte-for-byte. The real data
    // is entirely in the v2+ block below.
    let v1_stub = [LocalTimeType {
        utoff: 0,
        is_dst: false,
        abbr: String::new(),
    }];
    let no_transitions: [Transition; 0] = [];
    let no_leaps: [LeapRecord; 0] = [];
    write_block(
        &mut out,
        data.version,
        4,
        &v1_stub,
        &no_transitions,
        &no_leaps,
    )?;

    // --- Part 2: the version-2+ block (64-bit times) with the real data. The leap-second table
    // (if any) lives only in this authoritative block; the v1 stub above stays leap-free. ---
    write_block(
        &mut out,
        data.version,
        8,
        &data.types,
        &data.transitions,
        &data.leaps,
    )?;

    // --- Part 3: the footer: newline, POSIX TZ string, newline. ---
    out.push(b'\n');
    out.extend_from_slice(data.footer.as_bytes());
    out.push(b'\n');

    Ok(out)
}
