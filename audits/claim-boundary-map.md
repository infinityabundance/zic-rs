# Claim-boundary proof/audit map (T23 — anti-drift)

> **The priority line for all further audit/proof work.** Add evidence where a **false claim could hurt**; do
> **not** add evidence merely because a tool exists. Every row names a **claim boundary** (a place zic-rs
> could mislead — adoption, security, compatibility, provenance, or maintenance), its existing evidence, the
> remaining gap, the best tool, the next receipt *if worth doing*, and a priority. **No claim boundary → no
> priority.** This is not a new milestone; it lives inside T23 and governs what the audit suite, Kani, the
> reader gauntlet, cargo-vet, and the rest do next. It pairs with `docs/risk-register.md` (the failure modes)
> and the `dsfb-gray` self-audit (the in-house enforcer).
>
> **The question every new receipt must answer: *what lie does this prevent?***

## The precision ladder (keep this visible — every claim must name its rung)

The project is in the **precision phase**: the evidence skeleton exists; the job now is to prevent subtle
**category mistakes**. Each `≠` below is a rung a claim must not silently climb. Every receipt/report/row
states *which* rung it proves and refuses the next.

```text
memory-safe accepted        ≠  format-valid (RFC 9636)          # parse is lenient; rfc9636::validate is strict (T23.kani.3f.enforce)
format-valid                ≠  semantically equivalent          # tzif-validate ≠ semantic-report
semantically equivalent     ≠  civil-time truth                 # IANA/CLDR own that (RISK.TIME.1)
civil-time source admitted  ≠  vendor distribution policy       # admitted release ≠ what a distro ships
vendor receipt              ≠  vendor-family theorem            # one ecology, not a family (RISK.VENDOR.1)
audit tool ran              ≠  global safety                    # a receipt is one dimension, not a verdict
bounded proof               ≠  universal proof                  # reduced-surface Kani helpers, not whole-parser
archive witness             ≠  authority                        # canonical_url = authority; archive_url = witness (T18)
byte-identical output       ≠  reader-equivalent behaviour       # T23.reader-compat.1: 4/12 byte-id, 12/12 read-equal
proven predicate            ≠  enforced check                    # closed for the 4 RFC predicates (T23.kani.3f.enforce)
```

## Behaviour / hostile-input boundaries (the compiler core)

| Claim boundary | Existing evidence | Remaining gap | Best tool | Next receipt (if worth it) | Priority |
|---|---|---|---|---|---|
| Compiled TZif behaviour-matches reference | CORE.1 sweep (341/0/0) · `semantic-report` (`zdump`) | new tzdb releases (only 2026b) | oracle / release-admission | per-release CORE.1 on intake | **covered** (per release) |
| Evidence surface is not cherry-picked / curated | **T18.breadth-to-40** provenance archive (`reports/provenance/`): 40 diversity-selected stress cases (pre-1970 · neg-DST · double-summer · non-hour/sub-minute offsets · churn · renamed/linked · footers · underrepresented regions · reader stress) from sig-verified pristine 2026b region files → **40/40 behaviour zdump-MATCH** | broader still (598 full set) is CORE.1's job, not this archive | provenance gauntlet | more cases only if a new trap appears | **covered** (bounded, diverse); claims source-provenance + compiler-equivalence, **NOT timezone truth**; 1 named null finding (Europe/Lisbon slim residual) |
| Structurally-valid ≠ semantically-wrong TZif | `tzif-validate` (5 typed verdicts); `RISK.TZIF.1` | dual-block + indicator-relation invariants (T15.4 first-pass) | `tzif-validate` / Kani | indicator-relation verdict | medium |
| No panic / OOM on hostile input | `panic-analysis` (0 prohibited) · `miri` (tzif core, 0 UB) · `ResourceLimits` · `reports/t17-count-arithmetic-verdict.md` · **`tests/fuzz_regressions.rs` (4 seed replays) + 3 sharp unit tests** | ✅ **RESTORED for the known T23.cargo-fuzz.1 seeds + the bounded rerun — T23.cargo-fuzz.2 (2026-06-04) fixed 3/3.** F1 `tzif/validate.rs` footer now `strip_prefix`/`strip_suffix` (lone `\n` → typed Err) · F2 `model/time.rs` `h*3600` now `checked_mul`/`checked_add` (huge hour → typed Err) · F3 `source/parser.rs` `strip_prefix_ci` now byte-compares (multibyte → clean non-match). **The panic-policy claim is restored only for the known T23.cargo-fuzz.1 findings and the stated bounded fuzz rerun. It is NOT an exhaustive no-panic proof.** | bounded smoke + 6 regression tests | watch for new findings in longer/operator campaigns | **CLOSED for F1–F3; bounded-fuzz clean as of receipt** (`audits/cargo-fuzz/receipts/RECEIPT-2026-06-04-fuzz2.md`; seed replays rc=0; smoke 9/9 clean) |
| count×size arithmetic never wraps/OOMs | **T23.kani.1 ✅** (proven over `u32⁶×{4,8}`) | — | Kani | — | **covered** |
| cursor `take`/`skip`/`remaining` bounds | **T23.kani.2 ✅** (3 proofs) | — | Kani | — | **covered** |
| transition `type_index < typecnt` | **T23.kani.3a ✅** | — | Kani | — | **covered** |
| abbreviation index **slice-safety** (`idx ≤ charcnt` ⇒ no OOB read) | **T23.kani.3b ✅** | — | Kani | — | **covered (slice safety only)** |
| abbreviation index **RFC validity** (`desigidx < charcnt` + NUL at/after + `charcnt != 0`) | **T23.kani.3f.1 proven + ENFORCED** — `rfc_designation_index_valid` (proven to strictly imply the .3b slice-safety) now applied in `rfc9636::validate` (T23.kani.3f.enforce) | — *memory-safe acceptance ≠ format-valid acceptance*: a file parses yet is a structural `Violation` | `rfc9636::validate` + tests | ✅ done — `desigidx==charcnt` / `<charcnt` w/o NUL / `charcnt==0` → Violation; real zones + reference-zic stay conformant | **covered** (proven · enforced · tested) |
| `isdst` byte ∈ {0,1} | **T23.kani.3f.2 proven + ENFORCED** — `isdst_byte_valid` applied in `rfc9636::validate` (T23.kani.3f.enforce); `parse` stays lenient (`b[4] != 0`) but the validator now rejects | — | `rfc9636::validate` + tests | ✅ done — `isdst == 2` → Violation (parses, but format-invalid) | **covered** (proven · enforced · tested) |
| `utoff != i32::MIN` (and portability range) | **T23.kani.3f.4 proven + ENFORCED** — `utoff_structural_valid` (accept ⇒ negation cannot overflow) applied in `rfc9636::validate` (T23.kani.3f.enforce) | — | `rfc9636::validate` + tests | ✅ done — `utoff == i32::MIN` → Violation | **covered** (proven · enforced · tested) |
| indicator **byte values** + UT⇒std pairing | counts checked (`rfc9636`: isut/isstd ∈ {0,typecnt}); **T23.kani.3f.3 proven + ENFORCED** — `indicator_pair_valid` applied in `rfc9636::validate` over the now-captured `parsed.raw` indicator bytes (T23.kani.3f.enforce) | — (`parse` still skips them — lenient — but the validator reads `parsed.raw` and rejects) | `rfc9636::validate` + tests | ✅ done — byte ∉ {0,1} or `isut==1 && isstd==0` → Violation | **covered** (proven · enforced · tested) |
| footer ↔ last-transition consistency | `PosixFooterVerdict` = parseability only | RFC: TZ string evaluated at the last transition should match its type — **unassessed** | `semantic-report` / report axis | a `footer_last_transition_consistency` axis or explicit non-claim | medium |
| `ttisstd`/`ttisut` **count** ∈ {0, typecnt} | **RFC-9636 validator ✅** (`rfc9636.rs`) | — | — | — | **covered** |
| leap-record block sizing checked | `checked_block_len` (generic, T23.kani.1) | leap-specific width/expiry edge | Kani | T23.kani.3d | low-medium |
| path traversal / write-through | `ZIC008` · `install-materialization-contract.md` · `tests/hostile_output_tree.rs`; `RISK.PATH.1` | parent-component TOCTOU (`openat`) | tests / T20 | named residual (not closed) | named-residual |
| `right/` leap-profile transition times leap-adjusted (zones with transitions) | **FOUND + FIXED** (T23.reader-compat.2): `apply_leaps` shifts each transition by the cumulative leap correction; `right/{America/New_York,Europe/London,Etc/UTC}` `zdump`-match reference, +1 regression test (`RISK.LEAP.1`) | universal leap-profile parity beyond the tested `right/` zones (not claimed) | reader gauntlet / leap tests | covered for the tested `right/` zones; POSIX/default byte-unchanged | **covered (found → fixed; opt-in profile, CORE.1 unaffected)** |

## Trust-surface "the court cannot lie by bookkeeping error" boundaries (highest priority — mostly DOCTRINE-only today)

These are the project's own evidence surfaces. Several have a **doctrine** but **no machine check** — exactly
where a quiet bookkeeping bug could let a report or ledger overclaim. These are the strongest *new* targets
(small `dsfb-gray` invariant checks or tiny Kani helper proofs).

| Claim boundary | Existing evidence | Remaining gap | Best tool | Next receipt | Priority |
|---|---|---|---|---|---|
| `release-diff`: unassessed ≠ unchanged | typed `behaviour_unassessed`; `RISK.DIFF.1`; tests; **T23.release-diff-real.1 ✅** — real **2026a→2026b** (GPG-verified, hash-pinned): `oracle_mode reference_zdump`, 597 unchanged · 1 `behavior_past_and_future` (`America/Vancouver`, independently confirmed); oracle-omitted ⇒ `behaviour_unassessed`, never "unchanged" — `reports/release-diff/` | a `dsfb-gray`/Kani **machine invariant** on the unassessed≠unchanged rule is still doctrine-only | dsfb-gray/Kani | the bookkeeping invariant | **covered (real delta run); invariant medium** |
| does the compiler hold across tzdb source ERAS (not only 2026b)? | **RELEASE-LADDER.1** `docs/release-ladder.md` · `reports/release-ladder/` — 7 signature-verified releases (2015g→2026b), 7-zone fixture set **49/49 zdump behaviour-match · 0 divergent**; 3 non-fixture zones source-incompatible in 2018e/2020a (deferred features) | full release/zone admission (only 2026b fully admitted); old `zic` *binaries* not held (ref = current zic on old source) | release ladder | more releases/zones on demand | **covered (bounded fixture-match, sig-verified)**; source-incompat zones named, not hidden |
| which `zic` flag/mode surfaces are covered (not just "it compiles") | **ZIC-MATRIX.1** `docs/zic-operational-parity-matrix.md` — 19 flags run on BOTH compilers, strict verdicts (**11 match · 3 intentional-divergence · 2 class-parity · 1 unsupported-by-design · 1 deferred · 1 n/a · 0 divergent**); harness `reports/zic-matrix/` | full `zic` CLI history (decades of quirks) is not exhausted; bounded fixture set, 2026b ref | flag/mode gauntlet | expand flags/fixtures on demand | **covered (bounded, run-backed)**; the one `-D` divergence it found was **FIXED** (ZIC-MATRIX.1.D), regression-tested — the matrix proved actionable |
| reader ecology: real readers accept the output | **T23.reader-compat.1/.2/.3 ✅** `reports/reader-compat/` — `.1` `zdump`+Python `zoneinfo` (24/24 pairs); `.2` 15 edge fixtures (found+fixed right/-leap shift); **`.3` cross-reader ecology** — 6 T18-ledger fixtures × {glibc 2.43 · Go 1.26 · CCTZ/abseil} all interpret zic-rs ≡ reference (**16 match · 1 match-with-known-limit · 0 mismatch**); Java/PHP/ICU classified `unsupported_by_reader` (compiled-DB, not raw TZif) | broader zones + more readers always possible; not universal-consumer | reader-compatibility gauntlet | widen on demand | **covered (3 raw-TZif readers cross-checked; limitations recorded ≠ zic-rs failures)** |
| `archive_url` is witness, never authority (T18) | T18 doctrine (`canonical_url`=authority) | no machine check | dsfb-gray / Kani | a check that no row's authority derives from an archive URL | **high** |
| audit "green" requires a receipt (T23) | T23 doctrine (this suite) | no invariant check | dsfb-gray / Kani | `status==green ⇒ receipt_count>0` invariant | **high** |
| `HashReadStatus::Read` ⇒ a real 64-hex hash (doctor) | typed enum (T17.3) | no machine invariant | dsfb-gray / Kani | `Read(x) ⇒ x is 64-hex`; no prose in a `sha256` field | **high** |
| cargo-vet counts honest (audited+import+exempted=total) | T23.cargo-vet receipts | no machine invariant | dsfb-gray / Kani | `audited+import+exempted == total`; exempted never "reviewed" | medium |
| `size-report` bundle_hash covers every file once | `tests/size_report.rs` | pure-helper invariant | Kani / dsfb-gray | every file contributes once; symlink before follow | medium |
| `negative_capabilities` each enforced + unique | enum + `enforced_by` + drift test | registry totality invariant | dsfb-gray | stable id · non-empty · no dup · owning risk | medium |
| replacement-readiness not overclaimed | `replacement-readiness-ladder.md` | no monotonicity check | dsfb-gray | RRL-N requires RRL-(N-1) evidence | low-medium |
| report ≠ signed attestation | `ReportProvenance=unsigned_local_report`; `RISK.REPORT.1` | — | (doctrine) | — | covered |
| vendor receipt ≠ family theorem | `admit()` recomputes; `RISK.VENDOR.1` | — | (doctrine) | — | covered |
| supply-chain graph reviewed | `cargo-audit` clean · `cargo-vet` (**10 first-party + ~22 import**, T23.cargo-vet.5; 32 fully · 1 partial · 33 exempted) | **33 exemptions unaudited** (39→36→33) | cargo-vet | `semver` (47 sites) + `log` (global state) with dedicated review time (T23.cargo-vet.6) | medium |
| not a civil-time authority | `RISK.TIME.1` · `not-yet-ready.md` · S3 (RFC 6557) | — | (doctrine) | — | covered |

## How to use this map

1. **Before adding any audit/proof,** find (or add) the claim boundary it protects here. If there is none,
   park it — it is decoration, not evidence.
2. **The current high-priority frontier** is the second table's `high` rows: the **bookkeeping invariants**
   (archive≠authority · audit-green-requires-receipt · HashReadStatus · release-diff unassessed≠unchanged)
   provable by tiny `dsfb-gray`/Kani checks, plus the two new *evidence classes* — the **reader-compatibility
   gauntlet** and a **real 2026a→2026b release-diff**.
3. **Stopping rule (Kani):** the TZif hostile-input count/offset/cursor/index guards now have targeted
   bounded proofs (T23.kani.1/.2/.3a/.3b) or are explicitly out of scope (full-parser entrypoint = T23.kani.3,
   inconclusive). Further Kani only for a *named* boundary above, never for prestige.

> **Doctrine:** *zic-rs is judged less by how many artifacts it has, and more by whether every artifact
> protects a real adoption / security / compatibility / provenance / maintenance boundary.* Add evidence
> where a false claim could hurt; never merely because a tool exists.
