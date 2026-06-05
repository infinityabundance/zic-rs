#![no_main]
//! Fuzz the TZif **byte reader/validator** — the most dangerous hostile-input surface (it decodes
//! arbitrary `--input` via `tzif-validate`, plus reference-`zic` output). The contract (T17.1a/T17.5,
//! `docs/panic-policy.md`): malformed/hostile bytes must become a typed `Err`/verdict, **never a panic,
//! OOB index, wrap, or OOM**. This target asserts exactly that: any byte string is handed to both the
//! bounds-guarded `parse` and the RFC-9636 `validate`, and the only acceptable outcomes are `Ok`/`Err`
//! and a structural verdict — a crash is a finding.
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // `parse` must return Ok/Err, never panic (T17.1a type-index guard, T17.5 count arithmetic).
    let _ = tzcompile::tzif::validate::parse(data);
    // The RFC-9636 validator reuses `parse` and adds bounds-safe invariant checks; it returns a verdict
    // for every input (never panics) — exercise it too.
    let _ = tzcompile::tzif::rfc9636::validate(data);
});
