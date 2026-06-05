# Divergence Register

> Every known way zic-rs's behaviour differs from reference `zic`, in **one place**, each owned and bounded.
> A divergence is never hidden behind "mostly compatible": it is either a **defect** (found → fixed, with a
> regression test), a **deliberate divergence** (a safer/clearer choice, four-bucket-classified), or a
> **tracked gap** (a named residual with an owner). This register **consolidates + cross-links** the
> per-axis detail in `docs/differences-from-reference-zic.md` (the four-bucket map) and `docs/risk-register.md`
> (the `RISK.*` ids); it does not restate them. Pairs with `docs/PORTING-PHILOSOPHY.md` and
> `docs/PORTING-DECISION-LEDGER.md`.

## Legend

- **Bucket** (the standing four-bucket law, never blurred): **B1** implemented parity · **B2** explicit
  compatibility mode · **B3** intentional safer divergence · **B4** deferred full-parity work.
- **Class:** `defect_fixed` (was wrong, now corrected + regression-tested) · `intentional` (a deliberate
  choice) · `tracked_gap` (a named residual, not yet closed) · `resolved` (a former leniency now tightened).

## A. Defects found and fixed (the register earns its keep)

| ID | Surface | Reference `zic` | zic-rs (before → after) | Class | Evidence |
|---|---|---|---|---|---|
| `DIV.RIGHT-LEAP` | `right/` (leap) profile transition times | advances each transition by the **cumulative leap correction** (TAI-based `right/` encoding) | left transitions at POSIX values (drift behind reference by the running leap count) → **fixed:** `apply_leaps` shifts each transition by the cumulative correction, **`right/` path only** | `defect_fixed` | T23.reader-compat.2; regression test `right_profile_shifts_transitions_by_cumulative_leap_correction`; `RISK.LEAP.1`. **CORE.1 / default profile never affected** (POSIX path never calls `apply_leaps`); T11 missed it because it verified only `right/UTC` (no transitions). |
| `DIV.SIMULTANEOUS-TXN` | two rules for the same instant | fatal ("two rules for same instant", exit 1) | `debug_assert!` only → panic in debug / non-monotonic TZif in release → **fixed:** real `ensure_strictly_increasing()` guard → fatal `ZIC023` | `defect_fixed` | T14.4 pathology ledger; removed a **panic on untrusted input** |
| `DIV.OOB-TYPE-INDEX` | hostile TZif with transition `type_index ≥ typecnt` | n/a (reader concern) | latent OOB-index panic → **fixed:** `tzif::validate::parse` rejects as typed `Err` at the decode choke point | `defect_fixed` | T17.1a; `RISK.TZIF.1`; Kani `T23.kani.3a` |

## B. Deliberate divergences (chosen, classified, documented)

| ID | Surface | Reference `zic` | zic-rs | Bucket | Evidence |
|---|---|---|---|---|---|
| `DIV.ARGV-SHAPE` | CLI invocation | bare-flag argv (`zic -d DIR FILE…`), implicit system install | sub-command + **required `--out`**, no implicit system install | B3 | `drop-in-compatibility-contract.md`; `T23.drop-in-gauntlet.1` (file-set 598=598 on equivalent invocation) — **not a literal argv drop-in, by design** |
| `DIV.USAGE-EXIT-CODE` | usage error exit status | `1` | clap usage `2` (operational stays `1`) | B3 | `cli-compatibility-policy.md`; the 0/1/2 taxonomy keeps usage vs operational distinct |
| `DIV.DEFAULT-FAT` | default emission style | slim | **fat** default (`-b slim` / `--emit-style zic-slim` reproduces slim) | B3 (default) / B2 (slim mode) | `structural-parity.md`; behaviour-matched in both, byte parity claimed only where pinned |
| `DIV.CONTINUATION-DIAG` | stray continuation line | generic "input line of unknown type" | finer **`ZIC014`** (continuation-without-zone), split by column | B3 | T13.2; the **universal** vendor divergence (every vendor emits the generic class — see the T16.5b matrix) |
| `DIV.RESOURCE-CAPS` | input-size dimensions (zones/rules/links/leaps/chain-depth/bytes) | uncapped | bounded by `ResourceLimits` → `Error::config` (exit 1) on breach | B3 | T17.1b; defaults far above any real tzdb (2026b ≈350 zones), so no legitimate input is rejected; `RISK.RESOURCE.1` |
| `DIV.NAME-AS-PATH` | zone/link names as output paths | fatal `namecheck`/`componentcheck` (+ `zic -v` byte warnings) | same fatal policy **plus** B3 stricter (leading-`-` reject · NUL reject · UTF-8 required) + verbose `ZIC024`/`ZIC025` | B1 (fatal set) / B3 (stricter) | T14.5; `ZIC008`; reference `zic -v` parity on `Etc/GMT+5` |
| `DIV.MODE-METADATA` | `-m` file mode | symbolic + octal `chmod` | octal subset, validated-before-write, never follows symlinks | B1 (octal) / B3 (symbolic = simplification) | T9.5 |

## C. Tracked gaps (named residuals, not yet closed)

| ID | Surface | Reference `zic` | zic-rs | Bucket | Owner / status | Evidence |
|---|---|---|---|---|---|---|
| `DIV.SLIM-TIMECNT-RESIDUAL` | slim `timecnt` boundary | reference slim transition set | may keep a few footer-redundant transitions at the slim take-over (behaviourally null, `zdump`-identical) | B4 | enumerated, bounded (e.g. `Europe/Lisbon`) | `structural-parity.md`; the microcase test asserts prefix + ≤2-extra, not `timecnt` parity |
| `DIV.WHOLE-TREE-ATOMICITY` | crash mid-run | (no tree transaction either) | **per-file** crash-durable publish (Unix: content `sync_all` + atomic publish + parent-dir fsync); **whole-tree** atomicity refused | B4 | T17.4; `RISK.INSTALL.1`; non-claim `does_not_claim_whole_tree_crash_atomic_install` | `install-materialization-contract.md` |
| `DIV.PARENT-COMPONENT-TOCTOU` | concurrent parent-*component* symlink swap mid-run | — | leaf races closed (`O_EXCL`/atomic rename); the parent-component race needs `openat`/`O_NOFOLLOW` → unavailable without `unsafe`/a dep (both forbidden) | B4 | `RequiresOpenatStyleHardening`, owned by T20; non-claim `does_not_claim_full_toctou_resistance` | T14.6; `RISK.PATH.1` |
| `DIV.STDERR-WORDING` | exact diagnostic English | release/vendor-specific wording | class · severity · location matched; **exact wording last / not claimed** | B4 | by-design ordering (wording drifts across releases/vendors) | T13.close; `zic-warning-parity.md` wording ledger |
| `DIV.FOOTER-LAST-TXN` | footer ↔ last-explicit-transition consistency | (reader-equivalence axis) | **unassessed** as its own structural axis | B4 | tracked; a future reduced-surface Kani / validator follow-up | `audits/claim-boundary-map.md`; T23 forward queue |
| `DIV.MULTI-ERA-SAME-INSTANT` | multi-era same-instant where wall→UT `save_prev` separates activations | errors | still **accepts** (valid output) | B4 | T14.4 residual, recorded not dropped | `zic-pathology-ledger.md` |

## D. Out of scope by design (not divergences — refused capabilities)

These are not "differences from `zic`" — they are capabilities zic-rs deliberately does **not** offer, so a
reviewer never mistakes silence for a gap. `-p posixrules` (obsolete even in reference `zic` 2026b, which
itself warns *"-p is obsolete and likely ineffective"*) · `-u` ownership (privileged, deferred) · stdin
source `--input -` (documented non-capability) · any civil-time / display-name / CLDR / `tzselect` /
runtime-`localtime` role. See `docs/not-yet-ready.md` (the loud refusal surface) and `TRUST.md` §6.

## Maintenance rule

Every operational milestone adds its divergence rows here **in the same batch** as the code + tests + the
`differences-from-reference-zic.md` row that make them real. A defect moves from §A's "before" to "after"
only with a landed regression test. A tracked gap in §C closes only when its owner ships the fix + names the
evidence — never by quietly deleting the row.
