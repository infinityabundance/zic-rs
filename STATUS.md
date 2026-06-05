# STATUS — what may I rely on today?

> **Start here:** if you're new, read [docs/REVIEW-IN-10-MINUTES.md](docs/REVIEW-IN-10-MINUTES.md) first
> (the guided 10-minute path), then this page for the precise live state.
>
> **This is the single current-state page.** When a close receipt, ladder row, or older doc disagrees with
> this file, **this file wins** (those are historical-at-seal; this is kept current with every campaign).
> Last updated: campaign **AUDIT-SUITE-RUN.1** (all 17 audit tools actually run/confirmed; after LONG-FUZZ-SMOKE.1 · LONG-FUZZ-HARNESS.1 · T23.cargo-vet.8 · ATLAS-FORMAT.1 ·
> CROSS-FS.1 · LINK-MATERIALIZATION.1 · PACKAGER-POLICY.1 · INSTALL-SEMANTICS.1 · SHIM-CONTRACT.1 · DOC-CURRENCY.3 · HASH-HYGIENE.1).

## Current claims (what zic-rs does, today, with receipts)

- **Compiles a declared subset of IANA tzdb 2026b source into RFC 9636 TZif.** Library-first (`tzcompile`),
  thin `zic-rs` CLI; `#![forbid(unsafe_code)]`.
- **CORE.1 behaviour match — 341 / 341 canonical zones** behaviour-match reference `zic`/`zdump` over
  1900..2040 (0 mismatch · 0 fail-closed). The standing regression gate. → `bash /tmp/t9sweep.sh`.
- **Provenance breadth (T18.breadth-to-40):** 40 diversity-selected stress zones from the **signature-verified,
  hash-pinned pristine 2026b** → **40 / 40 behaviour `zdump`-MATCH**; readable ledger `docs/provenance-ledger.md`.
- **Cross-reader compatibility (T23.reader-compat.1/.2/.3):** every raw-TZif reader tested — **glibc 2.43 · Go
  1.26 · CCTZ/abseil** — interprets zic-rs output **identically to reference-`zic`** over the fixture set
  (**0 mismatch**); `zdump` + Python `zoneinfo` also agree. (Java/PHP/ICU consume a pre-compiled DB, not raw
  TZif → classified, not scored.)
- **Release-era reach (RELEASE-ALL.DATA.1):** **all 276 stable tzdata archives** (1993→2026b) compiled vs reference `zic` + zic-rs → **150 match · 66 reference-build-incompatible (`yearistype`, a *reference* limit — all 66 now replay via `--legacy-yearistype`+`--legacy-empty-footer` vs a historical oracle) · 60 source-shape-incompatible (Latin-1 — CLOSED by LEGACY-SOURCE.1) · 0 zic-rs-divergent** (1050/1050 fixtures behaviour-match where both built). **Under the explicit replay modes: 276/276 attempted, 0 zic-rs divergence** (the historical bands vs an admitted oracle). RELEASE-LADDER.1 (7 releases) is the pilot.
- **Bounded legacy-source replay (LEGACY-SOURCE.1):** `--legacy-latin1` admits non-UTF-8 (Latin-1) bytes
  **only inside `#` comments** of pre-2013 historical tzdb source (refused in any semantics-bearing field);
  the whole 2008a–2012j band (**60 releases, 420/420 fixtures**) now behaviour-matches reference `zic`. The
  default remains UTF-8-required (`ZIC012`) — the modern source contract is unchanged.
- **Historical Rule TYPE replay (YEARISTYPE.1):** `--legacy-yearistype` admits the obsolete `yearistype`
  predicates (`even`/`odd`/`uspres`/`nonpres`, anchored to `yearistype.sh` v7.4) as **internal deterministic
  functions** (zic-rs **never executes** the historical script); default rejects any non-`-` TYPE (`ZIC027`).
  **55/66** pre-2000f releases build & match an **admitted historical `zic` oracle** byte-identically
  (354/354 fixtures, 0 divergent) — *historical-source replay, not current-reference parity* (current `zic`
  removed `-y` in tzcode 2020a). The 11 pre-1995 perpetual-year-parity releases (whose recurring tail a POSIX
  footer cannot encode) are **closed by PERPETUAL-EXPANSION.1** (next bullet).
- **Empty-footer fallback (PERPETUAL-EXPANSION.1):** `--legacy-empty-footer` — on the `ZIC001`
  footer-synthesis-**failure** path only, emit the explicit transitions zic-rs already expands through
  `RECUR_HI`=2037 plus an **empty footer** (frozen beyond), instead of failing closed. A **footer-emission
  policy** (no transition-generation change). Closes the last 11 yearistype-band releases → **all 66/66 now
  behaviour-match the historical oracle over [1980,2037]** (371/371 fixtures, 0 divergent). Default still fails
  closed; footer-synthesising zones (all of CORE.1) are **byte-unchanged**; the beyond-horizon freeze is not claimed.
- **Install-semantics parity (INSTALL-SEMANTICS.1):** 13 package-relevant install scenarios vs reference
  `zic` (staged root, no host mutation) → **9 match · 2 install-policy-difference** (atomic no-clobber;
  copy-not-hardlink) **· 1 safer-no-partial · 1 unsupported-by-design · 0 divergent**. **Found + fixed a real
  divergence:** zic-rs created files at base mode **0666** (world-writable under umask 000) vs reference
  **0644** → now 0644 base (matches reference, never world-writable; CORE.1 byte-unchanged; +1 regression
  test). umask/`-m`/`-D`/symlink-to-outside-root all match.
- **Quality:** **520 tests**; `fmt` / `clippy -D warnings` / `doc` clean; zero `unsafe` in the crate.
- **Audit board (T23):** `cargo-audit` clean (0 vulns) · `cargo-geiger` (own 0 unsafe) · `cargo-auditable` ·
  `miri` (0 UB on tzif core) · `panic-analysis` · `dsfb-gray` · **`kani` 10 bounded helper proofs verified
  (0 fail)** · **`cargo-fuzz` bounded smoke found + fixed 3 panic bugs (F1–F3), re-ran 9/9 clean** ·
  **`cargo-vet` 42 fully · 1 partial · 23 exempted** (20 first-party full-source reviews — the unsafe-heavy
  tier itoa/anstyle-parse/anstyle-query + semver/log + thiserror/id-arena/unicode-ident/windows-link + now
  errno/anstream/anstyle-wincon, every `unsafe` site reasoned sound — T23.cargo-vet.8, with host-vs-all-target
  reachability recorded; the large syscall/serde/clap/proc-macro tiers stay honestly exempted). **Board closure (T23.AUDIT-BOARD.1):**
  every one of the 17 tools was **actually run/confirmed (2026-06-05 sweep) — 0 placeholders left**:
  **creusot ✅ RAN + PROVED** (Why3+SMT discharge ✔, after building the matched why3 from source) ·
  **crux-mir ✅ 5/5 goals Valid** · **loom ✅ proved** the `fetch_add` unique-sequence invariant (corrects
  the prior wrong N/A) · flux ✅ ran clean · hax ✅ ran (frontend extraction) · cargo-crev ✅ ran (empty WoT) ·
  cargo-scan ✅ confirmed non-functional stub · cargo-valgrind → **ASan** ✅ ran clean (151 tests, 0 errors).
  **The count-arithmetic invariant is now proven by THREE independent engines — Kani (CBMC) · crux-mir
  (Crucible) · creusot (Why3+SMT).** (The formal tools prove reduced-surface invariants only, not whole-crate correctness.)
- **Operational:** drop-in evidence across host + container + multiple real VMs + BSD/illumos source-builds;
  the crates.io source crate is the portable unit (output `bundle_hash` byte-identical across builds).

## Current non-claims (explicitly NOT claimed)

- **Not civil-time / timezone truth.** zic-rs compiles IANA source and checks itself against reference `zic`;
  it does not curate data, define display names, or settle legal time. (IANA / CLDR / governments own those.)
- **Not a universal / argv-level `zic` replacement.** The CLI shape is deliberately divergent + safer (subcommands,
  required `--out`). Behaviour + flag-concepts are compatible; argv is not a drop-in by design.
- **Not exhaustively fuzzed.** A *bounded* smoke ran (found + fixed F1–F3); no coverage-saturating campaign.
  The panic-policy "no panic on hostile input" claim is restored **only** for the known F1–F3 seeds + that rerun.
- **Not full supply-chain verification.** 23 dependencies remain explicitly exempted/UNAUDITED.
- **Not signed attestation.** Reports are `unsigned_local_report`; no SBOM/SLSA/signed release yet (planned).
- **Reader-equivalence ≠ compiler-equivalence ≠ civil-time truth.** All kept as separate axes.
- **Per-release / per-profile, not "all tzdb releases."** Only 2026b is **fully admitted** (signature +
  hash + CORE.1). RELEASE-LADDER.1 adds a *bounded fixture-set behaviour-match* across 7 signature-verified
  releases — that is era-reach evidence, **not** full admission of those releases.

## Current evidence receipts (where the claims are proven)

| claim | receipt |
|---|---|
| behaviour match (all canonical zones) | CORE.1 sweep · `semantic-report` (`zdump`) |
| provenance breadth (40 stress zones) | `reports/provenance/RECEIPT-2026-06-04-breadth-to-40.md` · `docs/provenance-ledger.md` |
| cross-reader ecology | `reports/reader-compat/RECEIPT-T23-reader-compat-3.md` (+ `.1`/`.2`) |
| fuzz found+fixed | `audits/cargo-fuzz/receipts/RECEIPT-2026-06-04.md` (found) · `…-fuzz2.md` (fixed+reran) |
| dependency vetting | `audits/cargo-vet/receipts/RECEIPT-2026-06-05-pass8.md` |
| standards currency (RFC 9636: **0 errata**) | `reports/rfc9636-errata/RECEIPT-RFC9636-ERRATA-1.md` |
| release-identity provenance (276 releases; **0 major** contradictions, 2 minor recorded) | `reports/release-metadata/RECEIPT-RELEASE-METADATA-1.md` |
| complete-bundle phase (**48/48** signed data+code bundles, data == standalone pair) | `reports/release-all/RECEIPT-RELEASE-ALL-TZDB-1.md` |
| source-profile parity (main/vanguard/rearguard + backzone — **all behaviour-match**) | `reports/source-variant/RECEIPT-SOURCE-VARIANT-1.md` |
| failure-mode parity (17 hostile fixtures; **14 class+exit match · 0 divergent**, 3 safer/intentional) | `reports/diag-parity/RECEIPT-DIAG-PARITY-1.md` |
| package-build slot acceptance (posix+right trees identical; only safer install policies differ) | `reports/package-acceptance/RECEIPT-PACKAGE-ACCEPTANCE-1.md` |
| shim compatibility contract (`zic-shim-v1`; conformance test 8/8 — found+fixed a word-split bug) | `reports/package-acceptance/SHIM-CONTRACT.md` · `shim-test.sh` |
| install-semantics parity (13 scenarios; **9 match · 0 divergent**; found+fixed 0666→0644 mode) | `reports/install-semantics/RECEIPT-INSTALL-SEMANTICS-1.md` |
| packager policy matrix (4 works-with-shim · 2 recipe-adaptation · **0 blocked**; crates.io source crate, no binary) | `reports/packager-policy/RECEIPT-PACKAGER-POLICY-1.md` |
| link-materialization footprint (copy 2.13× installed but **only 1.05× xz-shipped**; symlink option measured) | `reports/link-materialization/RECEIPT-LINK-MATERIALIZATION-1.md` |
| cross-fs/rootless materialization (**bundle_hash identical** across 2×ext4+tmpfs; relocation-safe; rootless) | `reports/cross-fs/RECEIPT-CROSS-FS-1.md` |
| atlas machine-readable exports (40-row TSV→**NDJSON+JSON-Schema+HTML**, 40/40 validate; external-consumer) | `reports/tzdb-atlas/RECEIPT-ATLAS-FORMAT-1.md` |
| Kani proofs | `audits/kani/receipts/` · `reports/t17-count-arithmetic-verdict.md` |
| source admission (sig+hash) | `reports/t12_5a2-reference-admission.md` |
| real release diff (2026a→2026b) | `reports/release-diff/RECEIPT-2026a-2026b.md` |
| which `zic` flags/modes are covered (match/divergent/unsupported/deferred) | `docs/zic-operational-parity-matrix.md` (ZIC-MATRIX.1; 19 rows, each a real run) |
| does the compiler hold across tzdb source eras (not only 2026b)? | `docs/release-ladder.md` · `reports/release-ladder/RECEIPT-RELEASE-LADDER-1.md` (RELEASE-LADDER.1; 7 sig-verified releases, 49/49 fixtures behaviour-match) |
| what is the COMPLETE upstream IANA release archive (every artifact, classified + provenance)? | `docs/iana-release-archive-ledger.md` · `reports/release-all/` (RELEASE-ALL.1; **785 entries indexed**, 571 archives classified, 55/55 signed-bundle+pilot GOODSIG) |
| does every stable tzdata release behaviour-match (across 30 years)? | `reports/release-all/RECEIPT-RELEASE-ALL-DATA-1.md` (RELEASE-ALL.DATA.1; **all 276 compiled** → 150 match · 66 ref-build-incompat · 60 source-shape-incompat · **0 divergent**; named yearistype + UTF-8 breakpoints) |
| where is a difference attributable (upstream-data / reference-zic / vendor-zic / reader / drop-in / zic-rs / bounded-legacy-source)? | **`docs/tzdb-evidence-atlas.md`** (TZDB-ATLAS.4) — joins upstream×vendor×drop-in×reader so a finding is explained by axis; **0 zic-rs divergence where both build**; the Latin-1 band **closed by LEGACY-SOURCE.1** and **all 66/66** of the `yearistype` band **closed by YEARISTYPE.1 + PERPETUAL-EXPANSION.1** (vs a historical oracle over [1980,2037]); **no stable-release replay band remains**; every vendor-shipped release in the match band |
| the whole trust map | `TRUST.md` · the anti-drift `audits/claim-boundary-map.md` |

## Known open debts (honest backlog)

- **cargo-vet.9+** — the remaining **23** exemptions stay honestly exempted: the large syscall/platform tiers
  (`windows-sys`, `linux-raw-sys`, `rustix`, `libc`, `r-efi`, `memchr`, `hashbrown`) and the big proc-macro/
  serde/clap tiers (`syn`, `serde*`, `clap*`, `getrandom`, `anyhow`, …) — several **not even host-reachable**
  (serde* are all-target/dev-graph only; reachability recorded in pass8). **`.8` admitted errno/anstream/
  anstyle-wincon (39→42 fully · 26→23 exempted), with host-vs-all-target reachability mapped** — after `.6`
  (`semver`+`log`) and `.7` (`thiserror`/`id-arena`/`unicode-ident`/`windows-link`, T23.AUDIT-BOARD.1). Slow by
  design; nothing admitted without a full-source review.
- **Exhaustive fuzzing** — a coverage-saturating (24 h) campaign; the bounded smokes are only a start. The
  reproducible **operator harness exists** (`fuzz/run-long-burnin.sh`, LONG-FUZZ-HARNESS.1 — provenance +
  receipt contract + `--campaign-hours`) and a **bounded 9-target × 5-min smoke ran `clean`** (LONG-FUZZ-SMOKE.1,
  0 crashes; `RECEIPT-LONG-FUZZ-20260605T090115Z.md`) — *not saturation*; the 24h-class run on a dedicated host
  is the future operator step.
- **Supply-chain attestation** — SBOM / SLSA provenance / signed releases (planned, not present).
- **TOCTOU** — the parent-component symlink-swap race needs `openat`/`O_NOFOLLOW` (`RequiresOpenatStyleHardening`).
- **Whole-tree crash-atomic install** — per-file durable (Unix) only; no tree transaction.
- **T18 source breadth** — toward ~40 *sources* + adjacent-compiler/TZif-reader ledgers; S10/S13 archival pending.
- **CI wiring** — the panic-grep / cross-target `cargo check` gates are intent, not run here.

*(Closed this batch: the `-D` directory-creation divergence ZIC-MATRIX.1 found is now FIXED —
`ZIC-MATRIX.1.D`, matrix row `divergent → match`, regression-tested.)*

---

*Maintenance rule: update this page in the same batch as every campaign that changes a claim, non-claim,
receipt, or open debt. The `scripts/doc-staleness-check.sh` gate guards the front-door docs against drift.*
