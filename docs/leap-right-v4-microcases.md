# Leap seconds / `right/` / TZif v4 — reference inventory + microcases (T11)

> **Status: T11 COMPLETE (T11.1–T11.6).** Reference-pinned first (the grammar/time model + byte fixtures
> from real `zic -L`), then implemented substep-by-substep: T11.1 inventory · T11.2 grammar isolation ·
> T11.3 Stationary emission · T11.4 Rolling · T11.5 Expires/v4 · T11.6 `right/` profile (`-L`). Leap
> support is **strictly opt-in** (`--leapseconds`); ordinary canonical-zone output is unchanged
> (CORE.1 341/0/0). Raw fixtures: [`fixtures/leap/reference/`](../fixtures/leap/reference/); inputs:
> [`fixtures/leap/src/`](../fixtures/leap/src/). Provenance: `zic` (tzcode) 2026b.

## Doctrine (the load-bearing distinctions)

- **Leap support is not canonical-zone support.** Leap seconds are a *separate grammar* in a *separate
  source file*; CORE.1 (341/341 over 1900..2040) is unaffected by, and independent of, leap support.
- **`right/` is not `posix/`.** The leap-corrected tree (`right/`) is a **build profile** (zones
  compiled *with* the leap table); the default/`posix/` tree is *without*. (Profile work is T12; the
  leap *grammar* is T11.)
- **TZif v4 is not "newer is better" — it is content-triggered.** Leap tables stay v2/v3 unless a v4
  trigger fires (below).
- **Leap grammar is never accepted in ordinary zone-source mode.** `Leap`/`Expires` are recognised
  *only* in a leapseconds file; in a zone file they are "input line of unknown type" (already enforced
  — see Boundary).

## Grammar (pinned from `zic.c` 2026b)

- **`Leap  YEAR MON DAY HH:MM:SS  CORR  ROLL`** (7 fields, `inleap`): `CORR` is `+` (correction `+1`) or
  `-` (`-1`); `ROLL` is **`Rolling`** or **`Stationary`** (`leap_types`: `Rolling=true`,
  `Stationary=false`). `leapadd(t, corr, roll)` inserts into a `(trans, corr, roll)` table sorted by
  `trans`; `adjleap()` makes `corr` **cumulative**.
- **`Expires  YEAR MON DAY HH:MM:SS`** (6 fields, `inexpires`): sets `leapexpires`; at most one.
- **`Rolling` vs `Stationary`:** `Stationary` = the instant is **UT**; `Rolling` = the instant is
  **local wall time** (it *rolls* with the zone's offset). Pinned empirically (zone `Test/East5`,
  offset +5h): the same `Leap … 23:59:60` gives `trans = 2017-01-01 00:00:00 UT` (Stationary) vs
  `2016-12-31 19:00:00 UT` (Rolling = Stationary − 18000). The default IANA `leapseconds` file is
  **all Stationary**.
- **`Rolling` + `-r` is a hard error** (`leapadd`: *"Rolling leap seconds not supported with -r"* →
  exit). zic-rs must reproduce this rejection.

## TZif v4 trigger (pinned from `zic.c` `writezone`, lines ~2766–2787)

`version = '4'` **iff** either:
1. **leap-table expiry** — an `Expires` line is present and within range (a **no-op** leap entry is
   appended at the expiry instant, same `corr`), **or**
2. **table truncation** — the first emitted leap entry's `corr` is not `±1` (only reachable under `-r`
   cutting into the cumulative table).

Otherwise a leap table is plain v2/v3. (`TZ_MAX_LEAPS` → a "too many leap seconds" *warning*, not an
error.)

## Reference microcases (byte fixtures)

| Fixture | Source | Version | Leap table |
|---------|--------|---------|------------|
| `stationary-utc.tzif` | IANA `right/UTC` (`-L leapseconds`) | **v2** | 27 cumulative `+1…+27` Stationary leaps, `1972-07-01 … 2017-01-01` |
| `stationary-east5.tzif` | `Test/East5` (+5) `-L stationary.list` | **v2** | one leap, `trans = 2017-01-01 00:00:00 UT`, `corr +1` |
| `rolling-east5.tzif` | `Test/East5` (+5) `-L rolling.list` | **v2** | one leap, `trans = 2016-12-31 19:00:00 UT` (= Stationary − offset), `corr +1` |
| `v4-expires.tzif` | `Test/UTC` `-L expires.list` (`Leap 2016…` + `Expires 2025 Jan 1`) | **v4** | 2 entries: real leap `2017-01-01 corr +1`, then **no-op expiry** `2025-01-01 corr +1` |

These are the ground truth T11.3 (Stationary), T11.4 (Rolling), T11.5 (Expires/v4) must reproduce.

## Boundary — the leap/zone wall (load-bearing, preserved through T11)

`Leap`/`Expires` are **only** recognised in an explicit leap-source (`--leapseconds`/`-L`); in an
ordinary zone file a `Leap` is "input line of unknown type" exactly as reference `zic` (pinned by
`src/source/{parser,leap}.rs` tests). Leap compilation is now **implemented but strictly opt-in** (the
`right/` build profile, T11.6): without `-L`, ordinary `tzdata.zi` output carries **no** leap table and
CORE.1 is unchanged. This fail-closed distinction is **load-bearing**: leap parsing/emission never
leaks into the normal zone path.

## Invariant (in docs **and** code — `src/model/leap.rs`)

> **Leap seconds are not local-time-type transitions.** They do not select an abbreviation, do not
> change `isdst`, and do not appear in the transition stream — they contribute *only* to the TZif
> leap-second correction table. The model types ([`LeapSecond`]/[`LeapTable`]) are deliberately
> separate from `Transition`/`LocalTimeType` so this distinction is visible in code; the leap grammar
> must never reuse the ordinary-transition machinery.

## Substep ladder (T11)

- **T11.1 — reference inventory ✅** (this doc + fixtures; reference-only, no behaviour change).
- **T11.2 — grammar isolation ✅ DONE (grammar wall only; no emission, no behaviour change):** a
  distinct leap-source parser `source::parse_leap_source(bytes, file) → model::LeapTable` recognises
  **only** `Leap`/`Expires` (a `Rule`/`Zone`/`Link` there is "input line of unknown type", matching
  `zic`'s `leap_line_codes`); the ordinary zone-source path still rejects `Leap`/`Expires`. Leap times
  allow the `:60` leap second (a *separate* time parser from `model::time`, which rejects `:60`);
  validates CORR `±`, ROLL Rolling/Stationary (prefix-matched), and **rejects pre-Epoch** (`t < 0`) and
  **multiple `Expires`** — both pinned from `zic.c`. Entries insertion-sorted by `trans`; stored
  verbatim (UT-as-written) + `rolling` flag (Rolling local-wall conversion deferred to T11.4). New
  `model::{LeapSecond, LeapTable}`. +7 tests; **CORE.1 untouched (341/0/0)**. No CLI surface / no TZif
  emission yet (those land in T11.3).
- **T11.3 — Stationary leap microcases ✅ DONE (Stationary + emission only):** `compile::apply_leaps`
  applies `zic`'s **`adjleap`** (cumulative `corr`; each occurrence shifted by the corrections that
  precede it; rejects leaps <28 days apart) and writes `TzifData::leaps` — emitted as data-block
  **record 5** in the **v2 block only** (the v1 stub stays leap-free; v2 is authoritative). Leaps are
  **orthogonal**: `apply_leaps` never touches `transitions`/`types`/`footer`/`version`. (At this step
  Rolling and Expires/v4 were deferred — both now done in T11.4 / T11.5.) Verified against the
  fixtures: `stationary-east5`
  byte-matches (and its leap is **UT-as-written**, *not* shifted by the +5h offset) and the full IANA
  `leapseconds` (27 `+1` Stationary) reproduces `right/UTC`'s cumulative `+1…+27` table. Ordinary zones
  keep `leapcnt = 0` (CORE.1 **341/0/0** unchanged). `tzif::{LeapRecord, ParsedTzif.leaps}` added;
  tests `tests/leap.rs` + `compile::leap::tests`. No CLI surface yet.
- **T11.4 — Rolling leap microcases ✅ DONE (two separate pieces):** **(A) Rolling conversion** —
  `apply_leaps` stores a Rolling leap's occurrence as `adjusted − utoff`, where `utoff` is the offset
  of the local-time-type **in effect at the leap instant** in the zone's transition stream (before the
  first transition: first non-DST type, else type 0) — exactly `zic`'s `todo = leap.trans -
  utoffs[j]`. Verified: `rolling-east5` byte-matches reference, and equals `stationary-east5 − 18000`
  (the +5h offset). **(B) Rolling + `-r`** — a Rolling entry with a range is a **hard config error**
  (`range.is_some()` ⇒ refuse, before any output), matching `zic`'s `leapadd`. Stationary unchanged.
  +3 tests; CORE.1 **341/0/0**. (General rule-based-zone Rolling uses the same in-effect-offset lookup;
  the pinned microcase is fixed-offset.)
- **T11.5 — `Expires` / TZif v4 ✅ DONE (two pieces):** **(A) Expires semantics** — `apply_leaps`
  appends a **no-op** terminal leap record at the expiry instant (itself shifted by the *total*
  cumulative correction — `zic`'s `leapexpires += last`); `corr` = the last cumulative (no ±1: a
  *marker*, not a leap second); the last real leap must precede it. **(B) v4 trigger** —
  content-driven `version = b'4'` *only* when an `Expires` is present (not "newer is better"; no
  Expires + no truncation ⇒ stays v2/v3). Verified: `v4-expires` byte-matches (real `2017-01-01 +1`
  + no-op `2025-01-01 00:00:01 +1`, version **v4**); Stationary/Rolling no-Expires stay non-v4;
  ordinary zones unchanged (CORE.1 **341/0/0**). Leap compilation under `-r` (table truncation) is
  deferred/fail-closed. +2 tests.
- **T11.6 — `right/` build profile ✅ DONE (profile wiring, opt-in):** **`--leapseconds`/`-L <file>`**
  (reference `zic`'s `-L`) parses the leap source (`source::parse_leap_source` → `LeapTable`, stored in
  `CompileConfig.leaps`) and, in `plan::run`, applies it to **every** compiled zone via the existing
  `apply_leaps` path — in **phase 1**, so a fatal (e.g. `Rolling` + `-r`, or a missing/malformed leap
  file) aborts before any write (T9.3 no-partial-install). Without `-L`, ordinary output has **no leap
  table** (`leapcnt = 0`) and canonical-zone conformance is unchanged. Verified end-to-end: CLI
  `--leapseconds <system leapseconds>` on `Etc/UTC` → leapcnt 27, **zdump-identical to reference
  `right/UTC`**; ordinary compile → leapcnt 0; missing leap file → config error 1 (nothing written);
  `Rolling` + `-r` → 1 (no partial install). +3 CLI tests; **CORE.1 341/0/0**. **T11 complete.**

> **Doctrine:** `right/` is an **explicit build profile**. It is **not** the default. It does **not**
> change ordinary canonical-zone conformance. It combines ordinary zone compilation with an explicit
> leap table — a different *output profile* with leap-second semantics, **not** "more correct time".
> (The operator chooses the output location, e.g. `--out right/`; zic-rs has no implicit `right/` tree.)
