# Drop-in compatibility contract (T19)

> The brutal, surface-by-surface answer to the packager's question: **"what breaks if I alias `zic` →
> `zic-rs`?"** No "mostly compatible" hand-waving — each surface is `matched` / `intentionally divergent`
> / `not claimed` / `future`, with where it is proven. Pairs with `docs/zic-operational-parity.md`,
> `docs/differences-from-reference-zic.md` (the four-bucket map), and `docs/replacement-readiness-ladder.md`.

## The headline (read this first)

**zic-rs is NOT a literal argv drop-in for `zic`, by design.** You **cannot** `ln -s zic-rs
/usr/sbin/zic` and expect `zic file` to work: zic-rs is a **subcommand** CLI (`zic-rs compile --input
FILE --out DIR …`) with a **required `--out`** and **no implicit system install** — a deliberate safety
posture (bucket-3 *intentional safer divergence*), not an oversight. What *is* compatible is the
**produced TZif behaviour** (CORE.1) and most flag **concepts**, reached through zic-rs's safer surface.
So "drop-in" here means *"the output and the operations are reproducible,"* not *"the command line is
interchangeable."*

## Surface-by-surface

| Surface | Reference `zic` | zic-rs | Status | Where |
|---|---|---|---|---|
| **argv shape** | `zic [opts] file…`, installs into system zoneinfo by default | `zic-rs compile --input … --out …` (subcommands; `--out` required; no implicit install) | **intentionally divergent** (aliasing breaks) | `zic-operational-parity.md` §shape |
| **exit codes** | `0` ok / non-zero fail | `0` ok · `1` operational/compiler/config · `2` clap usage | **matched + finer** (the 0/1 split holds; `2` = usage) | `cli-compatibility-policy.md` |
| **stdout / stderr** | warnings/errors to stderr | diagnostics to stderr; **structured JSON reports to stdout** (`--format json`) | **divergent (superset)** | `cli-compatibility-policy.md` |
| **diagnostic class/location** | `zic -v` text | typed `ZIC001`–`ZIC026` (class·severity·span); wording **not** claimed | **typed parity** (class/location) | `reports/t13/t14-close-receipt.md` |
| **output TZif bytes** | slim by default | behaviour-matched; **byte-identical only in `--emit-style zic-slim`/`-b slim` or where pinned** | **behaviour matched · bytes profile-dependent** | CORE.1 · `structural-parity.md` |
| **fat / slim / default** | default **slim** | default **fat-style** (behaviour-matched); slim via `-b slim`/`--emit-style zic-slim` | **intentional divergence-in-default** (bucket 3) | `zic-range-emission-policy.md` |
| **`-b` bloat / `-r` range / `-R` redundant** | as `zic.c` | implemented (T10; all `-r` profiles 341/341) | **matched** | `zic-range-emission-policy.md` |
| **`right/` / `posix` trees, leap (`-L`)** | build profile | opt-in `--leapseconds`; `posix` default | **matched (opt-in)** | T11 |
| **`-l`/`-t` localtime** | writes to arbitrary/system path | within `--out` only; safe-relative name | **intentional safer divergence** | T9.4 |
| **`-p posixrules`** | obsolete; `zic` itself warns | **deferred-legacy** (never a zone failure) | **future / out-of-scope** | T9.4 |
| **`-m` mode / `-u` owner** | `fchmod`/`fchown` | `--mode` octal-subset, Unix-only, validated-before-write, never on symlinks; **`-u` deferred** | **partial / deferred** | T9.5 |
| **path traversal / clobber** | warns under `-v` in cases | **hard reject** (`ZIC008`); no-clobber without `--force` | **intentional safer divergence** | `install-materialization-contract.md` |
| **install durability** | in-place write | per-file durable publish (Unix); **whole-tree atomicity not claimed** | **bounded** | `install-materialization-contract.md` |
| **source grammar** | full `zic` grammar | **admitted subset**; fail-closed on unsupported (`ZIC001`) | **subset + known divergence** | `supported-syntax.md` / `unsupported-syntax.md` |
| **stdin (`-`)** | accepted | documented **non-capability** (today) | **not claimed / future** | T16 note |

## What a packager should actually do

1. **Do not alias** `zic`→`zic-rs`. Invoke `zic-rs compile --input <tzdata.zi> --out <staged-dir> [--emit-style zic-slim] [-L <leapseconds>]`.
2. **Side-by-side first** (RRL-2): compile both, `compare`/`structural-report`/`release-diff` against
   reference `zic`/`zdump`, read the reports — **do not install** (the staged-DESTDIR path).
3. **Match the profile you need:** default is fat-style; pass `--emit-style zic-slim` to reproduce slim
   `zic` bytes where you need byte-parity.
4. **Accept the safer divergences** (required `--out`, hard traversal reject, no implicit install) or stay
   at a lower readiness level (`docs/replacement-readiness-ladder.md`).

## Non-claims

- **No argv/exit/stderr parity with reference `zic`** is claimed (the CLI shape is deliberately
  different + safer). This contract governs *what maps to what*, not interchangeability.
- **No byte parity** except in slim mode / where a pinned fixture proves it; the contract is **behaviour**
  parity (CORE.1), with structural/byte as separate axes.
- This is a **compatibility map**, not a statement that zic-rs is a full `zic` (see `not-yet-ready.md`).
- **No adjacent-format compatibility** is claimed unless separately admitted: **TZDIST · `VTIMEZONE` ·
  xCal · jCal · CLDR · ICU-format · Java-format · Windows-registry-format · geospatial zone lookup**. The
  contract above governs the **`zic`→TZif** surface only; the surrounding ecosystem (mapped by `tz-link`,
  S13 in the knowledge index) is explicitly out of scope — naming this prevents claim contamination, it is
  not a weakness.
