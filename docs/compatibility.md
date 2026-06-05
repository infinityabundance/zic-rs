# Compatibility

Status of each fixture zone against reference `zic` (tzcode 2026b). "Byte" = our output is
identical to reference `zic` byte-for-byte (pinned in `fixtures/expected/`). "Semantic" =
same meaning per `zdump`/decoded comparison. Verified by `cargo test` and `zic-rs compare`.

| Zone | Source | Construct | Semantic | Byte |
|------|--------|-----------|:--------:|:----:|
| `Etc/UTC` | `fixtures/minimal/utc.zi` | fixed offset 0, literal abbr | ✅ | ✅ |
| `UTC` | `fixtures/minimal/utc.zi` | `Link` (copy) | ✅ | ✅¹ |
| `Test/Fixed` | `fixtures/minimal/fixed.zi` | fixed offset −5:00, literal abbr | ✅ | ✅ |
| `Test/Simple` | `fixtures/minimal/dst.zi` | finite DST, `Sun>=8`/`Sun>=1`, wall `AT`, `%s` | ✅ | —² |
| `Test/Euro` | `fixtures/minimal/euro.zi` | finite DST, `lastSun`, UT `AT`, `%s` | ✅ | —² |
| `Test/Sle` | `fixtures/minimal/sle.zi` | finite DST, `Sun<=25`, standard `AT`, `%s` | ✅ | —² |
| `Test/Eastern` | `fixtures/minimal/eastern.zi` | **recurring** DST (`max`), POSIX footer `EST5EDT,M3.2.0,M11.1.0` | ✅³ | —² |
| `Test/FF` | `fixtures/minimal/multi_ff.zi` | **multi-era** fixed→fixed offset change at `UNTIL` | ✅ | ✅ |
| `Test/MidDst` | `fixtures/minimal/multi_mid.zi` | multi-era, era ends **mid-DST**; footer `AST4` | ✅ | —² |
| `Test/FR` | `fixtures/minimal/multi_fr.zi` | multi-era fixed→recurring (final-era footer anchoring) | ✅ | ✅⁴ |
| `Test/Q` | `fixtures/minimal/multi_q.zi` | multi-era, recurring `FROM 2015` → DST from era start 2000 | ✅ | ✅⁴ |
| `Test/Mixed` | `fixtures/minimal/mixed.zi` | **single-era** rule set mixing finite (April onset) + recurring (March onset) rows | ✅⁵ | —² |
| `Test/FinalEffective` | `fixtures/minimal/final_effective.zi` | multi-era final era: **effective-in-era** recurring-only (finite rows pre-era) | ✅ | ✅ |
| `Europe/London` | `fixtures/iana-slices/europe_london_2026b.zi` | **first real IANA slice** (T4.0): 5 eras, LMT→GMT/BST/BDST, EU footer `GMT0BST,M3.5.0/1,M10.5.0` | ✅⁶ | —² |
| `America/New_York` | `fixtures/iana-slices/america_new_york_2026b.zi` | **second real IANA slice** (T4.1): 6 eras, **genuinely mixed-in-era** final era (US finite 1967–2006 + recurring 2007–max), WWII war time; footer `EST5EDT,M3.2.0,M11.1.0` | ✅⁶ | —² |
| `Test/MinRule` | `fixtures/minimal/minrule.zi` | obsolete `FROM=minimum` → coerced to 1900 (T3.2a) | ✅ | —² |
| `Test/InlineLit` | `fixtures/minimal/inline.zi` | **inline-save** era (literal `HKWT`, +8:30, isdst) (T3.2b) | ✅ | ✅⁷ |
| `Test/InlineZ` | `fixtures/minimal/inline.zi` | inline-save era, `%z` → `+0720` from total offset (T3.2b) | ✅ | ✅⁷ |
| `Test/InlineSolo` | `fixtures/minimal/inline.zi` | single-era inline-save (+8:30 `HKWT`) (T3.2b) | ✅ | —² |

¹ The link is materialised as a copy of `Etc/UTC`, so its bytes match the (byte-identical)
target. With `--link-mode symlink` it becomes a relative symlink instead — a local
representation choice, not a semantic difference.

² Rule-driven zones are **behaviour**-match (verified by the `zdump` oracle); we do not claim
byte parity because `zic`'s local-time-type/abbreviation ordering and explicit-transition
horizon differ from ours while the meaning is identical.

³ Verified by the `zdump` behaviour oracle over `2019..2099` — the wide horizon exercises the
recurring POSIX footer well beyond the explicit-transition window.

⁴ `Test/FR`/`Test/Q` happen to be byte-identical because the **final recurring era footer
anchoring** rule emits exactly the same minimal output as reference `zic` (one anchor
transition + the recurring footer). Byte parity is incidental here, not a contract;
multi-era DST zones are validated semantically.

⁵ **Accepted slim/fat structural difference.** Reference `zic` emits a *slim* explicit set
(finite history + one recurring anchor = 39 transitions); zic-rs emits a *fat* set (every year
through `RECUR_HI` = 122). Both produce identical `zdump` behaviour over the finite window
`1976..1997` and share the footer — see the slim/fat note in
[reference-zic-semantics.md](reference-zic-semantics.md). Behaviour is the contract; the
explicit-transition representation is not.

⁶ **T4.0 — first real IANA-zone slice.** Verified by the `zdump` behaviour oracle over
`1830..2045`. The slice is the canonical (record-keys-only de-abbreviated) Europe/London from
tzdb 2026b; reference `zic` compiles it **byte-identically** to the unmodified abbreviated
`tzdata.zi` extract (faithfulness proof). zic-rs's explicit-transition count (159) differs from
`zic`'s by the same slim/fat orthogonality; behaviour matches across the full horizon.

⁷ `Test/InlineLit`/`Test/InlineZ` are **ttinfo byte-identical** to reference `zic` (the
local-time-type table — utoff/isdst/abbreviation — matches exactly), and `zdump`-identical over
their horizons. Byte parity of the whole file is still not a general contract.

### `Europe/London` — PASS (T4.0)

* **Source:** canonical de-abbreviated slice derived from `tzdata.zi` 2026b (record keys
  `R`/`Z`/`L` expanded; month/weekday/year prefixes left in `zic`-style form).
* **Faithfulness:** reference `zic` output is byte-identical to the abbreviated extract.
* **Oracle:** `zdump` behaviour match over `1830..2045`.
* **TZif:** v2; 5 local-time-types (`LMT`, `GMT`, `BST` dst, `BDST` double-dst, `BST` fixed);
  159 transitions; footer `GMT0BST,M3.5.0/1,M10.5.0`.
* **Why it became admissible:** the final EU era starts 1996 while Rule E's finite rows end in
  1995, so the *effective in-era* rule set is recurring-only (the
  [effective-in-era classification](reference-zic-semantics.md) law).
* **Source form (T4.0.1):** `Europe/London` is verified both from the **canonical** slice and
  from the **abbreviated** `tzdata.zi`-style extract — zic-rs now parses zishrink record keys
  (`R`/`Z`/`L`) directly, and produces byte-identical output for both forms. The canonical
  fixture is retained for readability/provenance; it is no longer *required* to get past the
  parser. zic-rs can also read `Europe/London` straight from the installed
  `/usr/share/zoneinfo/tzdata.zi`.

## Whole-database frontier (`support-report`) — compile-clean ≠ behaviour-verified

The rows above are individually **behaviour-verified** (each passes the `zdump` oracle over a
declared horizon). For the *whole* installed `tzdata.zi` (2026b), `zic-rs support-report` reports
a coarser, honest **compile** frontier:

* **338 / 341** canonical zones are **compile-clean** (zic-rs emits a valid TZif); **594 / 598**
  identifiers including links. Only 3 fail closed: `Europe/Prague` (negative inline SAVE),
  `Asia/Gaza` / `Asia/Hebron` (non-POSIX recurring day form).
* **Compile-clean is *not* behaviour-verified** (still tracked separately, even now that they
  coincide). A **comprehensive** `zdump` oracle sweep over **all 341** canonical zones
  (`1900..2040`, every zone compiled and diffed against reference `zic`) reports **341 match / 0
  mismatch / 0 fail-closed** (341 + 0 = 341 compile-clean) — *after T5 #1–#5 + law 7 + law 10*.
  **Every canonical zone behaviour-matches reference `zic` over `1900..2040`** — the canonical-zone
  behaviour frontier is closed. The match count rose **264 → 328 → 333 → 338 → 339 → 341** as the
  classes were fixed (an earlier 35-zone *sample* "28 / 6" caught only 6 of the original 74 — retired):
  **T5 #1** standard-`LETTER`, **#2** final-recurring anchor, **#3** standard-clock absorption (−64),
  **#4** prior-active rule-state seeding (−5), **#5** clock-reference normalization (−5), **law 7**
  inline negative SAVE (−1: Prague), **law 10** non-POSIX `ON` + **v3 footer** (−2: Gaza/Hebron, e.g.
  `Sat<=30` → `M3.4.4/50`). No canonical zone fails closed. (The remaining work is the **operational shell (T9–T15)**
  operational shell** — leap/`right`/v4, CLI/install, `-r`/`-R`/`-b`, warning parity — *not* a
  canonical-zone behaviour gap; **not** full-`zic`.) Full catalogue in
  [reference-zic-semantics.md](reference-zic-semantics.md) §10.

A zone is admitted to the **table above** (a behaviour-verified slice) only once it passes the
oracle and carries a receipt; "compile-clean" alone never earns a row. See
[conformance-ladder.md](conformance-ladder.md) for the admission gate and the status vocabulary.

## Not yet on this table

Inline-save eras with a `%s`/slash `FORMAT`, `24:00`/negative compiled times, recurring rules whose
`ON` is a *fixed numeric* day (no weekday), leap seconds, and further IANA slices are **not yet
supported** and fail closed today. (Negative inline SAVE and `Sun<=N`/`Sat<=N` recurring `ON` forms
are **now supported** — laws 7 and 10.) They join this table as the remaining
breadth (T3.2) and IANA-slice expansion (T4) land — each row added only once it passes the
oracle. We do not claim compatibility for any zone without a corresponding passing comparison.
