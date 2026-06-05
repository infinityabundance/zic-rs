#!/usr/bin/env bash
# RELEASE-ALL.TZDB.1 — complete tzdb-*.tar.lz bundle gauntlet.
# Classifies every signed complete-bundle (data+code in one lzip archive) as a combined release artifact
# and records how its data/code identity relates to the already-admitted RELEASE-ALL.DATA.1 / atlas rows.
#
# Per bundle (tzdb-<v>.tar.lz, all 48 already GOODSIG-verified by RELEASE-ALL.1):
#   identity      : filename version vs the internal `version` file (and NEWS heading)
#   combined?     : does it contain BOTH data (africa…) AND code (zic.c)?  [a complete bundle, by definition]
#   data==pair    : are the bundle's region files BYTE-IDENTICAL to the standalone tzdata<v> pair's? If so,
#                   zic-rs compiling the bundle data is identical to compiling the pair — so the behaviour
#                   verdict INHERITS RELEASE-ALL.DATA.1's already-verified result (no re-compile needed).
#   data1_verdict : the RELEASE-ALL.DATA.1 behaviour verdict for version <v>
#
# lzip is not installed here; the bundles are decoded with the pinned pure-Python decoder unlzip.py.
set -u
cd "$(dirname "$0")/../.." || exit 2
BUNDLES=${BUNDLES:-/tmp/ra-verify}            # the 48 GOODSIG tzdb-*.tar.lz (RELEASE-ALL.1 verify cache)
CACHE=${CACHE:-/tmp/data-gauntlet}            # standalone tzdata pairs: $CACHE/s_<v>
IDX=reports/release-all/index.tsv
VER=reports/release-all/verified.tsv
DG=reports/release-all/data-gauntlet.tsv
UNLZIP=reports/release-all/unlzip.py
OUT=reports/release-all/tzdb-bundles.tsv
TMP=/tmp/tzdb-gauntlet; mkdir -p "$TMP"
REGIONS="africa antarctica asia australasia europe northamerica southamerica etcetera backward"

declare -A SIG HASH D1
while IFS=$'\t' read -r art typ sha sig; do [ "$typ" = "tzdb_complete_bundle" ] && { SIG["$art"]="$sig"; HASH["$art"]="$sha"; }; done < <(tail -n +2 "$VER")
while IFS=$'\t' read -r line; do v=$(printf '%s' "$line" | cut -f1); d=$(printf '%s' "$line" | cut -f13); D1["$v"]="$d"; done < <(tail -n +2 "$DG")

printf 'bundle_id\tversion_file\tnews_top\tsig\thash16\thas_data\thas_code\tdata_vs_pair\tdata1_verdict\tverdict\n' > "$OUT"

for v in $(awk -F'\t' '$2=="tzdb_complete_bundle"{print $3}' "$IDX" | sort -u); do
  lz="$BUNDLES/tzdb-$v.tar.lz"; art="tzdb-$v.tar.lz"
  sig=${SIG[$art]:-unverified}; h=${HASH[$art]:-?}; h16=${h:0:16}
  if [ ! -f "$lz" ]; then printf '%s\t-\t-\t%s\t%s\t-\t-\t-\t%s\tunavailable\n' "$v" "$sig" "$h16" "${D1[$v]:-?}" >> "$OUT"; continue; fi
  d="$TMP/tzdb-$v"; rm -rf "$d"; mkdir -p "$TMP/x_$v"
  python3 "$UNLZIP" "$lz" 2>/dev/null | tar -x -C "$TMP/x_$v" 2>/dev/null
  B="$TMP/x_$v/tzdb-$v"; [ -d "$B" ] || B=$(find "$TMP/x_$v" -maxdepth 1 -type d -name "tzdb-*" | head -1)

  vf=$([ -f "$B/version" ] && tr -d '\n' < "$B/version" || echo "-")
  nt=$(grep -m1 -oE '^Release[[:space:]]+[0-9]+[a-z]*' "$B/NEWS" 2>/dev/null | awk '{print $2}'); nt=${nt:--}
  has_data=$([ -f "$B/africa" ] && echo yes || echo no)
  has_code=$([ -f "$B/zic.c" ] && echo yes || echo no)

  # data identity vs the standalone tzdata pair (byte-compare the 9 region files)
  S="$CACHE/s_$v"; dvp="no_pair"; same=0; tot=0
  if [ -d "$S" ]; then
    for f in $REGIONS; do
      [ -f "$B/$f" ] && [ -f "$S/$f" ] || continue
      tot=$((tot+1)); cmp -s "$B/$f" "$S/$f" && same=$((same+1))
    done
    [ "$tot" -gt 0 ] && { [ "$same" -eq "$tot" ] && dvp="match($same/$tot)" || dvp="DIFFER($same/$tot)"; }
  fi
  d1=${D1[$v]:-?}

  if [ "$has_data" = no ] || [ "$has_code" = no ]; then verdict=not_combined
  elif [ "$vf" != "-" ] && [ "$vf" != "$v" ]; then verdict=bundle_identity_contradiction
  elif [[ "$dvp" == DIFFER* ]]; then verdict=bundle_data_differs_from_pair
  elif [[ "$dvp" == match* ]] && [ "$d1" = "match" ]; then verdict=verified_combined_release
  else verdict=combined_unconfirmed; fi
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$v" "$vf" "$nt" "$sig" "$h16" "$has_data" "$has_code" "$dvp" "$d1" "$verdict" >> "$OUT"
  rm -rf "$TMP/x_$v"
done

echo "=== RELEASE-ALL.TZDB.1: $(($(wc -l < "$OUT")-1)) complete bundles ==="
echo "verdict tally:"; cut -f10 "$OUT" | tail -n +2 | sort | uniq -c
echo "GOODSIG: $(awk -F'\t' '$4=="GOODSIG"{c++}END{print c+0}' "$OUT")/48"
echo "data byte-identical to standalone pair: $(awk -F'\t' '$8 ~ /^match/{c++}END{print c+0}' "$OUT")/48"
echo "combined (data+code present): $(awk -F'\t' '$6=="yes"&&$7=="yes"{c++}END{print c+0}' "$OUT")/48"
echo "identity contradictions (version file != filename):"; awk -F'\t' '$10=="bundle_identity_contradiction"{print "  "$1": version="$2}' "$OUT" || echo "  none"
echo "data differs from pair:"; awk -F'\t' '$10=="bundle_data_differs_from_pair"{print "  "$1": "$8}' "$OUT" || echo "  none"
