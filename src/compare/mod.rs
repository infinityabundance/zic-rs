//! The compatibility oracle: compile a zone both ways and compare.
//!
//! `compare` is what turns this project from "a reimplementation" into a *verified* one.
//! For a chosen zone it compiles with **our** compiler and with the **reference** `zic`,
//! then compares the results in one of two modes:
//!
//! * [`CompareMode::Zdump`] (**default**) — the faithful semantic oracle: dump both files
//!   with `zdump -v -c LO,HI` over a declared year horizon and diff the behaviour (UTC
//!   instant, local time, UT offset, DST flag, abbreviation). This is the contract that
//!   matters, and the only mode appropriate for recurring zones (whose explicit-transition
//!   horizons differ between implementations while behaviour is identical). See [`zdump`].
//! * [`CompareMode::Structural`] — decode both TZif files and diff the decoded model
//!   (footer + transition list + types). Useful for fixed-offset/finite fixtures and for
//!   debugging, but **not** the correctness criterion for recurring zones, because it
//!   penalises valid representational differences. See [`semantic`].
//!
//! This module is the only one that runs an external `zic`/`zdump`, and only ever here.

pub mod reference_zic;
pub mod report;
pub mod semantic;
pub mod zdump;

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::fs::output_tree;
use crate::model::Database;
use crate::tzif;
use report::{ComparisonKind, ZoneComparison};

/// How a comparison is performed.
#[derive(Debug, Clone)]
pub enum CompareMode {
    /// Decode both TZif files and diff the decoded model. Default horizon-free; suited to
    /// fixed-offset/finite fixtures and debugging.
    Structural,
    /// Diff `zdump -v -c LO,HI` behaviour over `[lo, hi]` (inclusive years). The faithful
    /// semantic oracle. `program` is the `zdump` executable name/path.
    Zdump { program: String, lo: i32, hi: i32 },
}

/// Compare our compilation of `zone` against reference `zic`.
///
/// `inputs` are the source files fed to both compilers; `reference_zic` is the reference
/// compiler program; `work_dir` is a caller-controlled (absolute) directory for scratch
/// output (typically a tempdir).
pub fn compare_zone(
    db: &Database,
    inputs: &[PathBuf],
    zone: &str,
    reference_zic: &str,
    work_dir: &Path,
    mode: &CompareMode,
) -> Result<ZoneComparison> {
    // Our output, in memory.
    let ours_bytes = crate::compile_zone_to_bytes(db, zone)?;

    // Reference output, via the C `zic`, into work_dir/ref.
    let ref_root = work_dir.join("ref");
    std::fs::create_dir_all(&ref_root).map_err(|e| Error::io(&ref_root, e))?;
    reference_zic::compile_with_reference(reference_zic, inputs, &ref_root)?;
    let ref_path = reference_zic::compiled_path(&ref_root, zone);
    let theirs_bytes = std::fs::read(&ref_path).map_err(|e| Error::io(&ref_path, e))?;
    let byte_identical = ours_bytes == theirs_bytes;

    match mode {
        CompareMode::Structural => {
            // Decode both and diff the model.
            let ours = tzif::parse(&ours_bytes)?;
            let theirs = tzif::parse(&theirs_bytes)?;
            Ok(ZoneComparison {
                zone: zone.to_string(),
                kind: ComparisonKind::DecodedTzif,
                differences: semantic::diff(&ours, &theirs),
                byte_identical,
            })
        }
        CompareMode::Zdump { program, lo, hi } => {
            // Write our bytes to a file so `zdump` can read it (absolute path required).
            let ours_root = work_dir.join("ours");
            let ours_path =
                output_tree::write_zone_file(&ours_root, zone, &ours_bytes, true, false)?;

            let ours_lines = zdump::run(program, &ours_path, *lo, *hi)?;
            let theirs_lines = zdump::run(program, &ref_path, *lo, *hi)?;
            Ok(ZoneComparison {
                zone: zone.to_string(),
                kind: ComparisonKind::ZdumpBehaviour { lo: *lo, hi: *hi },
                differences: zdump::diff(&ours_lines, &theirs_lines),
                byte_identical,
            })
        }
    }
}
