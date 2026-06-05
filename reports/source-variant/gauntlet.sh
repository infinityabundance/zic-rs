#!/usr/bin/env bash
# SOURCE-VARIANT.1 — tzdb source-profile parity matrix.
# Run reference `zic` AND zic-rs across the major tzdb source profiles and classify each: does zic-rs
# match reference `zic` behaviour for that admitted profile? Profiles are generated from the admitted,
# GOODSIG-verified tzdb-2026b bundle via the PINNED ziguard.awk recipe (hashes verified vs T12.5d):
#
#   DATAFORM=main      -> main.zi      sha256 e0225823…   (the default encoding; == cat(TDATA))
#   DATAFORM=vanguard  -> vanguard.zi  sha256 49e16da4…   (newest features: negative SAVE, %z, 24:00+)
#   DATAFORM=rearguard -> rearguard.zi sha256 91c4f362…   (nonnegative SAVE only; oldest-compatible)
#   PACKRATDATA=backzone (backzone-included) -> main+backzone (pre-1970 history for backed-out zones)
#   PACKRATDATA= / PACKRATLIST= (default-empty) = backzone-excluded = main
#
# Generation (run once by gen.sh): awk -v DATAFORM=<form> -f ziguard.awk $TDATA [PACKRATDATA] [PACKRATLIST]
#   TDATA = africa antarctica asia australasia europe northamerica southamerica etcetera factory backward
#
# Reference = host zic 2026b. Compares bounded fixture `zdump` behaviour over 1900..2037.
set -u
cd "$(dirname "$0")/../.." || exit 2
ZRS=${ZRS:-target/release/zic-rs}
ZI=${ZI:-/tmp/sv/zi}                 # generated .zi (gen.sh output)
OUT=reports/source-variant/source-variant.tsv
TMP=/tmp/sv-gauntlet; mkdir -p "$TMP"
# standard fixtures + a backzone-sensitive zone (Montreal: a backward link in main, a full pre-1970 zone in backzone)
FIX="Etc/UTC Europe/Lisbon Europe/London America/New_York Asia/Singapore Pacific/Kiritimati Australia/Lord_Howe America/Montreal"
LO=1900; HI=2037

# profile -> source .zi
declare -A SRC
SRC[main]="$ZI/main.zi"
SRC[vanguard]="$ZI/vanguard.zi"
SRC[rearguard]="$ZI/rearguard.zi"
SRC[backzone_excluded]="$ZI/main.zi"
SRC[backzone_included]="$ZI/main-backzone.zi"

printf 'profile\tsource_zi\tref_files\tzrs_files\tfix_compared\tfix_match\tbyte_identical\tzrs_skipped\tverdict\tfinding\n' > "$OUT"

for prof in main vanguard rearguard backzone_excluded backzone_included; do
  src=${SRC[$prof]}
  if [ ! -f "$src" ]; then printf '%s\t%s\t0\t0\t0\t0\t0\t0\tunavailable\tno-source-zi\n' "$prof" "$(basename "$src")" >> "$OUT"; continue; fi
  rf="$TMP/r_$prof"; zf="$TMP/z_$prof"; rm -rf "$rf" "$zf"
  zic -d "$rf" "$src" 2>/dev/null
  "$ZRS" compile --input "$src" --all-supported --unsupported skip --out "$zf" >/dev/null 2>"$TMP/e_$prof"
  rfc=$(find "$rf" -type f 2>/dev/null | wc -l); zfc=$(find "$zf" -type f 2>/dev/null | wc -l)

  cmp_n=0; match=0; bytei=0; skip=0; finding=-
  for z in $FIX; do
    rp="$rf/$z"; zp="$zf/$z"
    [ -f "$rp" ] || continue            # zone not in this profile (e.g. Montreal only meaningful w/ backzone)
    if [ ! -f "$zp" ]; then skip=$((skip+1)); [ "$finding" = - ] && finding="zrs-skipped:$z"; continue; fi
    cmp_n=$((cmp_n+1))
    cmp -s "$rp" "$zp" && bytei=$((bytei+1))
    zdump -v -c "$LO,$HI" "$rp" 2>/dev/null | sed "s#$rp##" > "$TMP/_r"
    zdump -v -c "$LO,$HI" "$zp" 2>/dev/null | sed "s#$zp##" > "$TMP/_z"
    if cmp -s "$TMP/_r" "$TMP/_z"; then match=$((match+1)); else [ "$finding" = - ] && finding="zdump-diff:$z"; fi
  done

  if [ "$rfc" -eq 0 ]; then verdict=reference-build-incompatible
  elif [ "$zfc" -eq 0 ]; then verdict=unsupported-by-design
  elif [ "$match" -lt "$cmp_n" ]; then verdict=divergent
  elif [ "$skip" -gt 0 ]; then verdict=deferred
  elif [ "$bytei" -eq "$cmp_n" ]; then verdict=match
  else verdict=behaviour-match-byte-diff; fi
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$prof" "$(basename "$src")" "$rfc" "$zfc" "$cmp_n" "$match" "$bytei" "$skip" "$verdict" "$finding" >> "$OUT"
done

echo "=== SOURCE-VARIANT.1 ==="; column -t -s$'\t' "$OUT"
echo ""
echo "=== cross-encoding sanity: reference zic main vs vanguard vs rearguard must be behaviour-identical (same data) ==="
for z in Europe/Lisbon America/New_York Australia/Lord_Howe; do
  zdump -v -c $LO,$HI "$TMP/r_main/$z" 2>/dev/null | sed "s#$TMP/r_main/$z##" > "$TMP/_m"
  zdump -v -c $LO,$HI "$TMP/r_vanguard/$z" 2>/dev/null | sed "s#$TMP/r_vanguard/$z##" > "$TMP/_v"
  zdump -v -c $LO,$HI "$TMP/r_rearguard/$z" 2>/dev/null | sed "s#$TMP/r_rearguard/$z##" > "$TMP/_g"
  mv="same"; cmp -s "$TMP/_m" "$TMP/_v" || mv="MAIN!=VANGUARD"; cmp -s "$TMP/_m" "$TMP/_g" || mv="$mv MAIN!=REARGUARD"
  echo "  $z: $mv"
done
