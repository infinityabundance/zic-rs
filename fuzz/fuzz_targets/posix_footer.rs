#![no_main]
//! Fuzz the **POSIX-TZ footer** decode path. zic-rs has no standalone *public* footer parser — the
//! footer is decoded inside `tzif::validate::parse` (the trailing `\n<TZ>\n` region). This target
//! therefore shares the `parse` entry with `tzif_validate_bytes` but is **corpus-directed at the
//! footer**: its seed corpus is TZif files with varied/mutated footer tails, so libFuzzer evolves
//! footer-shaped inputs specifically. Contract: never panic; `Ok`/`Err` only. (A dedicated public
//! footer parser would let this target call it directly — a tracked future extraction.)
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = tzcompile::tzif::validate::parse(data);
});
