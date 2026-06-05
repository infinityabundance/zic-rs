# audits/cargo-fuzz — coverage-guided fuzzing (libFuzzer)

> **Status: F1–F3 FIXED + re-verified (T23.cargo-fuzz.2, 2026-06-04).** T23.cargo-fuzz.1's bounded smoke
> found 3 panic-on-hostile-input sites (F1–F3); **T23.cargo-fuzz.2 fixed 3/3**, each with its preserved
> minimized seed as a regression test, and the **bounded smoke re-ran 9/9 CLEAN · 0 crashes** + all 4 crash
> seeds replay `rc=0`. The panic-policy claim is **restored only for the known F1–F3 seeds + this bounded
> rerun — NOT an exhaustive no-panic proof.** An audit folder existing is not an audit result — a result is
> admitted only by a receipt in `receipts/` (tool · version · host · command · input+hash · duration · exit ·
> findings · fixed · residual · non-claims · next-owner). See `../README.md` · `receipts/RECEIPT-2026-06-04-fuzz2.md`.
>
> **Long burn-in (2026-06-05): LONG-FUZZ-HARNESS.1 sealed the reproducible operator harness**
> (`../../fuzz/run-long-burnin.sh` — modes/provenance/receipt-vocabulary; *no saturation claim*), and
> **LONG-FUZZ-SMOKE.1 ran 9 targets × 5 min → overall `clean`, 0 crashes, exit 0**
> (`receipts/RECEIPT-LONG-FUZZ-20260605T090115Z.md`; rustc 1.98-nightly, cargo-fuzz 0.13.1). A **bounded
> stability run, not saturation** — clean = no crash within the stated target/time/seed config. The
> 24h-class campaign (`--campaign-hours 24` on a dedicated host) remains the future operator step.

- **Scope (what it witnesses for zic-rs):** crashes / panics / OOM / timeouts on the fuzzed surfaces.
- **Command:** `cargo +nightly fuzz run <target> -- -max_total_time=25 -rss_limit_mb=4096` (verbatim in the receipt).
- **Tool / version:** `cargo-fuzz 0.13.1` · `rustc 1.98.0-nightly (31a9463c6 2026-05-25)` · `libfuzzer-sys 0.4.13`.
- **Input / corpus:** the 9 `fuzz/` targets over the public API; near-empty seed corpora (0–2 each → **shallow**).
- **Result:** **9/9 executed.** Clean (5): aux_table_validator · manifest_json · path_materialization_model ·
  source_lexer · vendor_oracle_json. **Crashed (4 → 3 sites):** posix_footer + tzif_validate_bytes → **F1**
  (`tzif/validate.rs:406`, single-`\n` footer slice); release_diff_tree → **F2** (`model/time.rs:96`,
  `h*3600` multiply overflow); zone_rule_link_parser → **F3** (`source/parser.rs:345`, non-char-boundary slice).
- **Cannot witness:** semantic correctness — a fuzzer finds crashes, not wrong-but-stable output; and a
  **bounded smoke** is not exhaustive (clean ≠ panic-free proof).
- **Findings:** minimized crash inputs preserved as regression seeds in `findings/F1*`, `findings/F2*`,
  `findings/F3*`; **FIXED in T23.cargo-fuzz.2** — each fix landed with its seed as a regression test
  (`tests/fuzz_regressions.rs` + 3 sharp unit tests); seeds replay `rc=0`.
- **Effect on claims:** T23.cargo-fuzz.1 **qualified** the panic-policy "no panic on hostile input" claim
  (3 implicit slice / `str`-slice / arithmetic-overflow panics the static `panic-analysis` grep census
  could not reach); **T23.cargo-fuzz.2 RESTORES it for the known F1–F3 seeds + the bounded rerun** — *not*
  an exhaustive no-panic proof.
- **Non-claims:** a target existing is NOT a fuzz result; a bounded smoke is execution evidence + a defect
  surface, not exhaustive fuzzing.
- **Receipt:** `receipts/RECEIPT-2026-06-04.md` (+ the earlier scaffold note). **Cross-reference:** `fuzz/` ·
  `fuzz/receipts/RUNS.md` · `docs/panic-policy.md` · `audits/claim-boundary-map.md`.
