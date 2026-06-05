#![no_main]
//! **RESERVED — surface_absent (honest scaffold).** The compile manifest (`zic-rs-compile-manifest-v8`)
//! is **write-only**: zic-rs *emits* it, it never *ingests* manifest JSON, so there is no
//! manifest-JSON parser to fuzz today. This target is kept (the audit list is authoritative) but is a
//! deliberate no-op until such a reader exists; its receipt status is **`surface_absent`**, not merely
//! `pending_capture`. The crate's only untrusted JSON-ingestion surface is the vendor-oracle receipt
//! reader — fuzzed by `vendor_oracle_json` (a *fuzz target existing is not a fuzz result*). If a
//! manifest reader is ever added, wire it here and flip the receipt to `pending_capture`.
use libfuzzer_sys::fuzz_target;

fuzz_target!(|_data: &[u8]| {
    // No manifest-JSON ingestion path exists (manifests are emitted, never parsed). Intentionally empty.
});
