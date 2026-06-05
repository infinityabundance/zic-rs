# Replacement-readiness ladder (T19)

> A typed, explicit ladder from "research compiler" to "default system `zic`" — so adoption is a
> **gated progression**, never the dangerous jump from *"it works"* to *"replace system `zic`."* Each
> level is reached only when its **required evidence** exists; zic-rs states its **current level** plainly
> and refuses higher claims. Pairs with `docs/drop-in-compatibility-contract.md`, `docs/not-yet-ready.md`,
> and `TRUST.md`.

## `ReplacementReadinessLevel`

| Level | Meaning | Required evidence (gate to *enter* the level) |
|---|---|---|
| **RRL-0** | research-only | builds; some fixtures compile |
| **RRL-1** | validator / report-only | the public reports run (`support`/`structural`/`semantic`/`tzif-validate`/`aux-table-validate`/`doctor`) + CORE.1 green; **no install claimed** |
| **RRL-2** | reference-comparison tool | `compare` + `release-diff` against reference `zic`/`zdump`; oracle identity recorded; differences are *reported*, not gated |
| **RRL-3** | package-build helper | staged `--out` materialization (no system install) + manifest/alias-map provenance + the install/materialization contract; a packaging-gauntlet receipt (T21) |
| **RRL-4** | secondary `zic` in a distro package | a distro packages it as a *non-default* `zic`; side-by-side vs reference in that distro's CI; fuzz-run receipts (not `pending_capture`); an external audit packet exercised |
| **RRL-5** | primary `zic` for a **selected release/profile** | bounded admission for that release + profile; drop-in-compat contract green for the used surface; whole-tree-install + TOCTOU residuals closed *or* explicitly accepted by the adopter |
| **RRL-6** | default system `zic` replacement | broad multi-release admission; complete operational parity for the adopter's modes; external audit + sustained maintenance posture |

## Current level (honest)

**zic-rs is at RRL-1 → RRL-2** (a validator / report generator / `doctor` / `release-diff` / bounded
compiler candidate), with **RRL-3 partially in reach** (staged materialization + provenance exist; the
packaging-gauntlet receipts of T21 do not yet). It is **not** RRL-4+ — no distro packaging, no admitted
fuzz runs, no external audit. The behaviour contract is exactly CORE.1 (2026b, 1900..2040).

## What each level requires that zic-rs does **not** yet have

- **RRL-3:** packaging-gauntlet receipts (T21).
- **RRL-4:** distro packaging + in-distro side-by-side CI · a **coverage-saturating** fuzz campaign (today: a
  *bounded* smoke ran — T23.cargo-fuzz.1/.2 found+fixed F1–F3; a 24 h campaign is the RRL-4 bar;
  `fuzz/receipts/RUNS.md`) · an exercised external audit (`docs/audit-readiness.md` is the packet, not a performed audit).
- **RRL-5:** the whole-tree-install non-claim + the parent-component-TOCTOU residual either closed
  (`openat`-style hardening, T20) or explicitly accepted by the adopter; drop-in-compat green for the used
  surface.
- **RRL-6:** broad multi-release admission (only 2026b today) + operational parity across the adopter's
  modes + sustained maintenance/audit.

## Doctrine

- A level is **claimed only when its evidence exists** — never inferred from a lower level. CORE.1 (an
  RRL-1/2 behaviour proof) does **not** imply RRL-5/6 operational replaceability.
- The ladder is **adopter-relative for RRL-5+**: "primary for a selected release/profile" is meaningful
  only against the adopter's declared release + operational profile, not as a universal claim.
- Moving up a level is a **dated, evidenced event** (a receipt), recorded in `CHANGELOG.md` — not a
  marketing decision.
