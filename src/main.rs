//! `zic-rs` binary entry point.
//!
//! Parses arguments and delegates to the `tzcompile` library. All real work lives in the
//! library; this file only handles argument parsing and mapping errors to an exit code so
//! the tool composes well in scripts.
//!
//! **Exit-status contract (campaign T9.2 — kept deliberately small; never surprise a script):**
//!
//! * `0` — success.
//! * `1` — an **operational / compiler / config** failure (parse error, unsupported construct,
//!   `--out` path-traversal `ZIC008`, no-clobber without `--force`, missing/unreadable input file,
//!   reference `zic` absent for `compare`/`structural-report`, missing `--out`). The diagnostic is
//!   printed to stderr.
//! * `2` — a **CLI usage** error (`clap`: unknown option, missing a structurally-required arg such
//!   as `--input`). Emitted by `clap` before the library runs.
//!
//! `--help` / `--version` print and exit `0` (clap). Diagnostics may evolve in wording, but this
//! success/failure *classification* is a stable contract (see `tests/cli_operational_parity.rs`).

#![forbid(unsafe_code)]

use clap::Parser;
use tzcompile::cli::{run, Cli};

fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("zic-rs: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}
