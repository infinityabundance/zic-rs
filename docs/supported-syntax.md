# Supported syntax (current declared subset)

This is the **exact** subset `zic-rs` compiles today. Anything not listed here is rejected
with a diagnostic (see [unsupported-syntax.md](unsupported-syntax.md)); it is never
approximated. The authoritative machine-checkable statement is `zic-rs supported-syntax`
plus the test suite.

## Lexical (fully supported)

* `#` comments to end of line (ignored inside double quotes);
* double-quoted fields (whitespace and `#` literal inside);
* blank-line skipping; whitespace field separators (space, tab, CR, form-feed, VT);
* 2048-byte maximum line length (including newline) — over-length lines rejected;
* NUL bytes rejected; input must be valid UTF-8.

## Records

### `Zone NAME STDOFF RULES FORMAT [UNTIL]` (single **or** multi-era)

Multi-era zones (continuation lines, each ending at an `UNTIL`) are supported: the era walker
carries cross-era state and converts each `UNTIL` in the *ending* era's context (with the
prevailing save). The final era owns the footer. See the **"final recurring era footer
anchoring"** rule and the other pinned behaviours in
[reference-zic-semantics.md](reference-zic-semantics.md). Each era is:


* `STDOFF`: `-`, or `[-+]h`, `[-+]h:mm`, `[-+]h:mm:ss`; fractional seconds rounded to nearest.
* `RULES`:
  * `-` → **fixed-offset** zone (T1);
  * a clock value (e.g. `1:00`) → **inline-save** era (T3.2b — see the section below);
  * a **named rule set** → a rule-driven zone: **finite** (concrete `FROM`/`TO`, T2) or
    **recurring** (`TO = maximum`, T3 — see the footer note below).
* `FORMAT`: literal, `%s` (LETTER substitution), `STD/DST` slash form, or `%z` (numeric UT
  offset). `%s` and the slash form need rule context (a LETTER / DST flag), so on a **no-rules**
  era (`RULES = -`) they fail closed; **`%z` works on a no-rules era** — it renders the era's
  numeric standard offset (`0:30 - %z` → `+0030`), exactly as reference `zic`.

### `Rule NAME FROM TO - IN ON AT SAVE LETTER`

Fully parsed and, for finite sets referenced by a compiled zone, expanded into transitions:

* `FROM`/`TO`: concrete years, or `only` (= `FROM`); `TO = maximum` is supported (recurring
  rule → POSIX footer); `FROM = minimum` is the obsolete spelling, coerced to 1900 like
  reference `zic` (see the section below). Year keywords match as unambiguous prefixes
  (`o`/`only`, `mi`/`minimum`, `ma`/`maximum`; bare `m` is ambiguous → rejected).
* `IN`: month name (unambiguous prefix, case-insensitive).
* `ON`: numeric day, `lastSun`..`lastSat`, `Sun>=N`, `Sun<=N` (with month spill).
* `AT`: time with suffix `w` (wall, default), `s` (standard), `u`/`g`/`z` (universal).
* `SAVE`: amount with suffix `s`/`d` (sign honoured; `0`→standard, non-zero→daylight by
  default).
* `LETTER`: the `%s` substitution (`-` → empty).

### `Link TARGET LINK-NAME`

* Materialised by copy (default) or symlink (`--link-mode symlink`).
* `--zone <alias>` resolves a link alias to its canonical zone and writes the alias.
* Multi-hop chains are resolved to the canonical zone (order-independent).

## Transition semantics (T2)

* Rules expanded across their concrete year span; each `AT` converted to a UT instant using
  the **prevailing offset just before** the transition (wall) / `STDOFF` (standard) / as-is
  (universal).
* Local-time-types de-duplicated; transitions strictly increasing.
* A finite rule set leaves the zone in its last state permanently → a **fixed** POSIX footer
  (e.g. `Test/Simple` → `EST5`).
* A **recurring** (`TO = maximum`) rule set is summarised by a **recurring POSIX footer**
  (e.g. `Test/Eastern` → `EST5EDT,M3.2.0,M11.1.0`), synthesised *exactly* — the DST offset
  is omitted when it is the default +1h, `/time` is omitted when it is the default `02:00`,
  `lastSun` → "last", and **both** `Sun>=N` *and* `Sun<=N`/`Sat<=N` are expressed: `zic`
  re-anchors a `weekday>=N`/`weekday<=N` form onto a clean nth-weekday and folds the skipped days
  into the transition time, which can exceed 24h and so triggers a **content-driven v3** footer
  (e.g. `Asia/Gaza`'s `Sat<=30 02:00` → `M3.4.4/50`). A *fixed numeric* `ON` day (no weekday) is
  the only recurring form that still **fails closed** (no `Mm.w.d` form). Explicit transitions are
  emitted through `RECUR_HI` (or the last finite one-shot rule year, whichever is later); the footer
  governs beyond.

## Output

* Valid **TZif**, **content-driven version**: v1 (32-bit) stub block + v2 (64-bit) block + POSIX
  `TZ` footer, big-endian, per RFC 9636. The version byte is **`2`** by default and **`3`** only when
  the footer needs the v3 extension (a transition time strictly outside `0..=24h`, e.g. `Asia/Gaza`'s
  `M3.4.4/50`) — never v3 gratuitously. See [tzif-notes.md](tzif-notes.md) and, for how our bytes
  compare to reference `zic` structurally, [structural-parity.md](structural-parity.md).

## Verified parity

* `Etc/UTC`, `Test/Fixed` — **byte-identical** to reference `zic` (pinned blobs).
* `Test/Simple` (finite US-style DST), `Test/Euro` (`lastSun`, UT-referenced `AT`),
  `Test/Sle` (`Sun<=25`, standard-time `AT`) — match reference `zic` under the **`zdump`
  behaviour oracle** over `2019..2022`.
* `Test/Eastern` (recurring US-Eastern DST) — matches under the `zdump` behaviour oracle over
  `2019..2099` (the wide horizon proves the recurring footer, not just the explicit
  transitions); footer `EST5EDT,M3.2.0,M11.1.0`.
* `Test/Mixed` / `Test/FinalEffective` — a rule set mixing finite + recurring rows where the
  final era's **effective in-era** activations are recurring-only (finite rows predate the era
  start). Matches under the `zdump` oracle; see the effective-in-era law in
  [reference-zic-semantics.md](reference-zic-semantics.md).
* **`Europe/London`** — the **first real IANA-zone slice** (T4.0), a canonical
  (record-keys-only de-abbreviated) slice of `tzdata.zi` 2026b. Matches under the `zdump`
  behaviour oracle over `1830..2045`; footer `GMT0BST,M3.5.0/1,M10.5.0`. See
  [compatibility.md](compatibility.md).

DST/recurring output is **behaviour-match** only: type/designation ordering and the
explicit-transition horizon (slim vs fat) may differ from `zic`, so byte-parity is not claimed
for rule-driven zones (it stays pinned to the fixed-offset blobs).

### Inline-save eras (`RULES` = a clock value)

A `Zone` era whose `RULES` column is a clock value (e.g. `0:30`) rather than `-` or a rule name
is an **inline-saving** era — a constant DST offset for that era's whole span (real tzdata: Hong
Kong / Jakarta wartime). It compiles to one fixed local-time type: `utoff = STDOFF + SAVE`,
`is_dst` set from the parsed save (`s`/`d` suffix honoured; no suffix → `seconds != 0`), and the
abbreviation rendered from `FORMAT` — **literal** (e.g. `HKWT`) or **`%z`**, where `%z` uses the
**total** effective offset (so `7:00 0:20 %z` → `+0720`). Verified against reference `zic`
(ttinfo byte-identical) for `Test/InlineLit`, `Test/InlineZ`, `Test/InlineSolo`. **`SAVE` is signed:
a *negative* inline save is supported** — the effective offset `STDOFF + SAVE` can be ≤ the standard
offset (e.g. `Europe/Prague`'s `1 -1 GMT` → `GMT`, isdst=1, gmtoff 0; law 7, pinned vs reference
`zic`). **Fail closed** (not yet pinned): inline save with a `%s` `FORMAT` (no LETTER) or the
`STD/DST` slash form.

### `FROM = minimum` — obsolete compatibility spelling

`FROM = minimum` is accepted as an **obsolete** compatibility spelling. Following reference
`zic`, zic-rs treats it as the year **1900** — *not* an infinite-past lower bound. (Current
tzdata 2026b does not use it at all; this is compatibility breadth, not a real-data feature.)
The year keywords are matched as `zic`-style unambiguous prefixes: `o`/`only`, `mi`/`minimum`,
`ma`/`maximum`; a bare `m` is ambiguous and rejected. Reference `zic` also prints a non-fatal
"minimum obsolete" warning; surfacing that during zic-rs compilation is a tracked follow-up.

### `tzdata.zi` (zishrink) source syntax — supported

The installed `/usr/share/zoneinfo/tzdata.zi` is the **zishrink-compressed** single-file form:
record keywords are abbreviated to `R`/`Z`/`L`, and month/weekday/year tokens to `zic`-style
prefixes (`O`, `Ap`, `lastSu`, `o`, `ma`). zic-rs reads it **directly** — record keywords match
as unambiguous prefixes of `Rule`/`Zone`/`Link` (the same way month/weekday names already do;
`zic`'s zone-file keyword table is exactly those three, so `L` is unambiguously `Link`). This
means a pinned IANA fixture can be either the verbatim abbreviated extract *or* a
record-keys-expanded canonical slice — zic-rs produces identical output for both (asserted by
`zic_rs_compiles_abbreviated_and_canonical_london_identically`). Leap-second / `Expires` lines
remain unrecognised in a zone file, matching reference `zic`.
