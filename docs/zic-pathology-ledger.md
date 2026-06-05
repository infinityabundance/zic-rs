# Pathological rule / time-law ledger (T14.4)

> **Classify-first, not exhaustive-support.** This ledger pins reference `zic`'s behaviour for the major
> parser/time-rule edge classes, records zic-rs's *current* behaviour, classifies each by diagnostic
> layer + four-bucket compatibility status, and identifies which are matched / intentionally divergent /
> deferred. It is the **artifact**; behaviour is changed only where a narrow, reference-pinned fix falls
> out safely. Each row is backed by an executable witness in `tests/pathology_ledger.rs` (the T14 lesson:
> a doctrine table becomes a machine-checked assertion). Probed against tzcode **2026b** `zic`/`zdump`.
>
> **Buckets** (from `docs/differences-from-reference-zic.md`): **1** implemented parity · **2** explicit
> compatibility mode · **3** intentional safer divergence · **4** deferred full-parity work.

## The ledger

| Pathology | Reference `zic` (pinned) | zic-rs | Layer | Bucket | Status |
|-----------|--------------------------|--------|-------|--------|--------|
| **Two rules for same instant** (single-era) | **fatal** "two rules for same instant" (warns one rule, errors the other → `errors=true` → exit 1) | **`ZIC023_SIMULTANEOUS_TRANSITION`**, fail-closed | semantic | **1** | ✅ **FIXED in T14.4** (was a `debug_assert!` → panic in debug / non-monotonic TZif in release) |
| **Two rules for same instant** (multi-era, wall-clock-separated) | fatal "two rules for same instant" | **accepts** — wall→UT conversion uses `save_prev`, so the two activations land at *different* UT and zic-rs keeps one (a valid, strictly-increasing stream) | semantic | **4** | known divergence: zic-rs's per-path wall conversion can separate what reference computes as one instant; recorded, deferred (the guard still prevents any *invalid* output) |
| Negative `SAVE` | accept | accept (law 7, T7) | semantic | 1 | ✅ matched |
| Large `SAVE` (≥ 2h) | accept | accept | semantic | 1 | ✅ matched |
| Sub-hour `STDOFF` (sec precision) | accept | accept | semantic | 1 | ✅ matched |
| `24:00` rule times | accept (folds to next day) | accept (T8-v3 day-shift) | semantic | 1 | ✅ matched |
| Year zero (`FROM 0`) | accept (proleptic Gregorian) | accept | semantic | 1 | ✅ matched |
| Large finite year range (e.g. `2000 9999`) | accept (~8000 transitions) | accept; under `-v` trips `ZIC020` (>1200) | semantic | 1 | ✅ matched |
| Far-past `minimum` | coerce → 1900 (obsolete keyword) | coerce → 1900 (T3.2a) | semantic | 1 | ✅ matched |
| Far-future `maximum` | recurring POSIX footer | recurring footer (T3) | semantic | 1 | ✅ matched |
| `UNTIL` exactly on a transition | boundary-coincident handling | matched (T5 #2/#3 boundary-coincidence) | semantic | 1 | ✅ matched |
| Duplicate-instant **no-op** era change (same offset/abbr across `UNTIL`) | dedup → no extra transition | dedup → 0 extra transitions | semantic | 1 | ✅ matched |
| **Offset > 24h** (e.g. `25:00`) | accept **+ `-v` warning** "values over 24 hours not handled by pre-2007 versions of zic" | accept **+ verbose-only `ZIC026`** (same magnitude rule) | warning | **1** | ✅ **FIXED in T15.5-remainder** — `ZIC026_VALUE_OVER_24_HOURS` over STDOFF/SAVE/AT/UNTIL, magnitude > 24:00:00 (exactly 24:00:00 silent), byte-preserving |
| Zone **name** with non-letter bytes (e.g. `T/T2400`) | accept **+ `-v` warning** "file name '…' contains byte 'N'" | accepts; name-as-path policy not yet at this granularity | operational | **4** | deferred → **T14.5 `ZoneNamePathPolicy`** (this is name admissibility, not a time-law) |

## The headline finding — "two rules for same instant" was a panic

The probe that found it (single-era):

```
Rule S 2000 only - Jun 1 2:00 1:00 D
Rule S 2000 only - Jun 1 2:00 0 S
Zone T/Sim 0:00 S %sT
```

* **Reference `zic`** (pinned `zic.c` ~3611): on the second rule it `warning()`s "two rules for same
  instant", and on the first it `error()`s the same — `errors = true` → **exit 1 (fatal)**.
* **zic-rs (before T14.4):** the two activations expanded to the *same* UT instant, and the only guard
  was a `debug_assert!(strictly increasing)` — so a debug build **panicked** and a release build would
  have emitted a **non-monotonic (invalid) TZif**. A panic on untrusted input violates the project's
  panic policy; an invalid TZif is worse.
* **zic-rs (T14.4):** a real runtime guard `ensure_strictly_increasing()` (in `src/compile/transitions.rs`,
  called on every compiled stream — single- and multi-era) **fails closed** with
  `ZIC023_SIMULTANEOUS_TRANSITION` at the zone's source line. This **matches reference at the
  class/exit-status level** (both reject the input fatally) and is the narrow, reference-pinned, safe fix
  the ledger surfaced. CORE.1 stays 341/0/0 — no canonical zone has a same-instant conflict, so the guard
  never fires on valid data.

**Honest scope of the fix.** The guard prevents *any* path from emitting a non-monotonic TZif. It fully
matches reference for the single-era case. The **multi-era, wall-clock-separated** case (where zic-rs's
chronological `save_prev` conversion lands the two activations at different UT) still *accepts* where
reference errors — a recorded bucket-4 divergence, not a safety bug (the output is valid). Reproducing
reference's exact same-`save` simultaneity test across the multi-era path is deferred.

## Two newly-surfaced `-v` warning divergences (both now closed)

Both were cases where reference *accepts* but emits a `-v` portability/name warning that zic-rs did not
yet emit (so zic-rs was *quieter*, never *wronger*); **both have since landed**:

1. **"values over 24 hours not handled by pre-2007 versions of zic"** — an offset/SAVE/AT/UNTIL magnitude
   portability warning → **✅ `ZIC026_VALUE_OVER_24_HOURS`** (T15.5-remainder; verbose-only, the exact
   `zic.c::gethms` magnitude rule, byte-preserving).
2. **"file name '…' contains byte 'N'"** — a zone-name admissibility warning → **✅ `ZIC024`** (T14.5
   `ZoneNamePathPolicy`, name-as-path), where it belongs.

Both are recorded so the gap was visible, not silently absent — and both are now emitted.

## Acceptance (T14.4)

> T14.4 is accepted when zic-rs records a table-driven pathology ledger for the major parser/time-rule
> edge classes, pins reference behaviour for each, records current zic-rs behaviour, assigns each a
> layer + bucket + status, and identifies matched / intentionally-divergent / deferred cases — without
> changing behaviour except where a narrow reference-pinned fix falls out safely. *(Met: the ledger +
> `tests/pathology_ledger.rs` executable witness; the one behaviour change is the `ZIC023` panic→fail-
> closed fix, reference-pinned and CORE.1-safe; two `-v` warning gaps + the multi-era nuance are
> recorded as deferred, not silently dropped.)*

## Non-claims (T14.4)

* Not an **exhaustive** pathology enumeration — the ledger is seeded with the major classes, not every
  conceivable degenerate input.
* `-v` warning parity for "values over 24 hours" (`ZIC026`, T15.5-remainder) and name-byte warnings
  (`ZIC024`, T14.5) is now **claimed** (verbose-only, magnitude/byte rules pinned to `zic.c`).
* Multi-era same-instant **parity** with reference is **not** claimed (single-era is; multi-era is a
  recorded divergence that still produces valid output).
* Matched rows assert **accept/reject disposition** here; deep behaviour parity for already-covered
  classes is carried by CORE.1 + their original milestone tests, not re-litigated in this ledger.
