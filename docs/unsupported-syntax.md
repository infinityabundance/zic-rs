# Unsupported and deferred syntax

> **No canonical zone in `tzdata.zi` 2026b currently fails closed** under the `1900..2040`
> behaviour sweep (CORE.1: 341/341 behaviour-match, 0 fail-closed). This document therefore now
> records two distinct things: **(a)** the *formerly* unsupported canonical blockers and how they
> were cleared (kept for the audit trail), and **(b)** the constructs and full-`zic` operational
> modes that remain **deferred** — none of which any 2026b canonical zone needs. It is a "what is
> still refused, and why" map, not a list of present canonical-zone gaps.

`zic-rs` **fails closed**: when it meets a construct it cannot compile *correctly*, it emits
an explicit diagnostic and refuses to write output for that zone (under the default
`--unsupported error` policy) rather than producing an approximate file. `--unsupported
skip` downgrades this to a warning and skips the zone.

## Remaining canonical-zone blockers (`tzdata.zi` 2026b)

**None.** All **341 / 341** canonical zones compile *and* behaviour-match reference `zic` over
`1900..2040` — the canonical-zone behaviour frontier is closed. The two former blockers were both
cleared:

| Former bucket | Zones | Resolution |
|---------------|-------|------------|
| **negative SAVE** | `Europe/Prague` | ✅ first-class **signed SAVE** (law 7): effective offset `STDOFF + SAVE`, `is_dst = save ≠ 0`, matching reference `zic` (negative-save *Rules*, e.g. Morocco, already worked). |
| **non-POSIX day form** | `Asia/Gaza`, `Asia/Hebron` | ✅ **law 10** + content-driven **v3 footer**: a recurring `Sat<=30`-style `ON` form is re-anchored onto a clean nth-weekday with the skipped days folded into the transition time (`Sat<=30 02:00` → `M3.4.4/50`), exactly as reference `zic`. |

The fail-closed mechanism remains in force for genuinely unrepresentable constructs (it is a
deliberate refusal — no approximate output — surfaced as a `ZIC001` diagnostic and bucketed by
`support-report`); there simply are no canonical zones that hit it in 2026b.

## Not yet supported

| Construct | Diagnostic | Planned |
|-----------|------------|---------|
| Inline DST saving (`RULES` = clock value) with a `%s` or `STD/DST` slash `FORMAT` (a **negative** inline save is now supported — law 7) | `ZIC001_UNSUPPORTED_DIRECTIVE` | when a real zone forces it |
| Recurring rule whose `ON` is a **fixed numeric** day (no weekday → no `Mm.w.d` footer; `Sun<=N`/`Sat<=N` weekday forms are now supported — law 10) | `ZIC001_UNSUPPORTED_DIRECTIVE` | later |
| `24:00`+ / negative `AT` times in compiled output (content-driven v3) | (parsed; compile path later) | T3.2 |

> **Mixed finite+recurring final eras — fully supported.** Both shapes work: (a) a rule set
> "mixed" only by *raw membership* (finite rows that all end **before** the era starts +
> recurring rows) is classified by *effective in-era* activations and treated as recurring-only
> (e.g. `Europe/London`); (b) a genuinely *mixed-in-era* final era (finite rows still firing
> inside the era + recurring rows) expands the finite history explicitly and projects the
> recurring tail via the footer (e.g. `America/New_York`, Rule US finite 1967..2006 + recurring
> 2007..max). See [reference-zic-semantics.md](reference-zic-semantics.md) §5.

Supported (no longer rejected): named **finite** and **recurring**
(`TO = maximum`) rule sets with their POSIX footer; **multi-era zones** (`UNTIL`
continuations); both **mixed finite+recurring** final-era shapes (effectively recurring-only
*and* genuinely mixed-in-era); **inline-save eras** (`RULES` = a clock value) with literal or
`%z` `FORMAT`;
`%s`/`%z`/`STD/DST` formats; the `ON` day forms `lastSun`/`Sun>=N`/`Sun<=N`; the first real IANA
slice (`Europe/London`); the **zishrink** record keys (`R`/`Z`/`L`) of the installed
`tzdata.zi`, which now parse directly; and the obsolete `FROM = minimum` spelling (coerced to
1900, as reference `zic`). See [supported-syntax.md](supported-syntax.md) and the behaviour
ledger [reference-zic-semantics.md](reference-zic-semantics.md).

## Deferred `zic` options / features

These reference-`zic` capabilities are intentionally out of scope for v0.1 and documented as
unsupported rather than silently ignored:

| Option / feature | Status |
|------------------|--------|
| `-L` leap-second input | Not implemented (`ZIC010` reserved) |
| `-u owner[:group]` | Not implemented (no privileged ownership) |
| `-m mode` | Not implemented |
| `-p posixrules` | **Intentionally not implemented** — legacy; reference `zic` itself warns *"-p is obsolete and likely ineffective"*. Never a zone failure (T9.4). |
| `-r` time-range truncation | Not implemented (→ T10) |

Implemented since this list was first written (kept here only as a pointer):

| Option / feature | Now available as |
|------------------|------------------|
| `-b slim` / `-b fat` emission style | `--emit-style {zic-slim\|zic-fat\|default}` (T8-slim; default is behaviour-matched fat-style) |
| `-l` / `-t` localtime links | `--localtime <zone>` / `--localtime-name <name>` — within `--out` only (T9.4) |

The set shrinks as milestones land; see [roadmap.md](roadmap.md). The authoritative,
machine-checkable statement of *current* support is `zic-rs supported-syntax` and the test
suite.
