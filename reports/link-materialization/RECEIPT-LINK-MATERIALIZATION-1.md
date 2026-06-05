# RECEIPT — LINK-MATERIALIZATION.1 (copy vs symlink vs hardlink footprint) — 2026-06-05

> **Claim wording (binding):** *LINK-MATERIALIZATION.1 does not require zic-rs to imitate reference `zic`'s
> inode sharing. It measures the package/build consequences of copy, symlink, and reference hardlink
> materialization so maintainers can choose policy with explicit size, inode, and behaviour evidence.*

## Method

The **same** `tzdata.zi` 2026b (598 zones, 257 links) materialized three ways, then measured for installed
footprint, inode count, and shipped (tar + gzip/xz/zstd) package size. Reproduce:
`bash reports/link-materialization/measure.sh` → `link-materialization.tsv`.

- **reference `zic`** — **hardlinks** (links share the target's inode).
- **zic-rs default** — **copy** (each link is an independent, relocatable file).
- **zic-rs `--link-mode symlink`** — **symlinks** (links point at the target path).

## The numbers

| variant | files | symlinks | unique inodes | **installed (du)** | tar | gzip | **xz** | zstd |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| ref (hardlink) | 598 | 0 | **341** | **204,352 B** (200 KiB) | 624,640 | 76,750 | **44,676** | 47,961 |
| zic-rs **copy** | 598 | 0 | 598 | **435,761 B** (426 KiB) | 931,840 | 117,620 | **47,228** | 51,002 |
| zic-rs **symlink** | 341 | 257 | 598 | **251,667 B** (246 KiB) | 665,600 | 78,886 | **46,528** | 49,828 |

## The two deltas that matter (and they are very different)

| | copy ÷ hardlink | symlink ÷ hardlink |
|---|--:|--:|
| **installed footprint (du)** | **2.13×** (+226 KiB) | **1.23×** (+46 KiB) |
| shipped package — gzip | 1.53× (+40 KiB) | 1.03× (+2 KiB) |
| **shipped package — xz** (`.rpm`/`.deb`/`.pkg.tar`/`.tar.lz`) | **1.057× (+2.5 KiB)** | 1.041× (+1.8 KiB) |
| shipped package — zstd | 1.063× | 1.039× |

**The headline for maintainers:** copy-not-hardlink roughly **doubles the *installed* size** (200 → 426 KiB),
but with a modern compressor (**xz/zstd**, what `.rpm`/`.deb`/Arch/the IANA bundle use) the **shipped package
is only ~5–6 % larger (+2.5 KiB)** — because the compressor dedups the identical zone content that hardlinks
would have shared on disk. Only the older **gzip** is materially affected (+53 %). And **`--link-mode
symlink` recovers most of the installed footprint** (1.23× instead of 2.13×) while staying a single
self-contained tree.

## The maintainer's policy choice (now evidence-backed, not directional)

| if you care about… | choose | why |
|---|---|---|
| **shipped package size** (the common case) | **copy** (default) — fine | +2.5 KiB on xz; the compressor erases the difference |
| **installed footprint** (embedded / rootfs / many-zone images) | **`--link-mode symlink`** | 1.23× vs 2.13×; one self-contained tree |
| **maximal compactness + same-filesystem install** | reference hardlink behaviour | 341 inodes; but hardlinks require a single filesystem and break under relocation/overlay copy-up |

zic-rs defaults to **copy** for a reason (the bucket-3 rationale): a copied tree is **relocatable and
self-contained** — no shared inodes to break under `cp`, overlayfs copy-up, container layering, or
cross-filesystem moves; no dangling-symlink risk. The cost is *installed* size, which `--link-mode symlink`
mitigates and which the *shipped* artifact barely feels.

## Non-claims

- zic-rs does **not** reproduce reference hardlink inode-sharing, by design (relocatable default).
- Sizes are for `tzdata.zi` 2026b on this host; absolute bytes vary by release/filesystem, but the **ratios**
  (copy ≈ 2.1× installed, ≈ 1.05× xz-shipped; symlink ≈ 1.23× installed) are the portable result.
- A hardlink `--link-mode` is **not** offered (it would reintroduce the same-filesystem/relocation fragility
  zic-rs's copy default avoids); symlink is the compact middle option.
- Not a per-distro package-size measurement (that is the actual `.deb`/`.rpm` build, PACKAGER-POLICY's next
  rung); this measures the zoneinfo-tree materialization the package wraps.

## Gate

Docs/report only — no `src/` change; CORE.1 341/0/0 + 520 tests unaffected; doc-staleness green. Cross-linked
from STATUS · `reports/packager-policy/` (turns its "directional size delta" into measured evidence) ·
`reports/install-semantics/` · `docs/differences-from-reference-zic.md` (the copy-vs-hardlink bucket-3 row).
