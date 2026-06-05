# Reader-compatibility microcase ledger (T23.reader-compat.1)

> **What this proves, and the tier it lives in.** This ledger asks one question: *do real, independent
> TZif **readers** consume zic-rs's compiled output the same way they consume reference-`zic`'s output,
> across the known RFC 9636 Appendix-A / reader-trap cases?* It is **output-consumer evidence** — the
> rung above format-validity and below civil-time truth:
>
> ```text
> memory-safe  ≠  format-valid  ≠  reader-compatible  ≠  semantic parity  ≠  civil-time truth
> ```
>
> A `matched` row means *these named readers, at these named instants, observed identical behaviour from
> zic-rs's TZif and reference-`zic`'s TZif*. It does **not** claim civil-time correctness, reference-`zic`
> parity beyond these readers/instants, or that any untested reader accepts the output.

This is **on-demand work inside T23** (`T23.reader-compat.1`), not a new milestone. Runs are receipt-borne
(`reports/reader-compat/`); a row is `matched`/`diverged`/`unassessed`, never a bare "compatible".

## Readers in the gauntlet

| Reader | Kind | Version (first cut) | Status |
|---|---|---|---|
| `zdump -v` | reference C reader (tzcode/glibc lineage) | tzcode **2026b-dirty** | ✅ run |
| Python `zoneinfo` | independent reader (CPython, `ZoneInfo.from_file`) | CPython **3.14.5** | ✅ run |
| Go `time.LoadLocationFromTZData` | independent reader | — | ▷ **unassessed** (Go not installed here) |
| CCTZ · Timelib/PHP · Java · ICU | further readers | — | ▷ planned (later cut) |

## The microcases (RFC 9636 Appendix-A traps + known reader hazards)

Each row: the **trap**, **why a reader can mishandle it**, the real **fixture zone** (compiled from admitted
`tzdata.zi` 2026b by *both* zic-rs and reference `zic`), whether the two TZif files are **byte-identical**,
the **readers' verdict** (zic-rs output vs reference output, per reader), and the **non-claim**. Probe set =
14 UTC instants spanning **~−2³¹ (1901-12-13) … ~+2³¹ (2038-01-19) … 2200**.

| Microcase / trap | Why readers can fail | Fixture zone | byte-id? | zdump | zoneinfo | Status |
|---|---|---|---|---|---|---|
| Baseline fixed UTC | — | `Etc/UTC` | yes | matched | matched | ✅ |
| **Irish negative DST** (winter is the *DST* type; `isdst` inverted) | readers that assume DST ⇒ +offset misreport winter/summer | `Europe/Dublin` | no | matched | matched | ✅ |
| **Content-driven v3 footer** (non-POSIX `ON`, Ramadan rules) | a v1/v2-only reader may ignore/mis-evaluate the v3 TZ string | `Asia/Gaza` | no | matched | matched | ✅ |
| **Time-of-day == 24:00** (`lastThu 24:00`, stays v2) | off-by-one-day or 24h-clamp bugs in transition expansion | `Africa/Cairo` | no | matched | matched | ✅ |
| **Non-hour, non-half-hour offset** (+5:45) | readers truncating to whole hours/half-hours | `Asia/Kathmandu` | yes | matched | matched | ✅ |
| **+8:45 offset** | same sub-hour truncation hazard | `Australia/Eucla` | no | matched | matched | ✅ |
| **Numeric `-03` abbreviation** | readers expecting alpha abbreviations | `America/Argentina/Buenos_Aires` | no | matched | matched | ✅ |
| **Numeric `+00`/`+02` abbreviations** (modern) | same | `Antarctica/Troll` | yes | matched | matched | ✅ |
| **Far-future recurring POSIX footer** + pre-1970 history | readers ignoring the footer project the last explicit transition forever | `Europe/London` | no | matched | matched | ✅ |
| **Pre-1901 LMT + 2³¹ boundary + far-future footer** | 32-bit `time_t` overflow / pre-first-transition heuristic divergence | `America/New_York` | no | matched | matched | ✅ |
| **Large +14 offset, date-line** | offset-range assumptions | `Pacific/Kiritimati` | yes | matched | matched | ✅ |
| **Historical sub-minute LMT** (+0:19:32) | readers rounding to whole minutes | `Europe/Amsterdam` | no | matched | matched | ✅ |

**First-cut result: 12 microcases × 2 readers = 24 reader-pairs → 24 matched · 0 diverged · 0 reader-error.**
The load-bearing finding: **only 4 of 12 fixtures are byte-identical** (zic-rs default *fat* vs reference
*slim*, plus footer/representation differences) — yet **both** readers consumed all 12 identically. *Reader
compatibility holds even where the bytes legitimately differ* — the exact reason this rung exists separately
from byte/structural parity.

## Non-claims (the loud column)

- Reader compatibility **does not** prove civil-time truth (IANA/CLDR own that).
- It **does not** prove reference-`zic` parity beyond these readers and these probe instants.
- It **does not** prove every downstream runtime — only that **these named readers** consumed **these named
  TZif artifacts** this way, on this host, at these instants.
- An `unassessed` reader (Go here) is **not** a pass — it is "not run", recorded honestly.

## T23.reader-compat.2 — RFC 9636 Appendix-A expansion (right/leap · v4 · `-r` truncation)

The first cut (above) was normal-ish zones. **T23.reader-compat.2** adds the **standards-derived edge
profiles** — `right/` leap profile, **TZif v4** (via a crafted leap-`Expires`, version byte `0x34`), `-r`
range truncation (the `-00` unspecified-local-time type), footer extension, pre-first-transition, ~2³¹
boundaries, numeric/unusual abbreviations, non-hour/non-minute offsets. Harness:
`reports/reader-compat/gauntlet-appendix-a.py`; receipt `RECEIPT-2026-06-03-appendix-a.md`.

**Result: 15 microcases × {`zdump`, Python `zoneinfo`} = 30 reader-results → 30 matched · 0 diverged · Go 15
unassessed** — *after a fix*. The expansion **found AND fixed a real defect**: the opt-in `right/` leap
profile left a zone's transition instants at POSIX values, so `right/America/New_York` drifted behind
reference `zic` by the accumulated leap count (`right/Etc/UTC` matched only because it has no transitions —
exactly why T11's `right/UTC` witness missed it). **Fixed** in `apply_leaps` (each transition shifted by the
cumulative leap correction; `right/` only — POSIX/default byte-unchanged, CORE.1 341/0/0) + a transition-bearing
regression test. Python `zoneinfo` is leap-blind (matched both; leap axis exercised by `zdump` only). Tier
non-claim unchanged: these readers/fixtures only; no universal leap-profile theorem; no civil-time truth.

## Reproduce

See `reports/reader-compat/README.md` + `reports/reader-compat/{gauntlet.py, gauntlet-appendix-a.py}` and the
dated receipts. The trees are built by `zic-rs compile --all-supported [-L leapseconds | -r @lo/@hi]` and
`zic -d [-L | -r]` from the same admitted `tzdata.zi`.
