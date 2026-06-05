# Review zic-rs in 10 minutes

> **This page is not new evidence. It is the shortest safe path through the existing evidence.** Every
> claim below points to a receipt; nothing here is true because this page says so. If this page ever
> disagrees with [STATUS.md](../STATUS.md), STATUS.md wins (it is the live current-state authority).

Read top to bottom. ~10 minutes. Deeper entry points: [STATUS.md](../STATUS.md) (live state) ·
[TRUST.md](../TRUST.md) (the evidence court) · [docs/reviewer-orientation.md](reviewer-orientation.md)
(the milestone map).

## 1. What zic-rs is (60 s)

A **memory-safe Rust TZif compiler**: it compiles a declared subset of IANA tzdb source (`Zone`/`Rule`/
`Link`) into binary **TZif** files per [RFC 9636](https://www.rfc-editor.org/rfc/rfc9636), and checks
every supported construct against the canonical C `zic`/`zdump`. Library-first (`tzcompile`), thin CLI,
`#![forbid(unsafe_code)]`. It is a **producer/verifier**, upstream of datetime *consumers* (jiff, tz-rs).

The honest one-liner: *a reference-admitted Rust TZif compiler candidate whose claims are typed,
witnessed, report-backed, and bounded — earning each claim through admitted sources, oracle checks, and
machine-readable receipts, not asserted from "it's Rust."*

## 2. What zic-rs is not (60 s) — the refusal surface, not weakened

- **Not civil-time / timezone truth.** It compiles IANA source and checks against reference `zic`; it
  does not curate data, define names, or settle legal time. (See [not-yet-ready.md](not-yet-ready.md).)
- **Not a universal / argv-level `zic` replacement.** The CLI is deliberately divergent + safer
  (subcommands, required `--out`). Behaviour/flag-concepts compatible; argv is not a drop-in by design.
- **Not exhaustively fuzzed.** A *bounded* smoke ran (found+fixed 3 bugs); no coverage-saturating campaign.
- **Not full supply-chain verification.** 23 dependencies remain explicitly exempted/UNAUDITED.
- **Not signed attestation.** Reports are `unsigned_local_report`; no SBOM/SLSA/signed release yet.

Full list with guards: [STATUS.md](../STATUS.md) "Current non-claims" · [risk-register.md](risk-register.md).

## 3. What may be relied on today (90 s) — each with its receipt

| you can rely on | receipt |
|---|---|
| **CORE.1** — 341/341 canonical zones behaviour-match reference `zic`/`zdump` over 1900..2040 (0 mismatch) | the full sweep (§4) · `semantic-report` |
| **Provenance breadth** — 40 diversity-selected stress zones from sig-verified pristine 2026b → **40/40 `zdump`-MATCH** | [T18.breadth-to-40 receipt](../reports/provenance/RECEIPT-2026-06-04-breadth-to-40.md) · [provenance ledger](provenance-ledger.md) |
| **Cross-reader compat** — glibc 2.43 · Go 1.26 · CCTZ all read zic-rs output ≡ reference, **0 mismatch** | [T23.reader-compat.3](../reports/reader-compat/RECEIPT-T23-reader-compat-3.md) |
| **No-panic on the known hostile inputs** — bounded fuzz found 3 panics, all fixed + regression-tested | [cargo-fuzz.1](../audits/cargo-fuzz/receipts/RECEIPT-2026-06-04.md) · [.2](../audits/cargo-fuzz/receipts/RECEIPT-2026-06-04-fuzz2.md) |
| **Bounded formal proofs** — 10 Kani helper invariants verified (count/cursor/index/standards predicates) | [audits/kani/receipts/](../audits/kani/receipts/) |
| **Dependency vetting** — `cargo vet` 42 fully · 1 partial · 23 exempted (20 first-party full reviews; host-vs-all-target reachability recorded) | [cargo-vet.8](../audits/cargo-vet/receipts/RECEIPT-2026-06-05-pass8.md) |
| **`zic` flag/mode coverage** — 19 flags run on both compilers, classified (11 match · 3 intentional-divergence · 0 divergent — the `-D` one it found was fixed) | [ZIC-MATRIX.1](zic-operational-parity-matrix.md) |
| **Release-era reach** — 7 sig-verified tzdb releases (2015g→2026b); 7-zone fixture set behaviour-matches reference, 49/49 zdump, 0 divergent | [RELEASE-LADDER.1](release-ladder.md) |
| **Complete IANA archive index** — all 785 release entries (1996l→2026b) classified; 571 archives; 55/55 signed-bundle+pilot GOODSIG | [RELEASE-ALL.1](iana-release-archive-ledger.md) |
| **Evidence atlas** — upstream×vendor×drop-in joined; differences attributed by axis; 0 zic-rs divergence; Latin-1 band + all 66/66 yearistype band closed (LEGACY-SOURCE.1 · YEARISTYPE.1 · PERPETUAL-EXPANSION.1) | [TZDB-ATLAS.4](tzdb-evidence-atlas.md) |
| **Quality** — 520 tests; `fmt`/`clippy -D warnings`/`doc` clean; zero `unsafe` | `cargo test` (§4) |

The single live list: **[STATUS.md](../STATUS.md)**.

## 4. Fastest reproduction path (2–3 min, needs the repo + reference `zic`/`zdump` 2026b)

```sh
cargo test                                                              # 520 tests green
cargo run -- support-report   --input /usr/share/zoneinfo/tzdata.zi     # the compile frontier map
cargo run -- structural-report --input /usr/share/zoneinfo/tzdata.zi --reference-zic zic   # byte/structural parity
```

- **CORE.1 (the 341/0/0 behaviour sweep)** = compile every canonical zone in `tzdata.zi` and compare each
  via `zdump` against reference `zic` over 1900..2040. The convenience full-sweep harness is operator-local;
  `cargo test` already exercises the deep-semantics + regression suite, and `structural-report` shows the
  per-zone parity classes.
- **Provenance archive:** `bash reports/provenance/gauntlet.sh` (re-fetches + sig-verifies pristine 2026b).
- **Cross-reader:** `reports/reader-compat/r3/` (probes + `run.py`).

## 5. Where the strongest receipts live (90 s)

- **Trust map / evidence court:** [TRUST.md](../TRUST.md) — every trust question → its artifact.
- **Anti-drift governor:** [audits/claim-boundary-map.md](../audits/claim-boundary-map.md) — each audit/proof
  names the *lie it prevents*.
- **Source admission (sig + hash):** [reports/t12_5a2-reference-admission.md](../reports/t12_5a2-reference-admission.md)
  (pristine 2026b, VALIDSIG `7E37…7E34`, sha `ffad46a0…`).
- **Real release diff:** [reports/release-diff/RECEIPT-2026a-2026b.md](../reports/release-diff/RECEIPT-2026a-2026b.md).
- **Drop-in / cross-platform evidence:** `reports/drop-in/` + the external vendor-oracle lab.

## 6. Current audit board (60 s)

All 17 tools run/confirmed (2026-06-05); **not all green, by design** (a folder existing is not a result):

| tool | state |
|---|---|
| `cargo-audit` | ✅ clean (0 vulns / 67 deps) |
| `cargo-geiger` | ✅ zic-rs own 0 unsafe; dep `unsafe` localised to syscall layers |
| `cargo-auditable` | ✅ embedded dependency manifest |
| `miri` | ✅ 0 UB on the `tzif` core |
| `panic-analysis` | ✅ census vs `panic-policy.md` |
| `dsfb-gray` | ✅ claim-boundary self-audit |
| `kani` | ✅ **10 bounded helper proofs verified, 0 fail** |
| `cargo-fuzz` | ✅ **T23.cargo-fuzz.1/.2** — bounded smoke found 3 panics, fixed 3/3, re-ran 9/9 clean |
| `cargo-vet` | ◐ **T23.cargo-vet.8** — 42 fully · 1 partial · 23 exempted (20 first-party; host-vs-all-target reachability recorded) |
| `cargo-valgrind` | ◐ inconclusive — valgrind SIGILLs on `/bin/true` here (env/CPU, **not** a zic-rs finding) |
| all 17 audit tools (2026-06-05 sweep) | ✅ **every tool RUN/confirmed** — creusot **PROVED** (Why3+SMT) · crux-mir **5/5 Valid** · loom **proved** (fetch_add invariant) · flux/hax ran · cargo-crev (empty WoT) · cargo-scan (stub) · cargo-valgrind→ASan clean. The count-arithmetic invariant is proven by **3 engines** (Kani · crux-mir · creusot) |

Detail: [audits/README.md](../audits/README.md) · [audits/index.html](../audits/index.html).

## 7. Current open debts (45 s)

- **cargo-vet.9+** — the 23 remaining exemptions (large syscall/serde/clap/proc-macro tiers; several not even host-reachable) stay honestly exempted; `semver`/`log` were audited in `.6`, errno/anstream/anstyle-wincon in `.8`.
- **Exhaustive fuzzing** — a coverage-saturating (24 h) campaign; the operator harness (`fuzz/run-long-burnin.sh`) exists + a bounded 9-target smoke ran clean, but the 24h burn-in is a future operator step.
- **Supply-chain attestation** — SBOM / SLSA / signed releases (planned, not present).
- **TOCTOU** — the parent-component symlink-swap race (`RequiresOpenatStyleHardening`).
- **Whole-tree crash-atomic install** — per-file durable (Unix) only.

Full backlog: [STATUS.md](../STATUS.md) "Known open debts".

## 8. How to avoid misreading historical receipts (45 s)

The project is receipt-bearing and long-running, so some receipts are **accurate-at-seal, not current**.
Two rules keep you safe:

1. **[STATUS.md](../STATUS.md) is the only live authority.** Milestone-ladder rows and `reports/t*-close-receipt.md`
   describe their seal moment; where they differ from STATUS.md, STATUS.md wins (close receipts carry a
   "historical receipt" banner saying what later superseded them — e.g. T17 closed with the fuzz harness
   `pending_capture`; it was *later* executed at T23.cargo-fuzz.1/.2).
2. **A drift gate enforces this.** [`scripts/doc-staleness-check.sh`](../scripts/doc-staleness-check.sh)
   (DOC-CURRENCY.1) fails if a front-door doc asserts a superseded status without a reconciliation marker.
   Run it: `bash scripts/doc-staleness-check.sh`.

If you read only one thing after this page: **[STATUS.md](../STATUS.md)**.
