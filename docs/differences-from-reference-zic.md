# Differences from reference `zic` — the consolidated map

> **One place, nothing hidden.** Every way zic-rs differs from reference `zic` (tzcode 2026b) is
> recorded here in exactly one of **four buckets**. This document does not invent claims; it
> *summarises* evidence each milestone already produced (linked per row), so it grows as the ladder
> lands. If a behaviour is not listed here, the intent is that it is **identical** to reference `zic`
> within the declared scope — and a gap in this table is a bug in the table.
>
> **The four buckets (the standing honesty rule):**
> 1. **Implemented parity** — zic-rs reproduces `zic`'s behaviour.
> 2. **Explicit compatibility mode** — `zic`-compatible behaviour reachable behind an opt-in flag
>    (never the default when it is the riskier choice).
> 3. **Intentional safer divergence** — zic-rs deliberately does something *safer* than `zic`. A
>    safer default is **never** called "parity."
> 4. **Deferred full-parity work** — not done yet; mapped to a milestone.
>
> Separate axes are never collapsed: **behaviour** parity (the contract) ≠ **structural** ≠ **byte** ≠
> **warning** ≠ **operational** parity. Byte parity is claimed only where pinned (or in `zic-slim`).

## The exact claim (and non-claims)

**Claim.** zic-rs behaviour-matches reference `zic`/`zdump` for all **341/341** canonical zones in
`tzdata.zi` 2026b over **`1900..2040`** (341 match · 0 mismatch · 0 fail-closed — "CORE.1"), and
reproduces `zic`'s **structural** TZif output (version byte, footer, `isutcnt`/`isstdcnt`/`leapcnt`/
`typecnt`/`charcnt`) across those 341.

**Platform note:** the compiler core is platform-neutral; the OS-specific install surfaces are gated
and fail closed off-platform — see [platform-portability.md](platform-portability.md) (a Redox-style
audit, not a Redox-support claim).

**Non-claims** (stated plainly, not buried): zic-rs is **not** a full drop-in `zic` across all modes;
it does **not** own time (no data curation, no display names/CLDR, no legal-time authority, no IANA
replacement); behaviour is claimed **only** over the declared horizon; `compile-clean` ≠
`behaviour-match`; byte parity is claimed only where pinned. See [TRUST.md](TRUST.md) (when present)
for the reviewer front door.

---

## Bucket 1 — Implemented parity

| Surface | Evidence |
|---------|----------|
| Canonical-zone **behaviour** (offsets, DST, abbreviations, footer) over 1900..2040, all 341 zones | CORE.1 zdump sweep; [roadmap.md](roadmap.md) T7 |
| Negative `SAVE` (inline + Rule); non-POSIX recurring `ON` (`Sun<=N`/`Sat<=N`) re-anchoring | [reference-zic-semantics.md](reference-zic-semantics.md) laws 7/10; [unsupported-syntax.md](unsupported-syntax.md) |
| **Structural** TZif: content-driven version byte; POSIX footer; abbreviation suffix-sharing packer; `charcnt` | [structural-parity.md](structural-parity.md) (T8-v3/abbrev) |
| Exit-status taxonomy `0`/`1`/`2` | [zic-operational-parity.md](zic-operational-parity.md) §exit-status (T9.2) |
| `-D` (do not create the output dir) ↔ `--no-create-dirs` | zic-operational-parity §filesystem (T9.3) |
| Default file mode **0644** + umask + `-m` octal mode + symlink-leaf safety + dir/file-collision errors | **INSTALL-SEMANTICS.1** matrix — 9/13 match vs reference; the default base mode was 0666 (world-writable under umask 000) and is now **0644** (reference-matching), `reports/install-semantics/` |
| `-l <zone>` localtime link ↔ `--localtime` | zic-operational-parity §localtime (T9.4) |
| `-m <mode>` (octal subset) ↔ `--mode` | zic-operational-parity §mode (T9.5) |
| `-b {slim\|fat}` emission bloat ↔ `--bloat`/`--emit-style` | [zic-range-emission-policy.md](zic-range-emission-policy.md) (T10.2) |
| `-R @hi` redundant tail ↔ `--redundant-until` (slim only; independent of `-b`; behaviour-preserving) | [zic-range-emission-policy.md](zic-range-emission-policy.md) (T10.3) |
| `-r '[@lo][/@hi]'` range truncation ↔ `--range` (incl. the `-00` unspecified boundary) — **all three declared profiles 341/341** (`bounded-2000-2038`, `post-2000`, `post-1970`); claim scoped to declared profiles | [zic-range-emission-policy.md](zic-range-emission-policy.md) (T10.4), [range-truncation-microcases.md](range-truncation-microcases.md) |
| `-L leapseconds` leap-second table ↔ `--leapseconds` (the `right/` profile): Stationary + Rolling leaps, cumulative `adjleap`, `Expires`→TZif-v4 no-op marker — **opt-in, never default**; ordinary zones unchanged | [leap-right-v4-microcases.md](leap-right-v4-microcases.md) (T11) |
| Input source files ↔ `--input` (also accepts directories) | zic-operational-parity flag table |
| `FROM = minimum` accepted as obsolete spelling, coerced to 1900 | [supported-syntax.md](supported-syntax.md) |

## Bucket 2 — Explicit compatibility modes (opt-in)

| Surface | Reference | zic-rs opt-in | Where |
|---|---|---|---|
| Latin-1 historical source (non-UTF-8 in pre-2013 comments) | byte-oriented (accepts) | **`--legacy-latin1`** (LEGACY-SOURCE.1) — admits Latin-1 **only in `#` comments**, refuses it in fields; default is UTF-8-required (`ZIC012`) | `reports/release-all/RECEIPT-LEGACY-SOURCE-1.md` |
| Historical Rule `TYPE` / `yearistype` (`even`/`odd`/`uspres`/`nonpres`) | removed `-y` in tzcode 2020a (can no longer build it) | **`--legacy-yearistype`** (YEARISTYPE.1) — admits the four predicates as **internal deterministic functions** (never executes the historical shell script); default rejects any non-`-` TYPE (`ZIC027`). Verified byte-identical to an admitted historical `zic` oracle on 55/66 pre-2000f releases. *Historical-source replay, not current-reference parity.* | `reports/yearistype/RECEIPT-YEARISTYPE-1.md` |
| Non-POSIX-expressible final recurrence (perpetual year-parity, `1990 max even/odd`) | renders explicit transitions + an empty footer (frozen beyond its private horizon) | **`--legacy-empty-footer`** (PERPETUAL-EXPANSION.1) — on the `ZIC001` footer-synthesis-failure path only, emits the already-expanded explicit transitions (through `RECUR_HI`=2037) + an **empty footer**; default still fails closed. A **footer-emission policy** (no transition-generation change); matches the historical oracle's behaviour over `[1980,2037]`; beyond-horizon freeze not claimed; CORE.1 byte-unchanged. | `reports/perpetual-expansion/RECEIPT-PERPETUAL-EXPANSION-1.md` |


| Mode | What it gives | Note |
|------|---------------|------|
| `--emit-style zic-slim` / `-b slim` | reference `zic`'s **slim** explicit-transition set | byte-pinned where measured; one enumerated residual (`Europe/Lisbon`, ref-fatter-by-1 no-op). [structural-parity.md](structural-parity.md) |
| `--emit-style zic-fat` / `-b fat` | reference `zic`'s **fat** emission | == zic-rs default |
| `--link-mode symlink` | relative symlinks instead of copies for `Link`s | default is `copy` (relocatable, self-contained) |
| `--unsupported skip` | warn-and-skip an unsupported zone | default is fail-closed (`error`) |
| `--force` | clobber existing output | default is atomic no-clobber |

## Bucket 3 — Intentional safer divergences (a safer default is *not* "parity")

| Divergence | Reference `zic` | zic-rs | Evidence |
|------------|-----------------|--------|----------|
| **Required `--out`; no implicit system install** | defaults to installing into `/usr/share/zoneinfo` | output only under an explicit `--out`; never a system path implicitly | zic-operational-parity §shape/safety |
| **No partial install after a fatal** | writes as it goes; a later failure can leave a half-built tree | compiles **all** selected zones to memory, then writes only if all succeed | zic-operational-parity §filesystem (T9.3) |
| **Atomic, no-clobber-by-default writes + cleanup-on-error** | overwrites in place | temp + atomic publish; re-compile without `--force` fails | zic-operational-parity §filesystem |
| **Hard path-traversal reject (`ZIC008`)** | warns under `-v` in some cases | absolute/`..`/traversal/leading-`-`/NUL names are always rejected, never written | zic-operational-parity §safety; `tests/output_safety.rs` |
| **`-t` localtime name constrained to a safe relative name under `--out`** | writes the localtime link to an arbitrary/system path (e.g. `/etc/localtime`) | safe relative name only; absolute/traversal → reject | zic-operational-parity §localtime (T9.4) |
| **`--mode` is Unix-gated and validated-before-write** | `fchmod` best-effort during write | octal-only, validated up front; non-Unix → config error before any write; never applied to symlinks | zic-operational-parity §mode (T9.5) |
| **Fat-style emission *default*** | default is `slim` | default is behaviour-matched **fat-style** (more explicit transitions; never wrong); slim reachable via bucket 2 | [structural-parity.md](structural-parity.md) (T8-slim) |
| **Fail-closed on unsupported constructs (default)** | best-effort / `-v` warnings | refuses to emit an approximate file; explicit `ZIC001` diagnostic | [unsupported-syntax.md](unsupported-syntax.md) |
| **Resource caps on input-driven dimensions** | no caps (source bytes / zone / rule / link / leap counts, link-chain depth all unbounded) | generous hard caps far above any real tzdb; a breach is a plain `Error::config` (exit 1, **not** a `ZIC###` code — an operational safety limit, not a grammar violation) | `src/limits.rs` (T17.1b); `panic-policy.md`; pairs with the TZif-read bounds-guard (T17.1a) |

## Bucket 4 — Deferred full-parity work (mapped to a milestone)

| Surface | `zic` flag/feature | Milestone | Note |
|---------|--------------------|-----------|------|
| **Non-POSIX-expressible perpetual recurring tail** (perpetual year-parity `1990 max even/odd`) | footer/expansion strategy | **✅ resolved (bucket 2)** | **PERPETUAL-EXPANSION.1 closed it:** the explicit-expansion + empty-footer fallback (`--legacy-empty-footer`) landed, gated behind the `ZIC001` synthesis-failure path so CORE.1's 341 zones are untouched. All 11 pre-1995 releases (93b–94f) now behaviour-match the historical oracle over `[1980,2037]`. See the bucket-2 row above + `reports/perpetual-expansion/RECEIPT-PERPETUAL-EXPANSION-1.md` |
| `-D` directory-creation semantics | `-D` | **✅ resolved (bucket 1)** | **ZIC-MATRIX.1 found it, ZIC-MATRIX.1.D fixed it:** zic-rs `-D` now matches reference — never creates a directory; a zone whose parent subdir is absent is skipped, zones whose parent exists are written, and the run exits 1 if any was skipped. Regression-tested; CORE.1 unchanged. See `docs/zic-operational-parity-matrix.md` |
| Verbose / portability warnings | `-v` | **T13** | a warning *taxonomy* (class + location) |
| Owner/group of created files | `-u 'owner[:group]'` | (deferred) | **privileged, Unix-only** install metadata; no flag yet, so it can't be mistaken for a no-op; lands as explicit Unix-only mode if a consumer needs it |
| Legacy `posixrules` link | `-p posixrules` | (deferred, legacy) | reference `zic` itself warns *"-p is obsolete and likely ineffective"*; **never** counted as a zone failure |
| Build-profile reproduction | `backward`/`backzone`/`rearguard`/`vanguard`/… | **T12** | manifest **v8** records the run's identity — detected-vs-claimed tzdb version + build profile (**T12.2 ✅**), the ordered `source_inputs` set (**T12.3 ✅**), the `link_profile` (counts + selected/omitted/failed + `alias_map_sha256`, `AliasMap::validate()` fail-closed) (**T12.4a/b/c ✅**), and `source_profile` evidence axes — `backward_evidence` (**T12.4d ✅**) + `backzone_evidence` (**T12.5b ✅**) hash-anchored to the **signature-verified, hash-pinned tzdb 2026b** reference (**T12.5a.1/a.2 ✅**, all-IANA release-admission matrix **T12.5a.3**) + `packratlist_evidence` backzone-*scope* (**T12.5c ✅** — from an admitted generation-policy input, **never** from compile `source_inputs`) + `dataform_evidence` encoding form (**T12.5d ✅** — hash-matched against the pinned 2026b `main`/`vanguard`/`rearguard.zi` artifacts + `recipe_hash`/`generated_from`; never content-inferred) — detected/claimed/status, hash-backed-or-claim-only, **never inferred** from alias/filename/link-count/output-shape/negative-SAVE; the source-variant arc is closed and `build_profile` carries no `"unknown"` placeholders |
| Warning & diagnostic parity (class/location) | `zic -v` class + location | **T13** | **T13.1** taxonomy inventory (`docs/zic-warning-parity.md`) + **T13.2 ✅** coded classes `ZIC013–017` (UnknownLineType/ContinuationWithoutZone/DuplicateZone/NulByteInInput/OverlongInputLine), `UnknownLineType` unconflated from `ZIC001`, **duplicate-zone detection added** (was silently accepted; now fails closed like reference), and a class/location-first comparison harness (`tests/diagnostic_parity.rs`) — verified ClassLocationMatch vs tzcode 2026b on unknown-kw/dup-zone/NUL/field-count. **T13.3 ✅** added the **warning-collection channel** + first non-fatal class `ZIC018_ABBREVIATION_POLICY_VIOLATION` (`warning ≠ error`; CORE.1 byte output unchanged). **T13.4 ✅** widened abbreviations to the full **`zic.c::checkabbr`** rule: extended ZIC018 length to `< 3`, added `ZIC019_ABBREVIATION_NOT_POSIX`, matched zic's **single-warning precedence** (non-POSIX > too-many > fewer-than-3) and recorded the **`-v`-gating of "fewer than 3"** as a *verbosity* divergence (zic-rs has no quiet mode → always collects; not a class divergence). All ClassLocationMatch vs `zic -v`. **T13.5 ✅** completed the diagnostic *contract*: machine-checked `DiagnosticCode::layer()` + an **operational/materialization** layer (output diagnostics are not source `-v` diagnostics); the **verbosity model** (`DiagnosticVerbosity`, per-diagnostic); **`ZIC020`** transition-count surface (coded; emission T13.6); the **diagnostic-code stability policy** (append-only · (class,location,severity) = stable surface); a **span-precision** audit; and a **reference-platform diagnostic matrix** (*upstream IANA = oracle; vendor/platform `zic` admitted-per-platform, never inferred*). **T13.6 ✅** made it executable: the **verbosity filter** (`compile --verbose`/`-v` — quiet matches `zic`, `-v` matches `zic -v`), **`ZIC020` emitted** (timecnt>1200, verbose-only, byte-preserving), and machine-checked per-code metadata (`layer()`/`span_precision()`/`default_severity()`). **Bucket 1** (implemented parity by class+location), except `ContinuationWithoutZone` = **bucket 3** (finer than reference's "unknown type") and the **`-v`-gating of "fewer than 3"** = a recorded *verbosity* divergence. Exact wording compared **last** (ledger only); further `-v` classes (values-over-24h, pre-1994/2007 clients) are parse-time/version-tail → T14/T15 |
| Hostile-input / parser-edge breadth | — | **T14** | **T14.1 ✅** admissibility inventory (`docs/zic-hostile-input-parity.md`) pinned from `zic.c::inputline`/`getfields` (tzcode 2026b), made **executable** via `tests/input_admissibility.rs` (the T13 lesson). **Line-cap 2048 MATCHES** pinned `_POSIX2_LINE_MAX` (the "511" manpage figure was stale — verified against the pinned source; verified before changing); NUL→`ZIC016`, quoted-`#`/lone-`-`/comment all match. **T14.2 ✅** closed the one leniency divergence the inventory found — **bucket 1, implemented parity**: missing-final-newline is now fatal **`ZIC021_UNTERMINATED_INPUT_LINE`** (matching `inputline`'s "unterminated line"; the witness flipped from lenient→fatal), and odd-quotes get the dedicated **`ZIC022_UNTERMINATED_QUOTE`** (matching `getfields`'s "Odd number of quotation marks"; was the generic `ZIC012`). Both ClassLocationMatch vs `zic -v` on line 1; CORE.1 341/0/0 unchanged. Field-count "too many fields" = documented shape divergence (lexer-arity cap vs per-record count; both reject). **T14.3 ✅** metamorphic source-ordering (`tests/metamorphic_ordering.rs`, bucket 1): permuting independent records / comments / inter-field whitespace → **byte-identical** per-zone output (pinned to `zic.c`'s `qsort` re-sorts), source identity differs; detached continuation rejected (`ContinuationWithoutZone`); diagnostic class stable as line moves. **T14.4 ✅** pathology ledger (`docs/zic-pathology-ledger.md` + `tests/pathology_ledger.rs`): major edge classes classified vs reference 2026b (most matched, bucket 1) — and **found + fixed a panic** (bucket 1 + safety): "two rules for same instant" (reference fatal) was a `debug_assert!` → now fails closed with **`ZIC023_SIMULTANEOUS_TRANSITION`** via an `ensure_strictly_increasing()` guard (removes a panic-on-untrusted-input; CORE.1 untouched). Recorded residuals: multi-era same-instant accepts (bucket 4); `-v` warning "values over 24 hours" (→ ✅ **`ZIC026`**, T15.5-remainder — verbose-only, magnitude rule pinned to `zic.c::gethms`). **T14.5 ✅** `ZoneNamePathPolicy` (`docs/zic-zone-name-path-policy.md` + `tests/zone_name_path_policy.rs`): fatal `ZIC008` structural policy already matched reference `namecheck` (empty·absolute·`//`·trailing-`/`·`.`/`..`) + safer divergences (leading-`-`/NUL/UTF-8-required, bucket 3); **added** verbose-only name-portability warnings **`ZIC024`** (non-benign byte, matches `zic -v` "contains byte", fires on `Etc/GMT+5`) + **`ZIC025`** (>14-byte component) over zones+links; non-UTF-8/reserved-names/case-collisions ledgered (bucket 3/4, platform-deferred). **T14.6 ✅** hostile-output-tree TOCTOU (`docs/zic-hostile-output-tree.md` + `tests/hostile_output_tree.rs`): pre-planted file/symlink/dir at the output leaf fails closed + **never written through** (`hard_link`/`O_EXCL`; `--force`=`rename` replaces, doesn't follow — bucket 1/3 safer); concurrent parent-swap race honestly `NotClaimed`/`RequiresOpenatStyleHardening` → T17/T20. stdin `-` = documented non-capability → T16. **T14 CLOSED** (`reports/t14-close-receipt.md`; ZIC001–ZIC025). |

---

## Reference-compatible surfaces (no divergence)

`--version`/`--help`; reading multiple input files; the obsolete `FROM = minimum` spelling; the
`zishrink` record keys (`R`/`Z`/`L`); `%s`/`%z`/`STD/DST` formats; the `ON` day forms
`lastSun`/`Sun>=N`/`Sun<=N`; multi-era `UNTIL` stitching; both mixed finite+recurring final-era
shapes. These are part of bucket 1 (implemented parity) and are listed here only so the map is
exhaustive rather than selective.

## How to keep this honest

- Every new operational milestone (T10.3 → T15) **adds rows here in the same batch** as its code +
  tests, classified into one of the four buckets — never "mostly compatible."
- A **safer default** goes in bucket 3 and is labelled as such; the reference-compatible behaviour, if
  riskier, is mode-gated (bucket 2), never the default.
- This file is a *summary*. The runnable receipts live in the linked per-surface docs and the test
  suite; the authoritative machine-checkable statement of current support is `zic-rs supported-syntax`
  and `zic-rs support-report`.
