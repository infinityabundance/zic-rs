# zic flag/mode operational parity matrix (ZIC-MATRIX.1)

> **Claim wording (binding):** *ZIC-MATRIX.1 does not claim total upstream `zic` replacement. It maps the
> operational surface of reference `zic` into receipt-backed verdicts so maintainers can see which modes
> match, which differ intentionally, which are unsupported by design, and which remain deferred or divergent.*

This is a **runnable** matrix, not a prose table. Every row is produced by `reports/zic-matrix/gauntlet.sh`,
which runs the flag on **both** reference `zic` and `zic-rs` over a committed fixture set
(`reports/zic-matrix/{sample,sample2,warn,bad,subdir}.zi`) and compares the real output: the produced
**file-set**, per-zone **behaviour** (`zdump` over 1970..2040), file **mode**, **exit status**, and
**diagnostic class** where applicable. The raw results are `reports/zic-matrix/matrix.tsv`.

## Verdict vocabulary (kept strictly distinct)

- **match** — equivalent observable result (file-set + `zdump` behaviour, or the mode/exit the flag governs).
- **class-parity** — the diagnostic *class/location* matches; exact wording is **not** claimed (wording last).
- **intentional-divergence** — a deliberate, documented difference (usually a safer default).
- **unsupported-by-design** — zic-rs deliberately does not implement it (with the user-facing alternative noted).
- **deferred** — a real `zic` surface zic-rs has not implemented yet (no silent no-op).
- **divergent** — an actual behavioural difference that is **a named finding**, to reconcile (not hidden).
- **not-applicable** — not a `zic` CLI flag (e.g. a source-set/Makefile concern).

*unsupported / deferred / divergent are never collapsed — that distinction is the whole value.*

## The matrix

| flag/mode | reference `zic` | zic-rs | ref exit | zrs exit | verdict | note |
|---|---|---|--|--|---|---|
| `default compile` | `zic -d OUT sample.zi` | `zic-rs compile --all-supported --input sample.zi --out OUT` | 0 | 0 | **✅ match** | basic Zone/Rule/Link compile |
| `-d / --out` | `zic -d DIR …` | `zic-rs compile --out DIR …` | 0 | 0 | **✅ match** | -d ≡ --out (output directory) |
| `-b slim` | `zic -b slim …` | `zic-rs … --emit-style zic-slim` | 0 | 0 | **✅ match** | slim structural: Test/Matrix null-diff (T8 slim residual class is behaviourally null) |
| `-b fat` | `zic -b fat …` | `zic-rs … -b fat` | 0 | 0 | **✅ match** | fat is zic-rs default |
| `-r @lo/@hi` | `zic -r @lo/@hi …` | `zic-rs … -r @lo/@hi` | 0 | 0 | **✅ match** | bounded range matches; open-ended -r has named residuals (T10.4e) |
| `-L leapseconds` | `zic -L leapseconds …` | `zic-rs … -L leapseconds` | 0 | 0 | **✅ match** | right/ leap profile; cctz cannot read leap files (reader limit, not zic-rs) |
| `-v verbose` | `zic -v warn.zi` | `zic-rs compile -v warn.zi` | 0 | 0 | **◑ class-parity** | both flag the >6-char abbreviation (class match; exact wording is NOT claimed — wording last) |
| `-l localtime` | `zic -l Test/Matrix …` | `zic-rs … -l Test/Matrix` | 1 | 0 | **⇄ intentional-divergence** | reference zic -l writes localtime to the SYSTEM default (TZDEFAULT, e.g. /etc — hence the non-root permission fail/exit≠0); zic-rs writes localtime UNDER --out only — deliberate safer-divergence (T9.4). The link's target behaviour is equivalent; the install LOCATION differs by design |
| `-t localtime-name` | `zic -l … -t Local/Custom` | `zic-rs … -t Local/Custom` | 0 | 0 | **✅ match** | custom localtime link name; zic-rs constrains to safe relative name under --out |
| `-D no-create-dirs` | `zic -D -d OUT(flat,Test/ missing)` | `zic-rs compile -D --out OUT(flat,Test/ missing)` | 1 | 1 | **✅ match** | FIXED (ZIC-MATRIX.1.D): -D forbids creating missing zone subdirs; zones needing them are skipped & the run exits 1, zones whose parent already exists are written (matches reference write-what-fits semantics) |
| `-m mode` | `zic -m 600 …` | `zic-rs -m 600 …` | 0 | 0 | **✅ match** | octal subset; symbolic chmod exprs = intentional-simplification (T9.5) |
| `-p posixrules` | `zic -p posixrules …` | `(no -p flag)` | 0 | 2 | **⊘ unsupported-by-design** | reference zic itself warns -p is obsolete & likely ineffective; zic-rs does not implement it (would be opt-in bucket-2 only if a real consumer needs it) |
| `-u owner` | `zic -u owner …` | `(no -u flag)` | 0 | n/a | **⋯ deferred** | privileged Unix-only install metadata; zic-rs has no -u (so it can't be mistaken for a silent no-op); lands as explicit Unix-only mode if a consumer needs it (T9.5) |
| `--version` | `zic --version` | `zic-rs --version` | 0 | 0 | **⇄ intentional-divergence** | different product/version string by design (separate tools) |
| `--help` | `zic --help` | `zic-rs --help` | 0 | 0 | **⇄ intentional-divergence** | zic-rs is a subcommand CLI (compile/compare/…); deliberately NOT an argv drop-in (drop-in-compatibility-contract.md) |
| `multiple input files` | `zic f1 f2 …` | `zic-rs --input f1 --input f2 …` | 0 | 0 | **✅ match** | order-independent record set (metamorphic, T14.3) |
| `links / aliases` | `Link Test/Matrix Test/MatrixAlias` | `(same source)` | 0 | 0 | **✅ match** | links materialised (copy by default); cycle/self-link = hard error in both |
| `backzone (source-set)` | `zic <backzone file> (just another input)` | `zic-rs --input <backzone> [--backzone included]` | n/a | n/a | **– not-applicable** | backzone is a Makefile/source-set choice, not a zic CLI flag; passing the file = ordinary input (compiles in both); zic-rs additionally records hash-backed provenance evidence (T12.5b), never inferred |
| `invalid input` | `zic bad.zi (unknown line type)` | `zic-rs compile bad.zi` | 1 | 1 | **◑ class-parity** | both reject (exit≠0); zic-rs is deliberately FINER on continuation-without-zone (ZIC014, intentional-divergence T13.2) |


**Tally (19 rows):** 11 match · 3 intentional-divergence · 2 class-parity · 1 unsupported-by-design ·
1 deferred · 1 not-applicable · **0 divergent.**

## Findings (surfaced by the runs)

1. **`-D` directory-creation suppression — FOUND then FIXED (`divergent` → `match`).** The first run of this
   matrix found a real divergence: reference `zic -D` forbids **all** directory creation (given a flat,
   pre-existing output dir it *skips* any zone needing a missing subdirectory, writing what it can and
   exiting 1), whereas zic-rs `-D` still created zone subdirectories. **ZIC-MATRIX.1.D closed it**
   (`src/compile/plan.rs`): under `-D` zic-rs no longer creates any directory — a zone whose parent subdir
   is absent is skipped, zones whose parent exists are written, and the run exits 1 if any was skipped.
   Verified equal to reference on all three cases (flat/missing-subdir → exit 1, 0 files · complete tree →
   exit 0, all files · mixed → exit 1, root-level zone written, subdir zone skipped); regression test
   `tests/cli_operational_parity.rs::no_create_dirs_skips_zone_needing_missing_subdir`. The default
   (non-`-D`) path is byte-unchanged → CORE.1 341/0/0. **This is the matrix proving itself actionable:
   it found a divergence and the divergence was closed.**

2. **`-l localtime` install location (`intentional-divergence`, for context).** Reference `zic -l` writes the
   `localtime` link to the **system default** (`TZDEFAULT`, e.g. `/etc/localtime`) — so as a non-root user it
   fails with a permission error and a non-zero exit. zic-rs writes `localtime` **under `--out` only** (T9.4),
   a deliberate safety posture; the link's *target behaviour* is equivalent, only the install location differs.

## Honest boundaries (what this matrix does and does not say)

- It covers the reference-`zic` **CLI flag/mode** surface, on a small representative fixture set — **not** all
  zones (CORE.1 covers all 341 canonical zones; T18 the 40-case stress set), **not** all releases (only the
  installed 2026b reference), **not** every diagnostic class (T13/T14 own that), **not** every `zic` build.
- `-b slim` and `-r` are "behaviour-match"; their *byte/structural* residuals are the documented T8 slim-tail
  and the T10.4e open-ended-`-r` named residuals — behaviourally null, separate axis.
- A `match` here is scoped to *these flags, this fixture set, this reference `zic` (2026b) build, this host*.

## Reproduce

```sh
bash reports/zic-matrix/gauntlet.sh    # needs reference zic/zdump 2026b; writes only /tmp; emits matrix.tsv
```

## Cross-references

`docs/zic-operational-parity.md` (the per-flag inventory + exit-status contract) ·
`docs/differences-from-reference-zic.md` (the four-bucket difference map) · `STATUS.md` (live state) ·
`TRUST.md` · `docs/REVIEW-IN-10-MINUTES.md` · `docs/zic-range-emission-policy.md` (`-r`/`-R`/`-b`) ·
`docs/replacement-readiness-ladder.md` · `docs/release-ladder.md` (RELEASE-LADDER.1) · `docs/tzdb-evidence-atlas.md` (TZDB-ATLAS — the upstream×vendor×drop-in join).

**Source-profile axis (SOURCE-VARIANT.1):** the CLI/flag matrix above is orthogonal to the *source-profile*
matrix — zic-rs behaviour-matches reference `zic` on **all** major tzdb source profiles (`DATAFORM`
main/vanguard/rearguard + backzone excluded/included), 8/8 fixtures each, 0 divergent, and normalises the
three `DATAFORM` encodings to byte-identical output. See `reports/source-variant/RECEIPT-SOURCE-VARIANT-1.md`
and `docs/build-profile-parity.md` (T12.5, the source-variant *evidence-axis* work this behaviour-verifies).
