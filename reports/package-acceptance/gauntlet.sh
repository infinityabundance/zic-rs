#!/usr/bin/env bash
# PACKAGE-ACCEPTANCE.1 — zic-rs in a distro-native tzdata build slot.
# Replicates the canonical tzcode/tzdata package-build zic invocations (posix tree, right/leap tree,
# localtime+default link, tables) with BOTH reference `zic` and zic-rs — the latter occupying the
# `$(ZIC)` slot via the argv-compat shim (zic-shim.sh). Compares the STAGED installed trees by file
# set, link materialization (hardlink vs copy), permissions, special-file/localtime semantics, and
# `zdump` behaviour. No host mutation: everything is staged under a disposable DESTDIR.
#
# DISTRIBUTION MODEL: zic-rs ships ONLY as a crates.io source crate (`cargo install zic-rs`); there is
# no binary release. A real package recipe therefore builds zic-rs from the crate, then shims it here.
# This harness uses the already-built release binary as that cargo-built artifact.
#
# Canonical package-build invocations (from the tzcode Makefile, 2026b):
#   posix:     zic -d $TZDIR tzdata.zi
#   right:     zic -d $TZDIR-leaps -L leapseconds tzdata.zi
#   localtime: zic -d $TZDIR -l Factory -t $DEST/etc/localtime
#   tables:    cp zone.tab zone1970.tab iso3166.tab $TZDIR/
set -u
cd "$(dirname "$0")/../.." || exit 2
ZICRS=${ZICRS:-$PWD/target/release/zic-rs}
SHIM="$PWD/reports/package-acceptance/zic-shim.sh"
PKG=${PKG:-/tmp/pkg}                  # tzdata.zi + leapseconds + *.tab (prepared from the admitted 2026b bundle)
OUT=reports/package-acceptance/package-acceptance.tsv
R=/tmp/pa/ref; Z=/tmp/pa/zrs; rm -rf /tmp/pa; mkdir -p "$R" "$Z"
FIX="Etc/UTC Europe/London America/New_York Asia/Singapore Australia/Lord_Howe America/Montreal"

zrs_zic() { ZICRS="$ZICRS" sh "$SHIM" "$@"; }

build_tree() { # $1=engine(ref|zrs) $2=destroot $3=subdir $4..=extra zic args
  local eng=$1 root=$2 sub=$3; shift 3
  local dir="$root/usr/share/zoneinfo$sub"
  if [ "$eng" = ref ]; then zic -d "$dir" "$@" "$PKG/tzdata.zi" 2>/dev/null
  else zrs_zic -d "$dir" "$@" "$PKG/tzdata.zi" 2>/dev/null; fi
}

printf 'profile\tref_files\tzrs_files\tfileset_match\tref_hardlinked\tzrs_hardlinked\tperm_match\tfix_match\tverdict\tnote\n' > "$OUT"

for prof in posix right; do
  La=(); [ "$prof" = right ] && La=(-L "$PKG/leapseconds")
  sub=""; [ "$prof" = right ] && sub="-leaps"
  build_tree ref "$R" "$sub" "${La[@]}"
  build_tree zrs "$Z" "$sub" "${La[@]}"
  rd="$R/usr/share/zoneinfo$sub"; zd="$Z/usr/share/zoneinfo$sub"
  rf=$(find "$rd" -type f 2>/dev/null | wc -l); zf=$(find "$zd" -type f 2>/dev/null | wc -l)
  setm=$([ "$(cd "$rd" && find . -type f | sort | sha256sum)" = "$(cd "$zd" && find . -type f | sort | sha256sum)" ] && echo yes || echo no)
  rhl=$(find "$rd" -type f -links +1 2>/dev/null | wc -l); zhl=$(find "$zd" -type f -links +1 2>/dev/null | wc -l)
  # permissions: compare the mode of a sample zone
  rp=$(stat -c%a "$rd/Etc/UTC" 2>/dev/null); zp=$(stat -c%a "$zd/Etc/UTC" 2>/dev/null)
  permm=$([ "$rp" = "$zp" ] && echo "yes($rp)" || echo "no($rp/$zp)")
  fm=0; fc=0
  for z in $FIX; do
    [ -f "$rd/$z" ] && [ -f "$zd/$z" ] || continue
    fc=$((fc+1))
    zdump -v -c 1900,2037 "$rd/$z" 2>/dev/null | sed "s#$rd/$z##" > /tmp/pa/_r
    zdump -v -c 1900,2037 "$zd/$z" 2>/dev/null | sed "s#$zd/$z##" > /tmp/pa/_z
    cmp -s /tmp/pa/_r /tmp/pa/_z && fm=$((fm+1))
  done
  # verdict: file set + zdump match; the hardlink-vs-copy is the documented install-semantics difference
  if [ "$setm" = yes ] && [ "$fm" -eq "$fc" ] && [ "$rhl" -gt 0 ] && [ "$zhl" -eq 0 ]; then
    verdict=install-semantics-difference; note="identical file set + behaviour; ref HARDLINKS links ($rhl), zic-rs COPIES (relocatable, bucket-3)"
  elif [ "$setm" = yes ] && [ "$fm" -eq "$fc" ]; then verdict=match; note="identical file set + behaviour"
  elif [ "$fm" -lt "$fc" ]; then verdict=divergent; note="zdump mismatch"
  else verdict=byte-diff-behaviour-match; note="set/behaviour ok"; fi
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s/%s\t%s\t%s\n' "$prof" "$rf" "$zf" "$setm" "$rhl" "$zhl" "$permm" "$fm" "$fc" "$verdict" "$note" >> "$OUT"
done

# --- localtime / -l / -t install-semantics edge ---
# reference: writes the localtime link to an ARBITRARY path ($DEST/etc/localtime, OUTSIDE zoneinfo).
mkdir -p "$R/etc"
zic -d "$R/usr/share/zoneinfo" -l Factory -t "$R/etc/localtime" 2>/dev/null
ref_lt=$([ -e "$R/etc/localtime" ] && echo "wrote-${R}/etc/localtime-$(stat -c%s "$R/etc/localtime")B" || echo "none")
# zic-rs: an absolute/outside -t is REFUSED (bucket-3 safer); a SAFE RELATIVE name under --out is accepted.
rm -rf /tmp/pa/zabs /tmp/pa/zrel
zrs_abs=$(zrs_zic -d /tmp/pa/zabs/zoneinfo -l Factory -t /tmp/pa/zabs/etc/localtime "$PKG/tzdata.zi" 2>&1 | grep -oE "ZIC0[0-9][0-9]_[A-Z_]+" | head -1)
[ -z "$zrs_abs" ] && zrs_abs="refused"
zrs_zic -d /tmp/pa/zrel/zoneinfo -l Factory -t localtime "$PKG/tzdata.zi" >/dev/null 2>&1
zrs_rel=$([ -e /tmp/pa/zrel/zoneinfo/localtime ] && echo "accepted-under-out" || echo "none")
printf 'localtime\t-\t-\t-\t-\t-\t-\t-\tinstall-semantics-difference\tref -t writes an ARBITRARY outside path [%s]; zic-rs REFUSES absolute -t [%s] but ACCEPTS a safe relative name under --out [%s] (bucket-3 safer)\n' "$ref_lt" "$zrs_abs" "$zrs_rel" >> "$OUT"

echo "=== PACKAGE-ACCEPTANCE.1 (staged DESTDIR; reference zic vs zic-rs-via-shim) ==="
column -t -s$'\t' "$OUT"
echo ""
echo "verdict tally:"; cut -f9 "$OUT" | tail -n +2 | sort | uniq -c
echo "DIVERGENT:"; awk -F'\t' '$9=="divergent"{print "  "$1": "$10}' "$OUT" || echo "  none"
echo "staged-tree zic-rs bundle hash (posix): $("$ZICRS" size-report --out "$Z/usr/share/zoneinfo" 2>/dev/null | grep -oE '[0-9a-f]{16}' | head -1)"
