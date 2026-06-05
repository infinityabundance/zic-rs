# Deep `zic` semantics tracked by zic-rs

This is the **named-law index**: every obscure `zic` behaviour that can change emitted output is
named here, tied to a reference source, and marked with its enforcement status (a test, a fail-closed
bucket, or a declared future campaign). The standard is deliberately strict —

> *Every obscure `zic` semantic that can change emitted behaviour must be named, tested, bucketed,
> and tied to an admitted zone, a fail-closed zone, or a future campaign.*

— because that is what makes a timezone compiler **boring and reliable** rather than "works on the
easy zones." Most of these are not in any tutorial; they are learned by being burned in production.
The per-zone evidence for each fix lives in [reference-zic-semantics.md](reference-zic-semantics.md)
(the behaviour ledger); this file is the *map* over those traps. The companion
[zic-v-hazards.md](zic-v-hazards.md) tracks the `zic -v` verbose-warning hazards.

## Authority stack

```
IANA tzdb source  →  reference zic / zdump  →  RFC 9636 (TZif validity)  →  zic-rs + oracle evidence
```

Wikipedia and articles are orientation only. Every subtle behaviour below is **pinned by experiment**
against reference `zic`/`zdump` (tzcode 2026b) over a declared horizon, never inferred from prose.
Normative sources: `zic(8)` / `zic.c` (grammar + semantics), RFC 9636 (TZif binary), RFC 6557 /
BCP 175 (governance). See [standards.md](standards.md) and [tzdb-governance.md](tzdb-governance.md).

## The state-model invariant (do not collapse into "offset")

A `Zone` is **not** a DST table; it is a **state machine over eras**. Almost every behaviour bug in
this project came from collapsing three distinct things into one. They are tracked separately and
must stay that way:

| Concept | What it is | Bounded by |
|---------|------------|------------|
| **era state** | which `Zone` continuation line is in force: its `STDOFF`, `RULES` column, `FORMAT`, `UNTIL`-context | era boundaries |
| **rule state** | the *prevailing* `(save, is_dst_semantics, letter, source_rule)` of a **named rule set**, evolving along that rule set's own timeline | rule activations |
| **local-time type** | the emitted TZif `ttinfo`: `(utoff = stdoff + save, is_dst, abbreviation)` — a *function* of era+rule state, never a substitute | a transition |

Two local-time types can share a `utoff` yet differ in `is_dst`/abbreviation (e.g. MSK-std vs
EEST-dst, both +3; EDT vs AST, both −4) — a **real** type change, never deduplicated.

## Scope honesty (permanent)

* **compile-clean ≠ behaviour-match.** A valid TZif is a *compile* claim; `zdump`-equivalence over a
  declared horizon is the *behaviour* claim. They are reported separately even when the counts
  coincide. As of `tzdata.zi` 2026b: **341/341 compile-clean; 341/341 behaviour-match reference `zic`
  over `1900..2040`; 0 fail-closed** — the entire canonical-zone behaviour frontier is closed.
* **Not full `zic`.** No leap-second modes, CLI policy, `right/`/`posix/`, backzone/rearguard,
  warning parity, or slim emission heuristics. zic-rs is a reference-verified *producer* for the
  supported canonical-zone subset over a declared horizon.
* **Pre-1970 humility.** A passing historical horizon proves *agreement with reference `zic`/tzdb for
  that named zone over that horizon* — not regional civil-time truth (law 17).

---

## Tracked laws

Each law: the trap, its **Reference**, **zic-rs status**, guard **Tests**, **Known zones**, and any
**Open bucket** (a fail-closed reason or future campaign).

### 1. A `Zone` is a state machine over eras
A zone is a sequence of eras (continuation lines), each compiled with state carried from the previous
era — not a line-by-line table conversion.
*Reference:* `zic(8)` zone/continuation grammar. *zic-rs status:* ✅ `compile_multi_era`
(`src/compile/transitions.rs`). *Tests:* the whole `tests/multi_era.rs` suite. *Known zones:* every
multi-era zone (London, New_York, …). *Open bucket:* —.

### 2. `UNTIL` is outgoing-era state, not an absolute literal
A `Zone` `UNTIL` is interpreted using the rules **in effect just before** the transition — the
*ending* era's `STDOFF` and the `save` prevailing at the `UNTIL`, plus its w/s/u reference. A wall
`UNTIL` during active DST lands one save earlier than naive.
*Reference:* `zic(8)` ("the UNTIL … is interpreted using the rules in effect just before the
transition"). *zic-rs status:* ✅ `convert_at(ul, stdoff, run.save, uref)` at the era-end.
*Tests:* `until_wall_time_uses_prevailing_save_when_dst_active` (`Test/MidDst`). *Open bucket:* —.

### 3. `Rule AT` / `Zone UNTIL` use wall / standard / universal frames
`-` = wall, `s` = standard, `u`/`g`/`z` = universal. The line describes the instant a clock *set to
that reference* would show. **Never compare two as raw local seconds without normalizing to a common
frame.**
*Reference:* `zic(8)` (AT/UNTIL suffixes). *zic-rs status:* ✅ T5 #5 — the era-end break compares
resolved UT; boundary-coincidence normalizes refs under the prevailing save (wall ≡ standard at
save 0). *Tests:* `central_asia_offset_drop_boundaries_normalize_clock_reference`,
`europe_mixed_reference_era_end_breaks_match_reference_zic`,
`era_boundary_normalizes_wall_standard_clock_reference`. *Known zones:* Tashkent, Ashgabat, Anadyr,
Simferopol, Warsaw. *Open bucket:* —.

### 4. Era-boundary ↔ rule-transition coincidences collapse to **one** transition ("Menominee law")
When a continuation changes the UT offset and a rule would otherwise fire in the next *N* seconds,
`zic` reads the boundary and the activation as **simultaneous** — one transition, not two. Absorption
uses **exact clock-frame equality, never a proximity tolerance**.
*Reference:* `zic(8)` (the `America/Menominee` example). *zic-rs status:* ✅ T5 #2/#3/#5 —
`boundary_until` + exact normalized clock-frame equality. *Tests:*
`russia_boundary_offset_drop_does_not_create_spurious_standard_hour`,
`europe_lisbon_final_era_boundary_coincident_dst_matches_reference_zic`. *Known zones:* Moscow,
Volgograd, Novosibirsk, Lisbon, St_Johns. *Open bucket:* —.

### 5. A named rule set has its own timeline across eras
A rule set's `(save, is_dst, letter)` persists until the rule *itself* changes it. An intervening era
that uses `-` or a different rule does **not** reset it; a later era reusing the rule seeds from its
most-recent activation at the era start (even one years earlier, outside the local window).
*Reference:* `zic(8)` rule semantics; pinned by experiment. *zic-rs status:* ✅ T5 #4 — seed scans
the rule's actual `FROM` range. *Tests:* `america_phoenix_1944_inherits_war_time_matches_reference_zic`,
`atlantic_bermuda_1930_inherits_standard_letter_matches_reference_zic`,
`asia_manila_and_europe_brussels_inherit_prior_dst_match_reference_zic`. *Known zones:* Phoenix (War
from 1942), Bermuda (letter S from 1918), Manila, Brussels, Macau. *Open bucket:* —.

### 6. `LETTER/S` is temporal rule state
`%s` renders the *active* rule's `LETTER/S`; `-` means the variable part is null. A **`SAVE=0` rule
can change it** (a standard-letter change). Timestamps before the earliest rule use the earliest
*standard*-time rule's letter.
*Reference:* `zic(8)` (LETTER/S; pre-first-rule letter). *zic-rs status:* ✅ T5 #1 — a
boundary-coincident `SAVE=0` activation seeds the boundary's letter. *Tests:*
`pacific_auckland_1946_changes_nzmt_to_nzst_and_matches_reference_zic`. *Known zones:* Auckland
(NZMT→NZST). *Open bucket:* —.

### 7. `SAVE` is signed state, not a DST boolean
`effective_offset = STDOFF + SAVE`; the suffix carries standard/daylight semantics; **negative SAVE
is valid** (Ireland is the canonical example). `zic` does not distinguish equivalent sums
(`10:30 + 0:30` ≡ `10:00 + 1:00`).
*Reference:* `zic(8)` (SAVE field; negative saves). *zic-rs status:* ✅ **done** — inline SAVE is
signed; a *negative* inline save renders the signed effective offset with `is_dst = (save ≠ 0)`,
exactly as reference `zic` does (implemented as first-class signed SAVE — negative-save *Rules*
already worked, e.g. Morocco; the inline guard was lifted). `%s`/slash inline still fail closed.
*Tests:* `inline_save_negative_renders_signed_effective_offset`,
`negative_inline_save_is_signed_state_matches_reference_zic`. *Known zones:* `Europe/Prague`
(`1 -1 GMT` → GMT, isdst=1, gmtoff 0). *Open bucket:* — (cleared; was the Prague fail-closed bucket).

### 8. `RULES` may be a named set, `-`, or an inline `SAVE` value
Inline SAVE constructs a **fixed** local-time type from `STDOFF + SAVE` (its own path) — not a lesser
rule set, and no named-rule timeline is consulted.
*Reference:* `zic(8)` ("RULES … can be a field in the same format as a rule SAVE"). *zic-rs status:*
✅ T3.2b inline-save. *Tests:* `inline_save_percent_z_uses_effective_offset` (`Test/InlineZ`),
`Test/InlineLit`. *Open bucket:* inline-save `%s` / slash (law 9).

### 9. `%s`, `%z`, and `STD/DST` slash are three distinct `FORMAT` paths
`%z` renders the **effective** UT offset (`STDOFF + SAVE`) in shortest lossless form (`±hh`/`±hhmm`/
`±hhmmss`); `%s` substitutes the active LETTER; slash **chooses** between two explicit names. Do not
infer one from another.
*Reference:* `zic(8)` (FORMAT); RFC 9636. *zic-rs status:* ✅ `%z` (incl. no-rules era + inline
save) and `%s` and slash on ruled eras; **inline-save `%s`/slash fail closed**. *Tests:*
`inline_save_percent_z_uses_effective_offset`. *Open bucket:* inline-save `%s`/slash (future).
*Note:* the `%z` unlock took compile-clean 162→338 — but a later sweep proved *compile-clean after
`%z` is not behaviour-verified* (law: scope honesty). Both numbers are reported separately.

### 10. `ON` day forms can leave the nominal month → v3 footer
`Sun>=31` may resolve into the next month, and `Sat<=30` is a `weekday <= N` form. `zic` expresses
these in a recurring footer by **re-anchoring onto a clean nth-weekday and folding the skipped days
into the transition time** — which can exceed 24h and needs the **v3 footer** extension (RFC 9636).
E.g. `Sat<=30 02:00` → `M3.4.4/50` (4th Thursday + 50h). The exact `zic` rule (verified): for
`W <= N` (not last-day), `wdayoff = N mod 7`, `d = W − wdayoff`, `week = N/7`, `+wdayoff days`;
`>= N` is symmetric (`wdayoff = (N−1) mod 7`, `week = 1+(N−1)/7`), reducing to the plain form at
`wdayoff = 0`. *Reference:* `zic(8)` / `zic.c` `stringrule`; RFC 9636 (v3). *zic-rs status:* ✅
**done** — `posix_footer::date_rule` + the content-driven v3 (`recurring` emits `b'3'` for an
out-of-`0..24h` time); the explicit horizon also extends to the last finite (one-shot) rule year so
Ramadan-dated rows are emitted explicitly while the footer covers the perpetual tail. *Tests:*
`recurring_sat_leq_30_uses_v3_extended_time_footer`, `recurring_sun_leq_25_uses_extended_v3_footer`,
`neighboring_month_on_form_matches_reference_zic`. *Known zones:* `Asia/Gaza`, `Asia/Hebron` (now
compile **v3** and zdump-match). *Open bucket:* — (cleared → 341/341).

### 11. Extended `AT` times need calendar carry + a version decision
`AT` may be `24:00`, `>24:00` (`260:00`), negative (`-2:30`), or fractional (rounded to the nearest
second, **ties to even**). These interact with calendar carry across day/month/year and with the
TZif-version footer decision (law 12).
*Reference:* `zic(8)` (AT range; fractional rounding). *zic-rs status:* parsed; **compile path for
`24:00`/negative not yet pinned** (fails closed). *Open bucket:* extended-AT + content-driven v3
(future campaign).

### 12. TZif version is behaviour-driven, not syntax-driven
Emit the **lowest** version that correctly represents the compiled behaviour. v3 only when the
*footer* needs the extension (e.g. signed transition hours −167…167) — never because source syntax
"looked weird."
*Reference:* RFC 9636 (version selection; v3 footer extensions). *zic-rs status:* ✅ content-driven
v2 today (no v3 content in scope). *Tests:* covered by the byte-pinned fixtures + the writer. *Open
bucket:* v3 trigger arrives with law 11.

### 13. The POSIX footer is the post-tail behaviour contract
For TZif v2+, the footer computes local time **after** the last explicit transition and must be
consistent with that last transition. Not decorative.
*Reference:* RFC 9636 (footer). *zic-rs status:* ✅ exact-or-fail recurring + fixed footers;
final-era anchoring; projection verified past `RECUR_HI`. *Tests:*
`footer_drives_post_tail_behavior` (`Test/Eastern`), the `ecosystem-tests` `tz-rs` bench. *Known
zones:* every recurring zone (London EU footer, US footer). *Open bucket:* —.

### 14. Slim/fat is structural, not behavioural
Reference `zic` (slim) may emit fewer explicit transitions and lean on the footer; zic-rs emits a fat
set. If `zdump` behaviour + footer match, they are equivalent. **Byte parity is claimed only where a
reference blob is pinned.**
*Reference:* `zic(8)` (`-b slim`/`fat`, `-R`). *zic-rs status:* ✅ accepted, documented. *Tests:*
`Test/Mixed` (zic 39 vs zic-rs 122 explicit transitions, `zdump`-equivalent). *Open bucket:* optional
`--emit-style` slim/byte-parity (future).

### 15. Links are production identifiers
`Link` targets may be zones or other links; links may precede their targets; chains name the same
zone. Production systems use aliases, not only canonical names. Accounted **separately**.
*Reference:* `zic(8)` (Link); jiff#258 alias-duplication. *zic-rs status:* ✅ resolve/cycle/missing
diagnostics; `--alias-map`; `support-report` counts canonical/links/total/chains/cycles separately.
*Tests:* link cycle/missing tests; `tests/support_report.rs`. *Open bucket:* —.

### 16. `Leap` lines are a separate file context
Main zone source is `Rule`/`Zone`/`Link`; leap-second input is `Leap` + an expiration line. The
`R`/`Z`/`L` zishrink prefixes are *zone-source* dispatch; `Leap`/`Expires` must **not** be accepted in
normal zone-source mode.
*Reference:* `zic(8)` (input forms). *zic-rs status:* ✅ zone-source dispatch only; leap modes out of
scope (fail closed / unknown). *Open bucket:* leap-file mode (future, out of scope).

### 17. Pre-1970 matching is reference compatibility, not omniscience
tzdb partitions the world so clocks agree *after* 1970; pre-1970 data is included where useful but is
not universal historical truth. Phrase every claim as "matches reference `zic`/tzdb for *this zone*
over *this horizon*."
*Reference:* tz `theory.html`. *zic-rs status:* ✅ wording enforced in docs (compatibility/ladder).
*Known zones:* London 1830..2045 (stated as reference-agreement, not regional truth). *Open
bucket:* —.

### 18. Governance + public-domain humility is correctness
zic-rs **compiles** tzdb per reference `zic` behaviour; it does not set timezone policy or "correct"
history. tzdb is public-domain, maintained by the TZ Coordinator via the RFC 6557 / BCP 175 process.
*Reference:* RFC 6557; tz `theory.html`. *zic-rs status:* ✅ doctrine in
[tzdb-governance.md](tzdb-governance.md); credit + no-endorsement in `ACKNOWLEDGEMENTS.md`. *Open
bucket:* —.

---

## Fail-closed buckets → law (the audit map)

`support-report --explain-buckets` and this file share **one** map, so they cannot drift:

| Bucket (support-report label) | Zones (2026b) | Deep law |
|-------------------------------|---------------|----------|
| `recurring footer: non-POSIX day form` | ✅ none — *cleared* (law 10 + v3 footer; `Asia/Gaza`/`Asia/Hebron` now compile) | **law 10** — neighbouring-month ON / footer expressibility |
| `inline-save: negative SAVE` | ✅ none — *cleared* (law 7; `Europe/Prague` now compiles) | **law 7** — signed SAVE (negative valid) |
| `no-rules era: %s or STD/DST slash FORMAT` | (none in 2026b) | **law 9** — FORMAT paths |

**No canonical zone fails closed in 2026b → 341/341 compile-clean and behaviour-match.** The map
retains the law-7/law-10 rows (the `deep_semantic` mapping is kept as documentation and would fire
again if a future tzdb release reintroduced such a construct) even though both buckets are now empty.
Any fail-closed remains a deliberate refusal (no approximate output), surfaced as a `ZIC001`
diagnostic and bucketed honestly.
