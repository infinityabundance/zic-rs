# TRUST.md — the front door into the evidence court

> **This is not a marketing summary. It is the entry point into the evidence court.** A reader may start
> here, but **no claim becomes true because TRUST.md says it** — claims are true only through the
> underlying typed reports, receipts, hashes, tests, schemas, ledgers, and non-claims this file points
> into. Every row below links *deeper*; nothing here replaces detail.
>
> **For the single live current-state answer, see [STATUS.md](STATUS.md)** — current claims, current
> non-claims, evidence receipts, and open debts, kept up to date with every campaign.
>
> **Reviewer fast path:** [docs/REVIEW-IN-10-MINUTES.md](docs/REVIEW-IN-10-MINUTES.md) — the shortest safe
> route through this evidence court (≈10 min), then return here for the per-question detail.

## 1. The claim (lead with this)

**zic-rs is a reference-admitted Rust TZif compiler candidate.** It compiles IANA tzdb source text
(`Zone`/`Rule`/`Link`) → binary **TZif** (RFC 9636) and checks itself against reference `zic`/`zdump`. It
does not ask to be trusted because it is rewritten in Rust; it **earns each claim** through admitted
sources, typed contracts, oracle checks, and machine-readable reports.

**The live behaviour claim (never inflated):** *zic-rs behaviour-matches reference `zic`/`zdump` for all
**341 canonical zones** in `tzdata.zi` **2026b** over **`1900..2040`** — 341 match · 0 mismatch · 0
fail-closed (CORE.1).* `zic` is the authority; zic-rs is a measured reimplementation candidate.

## 2. Two binding rules

> **Reference first · evidence second · behaviour third · claims last.**
>
> **If a human can read it as a claim, a machine must be able to locate the field that proves or bounds it.**

## 3. Reproduce it (the gate)

```sh
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test   # all green (count: STATUS.md)
bash /tmp/t9sweep.sh                                                           # CORE.1 → 341/0/0
zic-rs support-report --input /usr/share/zoneinfo/tzdata.zi --format json      # the conformance surface
```

`#![forbid(unsafe_code)]` · no `build.rs` · minimal dependencies · `overflow-checks` in all profiles ·
deterministic output independent of host `TZ`/`LC_ALL` (`tests/reliability.rs`).

## 4. Parity is several separate axes (never collapsed)

| Axis | Claim | Where it is proven / bounded |
|---|---|---|
| **Behaviour** | 341/341 over 1900..2040 (the contract) | CORE.1 sweep · `semantic-report` (`zdump` witnesses) |
| **Structural** | measured per-zone vs reference `zic` | `structural-report` (`ParityClass`); slim/fat |
| **TZif format** | RFC 9636 structural validity (≠ semantic) | `tzif-validate` (5 separate verdicts) |
| **Diagnostic** | class/location vs `zic -v` (wording last) | `ZIC001`–`ZIC026` (`reports/t13/t14-close-receipt.md`) |
| **Operational** | CLI/filesystem/install (safer defaults labelled) | `docs/zic-operational-parity.md`; `docs/differences-from-reference-zic.md` |

A **safer default is not "parity"** — it is a labelled bucket-3 divergence
(`docs/differences-from-reference-zic.md`, the four-bucket map).

## 5. The evidence map (each trust question → its artifact)

| You want to know… | Read |
|---|---|
| does it match reference behaviour? | the CORE.1 sweep · `semantic-report` (`oracle_mode` visible) |
| is the TZif structurally valid? | `tzif-validate` (`zic-rs-tzif-validation-v1`) |
| what diagnostics, by class/severity/span? | `docs/zic-warning-parity.md` · `reports/t13/t14-close-receipt.md` |
| which release/source is admitted? | the manifest `tzdb` block · `reports/t12_5a2-reference-admission.md` (only 2026b) |
| is the evidence cherry-picked, or does it cover the hard cases? | **`docs/provenance-ledger.md`** (T18.3) — 40 diversity-selected stress zones (15 trap categories), **40/40 `zdump`-MATCH** vs reference `zic` from sig-verified pristine 2026b; 80 stored witnesses; claims compiler-equivalence + source-provenance, **not** historical truth |
| which `zic` CLI flags/modes are matched vs intentionally-different vs unsupported vs deferred? | **`docs/zic-operational-parity-matrix.md`** (ZIC-MATRIX.1) — 19 flag/mode rows, **each backed by a real run** on both compilers; strict verdicts (match / class-parity / intentional-divergence / unsupported-by-design / deferred / divergent); one named `-D` finding |
| does the compiler hold across tzdb source eras, or only 2026b? | **`docs/release-ladder.md`** (RELEASE-LADDER.1) — 7 signature-verified tzdb releases (2015g→2026b); the 7-zone fixture set **behaviour-matches reference `zic`** (49/49 `zdump`, 0 divergent); 3 non-fixture zones source-incompatible in 2018e/2020a (deferred features). Bounded fixture-match, **not** full admission |
| where is a difference attributable (which evidence axis)? | **`docs/tzdb-evidence-atlas.md`** (TZDB-ATLAS.4) — joins upstream-archive × vendor-oracle × drop-in × reader so a difference is explained by axis (upstream-data / reference-`zic` / vendor-`zic` / reader / drop-in / zic-rs / bounded-legacy-source); **0 zic-rs divergence where both build**; the Latin-1 band **closed by LEGACY-SOURCE.1** and **all 66/66** of the `yearistype` band **closed by YEARISTYPE.1 + PERPETUAL-EXPANSION.1** (historical-source replay vs an admitted old-`zic` oracle over [1980,2037]); **no stable-release replay band remains**; every vendor-shipped tzdb release falls in the match band |
| what is the complete upstream IANA release archive — and does every release behaviour-match? | **`docs/iana-release-archive-ledger.md`** — RELEASE-ALL.1 mapped the **whole** directory (**785 entries**, 571 archives classified, 55/55 GOODSIG); **RELEASE-ALL.DATA.1 compiled all 276 stable tzdata** (1993→2026b) → **150 match · 66 ref-build-incompat (`yearistype`) · 60 source-shape-incompat (UTF-8 vs Latin-1) · 0 zic-rs-divergent**. Axis 1 (behaviour) of the planned TZDB Evidence Atlas (× vendor-oracle × drop-in) |
| do real TZif readers consume zic-rs output like reference output? | `reports/reader-compat/` — `T23.reader-compat.1/.2` (`zdump` + Python `zoneinfo`) + **`.3`** cross-reader ecology (glibc 2.43 · Go 1.26 · CCTZ/abseil all see zic-rs ≡ ref over 6 fixtures, **0 mismatch**; Java/PHP/ICU classified `unsupported_by_reader` — they consume a compiled DB, not raw TZif). **Not** universal consumer compatibility |
| what does "system `zic`" actually mean across vendors? | the vendor-oracle lab (17 rows / 19 receipts) · `../zic-rs-vendor-oracle-lab/RECEIPT-MATRIX.md` |
| how can it hurt me, and what guards each case? | **`docs/risk-register.md`** (15 claim-boundary risks) |
| what does the rewrite actually remove vs not? | `docs/security-rewrite-evaluation.md` (threat table · the honest rewrite-safety delta) |
| what may my audience conclude — and NOT conclude? | `docs/security-personas.md` (10 personas; the *may-NOT-conclude* column is the loudest) |
| how much time/memory does it cost, and is it bounded? | `docs/perf-ledger.md` + `reports/perf/` (measured + bounded receipts; deterministic anchors vs indicative timings; **not** a speed claim) |
| which parts touch the host (fs/process/env)? | `docs/effect-boundary-map.md` |
| no panic on hostile input? counts checked? | `docs/panic-policy.md` · `reports/t17-count-arithmetic-verdict.md` |
| install durability / path safety? | `docs/install-materialization-contract.md` |
| are the reports/CLI stable contracts? | `docs/schema-compatibility-policy.md` · `docs/cli-compatibility-policy.md` · `schemas/` |
| where do external facts come from? | `docs/zic-knowledge-index.md` + `claim-source-map.md` (T18) |
| what is it NOT ready for? | **`docs/not-yet-ready.md`** |
| how do I adopt it safely? | `docs/replacement-readiness-ladder.md` · `docs/drop-in-compatibility-contract.md` |
| how do I avoid believing something false? | `docs/misuse-resistance-ledger.md` |
| why this method (precision ladder · "trust earned, not from Rust")? | **`docs/PORTING-PHILOSOPHY.md`** (the front door to the doctrine) |
| which shortcuts were declined, and why? | **`docs/PORTING-DECISION-LEDGER.md`** (the rejected-shortcut ledger) |
| every known difference from reference `zic`, in one place? | **`docs/DIVERGENCE-REGISTER.md`** (defects-fixed · deliberate · tracked-gaps) |
| how do I audit it? | `docs/audit-readiness.md` |
| how is it maintained / who do I contact? | `docs/maintenance-policy.md` · `SECURITY.md` · `CONTRIBUTING.md` |

## 6. The exact non-claims (the refusal surface — guarded as hard as the success surface)

zic-rs **does not** claim: a full/universal drop-in `zic` replacement · civil-time-truth authority (IANA/CLDR
own that) · display-name/`tzselect`/CLDR-runtime territory · a signed attestation (reports are
`unsigned_local_report`) · whole-tree crash-atomic install (per-file durable on Unix only) · closure of the
parent-component symlink-swap race (needs `openat`; `RequiresOpenatStyleHardening`) · **exhaustive**
fuzzing (a *bounded* smoke ran — T23.cargo-fuzz.1/.2, found+fixed F1–F3, re-ran clean — but it is not a
coverage-saturating campaign) · schema instance-validation in the core (no
validator dep; registry/drift only) · vendor-family parity (a receipt is one measured ecology) · leap-smear
semantics. The machine-checkable list is `negative_capabilities` in `support-report` (each with its
`enforced_by` guard); the consequences are `docs/risk-register.md`.

## 6a. Four authorities — who owns what (never conflate them)

A claim is only as good as *which authority* backs it. zic-rs sits in exactly one of these lanes and defers
the other three (per RFC 6557/BCP 175 + RFC 9636):

| Authority | Owner | zic-rs's relationship |
|---|---|---|
| **Civil-time policy** (what the law/clocks are) | governments · on-ground consensus · the IANA tz process | **deferred** — zic-rs does not set or judge civil time (`RISK.TIME.1`) |
| **Release-artifact authority** (which bytes are *the* release) | admitted tzdb release tarballs · signatures · hashes | **admitted, recorded** — fetch + verify-sig + hash-pin (only 2026b; `reports/t12_5a2-…`) |
| **Compiler-behaviour authority** (what the bytes *do*) | reference `zic`/`zdump` (the oracle) + zic-rs's typed reports | **measured** — this is zic-rs's lane (CORE.1, the `*-v1` reports) |
| **Distribution authority** (what a platform actually ships) | distro packages · the vendor-oracle lab | **observed per receipt** — admitted receipts, never a family theorem (`RISK.VENDOR.1`) |

*A proof of compiler behaviour is not release-artifact authority; an admitted release is not civil-time
truth; a vendor receipt is not distribution-wide policy.* Numbering, names, and these lanes are evidence
boundaries.

## 7. Status & scope

Pre-1.0. Behaviour claims are **release-scoped** to the admitted release (2026b). Adoption is **staged**
(see `docs/replacement-readiness-ladder.md`): zic-rs is strong **now** as a validator / report generator /
`doctor` / `release-diff` / bounded compiler candidate; it is **not** a default system-`zic` replacement.
Milestone ladder + history: the plan + the `reports/t*-close-receipt.md` receipts; current state: `CHANGELOG.md`.

> **Doctrine:** *TRUST.md is the map; the territory is the typed evidence. Start here, then go deeper —
> and if a claim here is not backed by a linked report/receipt/test/schema/ledger, treat it as a bug.*
