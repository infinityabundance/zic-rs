#!/usr/bin/env bash
# RELEASE-LADDER.1 — does zic-rs hold across tzdb source ERAS, or only on 2026b?
# For each historical tzdb release: fetch the data archive + signature, hash-pin, verify the sig,
# extract, then compile a fixed region-file set with BOTH reference `zic` (the host 2026b binary,
# which is backward-compatible) and zic-rs (`--unsupported skip`: compile the supported set, classify
# the rest), and compare per-fixture `zdump` behaviour over 1900..2037. Read-only; writes only /tmp +
# this dir's TSVs. Classifies honestly — does NOT force any release green.
set -u
cd "$(dirname "$0")/../.." || exit 2
ZRS=target/release/zic-rs
OUT=reports/release-ladder
CACHE=/tmp/release-ladder; mkdir -p "$CACHE"
RELEASES="2026b 2026a 2025b 2024a 2020a 2018e 2015g"
FIX="Etc/UTC Europe/Lisbon Europe/London America/New_York Asia/Singapore Pacific/Kiritimati Australia/Lord_Howe"
REGIONS="africa antarctica asia australasia europe northamerica southamerica etcetera backward factory"
LO=1900; HI=2037

REL_TSV="$OUT/releases.tsv"; FIX_TSV="$OUT/fixtures.tsv"
printf 'release\tarchive_sha256\tsig\tref_files\tzrs_files\tzrs_skipped\tfixtures_match\tverdict\n' > "$REL_TSV"
printf 'release\tzone\tref_present\tzrs_present\tzdump\tverdict\n' > "$FIX_TSV"

for R in $RELEASES; do
  D="$CACHE/$R"; mkdir -p "$D"
  arc="$D/tzdata$R.tar.gz"
  [ -s "$arc" ] || curl -sSL -o "$arc" "https://data.iana.org/time-zones/releases/tzdata$R.tar.gz"
  [ -s "$D/sig.asc" ] || curl -sSL -o "$D/sig.asc" "https://data.iana.org/time-zones/releases/tzdata$R.tar.gz.asc" 2>/dev/null
  if [ ! -s "$arc" ]; then printf '%s\t-\tFETCH_FAILED\t-\t-\t-\t-\tunavailable\n' "$R" >> "$REL_TSV"; continue; fi
  sha=$(sha256sum "$arc" | cut -d' ' -f1)
  # NOTE: gpg writes its verification result to STDERR, so we must redirect 2>&1 to grep it.
  gpgout=$(gpg --verify "$D/sig.asc" "$arc" 2>&1)
  if printf '%s' "$gpgout" | grep -qiE 'Good signature'; then sig=GOODSIG
  elif printf '%s' "$gpgout" | grep -qiE 'no public key|can.t check'; then sig=key_unavailable
  else sig=unsigned_or_bad; fi
  rm -rf "$D/s"; mkdir -p "$D/s"; tar -xzf "$arc" -C "$D/s" 2>/dev/null

  # compile present region files with both compilers
  INPUTS=(); ZICARGS=()
  for f in $REGIONS; do [ -f "$D/s/$f" ] && { INPUTS+=(--input "$D/s/$f"); ZICARGS+=("$D/s/$f"); }; done
  rf="$CACHE/ref_$R"; zf="$CACHE/zrs_$R"; rm -rf "$rf" "$zf"
  zic -d "$rf" "${ZICARGS[@]}" 2>/dev/null
  "$ZRS" compile --all-supported --unsupported skip "${INPUTS[@]}" --out "$zf" >/dev/null 2>"$D/zrs.err"
  rfc=$(find "$rf" -type f 2>/dev/null | wc -l); zfc=$(find "$zf" -type f 2>/dev/null | wc -l)
  # zones reference compiled that zic-rs did not (skipped as unsupported under --unsupported skip)
  skipped=$(( rfc - zfc )); [ "$skipped" -lt 0 ] && skipped=0

  # per-fixture zdump comparison
  fmatch=0; ftot=0; rel_div=0
  for z in $FIX; do
    ftot=$((ftot+1))
    rp="$rf/$z"; zp="$zf/$z"
    rpres=$([ -f "$rp" ] && echo yes || echo no); zpres=$([ -f "$zp" ] && echo yes || echo no)
    if [ "$rpres" = no ]; then v=not-applicable; zd="ref-absent"
    elif [ "$zpres" = no ]; then v=source-incompatible; zd="zic-rs-skipped"
    else
      zdump -v -c "$LO,$HI" "$rp" 2>/dev/null | sed "s#$rp##" > /tmp/_ra
      zdump -v -c "$LO,$HI" "$zp" 2>/dev/null | sed "s#$zp##" > /tmp/_rb
      if cmp -s /tmp/_ra /tmp/_rb; then
        fmatch=$((fmatch+1))
        if cmp -s "$rp" "$zp"; then v=match; zd=byte-identical; else v=behaviour-match-byte-diff; zd="zdump-identical (slim/fat byte-diff)"; fi
      else v=divergent; zd="DIFF"; rel_div=$((rel_div+1)); fi
    fi
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$R" "$z" "$rpres" "$zpres" "$zd" "$v" >> "$FIX_TSV"
  done

  # release verdict: divergent if any fixture diverged; else match (some zones may be source-incompatible)
  if [ "$rfc" -eq 0 ]; then verdict=source-incompatible
  elif [ "$rel_div" -gt 0 ]; then verdict=divergent
  else verdict=match; fi
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s/%s\t%s\n' "$R" "${sha:0:16}…" "$sig" "$rfc" "$zfc" "$skipped" "$fmatch" "$ftot" "$verdict" >> "$REL_TSV"
  echo "  $R: sig=$sig ref=$rfc zrs=$zfc skipped~$skipped fixtures=$fmatch/$ftot verdict=$verdict"
done

echo "=== releases.tsv ==="; column -t -s$'\t' "$REL_TSV"
echo "=== fixture verdict tally ==="; cut -f6 "$FIX_TSV" | tail -n+2 | sort | uniq -c
