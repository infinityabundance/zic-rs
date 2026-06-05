# TZif writer notes

The writer targets [RFC 9636](https://www.rfc-editor.org/rfc/rfc9636) (the standards-track
TZif format, October 2024) and matches the on-disk output of reference `zic` (tzcode 2026b).

## File shape

A version-2+ TZif file is three parts, back to back:

```
┌─────────────────────────┐
│ v1 block  (32-bit times) │  44-byte header + data block
├─────────────────────────┤
│ v2+ block (64-bit times) │  44-byte header + data block
├─────────────────────────┤
│ footer  \n <POSIX TZ> \n │
└─────────────────────────┘
```

All integers are **big-endian**.

### Header (44 bytes)

`TZif` magic (4) · version byte (1) · 15 reserved NUL · then six `u32` counts in order:
`isutcnt`, `isstdcnt`, `leapcnt`, `timecnt`, `typecnt`, `charcnt`.

### Data block (seven arrays, in order)

1. transition times — `timecnt` × `time_size` (4 or 8 bytes), strictly ascending
2. transition type indices — `timecnt` × 1 byte
3. local-time-type records — `typecnt` × 6 bytes (`utoff:i32`, `isdst:u8`, `desigidx:u8`)
4. designation table — `charcnt` bytes (NUL-terminated abbreviations, may be shared)
5. leap-second records — `leapcnt` × (`time_size`+4)  [none emitted in T1]
6. standard/wall indicators — `isstdcnt` bytes        [none emitted in T1]
7. UT/local indicators — `isutcnt` bytes              [none emitted in T1]

## The slim v1 stub (important and surprising)

Modern `zic`'s default "slim" output writes the **v1 block as a near-empty stub**: zero
transitions and a single placeholder local-time-type with `utoff = 0` and an empty
abbreviation (`timecnt=0, typecnt=1, charcnt=1`). All real information lives in the v2+
block and the footer. We reproduce this **exactly** — it is what we byte-match — even though
it means a strictly v1-only reader sees UT for every zone. That is `zic`'s behaviour, not a
bug we introduced; v1-only readers are long obsolete.

`slim` vs `fat` is **orthogonal to the version byte**: it governs how many explicit transitions are
written versus relying on the POSIX footer for the open-ended future. zic-rs's **default** emission is
*fat-style* (explicit transitions through `RECUR_HI`) — the behaviour-matched, CORE.1-gated output;
**`--emit-style zic-slim`** reproduces reference `zic`'s slim set by truncating the footer-governed
recurring tail (campaign T8-slim — see [structural-parity.md](structural-parity.md) §1). Both are
`zdump`-equivalent (the footer reproduces the dropped transitions); byte parity vs reference `zic` is
claimed only in `zic-slim` mode.

## Version policy (content-driven)

The version byte is chosen by content, not stamped blindly. We follow `zic.c` exactly (tzcode
2026b, `outzone`): `version = compat < 2013 ? '2' : '3'`, where `compat` is the TZDB-release year
the synthesised footer needs. Concretely:

* `2` — the normal case (fixed-offset zones, and DST footers whose `Mm.w.d` rules are clean
  nth/last-weekday with an in-range transition time);
* `3` — when the footer needs the v3 extension. The precise trigger, from `zic.c`'s `stringrule`
  (`compat = 2013`), is **a `weekday>=N` / `weekday<=N` `ON` form re-anchored onto a clean
  nth-weekday with a non-zero day-shift** (`wdayoff != 0`), **or a negative folded transition
  time**. A folded transition time `>= 24h` *alone* is only `compat = 1994` → **still `2`** (e.g.
  `Africa/Cairo`'s literal `lastThu 24:00` → `/24`). So the day-shift, not the displayed time's
  range, is the real v3 trigger — see [structural-parity.md](structural-parity.md) §3 and
  `compile::posix_footer::recurring`;
* `4` — only for truncated/expired leap tables (out of scope; leap modes are a future campaign).

We never label a file `3` gratuitously, because readers gate behaviour on the version byte.

## Footer (POSIX `TZ`)

The footer is `\n`, a POSIX `TZ` string, `\n`. It governs instants after the last explicit
transition. **The offset sign is inverted** relative to TZif's `tt_utoff`: a zone at UT−5
(`utoff = -18000`) has POSIX offset `+5`, written `EST5`. Our policy is **exact-or-nothing**:
we synthesise a footer only for shapes we can represent precisely, never an approximation —
otherwise the zone fails closed. Supported shapes:

* **constant offset** (fixed-offset zones, finite rule tails) → `STD<offset>` (`UTC0`,
  `EST5`);
* **recurring DST** (`TO = maximum` rule sets) → the full recurring rule
  (`EST5EDT,M3.2.0,M11.1.0`), with the default +1h DST offset and the default `/02:00`
  transition time omitted exactly as `zic` does.

See `compile::posix_footer`.

**The footer is load-bearing but not prophetic.** It describes *projected* future behaviour
according to the current tzdb rule model — it is not a guarantee of future civil-time law.
Timezone rules are political and change; `zic-rs` (like `zic`) compiles the current data
deterministically and does not predict policy. The far-future tail a reader derives from the
footer is only as good as the rules in the pinned tzdata release.

## Validation

`tzif::validate::parse` decodes a file back to semantics. It is used both to round-trip our
own output in tests and to decode reference `zic` output for the oracle, so both sides of a
comparison are read by identical code.
