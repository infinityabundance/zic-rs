#!/usr/bin/env bash
# YEARISTYPE.1 — historical Rule TYPE / -y replay gauntlet.
# The 66 pre-2000f stable tzdata releases use the `even`/`odd` yearistype predicates (the `AS` rules for
# Australia/Adelaide & Broken_Hill, 1990-1994). Current reference `zic` REMOVED `-y`/yearistype in tzcode
# 2020a, so it cannot build these — they are the atlas `reference_zic` band. This gauntlet uses an
# ADMITTED HISTORICAL ORACLE (an old `zic` that still has `-y`, plus the pinned `yearistype.sh` v7.4) as
# the reference, and compares per-fixture `zdump` behaviour against zic-rs `--legacy-yearistype`.
#
# Oracle (admitted, hash-pinned — recorded in the receipt):
#   ORACLE = old zic built from tzcode2019c (the last era with `-y`; yearistype is script-delegated so
#            the TYPE semantics are invariant across tzcode versions — they live in yearistype.sh).
#   YIT    = yearistype.sh v7.4 (sha256 86cfb6b1…), the script every affected release shipped.
# Both default to the lab build under /tmp; override via env for reproduction.
set -u
cd "$(dirname "$0")/../.." || exit 2
ZRS=${ZRS:-target/release/zic-rs}
ORACLE=${ORACLE:-/tmp/tzcode2019c/zic_oracle}
YIT=${YIT:-/tmp/yit-oracle/yearistype}
CACHE=${CACHE:-/tmp/data-gauntlet}          # extracted release sources live in $CACHE/s_<rid>
OUT=reports/yearistype/yearistype.tsv
TMP=/tmp/yit-gauntlet; mkdir -p "$TMP"
REGIONS="africa antarctica asia australasia europe northamerica southamerica etcetera backward"
# Fixtures: the even/odd-bearing zone is Australia/Adelaide in 1995+ but was named Australia/South in the
# 1993-94 era (renamed later) — both are checked. (Lord_Howe uses the UN-typed LH rules, so it is only a
# control, not a yearistype witness.) The verdict turns on the even/odd zone (EOZONE) the oracle produced.
FIX="Australia/Adelaide Australia/South Australia/Broken_Hill Australia/Lord_Howe Europe/London America/New_York"
LO=1980; HI=2037                            # covers the even/odd era + the perpetual tail (pre-1995 max rules)

[ -x "$ORACLE" ] || { echo "no oracle zic at $ORACLE (build tzcode2019c: cc -w -o zic_oracle zic.c with a version.h stub)"; exit 2; }
[ -x "$YIT" ] || { echo "no yearistype script at $YIT"; exit 2; }

# the 66-release band = the data-gauntlet's reference-build-incompatible rows
RIDS=$(awk -F'\t' '$13=="reference-build-incompatible"{print $1}' reports/release-all/data-gauntlet.tsv)

printf 'release_id\tsrc_regions\toracle_files\tzrs_files\tfix_compared\tfix_match\tfix_mismatch\tverdict\tfinding\n' > "$OUT"

for rid in $RIDS; do
  S="$CACHE/s_$rid"
  [ -d "$S" ] || { printf '%s\t0\t0\t0\t0\t0\t0\tunavailable\tno-cached-source\n' "$rid" >> "$OUT"; continue; }
  INPUTS=(); ZREGS=(); n=0
  for f in $REGIONS; do [ -f "$S/$f" ] && { INPUTS+=(--input "$S/$f"); ZREGS+=("$S/$f"); n=$((n+1)); }; done
  of="$TMP/o_$rid"; zf="$TMP/z_$rid"; rm -rf "$of" "$zf"
  # oracle: old zic with -y pointing at the historical yearistype script
  "$ORACLE" -y "$YIT" -d "$of" "${ZREGS[@]}" >/dev/null 2>&1
  # zic-rs under explicit legacy replay
  "$ZRS" compile --all-supported --unsupported skip --legacy-yearistype "${INPUTS[@]}" --out "$zf" >/dev/null 2>"$TMP/e_$rid"
  ofc=$(find "$of" -type f 2>/dev/null | wc -l); zfc=$(find "$zf" -type f 2>/dev/null | wc -l)

  # EOZONE = the even/odd (yearistype) zone the ORACLE produced for this era (Adelaide or South).
  eozone=""
  for z in Australia/Adelaide Australia/South; do [ -f "$of/$z" ] && { eozone="$z"; break; }; done

  cmp_n=0; match=0; mis=0; finding=-
  for z in $FIX; do
    op="$of/$z"; zp="$zf/$z"
    [ -f "$op" ] && [ -f "$zp" ] || continue
    cmp_n=$((cmp_n+1))
    zdump -v -c "$LO,$HI" "$op" 2>/dev/null | sed "s#$op##" > /tmp/_yo
    zdump -v -c "$LO,$HI" "$zp" 2>/dev/null | sed "s#$zp##" > /tmp/_yz
    if cmp -s /tmp/_yo /tmp/_yz; then match=$((match+1)); else mis=$((mis+1)); [ "$finding" = - ] && finding="zdump-diff:$z"; fi
  done

  if [ "$ofc" -eq 0 ]; then verdict=oracle-build-incompatible
  elif [ "$zfc" -eq 0 ]; then verdict=source-shape-incompatible
  elif [ "$mis" -gt 0 ]; then verdict=zic-rs-divergent
  elif [ -n "$eozone" ] && [ ! -f "$zf/$eozone" ]; then
    # The oracle built the even/odd zone but zic-rs skipped it. Attribute by cause: a ZIC001 recurring
    # footer error = the SEPARATE perpetual-year-parity non-POSIX-footer deferred feature (pre-1995 `max`
    # even/odd rules), NOT a yearistype-admission failure (no ZIC027 — the TYPE was accepted).
    if grep -q "ZIC001.*recurring footer" "$TMP/e_$rid" 2>/dev/null; then verdict=deferred-perpetual-footer
    else verdict=deferred-other; fi
    finding="eozone-skipped:$eozone"
  elif [ -n "$eozone" ] && [ -f "$zf/$eozone" ] && [ "$match" -eq "$cmp_n" ] && [ "$cmp_n" -gt 0 ]; then
    verdict=match   # the even/odd zone built in both AND every compared fixture's zdump matches
  else verdict=inconclusive; fi
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$rid" "$n" "$ofc" "$zfc" "$cmp_n" "$match" "$mis" "$verdict" "$finding" >> "$OUT"
done

echo "=== YEARISTYPE.1: $(($(wc -l < "$OUT")-1)) releases ==="
echo "verdict tally:"; cut -f8 "$OUT" | tail -n +2 | sort | uniq -c
echo "any zic-rs-divergent?"; awk -F'\t' '$8=="zic-rs-divergent"{print "  "$1" "$9}' "$OUT" || echo "  none"
echo "total fixture comparisons: $(awk -F'\t' 'NR>1{c+=$5;m+=$6}END{print m"/"c" match"}' "$OUT")"
