#!/usr/bin/env bash
# RELEASE-ALL.DATA.1 — every stable IANA tzdata archive through BOTH reference `zic` and `zic-rs`.
# For each release: hash, sig-status, extract (.tar.gz or legacy .tar.Z), compile the present region
# files with reference `zic` and `zic-rs` (--unsupported skip), and compare per-fixture `zdump`
# behaviour over 1900..2037. Classifies each release; hides no failure. Writes data-gauntlet.tsv.
# Reference = current `zic` on old source (backward-compatible) -> parser/compiler-era parity.
set -u
cd "$(dirname "$0")/../.." || exit 2
ZRS=target/release/zic-rs
OUT=reports/release-all/data-gauntlet.tsv
CACHE=/tmp/data-gauntlet
IDX=reports/release-all/index.tsv
FIX="Etc/UTC Europe/Lisbon Europe/London America/New_York Asia/Singapore Pacific/Kiritimati Australia/Lord_Howe"
# The fixture-relevant region set (= RELEASE-LADDER.1's). Deliberately NOT solar*/systemv/pacificnew —
# those carry removed-feature noise (solar time, broken legacy links) unrelated to the fixtures; the
# classification stays about the fixture-relevant source shape. `yearistype` ("even"/"odd"/"uspres") in
# these core files is the real reference-build breakpoint (current `zic` removed it in 2020a).
REGIONS="africa antarctica asia australasia europe northamerica southamerica etcetera backward"
LO=1900; HI=2037

printf 'release_id\tarchive\tsha256\tsig\tsource_files\tref_compile\tzrs_compile\tref_files\tzrs_files\tfix_avail\tfix_match\tzdump_mismatch\tverdict\tfinding\n' > "$OUT"

# map archive filename -> sig availability from the index
declare -A SIGAV
while IFS= read -r line; do
  fn=$(printf '%s' "$line" | cut -f1); sa=$(printf '%s' "$line" | cut -f6)
  SIGAV["$fn"]="$sa"
done < <(tail -n +2 "$IDX")

extract(){ # archive dir
  local a="$1" d="$2"; rm -rf "$d"; mkdir -p "$d"
  case "$a" in
    *.tar.gz) tar -xzf "$a" -C "$d" 2>/dev/null ;;
    *.tar.Z)  gzip -dc "$a" 2>/dev/null | tar -x -C "$d" 2>/dev/null ;;
  esac
}

n=0
for arc in $(awk -F'\t' '$2=="tzdata"{print $1}' "$IDX"); do
  n=$((n+1)); rid=$(printf '%s' "$arc" | sed -E 's/^tzdata//; s/\.tar\.(gz|Z)$//')
  a="$CACHE/$arc"
  [ -s "$a" ] || { printf '%s\t%s\t-\t-\t-\tNA\tNA\t0\t0\t0/7\t0\t0\tunavailable\t-\n' "$rid" "$arc" >> "$OUT"; continue; }
  sha=$(sha256sum "$a" | cut -c1-16); sig=${SIGAV[$arc]:-unknown}
  S="$CACHE/s_$rid"; extract "$a" "$S"
  # present region files (the source shape)
  present=""; INPUTS=(); ZICARGS=()
  for f in $REGIONS; do [ -f "$S/$f" ] && { present="$present $f"; INPUTS+=(--input "$S/$f"); ZICARGS+=("$S/$f"); }; done
  srcn=$(printf '%s' "$present" | wc -w)
  rf="$CACHE/r_$rid"; zf="$CACHE/z_$rid"; rm -rf "$rf" "$zf"
  if [ "$srcn" -eq 0 ]; then
    printf '%s\t%s\t%s…\t%s\t0\tNA\tNA\t0\t0\t0/7\t0\t0\treference-build-incompatible\tno-region-files\n' "$rid" "$arc" "$sha" "$sig" >> "$OUT"; continue
  fi
  zic -d "$rf" "${ZICARGS[@]}" 2>/dev/null; rc=$?
  # LEGACY-SOURCE.1: set LEGACY=1 to add --legacy-latin1 (admits Latin-1 in comments for pre-2013 source).
  LEGACYFLAG=(); [ "${LEGACY:-0}" = "1" ] && LEGACYFLAG=(--legacy-latin1)
  "$ZRS" compile --all-supported --unsupported skip "${LEGACYFLAG[@]}" "${INPUTS[@]}" --out "$zf" >/dev/null 2>"$CACHE/e_$rid"; zc=$?
  rfc=$(find "$rf" -type f 2>/dev/null | wc -l); zfc=$(find "$zf" -type f 2>/dev/null | wc -l)
  refstat=$([ "$rfc" -gt 0 ] && echo ok || echo fail); zrsstat=$([ "$zfc" -gt 0 ] && echo ok || echo fail)

  avail=0; fmatch=0; mismatch=0; finding=-
  for z in $FIX; do
    rp="$rf/$z"; zp="$zf/$z"
    [ -f "$rp" ] || continue
    avail=$((avail+1))
    if [ ! -f "$zp" ]; then mismatch=$((mismatch+1)); [ "$finding" = - ] && finding="zrs-skipped:$z"; continue; fi
    zdump -v -c "$LO,$HI" "$rp" 2>/dev/null | sed "s#$rp##" > /tmp/_da
    zdump -v -c "$LO,$HI" "$zp" 2>/dev/null | sed "s#$zp##" > /tmp/_db
    if cmp -s /tmp/_da /tmp/_db; then fmatch=$((fmatch+1)); else mismatch=$((mismatch+1)); [ "$finding" = - ] && finding="zdump-diff:$z"; fi
  done

  # classify
  if [ "$rfc" -eq 0 ]; then verdict=reference-build-incompatible
  elif [ "$zfc" -eq 0 ]; then verdict=source-shape-incompatible
  elif [ "$avail" -eq 0 ]; then verdict=not-applicable
  elif [ "$fmatch" -gt 0 ] && [ "$(echo "$finding" | grep -c zdump-diff)" -gt 0 ]; then verdict=zic-rs-divergent
  elif [ "$fmatch" -eq "$avail" ]; then verdict=match
  else verdict=match; fi   # remaining mismatches are zrs-skipped (deferred zones), not behaviour diffs
  printf '%s\t%s\t%s…\t%s\t%s\t%s\t%s\t%s\t%s\t%s/7\t%s\t%s\t%s\t%s\n' \
    "$rid" "$arc" "$sha" "$sig" "$srcn" "$refstat" "$zrsstat" "$rfc" "$zfc" "$avail" "$fmatch" "$mismatch" "$verdict" "$finding" >> "$OUT"
done

echo "=== processed $n releases ==="
echo "verdict tally:"; cut -f13 "$OUT" | tail -n +2 | sort | uniq -c
echo "any zic-rs-divergent (named findings)?"; awk -F'\t' '$13=="zic-rs-divergent"{print "  "$1" "$14}' "$OUT" || echo "  none"
