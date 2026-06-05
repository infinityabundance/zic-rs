# RECEIPT — PACKAGE-ACCEPTANCE.1 (zic-rs in a distro-native tzdata build slot) — 2026-06-05

> **Claim wording (binding):** *PACKAGE-ACCEPTANCE.1 does not claim zic-rs is a universal system
> replacement. It tests zic-rs as the `zic` substitute inside bounded package-build environments and records
> installed-tree, permission, symlink/copy, special-file, and behaviour differences against the platform
> reference.*

## Setup (staged DESTDIR — no host mutation)

Replicates the **canonical tzcode/tzdata package-build** `zic` invocations (the pattern every distro
`tzdata` package wraps — Debian `debian/rules`, Arch/Alpine/RPM specs all call these), run with **both**
reference `zic` 2026b and zic-rs, into a disposable `DESTDIR`:

```text
posix tree : zic -d $TZDIR            tzdata.zi
right tree : zic -d $TZDIR-leaps  -L leapseconds  tzdata.zi
localtime  : zic -d $TZDIR  -l Factory  -t $DEST/etc/localtime
tables     : cp zone.tab zone1970.tab iso3166.tab  $TZDIR/
```

**Distribution model (important):** zic-rs ships **only as a crates.io source crate** (`cargo install zic-rs`
`--locked`) — there is **no binary release**. So a real package recipe *builds zic-rs from the crate*, then
substitutes it into the `$(ZIC)` slot. Because zic-rs is deliberately **not a literal argv drop-in**
(subcommand CLI, `--out` not `-d` — the documented bucket-3 CLI-shape divergence), the slot is filled by a
small **argv-compat shim** (`reports/package-acceptance/zic-shim.sh`) that translates `zic -d/-L/-l/-t/-b/-D/-m
files` → zic-rs's flags. The shim *is the bridge* that lets zic-rs occupy `$(ZIC)` without editing the
package Makefile. Inputs generated from the **admitted GOODSIG tzdb-2026b bundle** (`tzdata.zi` sha
`792c579a`, `leapseconds` 27 leaps). Reproduce: `bash reports/package-acceptance/gauntlet.sh`.

## Result — behaviour & file-set identical; differences are all documented *safer* install policies

| profile | ref/zrs files | file set | perms | behaviour (fixtures) | verdict |
|---|---|---|---|---|---|
| **posix** (`zic -d $TZDIR tzdata.zi`) | 598 / 598 | **identical** | `644` == `644` | **6/6 match** | install-semantics-difference |
| **right** (`-L leapseconds`) | 598 / 598 | **identical** | `644` == `644` | **6/6 match** | install-semantics-difference |
| **localtime** (`-l Factory -t …`) | — | — | — | — | install-semantics-difference |

**0 divergent.** The staged zic-rs tree's deterministic **`bundle_hash` = `453641ff2568d8b1`** — the **same
hash** as the host / container / VM / ecosystem-source-build drop-in receipts (cross-environment determinism
holds in the package-build slot too).

### The three install-semantics differences (each a documented bucket-3 *safer* divergence)

1. **Link materialization — hardlink vs copy.** Reference `zic` **hardlinks** identical zones (368 of the 598
   files share inodes — e.g. `America/Montreal` link-count 6); zic-rs writes **independent copies** (link-count
   1, relocatable/self-contained). The **file set and every zone's content are identical**; only the on-disk
   sharing differs. zic-rs's `--link-mode symlink` exists; copy is the safer relocatable default (bucket-3).
2. **`-t` localtime path safety.** Reference `zic -t $DEST/etc/localtime` writes the localtime link to an
   **arbitrary path outside** the zoneinfo dir (succeeded, 113 B). zic-rs **refuses an absolute/outside `-t`**
   (`ZIC008_OUTPUT_PATH_TRAVERSAL`) and **accepts a safe relative name under `--out`** — the documented
   bucket-3 "no write outside the explicit output root" policy. For a **staged** package build (`DESTDIR`)
   this is a *feature*: the localtime link is created within the staged tree, never to a host path.
3. **Slim vs fat default** (implied by the byte sizes): reference `zic` defaults slim, zic-rs defaults
   behaviour-matched fat — `zdump`-identical (the 6/6 behaviour match), the standard structural difference.

## What this proves for replacement

- **zic-rs can occupy the `$(ZIC)` slot** of the canonical tzdata package build (via the argv shim) and
  produce a **behaviour-identical, file-set-identical, same-permissions** installed tree (posix + right).
- The differences are **exactly zic-rs's documented safer install policies** (copy-not-hardlink, refuse
  out-of-root localtime), not data or behaviour divergences — and they are *safer* for staged/relocatable
  package builds.
- The output is **deterministic across every environment** (`bundle_hash 453641ff…`), including this
  package-build slot.

## The shim is a first-class surface (SHIM-CONTRACT.1)

Because zic-rs relies on `zic-shim.sh` to occupy the `$(ZIC)` slot, the shim is **not** test glue — it is a
versioned (`zic-shim-v1`) compatibility surface with a documented contract + threat model
(`SHIM-CONTRACT.md`) and a conformance test (`shim-test.sh`, **8/8**). Writing the conformance test
**found a real bug** (paths with spaces word-split because the shim built an unquoted `inputs` string); the
shim was rewritten to a two-phase argv rotation with no string interpolation, and the test now pins:
accepted/ignored flags, error-status propagation, multiple-`--input` order, **`-t` path-safety preserved**
(absolute refused, relative accepted), **ambient `TZ`/`LC_ALL` quarantine**, and spaces-in-paths. The blessed
`-t` replacement (LOCALTIME-STAGING) is in `SHIM-CONTRACT.md`: zic-rs materialises localtime as a safe
relative name under `--out`, and the *package script* does the write-into-`/etc` step (where it belongs).

## Non-claims

- **Not a universal system replacement.** A maintainer adopting zic-rs must either use the argv shim or adapt
  the recipe to zic-rs's subcommand CLI, and must accept copy-not-hardlink + the out-of-root `-t` refusal.
- Bounded to the canonical tzcode build pattern + 2026b; per-distro packaging wrappers (`.deb`/`.apk`/`.rpm`)
  and their **source-builds of the crate** are the separately-receipted Tier-3 drop-in work
  (`reports/drop-in/`), not re-run here.
- Staged `DESTDIR` only — **no host mutation**; `-u` ownership is deferred (Unix-gated); reference =
  current host `zic` 2026b.
- A hardlink-vs-copy difference increases installed size (no inode sharing); recorded as a fact, not hidden.

## Gate

Docs/report only — no `src/` change; CORE.1 341/0/0 + 519 tests unaffected; doc-staleness green. Cross-linked
from STATUS · `docs/zic-operational-parity-matrix.md` · `docs/differences-from-reference-zic.md` (bucket-3
install policies) · `reports/drop-in/` (Tier-3 source-builds).
