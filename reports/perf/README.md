# `reports/perf/` — the performance / resource ledger (T22)

Receipt-bearing resource profiles for the admitted workloads. **Measured and bounded, not a speed claim**
— see `docs/perf-ledger.md` for the method, the deterministic-anchor / indicative-timing split, and the
non-claims.

- `measure.py` — the harness (external operational tooling; not part of the crate). Drives the **release**
  binary, records wall time + peak RSS (`os.wait4`→`ru_maxrss`) + the deterministic anchors, emits a
  Markdown receipt.
- `RECEIPT-<machine>-<date>.md` — captured receipts (one per host/run). Append-only; a receipt is dated
  evidence, never overwritten.

## Receipts

| Receipt | Host | Anchors |
|---|---|---|
| `RECEIPT-x86_64-2026-06-02.md` | x86_64, 16 cpu, release, ref `zic`/`zdump` 2026b | 107,524 B `tzdata.zi` → 598 TZif files / 435,761 B (586 v2 · 12 v3) · `bundle_hash` `453641ff…` |

Reproduce: `cargo build --release && python3 reports/perf/measure.py target/release/zic-rs /usr/share/zoneinfo/tzdata.zi <date> zic zdump`.
