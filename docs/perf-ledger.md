# Performance / resource ledger (T22)

> A **receipt-bearing resource profile**, not a vanity benchmark: it proves that **normal admitted
> workloads are measured and bounded**, so a future regression is *catchable*. It is the measurement
> companion to the T17.1b resource caps (`src/limits.rs`) and the T21 builder profile
> (`docs/container-embedded-builder.md`). Receipts live in `reports/perf/`; the harness is
> `reports/perf/measure.py` (external operational tooling — the core crate stays pure, no `build.rs`, no
> measurement code in the library).

## The doctrine (kept in every receipt)

**T22 proves that normal admitted workloads are measured and bounded; it does NOT claim fastest-`zic`,
universal performance superiority, or adversarial-CPU-exhaustion immunity.** The `ResourceLimits` caps bound
the *input-size* tail (`RISK.RESOURCE.1`), not all adversarial CPU — that non-claim stands
(`docs/security-rewrite-evaluation.md` §C).

## The determinism split (why the ledger is trustworthy)

A perf number is only useful if you know which half is a contract and which is weather:

- **Deterministic anchors — the regression anchors (host-independent, must not drift):** input bytes + sha ·
  output tree file count + bytes · the `size-report` counts (`tzif_files`/`symlink_links`/`other_files`/
  `footer_present`) + version histogram · the deterministic **`bundle_hash`** · the `ResourceLimits` config.
  These are byte-deterministic (`tests/reliability.rs`); a change here is a real regression.
- **Indicative timings — this host, this run only:** wall time + peak RSS. Recorded with the host + build
  identity so two receipts are comparable *on the same host*, never quoted as a universal speed claim.

## Receipt schema (`reports/perf/RECEIPT-<machine>-<date>.md`)

Host & build identity (machine · cpus · kernel · rustc · profile=`release` with `overflow-checks` · zic-rs
version · reference `zic`/`zdump`) · the deterministic anchors · the limits-in-force envelope · a table of
**measured workloads** (wall ms · peak RSS MiB · exit) · the reproduce command. Workloads measured:

| Workload | what it exercises |
|---|---|
| `compile --all-supported` full tree | the whole admitted-release compile path (the dominant real job) |
| `size-report` | the read-only footprint + `bundle_hash` (T21.2) |
| `support-report` | the conformance frontier map |
| `structural-report --reference-zic zic` | the heavy oracle path (spawns reference `zic` per zone) |
| `doctor` | host-probe overhead (always exit 0) |
| `release-diff` self-diff | the diff engine cost (behaviour axis off — a self-diff isolates the engine) |

## Current receipt (this environment)

`reports/perf/RECEIPT-x86_64-2026-06-02.md` (x86_64, 16 cpu, release, reference `zic`/`zdump` 2026b). The
durable anchors: input `tzdata.zi` **107,524 bytes** → **598 TZif files / 435,761 bytes** (586 v2 + 12 v3,
all with a footer), `bundle_hash` `453641ff…`. Indicative: full-tree compile ≈ 12 ms, structural-report (vs
reference `zic`) ≈ 20 ms, peak RSS ≈ 22 MiB — i.e. **a full admitted-release compile is tens of milliseconds
and tens of megabytes on this host**, sitting *far* inside every `ResourceLimits` cap (≈350 zones / ≈600
links / 27 leaps vs caps of 1,000,000 / 100,000). Timings are indicative; the anchors are the contract.

## Reproduce

```sh
cargo build --release
python3 reports/perf/measure.py target/release/zic-rs /usr/share/zoneinfo/tzdata.zi <YYYY-MM-DD> zic zdump
```

(`measure.py` uses `os.wait4` → `ru_maxrss` for accurate per-workload peak RSS on Linux; no GNU `time`
dependency. Timings differ per host/run — the deterministic anchors must not.)

## Non-claims

- **Not a speed claim** — no "faster than `zic`"/"fastest" assertion; this is a *bounded-and-measured*
  record, not a comparison verdict.
- **Not universal** — one host, one run, the admitted 2026b release; another host will show different
  timings (and must show identical anchors).
- **Not adversarial-CPU immunity** — `RISK.RESOURCE.1` bounds the input-size tail; pathological CPU cost is
  not claimed solved (the caps reject oversized *input*, not all expensive computation).
- **Not a deterministic timing** — wall/RSS are weather; only the anchors are a contract.
