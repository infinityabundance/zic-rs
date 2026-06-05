#!/usr/bin/env bash
# PERPETUAL-EXPANSION.1 — empty-footer fallback for non-POSIX final recurrence.
# Re-runs the YEARISTYPE.1 band (the 66 pre-2000f releases) with BOTH legacy replay modes
# (--legacy-yearistype --legacy-empty-footer) and compares per-fixture zdump against the admitted
# historical oracle over the DECLARED replay window [1980,2037]. The 11 releases YEARISTYPE.1 left
# `deferred-perpetual-footer` (93b-94f: perpetual `1990 max even/odd` rules → non-POSIX-footer) should
# now build and behaviour-match; the 55 already-matching releases must stay matching (the empty-footer
# fallback is a no-op for them — their footer synthesises fine).
#
# Verifies BEHAVIOUR over [1980,2037], NOT byte identity to the oracle's private expansion horizon
# (tzcode2019c expands to 2392; zic-rs expands through RECUR_HI=2037 + empty footer, frozen beyond —
# that beyond-horizon region is intentionally not claimed).
set -u
cd "$(dirname "$0")/../.." || exit 2
ZRS=${ZRS:-target/release/zic-rs}
ORACLE=${ORACLE:-/tmp/tzcode2019c/zic_oracle}
YIT=${YIT:-/tmp/yit-oracle/yearistype}
CACHE=${CACHE:-/tmp/data-gauntlet}
OUT=reports/perpetual-expansion/perpetual-expansion.tsv
TMP=/tmp/pe-gauntlet; mkdir -p "$TMP"
REGIONS="africa antarctica asia australasia europe northamerica southamerica etcetera backward"
FIX="Australia/Adelaide Australia/South Australia/Broken_Hill Australia/Lord_Howe Europe/London America/New_York"
LO=1980; HI=2037   # the declared replay window (== RECUR_HI); beyond this is intentionally not claimed

[ -x "$ORACLE" ] || { echo "no oracle zic at $ORACLE"; exit 2; }
RIDS=$(awk -F'\t' '$13=="reference-build-incompatible"{print $1}' reports/release-all/data-gauntlet.tsv)

printf 'release_id\teozone\toracle_files\tzrs_files\tfix_compared\tfix_match\tfix_mismatch\tempty_footer\tverdict\tfinding\n' > "$OUT"

for rid in $RIDS; do
  S="$CACHE/s_$rid"
  [ -d "$S" ] || { printf '%s\t-\t0\t0\t0\t0\t0\tno\tunavailable\tno-cached-source\n' "$rid" >> "$OUT"; continue; }
  INPUTS=(); ZREGS=()
  for f in $REGIONS; do [ -f "$S/$f" ] && { INPUTS+=(--input "$S/$f"); ZREGS+=("$S/$f"); }; done
  of="$TMP/o_$rid"; zf="$TMP/z_$rid"; rm -rf "$of" "$zf"
  "$ORACLE" -y "$YIT" -d "$of" "${ZREGS[@]}" >/dev/null 2>&1
  # BOTH legacy replay modes
  "$ZRS" compile --all-supported --unsupported skip --legacy-yearistype --legacy-empty-footer "${INPUTS[@]}" --out "$zf" >/dev/null 2>"$TMP/e_$rid"
  ofc=$(find "$of" -type f 2>/dev/null | wc -l); zfc=$(find "$zf" -type f 2>/dev/null | wc -l)

  eozone=""
  for z in Australia/Adelaide Australia/South; do [ -f "$of/$z" ] && { eozone="$z"; break; }; done
  # did the even/odd zone get an empty footer in zic-rs? (the fallback fingerprint: trailing 0a0a)
  ef=no
  [ -n "$eozone" ] && [ -f "$zf/$eozone" ] && [ "$(tail -c2 "$zf/$eozone" | xxd -p)" = "0a0a" ] && ef=yes

  cmp_n=0; match=0; mis=0; finding=-
  for z in $FIX; do
    op="$of/$z"; zp="$zf/$z"
    [ -f "$op" ] && [ -f "$zp" ] || continue
    cmp_n=$((cmp_n+1))
    zdump -v -c "$LO,$HI" "$op" 2>/dev/null | grep -vE "failed" | sed "s#$op##" > /tmp/_peo
    zdump -v -c "$LO,$HI" "$zp" 2>/dev/null | grep -vE "failed" | sed "s#$zp##" > /tmp/_pez
    if cmp -s /tmp/_peo /tmp/_pez; then match=$((match+1)); else mis=$((mis+1)); [ "$finding" = - ] && finding="zdump-diff:$z"; fi
  done

  if [ "$ofc" -eq 0 ]; then verdict=oracle-build-incompatible
  elif [ "$zfc" -eq 0 ]; then verdict=source-shape-incompatible
  elif [ "$mis" -gt 0 ]; then verdict=zic-rs-divergent
  elif [ -n "$eozone" ] && [ ! -f "$zf/$eozone" ]; then verdict=eozone-skipped; finding="eozone-skipped:$eozone"
  elif [ "$match" -eq "$cmp_n" ] && [ "$cmp_n" -gt 0 ]; then verdict=match
  else verdict=inconclusive; fi
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$rid" "${eozone:--}" "$ofc" "$zfc" "$cmp_n" "$match" "$mis" "$ef" "$verdict" "$finding" >> "$OUT"
done

echo "=== PERPETUAL-EXPANSION.1: $(($(wc -l < "$OUT")-1)) releases (both legacy modes) ==="
echo "verdict tally:"; cut -f9 "$OUT" | tail -n +2 | sort | uniq -c
echo "even/odd zone got empty footer (the 11 perpetual band):"; awk -F'\t' '$8=="yes"{c++}END{print "  "c" releases"}' "$OUT"
echo "any zic-rs-divergent?"; awk -F'\t' '$9=="zic-rs-divergent"{print "  "$1" "$10}' "$OUT" || echo "  none"
echo "total fixture comparisons: $(awk -F'\t' 'NR>1{c+=$5;m+=$6}END{print m"/"c" match over [1980,2037]"}' "$OUT")"
