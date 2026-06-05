# zic-rs

A memory-safe Rust timezone compiler for IANA tzdata, producing TZif files with
deterministic output and reference-`zic` comparison.

## What this is

`zic-rs` compiles a **declared subset** of IANA tzdata source files (`Zone` / `Rule` /
`Link` lines) into binary **TZif** zoneinfo files, as specified by
[RFC 9636](https://www.rfc-editor.org/rfc/rfc9636) / `tzfile(5)`. It is built as a small
compiler with a hostile reference oracle: every supported construct is checked against the
canonical C `zic`.

It is **library-first** — the compiler lives in the `tzcompile` library; the `zic-rs` binary
is a thin CLI over it.

**A producer, not a datetime library.** Rust already has timezone *consumers* and *bundlers*
([`tz-rs`](https://github.com/x-hgg-x/tz-rs), [`tzdb`](https://crates.io/crates/tzdb),
[`jiff`](https://docs.rs/jiff)/`jiff-tzdb`). zic-rs sits *upstream* of them: it **compiles
and audits** the TZif those consumers read. If you need to read local time in an app, use a
consumer; if you need to compile IANA source into verified TZif for a pinned release, that's
zic-rs. See [docs/rust-ecosystem.md](docs/rust-ecosystem.md) and the
[generated-data contract](docs/generated-data-contract.md).

## Current trust ladder

`zic-rs` is best read not as "a Rust rewrite of `zic`" but as a **reference-admitted TZif compiler
candidate whose claims are typed, witnessed, report-backed, and bounded**. Each stage below answers a
distinct trust question and is sealed with a close receipt.

> **New here? Read [docs/REVIEW-IN-10-MINUTES.md](docs/REVIEW-IN-10-MINUTES.md) first** — the shortest safe
> path through the evidence (what it is/is not · what to rely on · fastest reproduction · strongest receipts ·
> audit board · open debts). Then [docs/reviewer-orientation.md](docs/reviewer-orientation.md) for the deeper frame.

> **For the single current-state answer — "what may I rely on today?" — read [STATUS.md](STATUS.md)**
> (current claims · current non-claims · evidence receipts · open debts). It is kept live with every
> campaign; ladder rows and close receipts below are accurate **at their seal** (historical snapshots).

| Stage | Trust question | State |
|-------|----------------|-------|
| **T12** | what source / build / profile **evidence** backs the artifact? | ✅ closed (manifest schema **v8**) |
| **T13** | what **diagnostics** are emitted, by class/severity/verbosity/platform? | ✅ closed (`ZIC001`–`ZIC020`) |
| **T14** | how does it behave under **malformed / hostile** input? | ✅ closed (`ZIC021`–`ZIC025`; found+removed a panic) |
| **T15** | can a reviewer **verify the conformance claim from machine-readable reports**? | ✅ closed (`ZIC026`; 15 guard-enforced non-claims; no global verdict) |
| **T16** | which **release / reference / platform / table / vendor-receipt** does a claim belong to? | ◐ in progress — T16.1–T16.6 sealed (inventory · `ReferenceBuildProfile` · locator+trust · aux-table validator · vendor-oracle receipt admission + ingestion · release-diff + doctor · **17 ecology rows / 19 receipts (18 admitted + 1 typed non-admission) — full BSD sweep (FreeBSD · OpenBSD · NetBSD · DragonFly) + illumos triad (OmniOS · SmartOS · OpenIndiana) + Linux (Alpine/musl-tzcode · Debian/NixOS/Ubuntu/AlmaLinux glibc · Gentoo/tzcode · openSUSE/tzcode-RPM) + Yocto/Poky (build-host `tzcode-native` 2026b admitted + target-runtime non-admission) + SLES 15-SP7 (commercial SUSE, container; tzcode-`zic` via `timezone` RPM ≡ openSUSE Leap) + Arch (rolling; tzcode-`zic` via the `tzdata` pacman pkg, glibc 2.43 unused)**; each admitted row rejects every fixture (safe), 3–4/5 class+location match + declared divergences; measured a real vendor lag spectrum, an old-fork input-compat gap, complete illumos convergence-on-three-independent-builds, **two `zic` implementation lineages — IANA tzcode `zic` vs glibc's own `zic`, the glibc lineage version-stratified with a bracketed inflection (glibc 2.34/2.39 old-fork-like + fat < 2.40/2.41 modern + slim, transition between 2.39 and 2.40)**, that **the `zic` lineage is a distro packaging choice independent of libc *and* package format** (Gentoo runs tzcode-`zic` via `timezone-data` despite glibc 2.42; openSUSE runs tzcode-`zic` via the `timezone` RPM where AlmaLinux's RPM gives glibc-`zic` — so **RPM ≠ glibc-`zic`**), that **bloat-default is its own packaging axis** (fat in old-glibc + Gentoo-tzcode; slim in Alpine/openSUSE-tzcode + modern-glibc), that **some artifacts ship no on-device `zic`** (the Yocto/Poky target image = TZif *consumer* — precompiled data, compiler off-device on the build host), and that **the embedded build-host *producer* ≠ the target-runtime *consumer*** (Poky's `tzcode-native` owns `zic`; the produced image ships only data → non-admission); the two non-admissions prove `vendor-oracle-admit` rejects-by-rule, handling negative results as first-class). **T16.6 ✅ sealed** — operator tooling: `release-diff` (per-identifier OLD↔NEW classification, typed `ReleaseChangeKind`, structural always + behavioural via two `zdump` year-windows, oracle absence visible) + `doctor` (read-only host probe, typed `ToolStatus`, always exit 0); both additive/read-only. The six-axis vendor-oracle matrix renderer stays T16.6.x (on demand) |
| **T17** | can the Rust implementation itself be made hard to embarrass? | ✅ closed — `reports/t17-close-receipt.md`. T17.1a parse bounds-guard + `panic-policy.md` · T17.1b `ResourceLimits` input caps (bucket-3 `Error::config`) · T17.2 CONTRACT.TYPING (6 free strings → typed enums, no schema bump) · T17.3 doctor/release-diff failure taxonomy (`ToolVersionStatus`/`HashReadStatus` → doctor v2 · `OracleFailureScope` · `--split` exclusive seam) · T17.4 install/materialization hardening (per-file crash-durable publish + leaf TOCTOU closed; `install-materialization-contract.md`) · T17.5 `CountArithmeticVerdict` (checked count/size/offset arithmetic + pre-allocation bound; `risk-register.md`) · T17.6 schema/CLI stability policy (`schema-compatibility-policy.md` + `cli-compatibility-policy.md`) · T17.FUZZ scaffold (`fuzz/` — 9 receipt-bearing targets; sat `pending_capture` at T17 close, **later executed at T23.cargo-fuzz.1/.2**) · T17.7 verify-here remainder (`tests/reliability.rs` · `SECURITY.md`/`CONTRIBUTING.md`/`CHANGELOG.md`/`RELEASE.md` · `schemas/` + registry/drift test); **480 tests at the T17 seal** (live count: `STATUS.md`), CORE.1 341/0/0 |
| **T18** | where does each claim's external knowledge come from? | ◐ in progress — `docs/zic-knowledge-index.md` + ledgers + `claim-source-map.md` + copyright policy (**13 sources S1–S13**, typed `SourceLedgerEntry`/`AuthorityKind`; archival capture COMPLETE). **T18.breadth-to-40 ✅** — provenance archive of 40 diversity-selected stress zones from sig-verified pristine 2026b → **40/40 behaviour zdump-MATCH** (`reports/provenance/`). **T18.3 ✅** — readable provenance ledger `docs/provenance-ledger.md` (40-case table + 15-category matrix + source-file matrix + named findings + claim boundaries). Source-breadth→40 + adjacent-compiler/reader ledgers continue |
| **T19** | how does a reviewer/packager/security person decide it is safe to depend on? | ✅ sealed — the **trust front door**: `TRUST.md` (the evidence-court entrypoint) + `docs/effect-boundary-map.md` (per-module effect class) + `docs/not-yet-ready.md` (the loud refusal surface) + `docs/replacement-readiness-ladder.md` (RRL-0…6; zic-rs at **RRL-1→2**) + `docs/drop-in-compatibility-contract.md` (surface-by-surface — **NOT an argv drop-in by design**) + `docs/misuse-resistance-ledger.md` (misuse · guard · test · non-claim) + `docs/audit-readiness.md` (the packet, not a performed audit) + `docs/maintenance-policy.md` (MSRV · cadence · emergency-tzdb · refresh); docs-only seal — CORE.1 341/0/0 (live test count: `STATUS.md`) |
| **T20** | what does the rewrite remove, and what may each audience conclude / NOT conclude? | ✅ sealed — `docs/security-rewrite-evaluation.md` (threat table · resource model · supply-chain [SBOM/SLSA/signing **planned**, not present] · Scorecard-readiness · the honest rewrite-safety delta) + `docs/security-personas.md` (10 personas × fears/evidence/commands/may-conclude/**may-NOT-conclude**/risks/owner); consumes risk-register + SECURITY.md + T19; docs-only seal — CORE.1 341/0/0 (live test count: `STATUS.md`) |
| **T21** | minimal deterministic timezone bundles for containers/embedded | ◐ first cut — `docs/container-embedded-builder.md` (bundle contract · container recipe · no-host-contamination · copy-mode baseline · producer≠consumer) + **`zic-rs size-report`** (footprint + deterministic `bundle_hash`; `zic-rs-size-report-v1`); named profiles/`--link-policy`/reader-gauntlet planned — **485 tests at the T21 seal** (live count: `STATUS.md`), CORE.1 341/0/0 |
| **T22** | is the resource cost measured and bounded? | ◐ first cut — `docs/perf-ledger.md` + `reports/perf/` (a real measured receipt: 107 KB `tzdata.zi` → 598 TZif files in ≈12 ms / ≈22 MiB RSS, far inside the `ResourceLimits` caps); **measured + bounded, not a speed claim**; deterministic anchors (bytes/counts/`bundle_hash`) are the regression contract, timings are host/run weather |
| **T23** | what does each audit tool witness — and what is actually run? | ◐ first cut + **external batch run** — `audits/` (the exact authoritative list: 17 tool folders + README + index.html, each **receipt-bearing**; *a folder existing is not a result*). **all 17 tools run/confirmed (2026-06-05):** `cargo-audit` ✅ clean (0 vulns / 67 deps), `cargo-geiger` ✅ (zic-rs own 0 unsafe; dep surface mapped), `cargo-auditable` ✅, `miri` ✅ clean over the `tzif` core (0 UB), `panic-analysis` ✅, `dsfb-gray` ✅, **`kani` ✅ — 10 bounded formal proofs** (T23.kani.1 count-arithmetic `checked_block_len` + .2 `Cursor` take/skip/≤-remaining + .3a `type_index < typecnt` + .3b abbr-index slice-safety + **.3f.1–.4 the four standards-precision predicates** [designation RFC validity · `isdst∈{0,1}` · indicator `isut⇒isstd` · `utoff≠i32::MIN`]; reduced-surface helpers only; all 0-fail), and **`cargo-fuzz` ✅ (T23.cargo-fuzz.1/.2)** — bounded smoke (9 targets, 25 s) **found 3 panic bugs F1–F3; .2 fixed 3/3 + regression seeds, smoke re-ran 9/9 clean** (panic-policy restored for the known seeds + bounded rerun, not an exhaustive proof). `cargo-vet` ◐ **T23.cargo-vet.8** (Vetting Succeeded: **42 fully · 1 partial · 23 exempted**; 20 first-party full-source reviews + trusted-import; 23 still UNAUDITED) · `cargo-valgrind`→**ASan** ◐→✅ (valgrind SIGILLs on `/bin/true`; ASan ran clean instead, 151 tests, 0 errors) · **creusot ✅ PROVED (Why3+SMT) · crux-mir ✅ 5/5 Valid · loom ✅ proved · flux/hax ran · cargo-crev (empty WoT) · cargo-scan (stub)** — the count-arithmetic invariant is proven by **3 engines** (Kani · crux-mir · creusot); **no status upgraded without a receipt; not all audits are green.** Also (on demand): **T18.breadth-to-40** provenance archive (40/40 zdump-MATCH) + **T18.3** readable ledger · **T23.reader-compat.1/.2/.3** cross-reader ecology (glibc/Go/CCTZ see zic-rs ≡ ref, 0 mismatch) · **T23.release-diff-real.1** (real 2026a→2026b). **520 tests, CORE.1 341/0/0.** |

**Public reports** (all `--format json`, hand-rolled + deterministic): `support-report` ·
`structural-report` · `semantic-report` · `tzif-validate` · `aux-table-validate` · `release-diff` · `doctor` · `size-report` · `compile --manifest`. Each carries an
`oracle_mode` (oracle absence is never silent), `negative_capabilities` (what is *not* claimed, each with
its enforcing guard), and a bounded `conformance_status` (never an unqualified `compatible`).

**Which `zic` flags/modes are covered?** See **[docs/zic-operational-parity-matrix.md](docs/zic-operational-parity-matrix.md)**
(ZIC-MATRIX.1) — 19 flag/mode rows, each a real run on both compilers, classified match / class-parity /
intentional-divergence / unsupported-by-design / deferred / divergent. The one divergence it found (`-D`
directory-creation) was **fixed** (ZIC-MATRIX.1.D) — current tally: **0 divergent.**

**Does it hold across tzdb source eras?** See **[docs/release-ladder.md](docs/release-ladder.md)**
(RELEASE-LADDER.1) — **7 signature-verified tzdb releases** (2015g→2026b); the 7-zone fixture set
behaviour-matches reference `zic` (**49/49 `zdump`, 0 divergent**); 3 non-fixture zones source-incompatible
in 2018e/2020a (documented deferred features). Bounded fixture-match, **not** full admission. **Scaled to the whole archive:** RELEASE-ALL.1 indexed all 785 IANA release entries; **RELEASE-ALL.DATA.1** compiled **all 276 stable tzdata** (1993→2026b) → 150 match · 66 ref-build-incompat (`yearistype`, **all 66 since replayed** by `--legacy-yearistype`+`--legacy-empty-footer` vs a historical oracle — YEARISTYPE.1 + PERPETUAL-EXPANSION.1) · 60 source-shape-incompat (UTF-8, **CLOSED** by LEGACY-SOURCE.1 → match with `--legacy-latin1`) · **0 zic-rs-divergent** (`docs/iana-release-archive-ledger.md`). **Under the explicit replay modes: 276/276 attempted, 0 divergent.** **TZDB-ATLAS.4** joins upstream × vendor-oracle × drop-in into one explainable evidence atlas (`docs/tzdb-evidence-atlas.md`) — differences attributed by axis; both historical bands fully replayable, **no stable-release replay band remaining**.

**The live behaviour claim** (never inflated): *zic-rs behaviour-matches reference `zic`/`zdump` for all
**341/341** canonical zones in `tzdata.zi` 2026b over `1900..2040` — 0 mismatch · 0 fail-closed (CORE.1).*
Structural / byte / diagnostic / operational parity are **separate axes**, never collapsed into one verdict.

## What this is not

* It is **not yet a full replacement** for reference `zic`.
* It does **not** install into `/usr/share/zoneinfo`; all output goes under an explicit
  `--out` directory.
* It **refuses unsupported syntax** with a clear diagnostic rather than emitting an
  approximate timezone file. Correctness is measured, not asserted.
* It **does not decide timezone policy or curate timezone history.** It compiles IANA tzdata
  according to reference-`zic` behaviour; it never editorialises or "normalises" civil-time
  rules beyond what `zic` does. (The tz database is maintained through IANA's process —
  [RFC 6557](https://datatracker.ietf.org/doc/html/rfc6557) / BCP 175.)

## Current implementation status

Implemented and verified:

* parsing of `Zone` / `Rule` / `Link` records, continuation lines, comments, quoting, and
  input-hardening limits;
* compilation of **fixed-offset zones** (`RULES` = `-`), **finite rule-driven DST zones**,
  **recurring (`TO = maximum`) DST zones** with a synthesised recurring POSIX footer
  (e.g. `EST5EDT,M3.2.0,M11.1.0`), **inline-save eras** (`RULES` = a clock value → a constant
  `STDOFF + SAVE` type, literal or `%z`), and **multi-era zones** (`UNTIL` continuations) with
  correct cross-era state — including `lastSun`/`Sun>=N`/`Sun<=N` day forms,
  wall/standard/UT `AT` suffixes, and `%s`/`STD/DST`/`%z` formats;
* **real IANA-zone slices** (tzdb 2026b): `Europe/London` (`zdump`-matched over `1830..2045`;
  final EU era effectively recurring-only) and `America/New_York` (`1883..2040`; a genuinely
  **mixed-in-era** final era — Rule US finite DST `1967..2006` plus recurring `2007..max`);
* `Link`s (copy or symlink), with `--zone <alias>` resolving to the canonical zone;
* a safe output tree (path-traversal-proof, atomic exclusive create, no clobber without
  `--force`);
* a two-mode oracle (`compare`): **`zdump` behaviour over a declared horizon** (default; the
  real correctness criterion, used for DST/recurring zones) and **decoded-TZif** structural
  comparison (fixed-offset/debugging). Byte parity is reported only against pinned blobs.

For the fixed-offset fixtures (`Etc/UTC`, `Test/Fixed`), our output is **byte-for-byte
identical** to the reference `zic` blobs **pinned in `fixtures/expected/`** (produced with
tzcode 2026b; see `fixtures/MANIFEST.toml`). For everything else the binding contract is
**semantic** parity, checked by `zic-rs compare` against your *local* reference `zic` via the
`zdump` behaviour oracle over a declared year horizon. Finite-DST, recurring-DST, and
multi-era zones all match the oracle (see [`docs/compatibility.md`](docs/compatibility.md)).
Source is read in both the canonical `Rule`/`Zone`/`Link` spelling and the
**zishrink-abbreviated** `R`/`Z`/`L` form, so the installed single-file
`/usr/share/zoneinfo/tzdata.zi` parses directly (record keywords match as `zic`-style
unambiguous prefixes, like month/weekday names).

**Whole-database status.** As of `tzdata.zi` 2026b, zic-rs **compiles all 341 / 341 canonical zones**
and **behaviour-matches reference `zic`/`zdump` for every one over `1900..2040`** — a comprehensive
per-zone sweep with **0 mismatches and 0 fail-closed** (`compile-clean` and `behaviour-match` are
still tracked separately as different claims). The entire canonical-zone behaviour frontier is
closed. This is reference-verified behaviour for the canonical zones over a declared horizon —
**not** a full `zic` replacement and **not** infinity: leap-second modes, CLI/install policy,
`right/`/`posix/`, backzone/rearguard, warning parity, slim/`-r`/`-R`/`-b` emission, and TZif v4
remain a declared operational roadmap (see [`docs/roadmap.md`](docs/roadmap.md), T9–T17).

**Structural parity is a separate, *measured* axis** (campaign T8). The `structural-report` command
inventories how our TZif *bytes* differ from reference `zic`: across all 341 zones,
`isutcnt`/`isstdcnt`/`leapcnt`/`typecnt`/**version**/**footer** are at full parity — `ttisstd`/`ttisut`
is **not** a gap (both emit zero in this slim-default build); **version+footer match 341/341** after
T8-v3 pinned `zic.c`'s `compat >= 2013` version rule (the day-shift, not the displayed time's range,
makes `America/Santiago`/`Pacific/Easter` v3); and **`charcnt` matches 341/341** after T8-abbrev
ported `zic`'s abbreviation **suffix-sharing** packer (`HST`⊂`AHST`, `LMT`⊂`PLMT`). The last axis, the
slim/fat `timecnt` window, is addressed by **T8-slim**: `--emit-style zic-slim` reproduces reference
`zic`'s slim explicit-transition set, collapsing the `slim/fat-timecnt` class **75 → 1** (only
`Europe/Lisbon`, a ref-fatter-by-1 no-op) and raising `byte-identical` to **141/341** — while the
**default** emission stays the behaviour-matched fat output. **Behaviour is 341/0/0 in both modes**
(all `zdump`-equivalent). See [`docs/structural-parity.md`](docs/structural-parity.md). Behaviour
parity stays the contract; byte parity is claimed only in `zic-slim` mode (or where a blob is pinned).

**Canonical-zone behaviour over `1900..2040` is closed for `tzdata.zi` 2026b** — *no* canonical zone
fails closed (negative SAVE → law 7, and the non-POSIX `Sat<=30`-style recurring `ON` → law 10, are
both **solved**). What remains is **deferred / outside the verified canonical-zone path**, none of it
forced by `tzdata.zi` 2026b: inline-save eras with a `%s` or `STD/DST` slash `FORMAT`; a recurring rule
whose `ON` is a *fixed numeric* day (no weekday, hence no `Mm.w.d` footer); `24:00`/negative *compiled*
`AT` times outside the implemented footer cases; leap-second / `right/` modes; and the operational
`zic` shell (CLI/install, `-r`/`-R`/`-b`, warning parity) — all tracked in [`docs/roadmap.md`](docs/roadmap.md)
(T9–T21). (Accepted breadth — matching reference `zic`: both mixed finite+recurring final-era shapes,
inline-save with a literal or `%z` `FORMAT`, and the obsolete `FROM = minimum` coerced to 1900.) See
the pinned behaviour ledger [`docs/reference-zic-semantics.md`](docs/reference-zic-semantics.md).

## Quick start

```sh
# Compile a fixed-offset zone and its link into ./zoneinfo
cargo run -- compile --input fixtures/minimal --out ./zoneinfo --zone Etc/UTC

# See what a zone would compile to (or why it is unsupported)
cargo run -- explain --input fixtures/minimal --zone Test/Fixed

# Compare our output against the system reference zic (semantic + byte)
cargo run -- compare --input fixtures/minimal --zone Etc/UTC --reference-zic zic

# Print the supported syntax subset
cargo run -- supported-syntax

# Map the whole installed tzdb: which zones compile, and why the rest don't (the frontier map)
cargo run -- support-report --input /usr/share/zoneinfo/tzdata.zi

# Inventory how our TZif *bytes* differ structurally from reference zic (separate from behaviour)
cargo run -- structural-report --input /usr/share/zoneinfo/tzdata.zi --reference-zic zic
```

## Documentation

| Doc | Contents |
|-----|----------|
| [docs/design.md](docs/design.md) | Compiler phases and module layout |
| [docs/supported-syntax.md](docs/supported-syntax.md) | Exactly what the current declared subset supports |
| [docs/unsupported-syntax.md](docs/unsupported-syntax.md) | What is rejected, and why |
| [docs/compatibility.md](docs/compatibility.md) | Fixture zones and oracle status |
| [docs/conformance-ladder.md](docs/conformance-ladder.md) | How IANA slices are admitted (the ladder + receipts); the `support-report` frontier map |
| [docs/security.md](docs/security.md) | Untrusted-input handling and path safety |
| [docs/tzif-notes.md](docs/tzif-notes.md) | TZif writer structure and RFC alignment |
| [docs/structural-parity.md](docs/structural-parity.md) | The measured structural-parity inventory (`structural-report`, T8): how our TZif bytes differ from reference `zic`, classified — a separate axis from behaviour parity |
| [docs/zic-operational-parity.md](docs/zic-operational-parity.md) | CLI / filesystem / install-mode parity (T9): the reference-`zic` flag inventory mapped to zic-rs, classified, with the safety boundary |
| [docs/oracle-testing.md](docs/oracle-testing.md) | Running comparisons against `zic` |
| [docs/reference-zic-semantics.md](docs/reference-zic-semantics.md) | Pinned subtle `zic` behaviours (the semantics ledger) |
| [docs/zic-deep-semantics.md](docs/zic-deep-semantics.md) | The named-law index: 18 `zic` state-machine laws, each tied to a zone/test/bucket |
| [docs/zic-v-hazards.md](docs/zic-v-hazards.md) | `zic -v` verbose-warning hazards → zic-rs status / test / bucket / campaign |
| [docs/zic-hidden-compatibility.md](docs/zic-hidden-compatibility.md) | Operational / reader-compat traps (input format, `-r`/`-00`, `ttisstd`, limits, …) |
| [docs/standards.md](docs/standards.md) | Normative (RFC 9636 / `zic` / `zdump`) vs orientation sources |
| [docs/tzdb-governance.md](docs/tzdb-governance.md) | The authority chain; why zic-rs is not a policy authority |
| [docs/legal-history.md](docs/legal-history.md) | Public-domain posture & provenance (brief) |
| [docs/commentary-style.md](docs/commentary-style.md) | Comment/doc/test conventions |
| [docs/rust-ecosystem.md](docs/rust-ecosystem.md) | Where zic-rs sits vs tz-rs/tzdb/jiff (producer vs consumers) |
| [docs/generated-data-contract.md](docs/generated-data-contract.md) | What zic-rs output guarantees a consumer |
| [docs/consumer-testbench.md](docs/consumer-testbench.md) | Optional `tz-rs` interop bench (`--features ecosystem-tests`); not the oracle |
| [docs/roadmap.md](docs/roadmap.md) | Remaining milestones and deferred syntax |
| [docs/prior-art.md](docs/prior-art.md) | Why this does not duplicate existing crates |
| [docs/PROMPT.md](docs/PROMPT.md) | The full project specification/prompt |

## Acknowledgements

zic-rs builds on decades of work by the IANA Time Zone Database community. See
[ACKNOWLEDGEMENTS.md](ACKNOWLEDGEMENTS.md) — and note that this project is **not affiliated
with or endorsed by** IANA/ICANN, the TZ Coordinator, or the tzdb maintainers; bugs here are
ours alone.

## License

Dual-licensed under either of [MIT](LICENSE-MIT) or
[Apache-2.0](LICENSE-APACHE), at your option.
