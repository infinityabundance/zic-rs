# RECEIPT — PERPETUAL-EXPANSION.1 (legacy empty-footer fallback) — 2026-06-05

> **Claim wording (binding):** *PERPETUAL-EXPANSION.1 does not alter normal POSIX footer synthesis and does
> not claim correctness beyond the explicit transition horizon. It permits an empty-footer fallback only in
> explicit legacy replay mode when a final recurring era cannot be represented as a POSIX footer, after
> transitions have already been expanded through `RECUR_HI = 2037`.*
>
> **Non-claim:** *PERPETUAL-EXPANSION.1 does not reproduce tzcode2019c's private `2392` expansion horizon
> byte-for-byte. It reproduces the admitted historical oracle's behaviour over the declared `[1980, 2037]`
> replay window and emits an empty footer with an explicit beyond-horizon freeze boundary.*

## The gap it closes

YEARISTYPE.1 closed 55/66 of the pre-2000f band but left **11** releases (93b–94f) `deferred-perpetual-footer`:
their `AS` even/odd rules were **perpetual** (`1990 max even` / `1990 max odd`), so the final recurring era has
**two perpetual standard rules that alternate by year parity** — a shape a POSIX `TZ` footer **cannot encode**.
zic-rs *accepted* the Rule `TYPE` (no `ZIC027`) but then **failed closed** on footer synthesis (`ZIC001`).

This was always a **footer-emission** gap, not a transition-generation gap: zic-rs already expands explicit
transitions across `[lo, RECUR_HI = 2037]` for recurring zones (the exact comparison window). The only missing
piece was *what footer to emit when synthesis fails*.

## The fix (surgical, gated, fail-closed-by-default)

A new **`--legacy-empty-footer`** flag → `EmitOptions::allow_empty_footer_on_legacy_nonposix_recurrence`
(deliberately verbose so it can't be mistaken for a general relaxation). It is a **footer-emission policy** —
**it does not change transition generation**. On the *existing* `build_recurring_footer` failure path **only**,
and only when the flag is set, zic-rs emits an **empty footer** (`\n\n`) instead of `ZIC001` — the explicit
transitions it already expanded through `RECUR_HI` stand, frozen beyond the last one.

Applied at the two sites that reach the failure path **after** explicit expansion: `compile_rule_zone`
(single-era — e.g. `Australia/South` 93b–93f) and `compile_multi_era`'s shape-b / mixed-in-era final-era path
(e.g. `Australia/Adelaide` 93g–94f, which carries finite 1971–89 history so its tail is already expanded). The
**shape-a recurring-only path stays fail-closed** even under the flag — it has no explicit expansion, so an
anchor-only empty-footer file would be wrong; refusing it is the correct conservative behaviour (the
precondition "transitions already cover the window" is not met there).

**No coupling to the oracle's horizon.** tzcode2019c expands to year **2392**; zic-rs expands through its own
policy horizon `RECUR_HI = 2037` + empty footer. Verification is **behavioural over `[1980, 2037]`**, never
byte-identity to `2392`. Beyond the last explicit transition both freeze; that region is **not claimed**.

## Result (acceptance #5) — the band closes

`bash reports/perpetual-expansion/gauntlet.sh` (oracle `zic -y` vs zic-rs `--legacy-yearistype
--legacy-empty-footer`, per-fixture `zdump` over the declared `[1980, 2037]` window):

| | YEARISTYPE.1 (`--legacy-yearistype`) | + `--legacy-empty-footer` |
|---|---|---|
| match | 55 | **66** |
| deferred-perpetual-footer | 11 | **0** |
| zic-rs-divergent | 0 | **0** |
| even/odd zone gets empty footer | — | **11 / 11** |
| fixture `zdump` comparisons | 354/354 | **371/371 match over [1980,2037]** |

**All 66 pre-2000f releases now build and behaviour-match the admitted historical oracle.** Combined with the
default and Latin-1 modes: under the explicit replay modes, **276/276 stable tzdata releases attempted,
0 zic-rs behaviour divergences**, with the historical bands replayed against an admitted oracle.

## Gate (the strict 10-point gate, all met)

1. Branches from the existing `ZIC001` footer-synthesis failure path. ✅
2. Fallback only for final recurring eras that cannot synthesize a POSIX footer. ✅
3. Explicit transitions already expand through `RECUR_HI = 2037` (no expansion-engine change). ✅
4. Emits an empty footer. ✅
5. 11 residual releases → behaviour-match the oracle over `[1980, 2037]` (66/66, 0 divergent). ✅
6. No byte-identity required where the oracle expands farther (2392). ✅
7. The oracle's 2392 horizon is documented as oracle-internal, not a zic-rs contract. ✅
8. **CORE.1 341/0/0 byte-unchanged** (flag off; sweep + unit byte-identity on Europe/London). ✅
9. Existing POSIX-footer-synthesising zones never take the fallback (`legacy_empty_footer_does_not_change_posix_footer_zone`, `core_fixture_byte_unchanged_under_the_flag`). ✅
10. Atlas row reclassified `deferred` → `bounded_empty_footer_replay`. ✅

## Tests (acceptance #7) — 519 total (+4)

`tests/perpetual_expansion.rs`: `default_fails_closed_on_nonposix_final_recurrence` (ZIC001 preserved) ·
`legacy_empty_footer_emits_empty_footer` (empty footer + >50 explicit transitions, not anchor-only) ·
`legacy_empty_footer_does_not_change_posix_footer_zone` (bytes identical on/off) ·
`core_fixture_byte_unchanged_under_the_flag` (Europe/London byte-identical on vs off).

## Non-claims

- Footer-emission policy only; **transition generation is unchanged**, the default is unchanged (fail-closed).
- **Beyond the last explicit transition is intentionally not claimed** — an empty-footer file freezes; this is
  a faithful replay of the oracle's lossy behaviour for a rule shape POSIX cannot represent forever, not a
  forever-correct artifact.
- Verified behaviourally over `[1980, 2037]` against the admitted tzcode2019c oracle (+ `yearistype.sh` v7.4)
  on the bounded fixture set; not byte parity, not all zones/all time, not current-reference parity.

## Reproduce

`bash reports/perpetual-expansion/gauntlet.sh` (needs the YEARISTYPE.1 oracle: `tzcode2019c` built `zic` +
`yearistype.sh` v7.4 — see `reports/yearistype/RECEIPT-YEARISTYPE-1.md`).
