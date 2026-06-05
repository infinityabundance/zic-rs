# zic-rs fuzzing harness (T17.FUZZ)

> **A fuzz target existing is not a fuzzing result.** A fuzz run is admitted **only** by a receipt
> recording toolchain, target, corpus hash, duration, result, and artifacts (see
> [`receipts/TEMPLATE.md`](receipts/TEMPLATE.md)). **`pending_capture`** means the harness exists but
> **no fuzzing claim is made** — the structure is real, the campaign is an operator/lab task.

This is a **separate, non-published crate** (`zic-rs-fuzz`) that exercises only the **public API** of
`tzcompile`. It is built **only** by `cargo +nightly fuzz run <target>` from inside `fuzz/` (needs nightly
+ libFuzzer); the parent crate is a single package with no `[workspace]` and does not depend on this, so
the parent gate (`cargo build`/`test`/`clippy`/`fmt` in `..`) never compiles it. In a stable, no-network
environment (e.g. the one this scaffold was authored in) it **cannot be built** — which is why the targets
sat `pending_capture` until a bounded smoke ran on a nightly+network host (see "T23.cargo-fuzz.1" below).

## Why this exists

Fuzzing is a **first-class evidence surface** here, in the project's receipt-bearing style: named targets,
seed corpora, a fixed receipt format, explicit non-claims, and an honest run status — not a vague "we
fuzzed" badge. It complements `docs/panic-policy.md` (no panic on hostile input), `risk-register.md`, and
the T17.1a/T17.5 input-hardening: fuzzing is how those guards are *adversarially* exercised over time.

## Targets (priority order)

The first three defend the most dangerous hostile-input / report-ingestion surfaces.

| # | Target | Public surface fuzzed | Contract |
|---|---|---|---|
| 1 | `tzif_validate_bytes` | `tzif::validate::parse` + `tzif::rfc9636::validate` | malformed/hostile TZif → typed `Err`/verdict, never panic/OOB/wrap/OOM (T17.1a/T17.5) |
| 2 | `vendor_oracle_json` | `vendor_oracle::VendorOracleReceipt::from_json` | hostile receipt JSON → `Ok`/`Err(ReceiptParseError)`, never panic; self-assessment ignored |
| 3 | `manifest_json` | **none — `surface_absent`** | manifests are **write-only**; no ingestion path exists. Reserved no-op (the list is authoritative); activates if a reader is ever added |
| 4 | `source_lexer` | `source::lexer::tokenize` | hostile `.zi` text → typed `Ok`/`Err`, never panic (T14 admissibility) |
| 5 | `zone_rule_link_parser` | `source::parser::parse_into` | hostile records → typed `Ok`/`Err`, never panic (T13) |
| 6 | `posix_footer` | `tzif::validate::parse` (footer path; corpus-directed at footer tails) | never panic; `Ok`/`Err` only |
| 7 | `aux_table_validator` | `aux_tables::{validate_zone_table, iso3166_codes}` | every table input → a verdict, never panic (T16.4) |
| 8 | `release_diff_tree` | `release_diff::build_release_diff` (structural-only, `zdump=None`) | two parsed halves diffed deterministically; never panic |
| 9 | `path_materialization_model` | `fs::output_tree::{safe_relative_path, is_contained}` | name→path policy; accepted names are relative + contained, never panic (T14.5) |

## Corpus

`corpus/<target>/` holds seed inputs. Priority targets are seeded from real committed fixtures
(`fixtures/leap/reference/*.tzif`, `fixtures/minimal/*.zi`, `fixtures/vendor-oracle/*.json`) plus a few
synthetic seeds. The operator may add more (the diagnostic/range/hostile fixtures under `fixtures/` and
`tests/` are good additional seeds). `manifest_json/` is intentionally empty (`surface_absent`).

## How to run (operator / lab task)

```sh
cargo install cargo-fuzz                  # once; needs network
cd fuzz
cargo +nightly fuzz run tzif_validate_bytes -- -max_total_time=86400   # a 24h campaign
# then record a receipt from receipts/TEMPLATE.md into receipts/RUNS.md (status: completed)
```

## The long burn-in harness — `run-long-burnin.sh` (LONG-FUZZ-HARNESS.1)

`fuzz/run-long-burnin.sh` is the **reproducible operator/lab harness** for an extended campaign: it runs
the 9 targets for a declared per-target duration, records the full toolchain/host/seed/git provenance, and
writes `audits/cargo-fuzz/receipts/RECEIPT-LONG-FUZZ-<UTC>.md` automatically. **It does not claim coverage
saturation** — the harness is the artifact; the 24h-class run is the future operator step.

```sh
bash fuzz/run-long-burnin.sh --smoke                       # per-target 60s sanity pass
bash fuzz/run-long-burnin.sh --per-target-minutes 10       # 10 min/target (the LONG-FUZZ-SMOKE.1 shape)
bash fuzz/run-long-burnin.sh --campaign-hours 24           # 24h total, split evenly across the 9 targets
bash fuzz/run-long-burnin.sh --target tzif_validate_bytes --per-target-minutes 60
```

Precedence: `--per-target-minutes` > `--campaign-hours` (split over the selected targets) > `--smoke`
(60s) > default (300s). The receipt's **overall verdict** uses a fixed vocabulary:

| verdict | meaning |
|---|---|
| `clean` | every selected target ran to time, exit 0, **no new crash artifact** (no saturation claimed) |
| `crash_found` | a new `crash-*`/`oom-*`/`timeout-*` artifact appeared in `fuzz/artifacts/<target>/` |
| `inconclusive_environment` | a target's libFuzzer exited non-zero with no new artifact (build/host issue — **not** a zic-rs finding) |
| `interrupted` | SIGINT/SIGTERM before all selected targets completed (partial receipt written) |
| `not_run` | nightly + cargo-fuzz absent — nothing executed |

On a crash: `cargo +nightly fuzz tmin`, add the seed as a regression test in **`tests/fuzz_regressions.rs`**
(main crate), land the fix with that test, and record both in the receipt (the T23.cargo-fuzz.2 discipline).

## T23.cargo-fuzz.1 — bounded smoke (RAN 2026-06-04)

A **bounded smoke** has now run: `cargo-fuzz 0.13.1`, nightly rustc 1.98.0, libFuzzer, **25 s per target**.
**9/9 targets executed · 5 clean · 4 crashed → 3 distinct panic sites (F1–F3)**, preserved as regression
seeds in `../audits/cargo-fuzz/findings/` and **not fixed** (follow-up campaign **T23.cargo-fuzz.2**). Per-
target rows are in `receipts/RUNS.md`; the full evidence is `../audits/cargo-fuzz/receipts/RECEIPT-2026-06-04.md`.
This is **execution evidence + a defect surface**, *not* exhaustive fuzzing and *not* a safety certificate;
the longer (24 h) campaigns above remain an operator/lab task. The smoke **qualified the panic-policy claim**
— it found three panic-on-hostile-input sites that the static `panic-analysis` census could not reach.

## T23.cargo-fuzz.2 — fixed + re-verified (2026-06-04)

The three findings are **fixed (3/3)**, each with its preserved minimized seed as a regression test
(`tests/fuzz_regressions.rs` seed replays + 3 sharp per-function unit tests). All 4 crash seeds now replay
`rc=0`, and the **bounded smoke re-ran 9/9 CLEAN · 0 crashes** (same 9 targets, same 25 s budget; the 4
formerly-crashing targets now explore millions of execs to completion). Fixes: F1 footer
`strip_prefix`/`strip_suffix`, F2 `checked_mul`/`checked_add`, F3 byte-compare in `strip_prefix_ci` — no
semantics or TZif-output change (CORE.1 341/0/0). This **restores the panic-policy claim for the known
F1–F3 seeds and this bounded rerun only — it is *not* an exhaustive no-panic proof.** Full receipt:
`../audits/cargo-fuzz/receipts/RECEIPT-2026-06-04-fuzz2.md`.

## Required non-claim

*A fuzz target existing is not a fuzzing result. A fuzz run is admitted only by a receipt recording
toolchain, target, corpus hash, duration, result, and artifacts. A **bounded smoke** (short budget, near-empty
corpora) is execution evidence + a defect surface — not exhaustive fuzzing, not coverage saturation, and a
clean target in it is not a panic-free proof.*
