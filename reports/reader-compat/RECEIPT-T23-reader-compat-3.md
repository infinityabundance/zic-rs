# T23.reader-compat.3 — cross-reader TZif ecology gauntlet (2026-06-04)

> **Claim wording (binding):** *T23.reader-compat.3 does not prove universal consumer compatibility. It
> classifies how a bounded fixture set from the T18 provenance ledger is interpreted by additional TZif
> reader ecosystems and records reader limitations separately from compiler mismatches.*

This widens the reader axis that `docs/provenance-ledger.md` (T18.3) cross-references. For each fixture it
asks the sharp question: **does a real TZif reader interpret zic-rs's output the same as it interprets
reference-`zic`'s output?** — i.e. reader-on-`zic-rs-file` vs reader-on-`reference-file` at a spread of UTC
instants. A reader that *cannot consume raw TZif at all* is classified, **not** scored as a zic-rs failure.

## Fixture set (from the T18 40-case ledger)

| fixture | trap it stresses |
|---|---|
| `Europe/Lisbon` | the slim structural residual (T8 ref-fatter-by-1) |
| `Asia/Singapore` | odd historical offsets (+6:55:25 → +7:20 → +7:30 → +8:00) |
| `Pacific/Kiritimati` | +14 maximum offset |
| `Europe/London` | footer-heavy (far-future projection) + WW2 double summer time |
| `Europe/Dublin` | negative DST (winter-time inversion) |
| `America/New_York` **[right/]** | leap/right profile (compiled with `-L leapseconds`) |

Probe = 12 UTC instants spanning ~−2³¹ (1901) · pre-1970 · 2000/2019/2020 · the 2³¹ boundary (2038) ·
**year 2200 (footer projection)** · and a post-2017-leap instant (for `right/`).

## Readers — capabilities (acceptance #2)

| reader | version | how invoked | reads **raw TZif**? | honors POSIX footer? | leap/`right` support? |
|---|---|---|---|---|---|
| **glibc** `localtime` | glibc **2.43** | `TZ=:<file>` + `localtime_r` (C probe) | ✅ raw file | ✅ yes | ✅ **applies leaps** (right/) |
| **Go** `time` | go **1.26.3** | `time.LoadLocationFromTZData(name, bytes)` | ✅ raw bytes | ✅ yes | ⚠️ **ignores** leap table (no TAI); also does not expose `is_dst` |
| **CCTZ** / abseil | absl **20260107** | `absl::LoadTimeZone(name)` + `$TZDIR` | ✅ raw file (`TZDIR` honored — **verified** by a planted-zone discriminating test) | ✅ yes | ❌ **refuses** leap-bearing files (`LOAD_FAIL`; confirmed on the *system* `right/` zone too) |
| Java `java.time` | OpenJDK **17.0.19** | `java.time` | ❌ — consumes `$JAVA_HOME/lib/tzdb.dat` (TZDB format); no public raw-TZif loader | n/a | n/a |
| PHP / timelib | PHP **8.5.6** | `DateTimeZone` | ❌ — bundled timelib DB (`php -i`: *Timezone Database → internal*, Olson 2026.1) | n/a | n/a |
| ICU4C | ICU **78.3** | `icu::TimeZone` | ❌ — ICU `zoneinfo64.res` resource bundle | n/a | n/a |

**Note:** "reads raw TZif? → ❌" is a reader *architecture* fact (they consume their own pre-compiled
database), **not** a statement about zic-rs's output. Those three are classified `unsupported_by_reader`.

## Verdict matrix (acceptance #2/#3)

| reader | Lisbon | Singapore | Kiritimati | London | Dublin | NY [right/] |
|---|---|---|---|---|---|---|
| **glibc** | match | match | match | match | match | **match** (leaps applied; zic-rs right/ ≡ ref right/) |
| **Go** | match | match | match | match | match | **match_with_known_reader_limitation** (Go ignores leaps; still equivalent) |
| **CCTZ/absl** | match | match | match | match | match | **unsupported_by_reader** (cctz cannot load leap-bearing TZif; fails on ref + system too) |
| **Java** | unsupported_by_reader | ″ | ″ | ″ | ″ | ″ |
| **PHP** | unsupported_by_reader | ″ | ″ | ″ | ″ | ″ |
| **ICU4C** | unsupported_by_reader | ″ | ″ | ″ | ″ | ″ |

**Tally: 16 `match` · 1 `match_with_known_reader_limitation` · 19 `unsupported_by_reader` · 0 `mismatch` ·
0 `unavailable`.**

Headline: **every reader that can consume raw TZif (glibc, Go, CCTZ) interprets zic-rs's output identically
to reference-`zic`'s output across all 6 fixtures and 12 instants — 0 mismatches.** The footer-projection
instant (year 2200) matched for all three (POSIX TZ-string extrapolation, not stored transitions).

## Named reader limitations (recorded separately — NOT zic-rs failures, acceptance #4)

- **CCTZ/abseil cannot load leap-bearing (`right/`) TZif files** — `absl::LoadTimeZone` returns `LOAD_FAIL`
  on the `right/` fixture. Verified this is a **universal CCTZ limitation**, not a zic-rs defect: it fails
  identically on reference-`zic`'s `right/` output *and* on the host's system `/usr/share/zoneinfo/right/`
  zone. → `unsupported_by_reader`.
- **Go's `time` ignores the leap-second table** (applies no TAI offset). For the `right/` fixture this is
  `match_with_known_reader_limitation`: Go reads both files equivalently, it simply does not act on leaps.
  Go also does not expose `is_dst`, so the Go verdict compares `(offset, abbreviation)` only.
- **Java / PHP / ICU consume their own pre-compiled databases** (`tzdb.dat` / internal timelib / `zoneinfo64.res`),
  with no public API to load a raw TZif *file*. They are real, ubiquitous readers — but of a *compiled
  database*, not of arbitrary TZif bytes; so this gauntlet cannot (and does not) score them on zic-rs output.

A correctness aside that strengthens the result: an early run risked reading **system** zoneinfo instead of
our files. A discriminating test (planting a `Pacific/Kiritimati` +14 file under the name `Europe/Lisbon` in
a temp `$TZDIR` → CCTZ printed +50400) **confirmed CCTZ honors `$TZDIR`**, so the cctz matches genuinely
read zic-rs's bytes.

## Non-claims

- **Not universal consumer compatibility.** 6 fixtures × a bounded reader set over 12 instants; not all 598
  zones, not all readers, not all instants.
- **Reader equivalence ≠ compiler equivalence ≠ civil-time truth.** This shows readers see zic-rs ≡ ref;
  the behaviour-vs-reference contract is CORE.1 + T18, and timezone *truth* is never claimed (IANA's).
- A `match` is scoped to *these readers, fixtures, instants, host, and reader versions* (recorded above).
- `unsupported_by_reader` records a reader's architecture, not a zic-rs gap.

## Gate

Docs/harness only — **no `src/` change**; CORE.1 sweep unchanged at **341/0/0**. Harness + raw result in
`reports/reader-compat/r3/` (`run.py`, `glibc_probe.c`, `goprobe.go`, `absl_probe.cc`, `result.tsv`).
Reproduce: install glibc/Go/abseil, build the probes, run `python3 run.py` against the T18 fixtures in
`/tmp/prov/{zrs,ref,zrs-right,ref-right}`. Builds on `T23.reader-compat.1/.2` (zdump + Python `zoneinfo`).
