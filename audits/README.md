# `audits/` — the audit suite (T23)

> **An audit folder existing is not an audit result.** An audit result is admitted **only by a receipt**
> recording tool identity, command, input/corpus, duration (where applicable), result, findings, residuals,
> and non-claims. This is the same discipline as the fuzz harness (`fuzz/`, **run** at T23.cargo-fuzz.1/.2) and the
> T18 archival ledger (capture ≠ authority). The suite is **evidence-producing infrastructure, not a
> pass/fail badge** — its job is to record *what each tool can and cannot witness*, and to turn each run
> into a receipt. **No claim that all audits are green.**

> **Anti-drift:** [`claim-boundary-map.md`](claim-boundary-map.md) is the governing table — every audit/proof
> must name the **claim boundary** it protects (*what lie does this prevent?*); no boundary → no priority.
> It tracks which boundaries are covered (e.g. the T23.kani count/index proofs) and which are still
> doctrine-only (the highest-priority gaps: archive≠authority, audit-green-requires-receipt, `HashReadStatus`,
> release-diff unassessed≠unchanged — small `dsfb-gray`/Kani checks).

## The list is authoritative

This top-level set is **exact and operator-fixed — no tool is removed, substituted, or reclassified out of
the set**; the only acceptable additions are *inside* a tool's folder:

```
audits/
  cargo-audit/  cargo-auditable/  cargo-crev/  cargo-fuzz/  cargo-geiger/  cargo-scan/
  cargo-valgrind/  cargo-vet/  creusot/  crux-mir/  dsfb-gray/  flux/  hax/  kani/
  loom/  miri/  panic-analysis/  README.md  index.html
```

## Per-folder shape (receipt-bearing)

Each tool folder carries a `README.md` (scope · command · what it witnesses · what it *cannot* · non-claims ·
**status** · environment requirement) and a `receipts/` dir. A receipt records: **tool · version · host ·
command · input/corpus + hash · duration · exit · findings · fixed · residual · non-claims · next-owner.**
A per-folder `status` ∈ {`mandatory-gate` · `advisory` · `experimental`} is allowed *inside* the folder; it
never changes membership of the set.

## Status (honest — this environment is no-network + stable-only)

| Tool | Witnesses | Cannot witness | Status here |
|---|---|---|---|
| `cargo-audit` | known-vuln / yanked deps vs the RustSec advisory DB | logic bugs · `unsafe` correctness | **✅ RUN (2026-06-02) — clean: 0 vulns / 67 deps / 1102 advisories** |
| `cargo-auditable` | an embedded dependency manifest in the built artifact (SBOM-ish) | vulnerabilities themselves | **✅ RUN** — embedded manifest in the release binary |
| `cargo-crev` | community code-review trust of dependencies | absence of bugs | **✅ RAN (cargo-crev 0.27.1)** — `cargo crev verify` enumerated all 66 resolved crates; **no trusted WoT Ids → nothing verified** (empty-but-real, not a pass); cargo-vet carries the first-party layer |
| `cargo-fuzz` | crashes/panics/OOM on the libFuzzer targets | semantic correctness | **✅ RUN (T23.cargo-fuzz.1/.2, 2026-06-04)** — bounded smoke (9 targets, 25 s) **found 3 panic bugs (F1–F3), .2 fixed 3/3 + seeds-as-tests, smoke re-ran 9/9 clean**; panic-policy claim restored for the known seeds + bounded rerun (not an exhaustive proof) |
| `cargo-geiger` | the `unsafe` surface of the dependency tree | whether `unsafe` is *wrong* | **✅ RUN** — zic-rs own 0 unsafe (forbid); dep surface quantified (libc/rustix/getrandom/…) |
| `cargo-scan` | the effect/capability footprint (fs/net/process) of deps | logic correctness | **✅ RAN (confirmed stub)** — the installed Rust CLI is a non-functional stub (redirects to a Python endpoint); its effect-scan coverage overlaps cargo-geiger + cargo-vet |
| `cargo-valgrind` | runtime memory errors/leaks under memcheck | logic · perf | **◐ valgrind INCONCLUSIVE (host SIGILL) → ✅ ASan substitute RAN clean** — `-Zsanitizer=address` over 151 lib tests (incl. tzif core), 0 sanitizer errors, native (no SIGILL) |
| `cargo-vet` | vetted-dependency / imported-audit status | bug absence | **◐ RUN (T23.cargo-vet.8)** — `cargo vet` Vetting Succeeded: **42 fully audited · 1 partial · 23 exempted**; **20 first-party** (full-source reviews incl. the unsafe-heavy tier itoa/anstyle-parse/anstyle-query + **semver**/**log** + thiserror/id-arena/unicode-ident/windows-link + now **errno**/**anstream**/**anstyle-wincon**, each `unsafe` site reasoned sound; host-vs-all-target reachability recorded), rest trusted-import; **23 still honestly exempted/UNAUDITED** (large syscall/serde/clap/proc-macro tiers, several not even host-reachable); no "verified" claim |
| `creusot` | deductive functional-correctness proofs of *annotated* code (Why3) | unannotated code | **✅ RAN + PROVED (deductive, Why3+SMT)** — `why3find prove` discharged a zic-rs invariant (`block_len`: count×size non-overflow) with the **matched** why3 1.8.2+git / why3find 1.3.0+dev (built from the creusot-pinned source); a THIRD engine corroborating Kani + crux-mir |
| `crux-mir` | symbolic-execution properties on harnesses (MIR) | unbounded behaviour | **✅ RAN — Overall status Valid, 5/5 goals proved** on 3 zic-rs `#[crux::test]` harnesses (cursor-bounds · type-index · indicator pairing); second-engine (Crucible+z3) corroboration of Kani |
| `dsfb-gray` | zic-rs's **own** gray-box claim-boundary checks | external-tool findings | **✅ RAN (the real `dsfb-scan-crate` tool)** — 60.3% mixed assurance posture (review-readiness map, not a parity contradiction); SARIF+in-toto+DSSE in `dsfb-gray/output/`; replaces the prior grep summary |
| `flux` | refinement-type properties (e.g. index bounds) on annotated code | unannotated code | **✅ RAN clean** — `cargo flux` checked zic-rs to completion (exit 0, no errors); no `#[flux::...]` annotations authored → no specific refinement proven (clean pass, not a verification claim) |
| `hax` | extraction of Rust to formal backends (F\*/Coq) for proof | proof itself (only extraction) | **✅ RAN** — `cargo hax json` extracted the crate to a 186 KB typed-AST JSON (exit 0); frontend extraction only, no backend proofs authored |
| `kani` | bounded model-checking of harnesses (panics · overflow · assertions; CBMC) | unbounded behaviour | **✅ RUN — 10 bounded helper proofs VERIFIED, 0 failed** (T23.kani.1 count-arithmetic + T23.kani.2 `Cursor` take/skip/≤-remaining + T23.kani.3a `type_index < typecnt` + T23.kani.3b abbr-index slice-safety + T23.kani.3f.1–.4 the four standards-precision predicates; reduced-surface helpers only) |
| `loom` | concurrency-interleaving correctness of shared-state structures | single-threaded logic | **✅ RAN — `loom::model` proved the `fetch_add` unique-sequence invariant** (temp-name counters) across all interleavings; corrects the prior wrong N/A (zic-rs DOES have lock-free shared state) |
| `miri` | undefined behaviour / leaks in tests (MIR interpreter) | logic · perf | **✅ RUN** — clean over the `tzif` core (16 tests, 0 UB; incl. the bounds-guard) |
| `panic-analysis` | the panic-primitive surface vs `docs/panic-policy.md` | a *proof* of no-panic (that is kani/miri) | **✅ RUN here** — real receipt (`panic-analysis/receipts/`) |

> **What has actually run (receipts present):** `cargo-audit` ✅ clean (0 vulns / 67 deps) · `cargo-geiger`
> ✅ census (zic-rs own 0 unsafe; dep surface quantified) · `cargo-auditable` ✅ embedded manifest · `miri` ✅
> clean over the `tzif` core (0 UB) · `panic-analysis` ✅ · `dsfb-gray` ✅ · `kani` ✅ 10 proofs · **`cargo-fuzz`
> ✅ (T23.cargo-fuzz.1/.2) — bounded smoke found 3 panic bugs F1–F3, .2 fixed 3/3 + regression seeds, smoke
> re-ran 9/9 clean** — **eight real audit results.** `cargo-vet` ◐ **T23.cargo-vet.8** — Vetting Succeeded:
> **42 fully · 1 partial · 23 exempted**; **20 first-party** (full-source reviews incl. the unsafe-heavy tier
> itoa/anstyle-parse/anstyle-query + **semver**+**log** + now **errno**/**anstream**/**anstyle-wincon**, each
> `unsafe` site reasoned; host-vs-all-target reachability recorded) + trusted-import;
> **23 still honestly exempted/UNAUDITED**; no "verified" claim. `cargo-valgrind`
> ◐ ran but is environment-incompatible here (valgrind SIGILLs on `/bin/true` — inconclusive, not a finding).
> `kani` (0.67.0) **landed 10 bounded helper proofs ✅ VERIFICATION SUCCESSFUL, 0 failed** — T23.kani.1
`checked_block_len_never_panics` (the T17.5 count-arithmetic guard) + T23.kani.2 `take`/`skip`/`skip-within-remaining`
(the `Cursor` bounds arithmetic: no out-of-bounds read, no offset wrap, `≤ remaining ⇒ skippable`) + T23.kani.3a
`type_index_guard_is_sound` (`type_index < typecnt`) + T23.kani.3b `abbr_index_guard_prevents_oob_slice` (designation
index slice-safety) + **T23.kani.3f.1–.4** the four reduced-surface standards-precision predicates (designation RFC
validity · `isdst∈{0,1}` · indicator `isut⇒isstd` pairing · `utoff≠i32::MIN`), all 0 failed in ~0.01–4 s each.
*Kani targets sharp reduced-surface helper invariants only.* **The remaining tools were then actually RUN
(2026-06-05), not left as scaffolds:** `cargo-crev` ✅ (66 crates enumerated, empty WoT) · `cargo-scan` ✅ (confirmed
non-functional Rust-CLI stub) · `cargo-valgrind`→**ASan** ✅ (151 tests, 0 sanitizer errors, native) ·
`crux-mir` ✅ (**Overall status Valid, 5/5 goals proved** — second-engine corroboration of Kani) · `flux` ✅
(checked to completion, exit 0; no refinement annotations) · `hax` ✅ (frontend extraction, 186 KB JSON) ·
`loom` ✅ (**`fetch_add` unique-sequence invariant proved across all interleavings** — corrects the prior
wrong N/A) · `dsfb-gray` ✅ (the **real** tool, 60.3% review-readiness map) · `panic-analysis` ✅ (real
`scan.sh`). `creusot` ✅ (**RAN + PROVED** — `why3find prove ✔`, deductive Why3+SMT proof of the count×size invariant, after building the creusot-pinned why3/why3find from source to fix a version skew). **The count-arithmetic invariant is now proven by THREE independent engines — Kani (CBMC) · crux-mir (Crucible) · creusot (Why3+SMT).** **All 17 tools were actually run/confirmed; no status was upgraded without a receipt + captured raw output.**
>
> **Sequencing:** T23 is built **near the end** (after T17.FUZZ/T19/T20/T21/T22). Cross-references: `fuzz/`
> (cargo-fuzz), `docs/panic-policy.md` (panic-analysis/kani/miri), `docs/effect-boundary-map.md` (cargo-scan),
> `docs/security-rewrite-evaluation.md` §D/§E (cargo-audit/vet/auditable/geiger),
> `reports/t17-count-arithmetic-verdict.md` (the kani target).
