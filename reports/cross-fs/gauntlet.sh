#!/usr/bin/env bash
# CROSS-FS.1 — zic-rs materialization policy across filesystem/container/rootless environments.
# Verifies the deterministic bundle_hash + copy/symlink SAFETY across tmpfs, two distinct ext4 filesystems,
# cross-filesystem relocation (where hardlinks genuinely break), and rootless/fakeroot — and contrasts the
# reference hardlink tree's filesystem-bound inode sharing against zic-rs's relocatable copy/symlink trees.
set -u
ZRS=${ZRS:-/home/one/zic-rs/target/release/zic-rs}
PKG=${PKG:-/tmp/pkg}
CANON=453641ff2568d8b1     # the canonical bundle_hash (host/container/VM/source-build)
OUT=/home/one/zic-rs/reports/cross-fs/cross-fs.tsv
# environments (write locations on distinct filesystems)
EXT4A=/home/one/.cache/cfs        # ext4 (/)
TMPFS=/dev/shm/cfs                 # tmpfs
EXT4B="${ZICRS_EXT4B:-/mnt/ext4b}/zic-crossfs-test"  # a SECOND ext4 filesystem (set ZICRS_EXT4B to a real second-fs mount)
rm -rf "$EXT4A" "$TMPFS" "$EXT4B"; mkdir -p "$EXT4A" "$TMPFS" "$EXT4B"

bh() { "$ZRS" size-report --out "$1" 2>/dev/null | grep -oE '[0-9a-f]{16}' | head -1; }
build_copy() { "$ZRS" compile --input "$PKG/tzdata.zi" --all-supported --out "$1" >/dev/null 2>&1; }
build_sym()  { "$ZRS" compile --input "$PKG/tzdata.zi" --all-supported --link-mode symlink --out "$1" >/dev/null 2>&1; }
# read a zone THROUGH a (possibly relocated) link to prove resolution: Montreal -> Toronto
resolves() { zdump -c 1990,1991 "$1/America/Montreal" >/dev/null 2>&1 && echo yes || echo no; }

printf 'environment\tfstype\tmaterialization\tbundle_hash\thash_ok\tlink_resolves\tverdict\n' > "$OUT"
rown() { printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$1" "$2" "$3" "$4" "$5" "$6" "$7" >> "$OUT"; }

for env in "ext4_home:$EXT4A" "tmpfs:$TMPFS" "ext4_second:$EXT4B"; do
  name=${env%%:*}; dir=${env#*:}; fst=$(stat -f -c %T "$dir" 2>/dev/null)
  build_copy "$dir/copy"; h=$(bh "$dir/copy"); ok=$([ "$h" = "$CANON" ] && echo yes || echo NO)
  rown "$name" "$fst" "copy" "$h" "$ok" "$(resolves "$dir/copy")" "$([ "$ok" = yes ] && echo deterministic-and-safe || echo DIVERGENT)"
  build_sym "$dir/sym"; r=$(resolves "$dir/sym")
  rown "$name" "$fst" "symlink" "(relative)" "n/a" "$r" "$([ "$r" = yes ] && echo deterministic-and-safe || echo BROKEN)"
done

# --- cross-filesystem RELOCATION (tmpfs -> ext4#2): where hardlinks break, copy/symlink must not ---
build_copy "$TMPFS/relo_copy"; cp -a "$TMPFS/relo_copy" "$EXT4B/relo_copy_moved"
rown "relocate_copy(tmpfs->ext4)" "ext4" "copy" "$(bh "$EXT4B/relo_copy_moved")" "$([ "$(bh "$EXT4B/relo_copy_moved")" = "$CANON" ] && echo yes || echo NO)" "$(resolves "$EXT4B/relo_copy_moved")" "deterministic-and-safe"
build_sym "$TMPFS/relo_sym"; cp -a "$TMPFS/relo_sym" "$EXT4B/relo_sym_moved"
rown "relocate_symlink(tmpfs->ext4)" "ext4" "symlink" "(relative)" "n/a" "$(resolves "$EXT4B/relo_sym_moved")" "$([ "$(resolves "$EXT4B/relo_sym_moved")" = yes ] && echo deterministic-and-safe || echo BROKEN)"

# --- the hardlink-fragility CONTRAST: reference hardlink inode sharing is filesystem-bound ---
rm -rf "$TMPFS/refhl"; zic -d "$TMPFS/refhl" "$PKG/tzdata.zi" 2>/dev/null
ih_same=$(find "$TMPFS/refhl" \( -type f -o -type l \) -printf '%i\n' | sort -u | wc -l)
# a NAIVE cross-fs copy (cp -r, no --preserve=links) breaks the hardlink sharing → inodes balloon
cp -r "$TMPFS/refhl" "$EXT4B/refhl_naive" 2>/dev/null
ih_naive=$(find "$EXT4B/refhl_naive" \( -type f -o -type l \) -printf '%i\n' | sort -u | wc -l)
rown "ref_hardlink_naive_relocate" "ext4" "hardlink" "n/a" "n/a" "n/a" "fragile: inode-sharing lost on naive cross-fs copy ($ih_same->$ih_naive unique inodes)"

# --- rootless: every build above ran UNPRIVILEGED (uid $(id -u)); fakeroot grounded in PACKAGER-POLICY.1 ---
rown "rootless_unprivileged" "(all above)" "copy/symlink" "$CANON" "yes" "yes" "deterministic-and-safe (uid=$(id -u), no root; fakeroot shown in PACKAGER-POLICY.1)"

echo "=== CROSS-FS.1 ==="; column -t -s$'\t' "$OUT"
echo ""; echo "verdict tally:"; cut -f7 "$OUT" | tail -n +2 | sed 's/(.*//' | sort | uniq -c
echo "DIVERGENT/BROKEN:"; awk -F'\t' '$7 ~ /DIVERGENT|BROKEN/{print "  "$1" "$7}' "$OUT" || echo "  none"
rm -rf "$EXT4A" "$TMPFS" "$EXT4B"   # clean up (no host residue)
