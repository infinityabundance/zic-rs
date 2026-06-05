# Fuzz-run receipt — TEMPLATE

> Copy this per real run and append the filled form to [`RUNS.md`](RUNS.md). **A run is admitted only by
> a completed receipt.** Do not mark a target `completed` without one. `pending_capture` = the harness
> exists but no run was executed (no claim). `surface_absent` = there is no fuzzable surface for this
> target (e.g. `manifest_json` — manifests are write-only).

```text
target:                 # e.g. tzif_validate_bytes
status:                 # pending_capture | completed | surface_absent
tool:                   # cargo-fuzz / libFuzzer
tool_version:           # cargo-fuzz X.Y.Z, libFuzzer (rustc bundled)
rust_toolchain:         # e.g. nightly-YYYY-MM-DD
host_identity:          # os / arch / cpu
start_time:             # ISO-8601 (UTC) — NEVER fabricated; recorded by the runner
duration:               # e.g. 24h / 86400s
seed_corpus_hash:       # sha256 over the sorted seed-file hashes (the input set's identity)
generated_corpus_hash:  # sha256 of the post-run corpus (coverage-expanded), if retained
max_len / limits:       # -max_len, -rss_limit_mb, etc.
sanitizers:             # address / undefined / none (libFuzzer default is ASan)
exit_status:            # 0 (clean) / non-zero (crash/leak/timeout)
crashes:                # count + minimized artifact paths
ooms:                   # count
timeouts:               # count
minimized_artifacts:    # paths under crash-*/ or artifacts/
fixed_regressions:      # commit hashes of fixes + the regression test added for each
residual_non_claims:    # what this run does NOT establish (coverage gaps, surfaces not run, etc.)
next_owner:             # who owns the next run / follow-up
```

**Discipline:** timestamps/hashes are recorded by the runner, never hand-written; a crash becomes a
**minimized artifact + a regression test in the main crate** (the fix lands with its test, the receipt
records both); a clean run records its `residual_non_claims` (a clean 24h run is evidence, not a proof of
absence of bugs).
