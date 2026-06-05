#!/usr/bin/env bash
# LINK-MATERIALIZATION.1 — measure copy vs symlink vs reference-hardlink footprint for the SAME zoneinfo tree.
# Builds tzdata.zi 2026b three ways and reports installed (du) + shipped (tar + gzip/xz/zstd) sizes + inodes.
set -u
cd "$(dirname "$0")/../.." || exit 2
ZRS=${ZRS:-$PWD/target/release/zic-rs}; PKG=${PKG:-/tmp/pkg}
T=/tmp/lm; rm -rf "$T"; mkdir -p "$T"
OUT=reports/link-materialization/link-materialization.tsv
zic -d "$T/ref" "$PKG/tzdata.zi" 2>/dev/null
"$ZRS" compile --input "$PKG/tzdata.zi" --all-supported --out "$T/copy" >/dev/null 2>&1
"$ZRS" compile --input "$PKG/tzdata.zi" --all-supported --link-mode symlink --out "$T/sym" >/dev/null 2>&1
printf 'variant\tfiles\tsymlinks\tunique_inodes\tinstalled_du_bytes\ttar_bytes\tgzip_bytes\txz_bytes\tzstd_bytes\n' > "$OUT"
for v in ref copy sym; do
  d="$T/$v"
  f=$(find "$d" -type f|wc -l); s=$(find "$d" -type l|wc -l)
  i=$(find "$d" \( -type f -o -type l \) -printf '%i\n'|sort -u|wc -l)
  du=$(du -sb "$d"|cut -f1)
  tar -C "$d" -cf "$T/$v.tar" .; tb=$(stat -c%s "$T/$v.tar")
  g=$(gzip -9 -c "$T/$v.tar"|wc -c); x=$(xz -9 -c "$T/$v.tar"|wc -c); z=$(zstd -19 -c "$T/$v.tar" 2>/dev/null|wc -c)
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$v" "$f" "$s" "$i" "$du" "$tb" "$g" "$x" "$z" >> "$OUT"
done
column -t -s$'\t' "$OUT"
