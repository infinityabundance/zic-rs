> **T23.cargo-fuzz.1 — bounded smoke RAN 2026-06-04** (`cargo-fuzz 0.13.1`, nightly rustc 1.98.0,
> 25 s/target): **9/9 executed · 5 clean · 4 crashed → 3 distinct panic sites (F1–F3)**, preserved as
> regression seeds in `../../audits/cargo-fuzz/findings/`. Receipt: `RECEIPT-2026-06-04.md`.
>
> **T23.cargo-fuzz.2 — FIXED + re-verified 2026-06-04**: F1/F2/F3 fixed 3/3 (each with its seed as a
> regression test); **bounded smoke re-ran 9/9 CLEAN · 0 crashes**; all 4 crash seeds replay `rc=0`. The
> panic-policy claim is restored for the known F1–F3 seeds + this bounded rerun — **not** an exhaustive
> no-panic proof. Receipt: `../../audits/cargo-fuzz/receipts/RECEIPT-2026-06-04-fuzz2.md`. Bounded smoke ≠ exhaustive fuzzing.

# Fuzz-run ledger (RUNS)

> Append-only. One section per real run, filled from [`TEMPLATE.md`](TEMPLATE.md). **No fuzz-run or
> fuzz-duration claim is made by zic-rs until a `completed` receipt appears here.** The scaffold
> (`fuzz/`, the 9 targets, the seed corpora, this ledger) exists as of **T17.FUZZ**; the campaigns are an
> operator/lab task — this environment is stable + no-network, so `cargo-fuzz`/nightly/libFuzzer are
> unavailable and **no run was executed**.

## Status (T17.FUZZ scaffold — runs pending)

| Target | Status | Seeds | Note |
|---|---|---|---|
| `tzif_validate_bytes` | ✅ **clean** (3.64M execs, 25 s) — was F1 `validate.rs` footer slice, **fixed T23.cargo-fuzz.2** | 2 real TZif (`stationary-east5`, `v4-expires`) | priority 1 — most dangerous hostile-input surface |
| `vendor_oracle_json` | ✅ **clean** (3.68M execs, 25 s) | 1 real receipt sample | priority 2 — third-party JSON ingestion |
| `manifest_json` | ✅ **clean** (52.5M execs; write-only no-op surface) | — | priority 3 *slot*; **no manifest-JSON reader exists** (write-only). Reserved no-op; activates if a reader is added |
| `source_lexer` | ✅ **clean** (1.95M execs, 25 s) | 1 real `.zi` (`dst.zi`) | priority 4 |
| `zone_rule_link_parser` | ✅ **clean** (2.17M execs, 25 s) — was F3 `parser.rs` `str`-slice, **fixed T23.cargo-fuzz.2** | 1 real `.zi` (`dst.zi`) | priority 5 |
| `posix_footer` | ✅ **clean** (8.04M execs, 25 s) — was F1 `validate.rs` footer slice, **fixed T23.cargo-fuzz.2** | 1 real TZif (footer-bearing) | corpus-directed at footer tails via `parse` |
| `aux_table_validator` | ✅ **clean** (669K execs, 25 s) | 1 synthetic `zone.tab` row | — |
| `release_diff_tree` | ✅ **clean** (2.37M execs, 25 s) — was F2 `time.rs` `h*3600` overflow, **fixed T23.cargo-fuzz.2** | 1 synthetic `.zi` pair | structural-only (`zdump=None`), deterministic |
| `path_materialization_model` | ✅ **clean** (10.7M execs, 25 s) | 1 name string | pure + fast — ideal high-throughput target |

**No run receipts yet.** When a campaign runs, append a filled `TEMPLATE.md` block below this table and
flip the row's status to `completed` (with the receipt's key fields).

---

<!-- Completed run receipts are appended below, newest last (append-only). None yet. -->
