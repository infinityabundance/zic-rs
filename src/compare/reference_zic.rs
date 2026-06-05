//! Driving the **reference C `zic`** to produce oracle output.
//!
//! This is the *only* place in the crate that executes an external `zic`, and it is never
//! on the compile path — it exists purely so `compare` can hold our output up against the
//! canonical implementation. Everything here is plain `std::process`.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{Error, Result};

/// Is a usable reference `zic` available at `program`? (Tests and CI gate on this so the
/// default test run never *requires* a system `zic`.)
pub fn is_available(program: &str) -> bool {
    Command::new(program)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Run `zic -d <out_dir> <inputs...>`, compiling the given source files.
///
/// `zic` writes a `/usr/share/zoneinfo`-style tree under `out_dir`. We pass `-d` explicitly
/// and never let it touch a system directory.
pub fn compile_with_reference(program: &str, inputs: &[PathBuf], out_dir: &Path) -> Result<()> {
    let mut cmd = Command::new(program);
    cmd.arg("-d").arg(out_dir);
    for i in inputs {
        cmd.arg(i);
    }
    let output = cmd
        .output()
        .map_err(|e| Error::message(format!("failed to run reference zic {program:?}: {e}")))?;
    if !output.status.success() {
        return Err(Error::message(format!(
            "reference zic failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(())
}

/// The path at which reference `zic` will have written a zone, given the output root.
pub fn compiled_path(out_dir: &Path, zone: &str) -> PathBuf {
    out_dir.join(zone)
}
