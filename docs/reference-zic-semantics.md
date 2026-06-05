# Reference-`zic` semantics ledger

Pinned records of subtle reference-`zic` (tzcode 2026b) behaviours that a casual port would
get wrong. Each was established **by experiment** — decoded explicit-transition counts
**and** `zdump -v` behaviour **and** the footer — *before* being encoded, never inferred
from prose. Authority order: `zic.c` / `zic(8)` / `tzfile(5)` (RFC 9636) / reference
`zic`+`zdump`. Wikipedia and articles are orientation only.

---

## 1. `UNTIL` is interpreted in the *ending* era's context, with the prevailing save

A zone era's `UNTIL` is converted to a UT instant using **that era's** `STDOFF` and the
`save` **in effect at the UNTIL moment** — *not* the next era's offset, and *not* save 0 if
DST is active there.

* `Test/FF` (`… EST 1975 Apr 1 2:00` then `-6:00 - CST`): `2:00` wall under the old `-5:00`
  → boundary at **1975-04-01 07:00Z**; footer `CST6` (final era).
* `Test/MidDst` (era ends `1990 Jul 1 0:00` **while DST is active**): `00:00 − (−5h) − 1h` →
  **1990-07-01 04:00Z**, not `05:00Z`. The one-hour trap.

Implementation: `compile::transitions::compile_multi_era` converts `UNTIL` with the running
`save` after applying the era's in-range rule activations.
Tests: `until_wall_time_uses_prevailing_save_when_dst_active`, `footer_comes_from_final_era`.

## 2. Equal UT offset across a boundary is still a real type change

The `Test/MidDst` boundary is `EDT → AST`, both `utoff = −14400`, but the DST flag and
abbreviation differ. It must be emitted, not de-duplicated. Local-time-type identity is at
least `(utoff, is_dst, abbreviation)` (plus the std/wall + UT/local indicators where
represented), not the UT offset alone.
Test: `equal_utoff_boundary_type_change_is_not_deduped`.

## 3. Final recurring era: footer anchoring (the big one)

A recurring (`TO = maximum`) rule set is **not** expanded into per-year explicit transitions
by `zic`. It writes a single **anchor** transition plus the recurring POSIX footer; `zdump`
then projects the footer's recurrence forward **from the anchor**.

* **Single-era recurring zone** (`Test/Eastern`): the anchor is the rule's first activation
  (≈ its `FROM` year). Pre-`FROM` years remain **standard**. `zic` does **not** treat `FROM`
  as globally ignored.
* **Final continuation recurring era** (`Test/FR`, `Test/Q`): the anchor is the **era-start
  boundary**. The footer governs the whole era from the era start, so DST appears from the
  era start *even when the rule's `FROM` year is later* (`Test/Q` proves this: `FROM 2015`,
  DST from 2000). The anchor is **load-bearing** — without it `zdump` projects the recurrence
  backwards over the previous era / all prior time. It is therefore emitted **forced** (never
  de-duplicated, even when its type equals the prevailing one).

> `zic-rs` matches this by emitting a forced final-era anchor transition + the exact
> recurring footer, rather than expanding per-year transitions for the tail. We do **not**
> change `Rule FROM` semantics globally; the single-era compiler's "fat" expansion is
> `zdump`-behaviour-equivalent in its window and is kept. A final era *mixing* finite +
> recurring rules is not yet supported (fails closed; no fixture needs it).

### Per-fixture evidence
| fixture | source (final era + rule) | ref explicit transitions | footer | zdump horizon | observed behaviour |
|---------|---------------------------|:------------------------:|--------|---------------|--------------------|
| `Test/Eastern` | one era `-5:00 US E%sT`; `Rule US 2007 max` | **1** | `EST5EDT,M3.2.0,M11.1.0` | 2000–2010 | standard before 2007, DST from 2007 |
| `Test/FR` | `… EST 2000` then `-5:00 US E%sT`; `Rule US 2007 max` | **1** (era-start anchor) | `EST5EDT,M3.2.0,M11.1.0` | 1995–2099 | EST before 2000, **DST from 2000** |
| `Test/Q` | `… EST 2000` then `-5:00 Q E%sT`; `Rule Q 2015 max` | **1** (era-start anchor) | `EST5EDT,M3.2.0,M11.1.0` | 2000–2018 | EST before 2000, **DST from 2000** (not 2015) |

Implementation rule: a final recurring era → forced era-start anchor transition + exact
recurring footer; no per-year expansion.
Tests: `single_era_recurring_rule_does_not_backfill_before_from_year`,
`final_continuation_recurring_era_footer_projects_from_era_start`,
`final_continuation_from_2015_projects_from_era_start`,
`final_recurring_era_emits_anchor_transition_not_fat_year_expansion`,
`footer_anchor_prevents_projection_into_previous_era`.

## 4. Recurring footer is exact-or-fail

The recurring POSIX footer is synthesised only for shapes `zic` represents exactly: omit the
DST offset when it is the default +1h; omit `/time` when it is the POSIX default `02:00`;
`Sun>=N` → `M{month}.{(N-1)/7+1}.{weekday}` (and `lastSun` → week 5); `AT` converted to local
wall. `Sun<=N`/fixed-day recurring rules have no exact POSIX form and **fail closed** rather
than emit an approximate footer.

## 5. Effective-in-era rule classification (the Europe/London gate)

A final era is classified by the rule activations **relevant inside that era's interval**, not
by raw rule-set membership. A finite rule (`TO = <year>`) whose whole span ends before the era
starts never fires inside the era — it governed an earlier era and is already accounted for
there — so its mere presence must not force the set to be treated as "mixed finite+recurring".

Two shapes, **both supported**:

* **(a) effectively recurring-only** — no finite rule fires in or after the era start.
  Reference `zic` (and zic-rs) represent this as a single era-start **anchor** + the recurring
  footer (§3). *Example* — **Europe/London**: final era `0 E GMT/BST` from **1996**, Rule `E`'s
  finite rows end **1995**, so only the perpetual rows activate in-era → footer
  `GMT0BST,M3.5.0/1,M10.5.0`, `zdump` match over `1830..2045`. (`Test/FinalEffective` is the
  artificial miniature; `Test/Mixed` the single-era analogue.)
* **(b) genuinely mixed-in-era** — finite rules *do* fire inside the era, alongside recurring
  rows. zic-rs expands the finite history explicitly and projects the recurring tail via the
  footer (the same path as the single-era `Test/Mixed`); the explicit transitions cover the era
  from its start, so there is no FROM-after-start gap needing the (a) anchor. *Example* —
  **America/New_York** (T4.1): final era `-5 US E%sT` from **1967**, Rule US finite DST
  1967..2006 + recurring 2007..max → footer `EST5EDT,M3.2.0,M11.1.0`, `zdump` match over
  `1883..2040`. (`Test/MixedInEra` is the artificial miniature.)

The (a)/(b) split is *year-level* (a finite rule is "in-era" if its `TO` year ≥ the era's start
year); this is safe — a borderline mis-route from (a) to (b) still produces correct output via
explicit expansion, just one extra transition. Tests:
`final_era_ignores_pre_era_finite_rows_when_classifying_recurring_tail`,
`mixed_in_era_finite_and_recurring_final_era_is_supported`,
`america_new_york_2026b_matches_reference_zic_over_1883_2040`.

Tests: `final_era_ignores_pre_era_finite_rows_when_classifying_recurring_tail`,
`europe_london_2026b_matches_reference_zic_over_1830_2045`.

## 6. Accepted slim/fat structural difference (behaviour is the contract)

Reference `zic` emits a **slim** explicit-transition set; `zic-rs` currently emits a **fat**
one for rule-driven recurring zones. Pinned by decoding TZif `timecnt` directly (never by
counting `zdump` lines — see §7):

* **`Test/Mixed`** (single-era, finite-April + recurring-March rule set): reference `zic`
  `timecnt = 39` (38 explicit finite-history transitions + 1 recurring anchor); `zic-rs`
  `timecnt = 122` (every year expanded through `RECUR_HI`). The raw normalised `zdump` diff
  over the finite window `1976..1997` is **empty** and the footers are identical
  (`GMT0BST,M3.5.0/1,M10.5.0`), so the historical April onsets are honoured by *both* — only
  the explicit representation differs. **Behaviour (`zdump`) is the contract; the
  explicit-transition representation is not.** Matching `zic`'s slimming heuristic is a
  separate (byte-parity) goal, deliberately not pursued here.
* For **pure** recurring zones with no in-era finite history (`Test/Eastern`, `Test/FR`,
  `Test/Q`), reference `zic` `timecnt = 1` (a single anchor), which the multi-era path matches.

## 7. Oracle hazard: `zdump` requires absolute paths

`zdump` must be invoked on **absolute** TZif file paths. A relative name may be interpreted as
a zone identifier via `TZDIR`/the system zoneinfo database (or fail), producing **misleading**
output. During the `Test/Mixed` audit a relative-path invocation produced a false
"one transition" reading; re-running with absolute paths showed reference `zic` and `zic-rs`
agreed across the finite window. The `compare` command always uses absolute paths for exactly
this reason; ad-hoc `zdump` debugging must too. (See [oracle-testing.md](oracle-testing.md).)

## 8. `FROM = minimum` is obsolete — coerced to 1900 (not infinite past)

Pinned against reference `zic` 2026b: a rule whose `FROM` year is `minimum` produces the
warning *"FROM year 'minimum' is obsolete; treated as 1900"* and is compiled **as if the FROM
year were 1900** — the earliest transition lands in 1900, not at the start of representable
time. tzdata 2026b uses `minimum` **zero** times (`grep -c '^R [^ ]* mi' tzdata.zi` → 0), so it
is a legacy-compatibility spelling only. zic-rs coerces `minimum` → 1900 at parse time, matching
this exactly; it does **not** model an unbounded past. Verified: `Test/MinRule`
(`Rule X minimum max … Apr lastSun / Oct lastSun`, `Zone … -5:00 X E%sT`) → first transition
`1900-04-29 07:00:00Z`, footer `EST5EDT,M4.5.0,M10.5.0`, `zdump` match over `1899..1910` *and*
`2019..2040`. The year keywords `only`/`minimum`/`maximum` are matched as `zic`-style unambiguous
prefixes (`mi`→min, `ma`→max, bare `m` is ambiguous → error).
**Tracked follow-up:** zic-rs does not yet surface the "minimum obsolete" *warning* during
compilation (the compile path has no non-fatal-warning sink); the acceptance bar here is
semantic parity (the 1900 coercion), not warning-text parity.

## 9. Inline-save eras construct a fixed type from STDOFF + SAVE

A `Zone` era whose `RULES` column is a clock value (e.g. `0:30`) does **not** consult a named
rule set; it is a constant-saving era. Reference `zic` builds a single local-time type:
`utoff = STDOFF + SAVE`, `isdst = 1` for a non-zero save, abbreviation from `FORMAT`. The
critical trap: a **`%z`** `FORMAT` renders from the **total** effective offset, not `STDOFF`.

Pinned (ttinfo decoded directly from the reference TZif — byte-identical to zic-rs):
* `8:00 0:30 HKWT` → ttinfo `(30600, isdst=1, "HKWT")` (literal). [`Test/InlineLit`]
* `7:00 0:20 %z`  → ttinfo `(26400, isdst=1, "+0720")` — `%z` over `7:20`, **not** `7:00`.
  [`Test/InlineZ`]
* single-era `8:00 0:30 HKWT` → one type `(30600, isdst=1, "HKWT")`, fixed footer `HKWT-8:30`.
  [`Test/InlineSolo`]

`Save { seconds, is_dst }` from `parse_save` already encodes the daylight flag (`s`→0, `d`→1,
no-suffix→`seconds != 0`), so zic-rs uses it directly. **Fail closed** (not yet pinned): inline
save with `%s` (no LETTER to substitute), the `STD/DST` slash form, and a **negative** save.
Tests: `inline_save_literal_format_marks_dst_and_adds_save_to_offset`,
`inline_save_percent_z_uses_effective_total_offset_for_abbreviation`,
`inline_save_works_across_multi_era_boundaries`, plus the three fail-closed and three oracle tests.

## 10. compile-clean ≠ behaviour-verified (a frontier, honestly stated)

The `%z`-on-no-rules-era unlock took `support-report` from 162 → **338 / 341** canonical zones
**compile-clean** (a valid TZif is produced) over `tzdata.zi` 2026b; the law-7 negative-SAVE fix
then took it to **339 / 341** (law 7, `Europe/Prague`), and law 10 to **341 / 341**. That number is
*not* a
behaviour claim. The behaviour contract is the **`zdump` oracle**, and we now run it
**comprehensively, not as a sample**: every one of the 341 canonical zones is compiled and
`zdump`-diffed against reference `zic` over `1900..2040`. The result:

> **341 match / 0 mismatch / 0 fail-closed** (of 341), `1900..2040` — *after T5 #1–#5 + law 7 + law
> 10*. Accounting: 341 + 0 = 341 compile-clean; + 0 fail-closed = 341. Consistent with
> `support-report` (unsupported zones: none). Progression: **264 → 328 → 333 → 338 → 339 → 341 match**
> (T5 #3 −64, #4 −5, #5 −5; law 7 −1 Prague; law 10 −2 Gaza/Hebron).

**MILESTONE (measured, scoped):** zic-rs **behaviour-matches reference `zic`/`zdump` over `1900..2040`
for *every* canonical zone** in `tzdata.zi` 2026b — **341 / 341**, zero mismatches, **zero
fail-closed**. This is the *last of the core* — there is no remaining canonical-zone behaviour gap
over the declared horizon. It is **still not** a claim of full `zic` replacement (no leap-second
modes, CLI policy, `right/`/`posix/`, backzone/rearguard, warning parity, slim emission, `-r`/`-R`/
`-b`/`-p` modes — the operational shell (T9–T15) remains a declared roadmap) and **not** infinity — it is
reference-verified *behaviour* for the canonical zones over a declared horizon. **compile-clean (341)
and behaviour-match (341) are still tracked separately** — different claims, permanent doctrine.

**Law 7 (signed SAVE) — DONE.** `Europe/Prague`'s 1946–47 era `1 -1 GMT` is an **inline negative
SAVE**: effective offset `STDOFF + SAVE = +1 + (−1) = 0`, `is_dst = (save ≠ 0) = true`, literal abbr
`GMT` — reference `zic` renders `GMT isdst=1 gmtoff=0`, and zic-rs now matches it. Lifted the
defensive "negative inline SAVE not pinned" guard (negative-save *Rules* already worked — e.g.
Morocco). Tests `inline_save_negative_renders_signed_effective_offset`,
`negative_inline_save_is_signed_state_matches_reference_zic`.

**Law 10 (non-POSIX `ON` day form + v3 footer) — DONE → 341/341.** `Asia/Gaza`/`Asia/Hebron`'s
perpetual Palestine rule uses `Sat<=30`. Reference `zic` re-anchors a `weekday W <= N` form onto a
clean nth-weekday and folds the skipped days into the transition time, which can exceed 24h and
therefore needs a **v3 footer**: `Sat<=30 02:00` → `M3.4.4/50` (4th Thursday + 50h). Implemented the
exact `zic` `stringrule` algorithm (`posix_footer::date_rule` now returns an `(Mm.w.d, day-shift)`;
`recurring` folds the shift into the time and returns the **content-driven version** — `b'3'` when a
transition time is outside `0..24h`). Two coupled pieces: (a) the day-form conversion (`Sat(6)<=30`
→ wdayoff 2, Thu(4), week 4, +2 days; the `>=N` form is symmetric and reduces to the existing clean
case when wdayoff=0); (b) the **explicit-horizon fix** — Rule P's one-shot Ramadan-dated rows extend
to **2086**, far past `RECUR_HI`, and are *not* footer-representable, so the explicit expansion now
runs to the last finite rule year and the footer covers only the perpetual tail. Both zones compile
**v3** and zdump-match reference `zic`; the synthetic `Sun<=25` case (`M10.3.3/98`) is byte-identical
to reference `zic` too. Tests `neighboring_month_on_form_matches_reference_zic`,
`recurring_sat_leq_30_uses_v3_extended_time_footer`, `recurring_sun_leq_25_uses_extended_v3_footer`.

**Honesty note — this corrected an earlier under-count.** A 35-zone *stratified sample* had reported
"28 match / 6 mismatch," which made the frontier look ~6 bugs deep; the first comprehensive sweep
showed it caught only **6 of 74** real mismatches. None of the mismatches were introduced by the
`%z` unlock. The fixes **clustered by shared rule sets** — T5 #3 collapsed the
Russia/Central-Asia/Argentina/Indiana/Chile/Europe family (74 → 10), T5 #4 the prior-active-carry
family (10 → 5), and T5 #5 the clock-reference family (5 → 0).

### State model (do not collapse these into "offset")

The T5 bugs are all **state-model** bugs, so the ledger names three distinct things and never
conflates them:

* **era state** — which `Zone` continuation line is in force at an instant (its `STDOFF`, its
  `RULES` column, its `UNTIL`). Bounded by era boundaries.
* **rule state** — the *prevailing* `(save, is_dst, LETTER)` of a **named rule set**, evolving
  along that rule set's **own** timeline of activations. A rule set's state persists across eras
  that do **not** use it; **entering a later era that uses the rule set does not reset it.**
* **local-time type** — the emitted TZif `ttinfo`: `(utoff = stdoff + save, is_dst, abbreviation)`.
  This is a *function* of era state + rule state, not a substitute for either. Two types can share
  a `utoff` yet differ in `is_dst`/abbr (e.g. MSK-std vs EEST-dst, both +3) — a real type change,
  never deduplicated.

**Fixed so far (generic, no zone special-casing):**

* ✅ **FIXED (T5 #1) — standard `LETTER` is temporal state, not a rule-set constant.** `%s`
  renders the *active* rule's `LETTER`, and a `SAVE=0` rule can *change* it. `Pacific/Auckland`:
  NZ's `1946 Jan 1 0 0 S` rule lands **exactly on the +11:30→+12:00 era boundary** and flips the
  standard abbreviation NZMT→NZST. zic-rs dropped that boundary-coincident activation behind the
  era-start transition (which used the stale seed letter `M` → NZMT). **Fix:** a rule activating
  at-or-before the boundary instant (`ut <= s`, not `< s`, in `compile_multi_era`) now seeds the
  boundary state, so the boundary carries the activation's letter (`S` → NZST). Verified:
  `Pacific/Auckland` zdump-matches reference `zic` over `1900..2040`
  (test `pacific_auckland_1946_changes_nzmt_to_nzst_and_matches_reference_zic`).
* ✅ **FIXED (T5 #2) — the final *recurring* era anchor must seed boundary-coincident activations
  too.** The shape-(a) (effectively recurring-only) short-circuit emitted its era-start anchor from
  the era's *standard* seed, skipping the `ut <= s` absorption the general path does.
  `Europe/Lisbon`'s final era `0 E WE%sT` begins **1996-03-31 01:00u**, which is exactly Rule E's
  last-Sunday-of-March spring-forward, so the anchor must be **WEST (isdst, +1h)**, not standard
  **WET**. **Fix:** before building the anchor, walk the rule activations in `[start_year−1,
  start_year+1]` and seed `run` from the latest one at-or-before the era start (same `ut <= s`
  rule). Verified: `Europe/Lisbon` **and** `America/St_Johns` (a different zone, same bug) both
  zdump-match reference `zic` over `1900..2040` (tests
  `europe_lisbon_final_era_boundary_coincident_dst_matches_reference_zic`,
  `america_st_johns_matches_reference_zic_after_anchor_fix`).
  **Correction to the earlier ledger:** Lisbon was previously grouped with
  `America/Argentina/Buenos_Aires` as one "wrong DST at a rule/era change" class. The oracle
  disproved that — the anchor fix clears Lisbon and St_Johns but **not** Buenos_Aires.
* ✅ **FIXED (T5 #3) — boundary-coincident *standard-clock* activation absorption (cleared 64
  zones).** A rule activation that names the **same standard-clock instant** as an era boundary must
  seed the boundary, but the general path only absorbed it when its *converted UT* satisfied
  `ut <= s`. When the new era's standard offset is **1h smaller** than the old era's, the boundary
  `s` is computed at the previous era's UNTIL with the *old* (larger) stdoff, while the activation's
  `ut` is computed with the *new* (smaller) stdoff — so `ut` overshoots `s` by exactly the offset
  difference and was emitted as a separate transition, inserting a **spurious 1h standard interval**.
  Example: `Europe/Moscow` 1991 — `3 R MSK/MSD 1991 Mar 31 2s` → `2 R EE%sT`, both "Mar 31 02:00
  standard"; reference `zic` makes the boundary itself the spring (EEST, isdst=1, UT offset
  continuous +3→+3), not standard-then-spring. **Fix:** carry the previous era's UNTIL as a *naive
  local + clock reference* (`boundary_until`) and absorb an activation whose `(local_seconds,
  w/s/u reference)` **exactly equals** the boundary's — an exact clock-frame equality, deliberately
  **not** a time tolerance (which would wrongly absorb ordinary near-boundary transitions). Generic;
  it collapsed the whole Russia/Central-Asia/Argentina/Indiana/Chile/Europe spring-at-offset-drop
  family (74 → 10). Tests: `era_boundary_absorbs_new_era_standard_clock_spring_forward` (hermetic
  synthetic fixture `boundary_spring.zi`), `europe_moscow_1991_matches_reference_zic`,
  `europe_volgograd_boundary_spring_forward_matches_reference_zic`,
  `asia_novosibirsk_1991_matches_reference_zic`,
  `russia_boundary_offset_drop_does_not_create_spurious_standard_hour`.
* ✅ **FIXED (T5 #4) — prior-active rule-state seeding: a named rule set has its own timeline
  (cleared 5 zones).** A **named rule set's state persists across eras that do not use that rule
  set; entering a later era that uses the rule set does not reset it to standard.** If the last
  activation before the era start was `SAVE=1 LETTER=W`, the era *begins* in that rule state until a
  later activation changes it. zic-rs seeded a ruled era from `seed_run` (standard) and only evolved
  it from activations inside the era's local window `[start_year−1, …]`, so an activation that set
  the state **years earlier** (and is still in force) was missed. Forced by:
  `America/Phoenix` (Rule US set War in **1942**, not cleared until 1945 → the 1944-04-01
  `-7 US M%sT` era must start in **MWT**, not MST); `Atlantic/Bermuda` (Rule Be's last pre-1930
  activation, **1918**, carries `LETTER=S` → `AST`, not `AT`); `Asia/Manila` (DST from **1941**);
  `Europe/Brussels` (DST from **1940**, persisting across the intervening `1 c` era until Rule b
  falls back 1944-09-17). **Fix:** when seeding a ruled era with a start instant, scan the rule
  set's *actual* `FROM` range up to the era start and seed `run` from the latest activation
  at-or-before it (carrying `save` **and** `LETTER` together, via the same w/s/u `convert_at`); if
  none precedes the era start, the standard seed stands. Generic; cleared **Phoenix, Bermuda,
  Manila, Brussels, and `Asia/Macau`** (a 5th that shared the class). Tests:
  `ruled_era_inherits_prior_active_rule_state` (hermetic synthetic fixture `prior_active_carry.zi`,
  which uses a fixed first era so it isolates the carry from zic's ruled-first-era initial-type
  choice), `america_phoenix_1944_inherits_war_time_matches_reference_zic`,
  `atlantic_bermuda_1930_inherits_standard_letter_matches_reference_zic`,
  `asia_manila_and_europe_brussels_inherit_prior_dst_match_reference_zic`.
* ✅ **FIXED (T5 #5) — clock-reference normalization at era boundaries (cleared the last 5 zones).**
  tzdata freely mixes `w`/`s`/`u` references, but two boundary comparisons used **naive local-second
  values / exact reference equality** instead of resolved instants. All 5 residuals were a boundary
  where the `UNTIL` and the responsible rule's `AT` used *different* references. Two sub-sites:
  - **Era-end break** — `act.local_seconds >= until_local` compared naive locals across references.
    `Europe/Simferopol` ends `2 E EE%sT 2014 Mar 30 2` (**wall**) while Rule E springs `Mar lastSu
    1u` (**universal**): naive `01:00 < 02:00` kept the spring, but in UT it is past the boundary and
    belongs to the next era. **Fix:** break on `ut >= convert_at(until_local, …)` — resolved UT, not
    naive local. For a same-reference era this is *algebraically identical* (offsets cancel), so only
    mixed-reference boundaries change. Breaking before `run` advances also keeps the prevailing save
    correct for the `UNTIL`, which fixes `Europe/Warsaw` (Rule c fall-back `2s` ≡ wall 03:00 at the
    `1 c CE%sT 1918 S 16 3` boundary).
  - **Boundary-coincidence absorption** (the T5 #3 test) required `act.at_ref == boundary_ref`.
    `Asia/Tashkent`/`Ashgabat` end `…Mar 31 2` (**wall**) vs Rule R `2s` (**standard**); `Asia/Anadyr`
    ends `…Apr 1 0s` (**standard**) vs Rule R `Apr 1 0` (**wall**) — equivalent because `save == 0` at
    the boundary, so wall 02:00 ≡ standard 02:00. **Fix:** require the same local seconds AND that the
    two references resolve to the **same UT under the prevailing `(stdoff, save)`** — exact equality
    after deterministic normalization, never a time tolerance.
  Tests: `era_boundary_normalizes_wall_standard_clock_reference` (hermetic `boundary_clockref.zi`),
  `central_asia_offset_drop_boundaries_normalize_clock_reference` (Tashkent/Ashgabat/Anadyr),
  `europe_mixed_reference_era_end_breaks_match_reference_zic` (Simferopol/Warsaw).

**Open behaviour-mismatches: none. Open fail-closed: none.** **341 / 341** canonical zones compile
*and* behaviour-match reference `zic` over `1900..2040` — the entire canonical-zone behaviour frontier
is closed. (The remaining work is the **operational shell (T9–T15)** — leap/`right`/v4, CLI/install modes,
`-r`/`-R`/`-b`, warning parity — and is *not* a canonical-zone behaviour gap; see `roadmap.md` and
`zic-hidden-compatibility.md`.)

Doctrine: `support-report` reports *compile* support; the comprehensive `zdump` sweep above is the
behaviour contract. The sweep is reproducible (`/tmp/t5sweep.sh`-style: compile every `Z`-line zone,
`zdump -v -c 1900,2040` diff vs reference `zic`); folding it into a `support-report --verify` mode
is the next tooling step.

---

### Method note (the tz database is multi-era; classify by evidence)
The tz database model is multi-era: a zone is a sequence of zone eras, each ending at an
`UNTIL` boundary; rule sets define transition patterns but final TZif behaviour may also
depend on the generated POSIX footer. Therefore apparent recurring behaviour before a rule's
`FROM` year must be **classified by evidence** — explicit TZif transitions vs footer-driven
`zdump` projection vs `zic`-era synthesis. `zic-rs` must not globally change `Rule FROM`
semantics without a fixture proving the exact reference-`zic` behaviour. (`docs/zic.c`
function leads for future digs: `outzone`, `stringzone`, `rpytime`, `rule_cmp`, slim/`want_bloat`.)
