#!/usr/bin/env bash
# RELEASE-ALL.1 Phase 1 — signature verification pass over the complete IANA release index.
# Verifies the signed complete tzdb bundles (the modern provenance backbone) + records the 7 pilot
# tzdata (already GOODSIG in RELEASE-LADDER.1). Hash + full-corpus verify of the remaining archives is
# the incremental work of the compile phases (.DATA/.TZDB/.CODE). Writes verified.tsv.
set -u
cd "$(dirname "$0")/../.." || exit 2
IDX=reports/release-all/index.tsv
OUT=reports/release-all/verified.tsv
CACHE=/tmp/ra-verify; mkdir -p "$CACHE"
printf 'artifact\ttype\tsha256\tsig_status\n' > "$OUT"

n=0; good=0
# awk-extract the signed complete-bundle filenames (avoids IFS=tab empty-field collapse)
for fn in $(awk -F'\t' '$2=="tzdb_complete_bundle" && $6=="asc_available"{print $1}' "$IDX"); do
  n=$((n+1))
  a="$CACHE/$fn"; s="$CACHE/$fn.asc"
  [ -s "$a" ] || curl -sSL -o "$a" "https://data.iana.org/time-zones/releases/$fn"
  [ -s "$s" ] || curl -sSL -o "$s" "https://data.iana.org/time-zones/releases/$fn.asc"
  sha=$(sha256sum "$a" 2>/dev/null | cut -c1-16)
  if gpg --verify "$s" "$a" 2>&1 | grep -qi "Good signature"; then st=GOODSIG; good=$((good+1)); else st=unverified; fi
  printf '%s\ttzdb_complete_bundle\t%s…\t%s\n' "$fn" "${sha:-?}" "$st" >> "$OUT"
done

# the 7 pilot tzdata (already GOODSIG in RELEASE-LADDER.1)
for R in 2026b 2026a 2025b 2024a 2020a 2018e 2015g; do
  sha=$(sha256sum "/tmp/release-ladder/$R/tzdata$R.tar.gz" 2>/dev/null | cut -c1-16)
  printf 'tzdata%s.tar.gz\ttzdata\t%s…\tGOODSIG\n' "$R" "${sha:-?}" >> "$OUT"
done

echo "complete bundles verified: $good / $n signed bundles GOODSIG"
echo "verified.tsv tally:"; cut -f4 "$OUT" | tail -n +2 | sort | uniq -c
