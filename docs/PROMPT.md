# zic-rs — implementation notes (reviewed, corrected)

> This is the refined, corrected implementation prompt for `zic-rs`. It descends from the
> original draft (kept verbatim at `research/zic-rs.txt`) but folds in the corrections an
> expert review made after reading `zic(8)`, RFC 9636 / `tzfile(5)`, and after capturing
> exact reference-`zic` (tzcode 2026b) output. Where the draft was technically loose, this
> version is precise. Treat **this** file as the source of truth for the spec.

## System mode

Strict Rust infrastructure port + timezone-compiler correctness + compatibility oracle.

## Mission

Build a memory-safe Rust implementation of (a declared subset of) the IANA timezone compiler
`zic`. `zic-rs` compiles IANA tzdata source files into TZif zoneinfo files: deterministic
output, explicit subset support, reference comparison against classic `zic`, and secure
handling of untrusted source input.

This is **not** a toy parser, a display utility, a TZif inspector, a wrapper around system
`zic`, or a full-replacement claim in v0.1.

**Project identity.** A memory-safe, deterministic timezone compiler for a declared subset
of IANA tzdata, with a compatibility harness against reference `zic`.

**Core principle.** Correctness is measured, not asserted. Develop as a compiler:
`source text → tokens → parsed records → semantic model → transitions → TZif writer → oracle`.

## Hard goals

Safe Rust only (`#![forbid(unsafe_code)]`). Deterministic output. Clear unsupported-syntax
reporting. Reference comparison against classic `zic`. Library-first architecture. CLI
usable in scripts. Fixture-driven development. No silent fallback to system `zic`. No writing
into system directories by default. No mutation of `/usr/share/zoneinfo`. No path traversal
through zone names or link targets. Timeless, well-commented, maintainable systems code.

## Hard non-goals for v0.1

Do not claim full `zic` replacement. Do not implement every historical edge case
immediately. Do not install into `/usr/share/zoneinfo`. Do not set privileged
ownership/mode. Do not implement `-L` leap seconds (unless everything else is solid). Do not
support every `zic` option. Do not shell out to `zic` on the compile path. Do not hide
unsupported input behind approximate output. **Never emit incorrect TZif when syntax is
unsupported — fail explicitly.**

## Corrections folded in (read these — the draft got them subtly wrong)

1. **TZif version is content-driven, not blanket "v2/v3".** Emit version byte `2` normally;
   emit `3` *only* when v3-only content appears (e.g. transition/`AT` times > 24:00, or other
   POSIX.1-2024 footer extensions); emit `4` only for truncated leap tables (out of scope).
   Readers gate behaviour on the version byte — never label a file `3` gratuitously.
2. **The v1 (32-bit) block is always present** in a v2+ file (RFC 9636 mandates header + v1
   block, then v2+ header + block, then footer). Moreover, modern `zic`'s **slim** default
   writes the v1 block as a near-empty **stub** (`timecnt=0, typecnt=1, charcnt=1`, one type
   `utoff=0`, empty abbreviation); the real data lives only in the v2+ block. Reproduce the
   stub exactly — it is what you byte-match.
3. **slim/fat is orthogonal to the version byte.** It governs how many explicit transitions
   are written versus relying on the POSIX footer for the open-ended future. Emit slim-style.
4. **The footer is load-bearing and must be exact.** Synthesise the POSIX `TZ` footer only
   for shapes you can represent precisely (no-DST → `STD<offset>`; simple annual last-weekday
   DST → full rule). Otherwise fail closed — never an approximate footer. Mind the **sign
   inversion**: TZif `tt_utoff` is seconds *east* of UT; POSIX `TZ` offset is west-positive,
   so `utoff=-18000` → `EST5`.
5. **The oracle is semantic first.** Semantic parity via `zdump`/decoded comparison is the
   binding contract; **byte-parity is asserted only where a pinned reference blob is checked
   in.** Pin the reference toolchain (here: `zic`/`zdump` tzcode 2026b; tzdata 2026b from
   `/usr/share/zoneinfo/tzdata.zi`). `zdump` needs **absolute** paths to compiled files
   (relative names are treated as `TZDIR` lookups).
6. **Honour the manpage's hardening details:** 2048-byte max line (incl. newline),
   fractional seconds rounded to nearest, NUL rejection, and the leap-second-in-`AT`
   convention (defer leap handling, but know it exists).

## CLI shape

```
zic-rs compile --input tzdata/ --out zoneinfo/ --zone Europe/London
zic-rs compile --input tzdata/ --out zoneinfo/ --zones zones.txt
zic-rs compile --input tzdata/ --out zoneinfo/ --all-supported
zic-rs compare --input tzdata/ --zone Europe/London --reference-zic zic
zic-rs explain --input tzdata/ --zone Europe/London
zic-rs supported-syntax
```

All output goes to an explicit `--out`; a missing `--out` is a clean error. No default to
`/usr/share/zoneinfo`.

## Security posture

Treat tzdata input as untrusted. Reject NUL bytes; enforce max line length and record/
transition limits; reject path traversal in zone names and link targets; reject absolute
names, `.`/`..` components, and names beginning with `-`; write atomically (temp + rename);
never overwrite without `--force`; never follow/write through pre-existing symlinks unsafely;
keep deterministic ordering of records, transitions, files, diagnostics, and reports.

## Diagnostics

Every diagnostic carries: severity (error/warning/info), stable code (`ZIC0xx`), message,
source file, line, optional field span, optional suggestion. Codes (stable contract):
`ZIC001_UNSUPPORTED_DIRECTIVE`, `ZIC002_INVALID_FIELD_COUNT`, `ZIC003_INVALID_MONTH`,
`ZIC004_AMBIGUOUS_NAME_ABBREVIATION`, `ZIC005_INVALID_DAY_RULE`, `ZIC006_INVALID_TIME_SUFFIX`,
`ZIC007_UNSUPPORTED_YEAR_TYPE`, `ZIC008_OUTPUT_PATH_TRAVERSAL`, `ZIC009_TOO_MANY_TRANSITIONS`,
`ZIC010_UNSUPPORTED_LEAP_SECONDS`, `ZIC011_REFERENCE_ZIC_MISMATCH`, `ZIC012_INVALID_VALUE`.

## Dependency policy

Minimal and serious: `clap` (CLI), `thiserror` (errors); `tempfile` dev-only. Implement
calendar arithmetic in-house (no `time`/`jiff`) to avoid semantic drift from `zic`. Avoid
parser-generator frameworks, async runtimes, and large timezone libraries.

## Build doctrine — oracle-first, fixture-class-driven

This is "a small compiler with a hostile reference oracle," not "port a C file." Stand up the
TZif writer and the `compare` oracle **early** (on `Etc/UTC`), then grow the
parser/calendar/transition engine *into* the writer one fixture class at a time:
fixed-offset → link → simple DST rule → `UNTIL` → `lastSun` → `Sun>=8`/`Sun<=25` → `24:00` →
`%s` → `%z` → one real IANA slice. **Add syntax only when a fixture forces it.** Fail closed
on everything else.

## Milestones

* **T1 (delivered):** skeleton + fixed-offset zones + `Link` + full TZif writer + safe output
  tree + semantic/byte oracle. `Etc/UTC` and a fixed-offset zone byte-match reference `zic`.
* **T2:** transition compiler for simple DST (`Rule` expansion, wall/std/UT→UT conversion via
  the prevailing offset, `%s`). Target: `Test/Simple` semantically matches.
* **T3:** `UNTIL`/multi-era, `24:00`/negative times (content-driven v3), `%z`, `STD/DST`,
  negative SAVE, transition-limit enforcement, byte-parity mode.
* **T4:** IANA slice expansion from pinned `tzdata.zi`, each zone gated by the oracle.

## Acceptance criteria (v0.1 trajectory)

No `unsafe`. Builds clean. Parses the `Rule`/`Zone`/`Link` subset. Compiles fixed-offset
zones (and, per milestone, rule-driven zones) to valid TZif v2/v3. Writes only under `--out`.
Has path-traversal tests, deterministic output, a reference-comparison harness, fixture
provenance, a clear unsupported-syntax report, and README/docs explaining non-goals. At least
one real IANA tzdata slice compiles and semantically matches reference `zic` (by T4).

## When uncertain

Prefer an explicit unsupported diagnostic over approximate output. Prefer reference
comparison over intuition. Prefer deterministic simple output over cleverness. Prefer the
library API over CLI-only design. Prefer a small correct conformance slice over broad
incorrect coverage.
