//! The **`zdump` behaviour oracle**: compare two compiled zone files by what they actually
//! *do*, as reported by the reference `zdump` tool, over a declared year horizon.
//!
//! Why this rather than decoding the TZif structurally: reference `zic` is free to choose
//! different explicit-transition horizons, type orderings, and how much it leans on the
//! POSIX footer for the tail. Those are valid representational choices that a structural
//! diff would flag as "mismatches". The contract users actually care about is local-time
//! *behaviour*: for every instant in the declared window, do both files agree on the UTC
//! instant, local time, UT offset, DST flag, and abbreviation? `zdump -v -c LO,HI`
//! enumerates exactly that (it evaluates transitions **and** the POSIX footer), so diffing
//! its normalised output is the faithful semantic comparison.
//!
//! ## Path handling (a real foot-gun)
//!
//! `zdump` treats a bare name as a timezone-database lookup (`TZDIR`), not a file. To dump
//! an arbitrary compiled file you must pass an **absolute path**. We always do.

use std::path::Path;
use std::process::Command;

use crate::error::{Error, Result};

use super::semantic::Difference;

/// Whether the `zdump` oracle program is runnable (mirrors `reference_zic::is_available`). Used by
/// the T15.3 semantic-witness builder to render `OracleMode::Unavailable` (with a reason) rather than
/// silently weakening when `zdump` is absent.
pub fn is_available(program: &str) -> bool {
    std::process::Command::new(program)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Run `zdump -v -c LO,HI <abs path>` and return its output as **normalised** lines.
///
/// Normalisation strips the leading path token from each line (every line begins with the
/// file path), so two dumps of the same behaviour at different paths compare equal.
pub fn run(program: &str, path: &Path, lo: i32, hi: i32) -> Result<Vec<String>> {
    // `path` must be absolute (see module docs). The caller builds it under a tempdir, which
    // is absolute; assert defensively so a future caller can't silently regress to a lookup.
    debug_assert!(path.is_absolute(), "zdump needs an absolute path");
    let path_str = path.to_str().ok_or_else(|| {
        Error::message("compiled zone path is not valid UTF-8 (cannot pass to zdump)")
    })?;

    let output = Command::new(program)
        .arg("-v")
        .arg("-c")
        .arg(format!("{lo},{hi}"))
        .arg(path_str)
        .output()
        .map_err(|e| Error::message(format!("failed to run {program:?}: {e}")))?;
    if !output.status.success() {
        return Err(Error::message(format!(
            "{program} failed for {path_str}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let normalised = stdout
        .lines()
        .map(|line| normalise(line, path_str))
        .collect();
    Ok(normalised)
}

/// Strip the leading path token from one `zdump -v` line so behaviour can be compared
/// independent of where the file lives.
fn normalise(line: &str, path: &str) -> String {
    line.strip_prefix(path)
        .unwrap_or(line)
        .trim_start()
        .to_string()
}

/// Diff two normalised `zdump` line lists into human-readable [`Difference`]s.
///
/// Compared line-by-line in order; `zdump -v` output is deterministic for a given file and
/// horizon, so a positional diff is exact. Any divergence — count or content — is reported.
pub fn diff(ours: &[String], theirs: &[String]) -> Vec<Difference> {
    let mut diffs = Vec::new();
    if ours.len() != theirs.len() {
        diffs.push(Difference {
            what: "zdump line count".into(),
            ours: ours.len().to_string(),
            theirs: theirs.len().to_string(),
        });
    }
    for (i, (a, b)) in ours.iter().zip(theirs).enumerate() {
        if a != b {
            diffs.push(Difference {
                what: format!("zdump line {i}"),
                ours: a.clone(),
                theirs: b.clone(),
            });
        }
    }
    diffs
}
