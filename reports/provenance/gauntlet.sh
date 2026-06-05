#!/usr/bin/env bash
# T18.breadth-to-40 — provenance gauntlet.
# Compiles the pristine, signature-verified IANA tzdb 2026b region source with BOTH reference `zic`
# and `zic-rs`, then for 40 diversity-selected zones records: source file, reference TZif sha256 +
# zdump, zic-rs (fat default + zic-slim) sha256 + zdump, the behaviour (zdump 1900..2040) verdict,
# and the slim structural byte verdict. Output: prov.tsv (machine-readable) + a per-zone zdump dir.
#
# It does NOT claim timezone truth; it claims zic-rs preserves the admitted source semantics against
# reference zic/zdump for the selected cases. Any mismatch is printed as a FINDING, never hidden.
set -u
SRC=/tmp/prov/src/tzdb-2026b
ZRS=/home/one/zic-rs/target/release/zic-rs
OUT=/home/one/zic-rs/reports/provenance
REGIONS="africa antarctica asia australasia europe northamerica southamerica etcetera backward factory"
HORIZON="1900,2040"

cd "$SRC" || { echo "no source at $SRC"; exit 1; }

# 1. Pin region-file source hashes.
sha256sum $REGIONS > "$OUT/source-files.sha256"

# 2. Compile with both compilers (reference zic = slim default; zic-rs fat default + zic-slim).
rm -rf /tmp/prov/ref /tmp/prov/zrs /tmp/prov/zrs-slim
zic -d /tmp/prov/ref $REGIONS 2>"$OUT/ref-zic.stderr"
INPUTS=(); for r in $REGIONS; do INPUTS+=(--input "$r"); done
"$ZRS" compile --all-supported "${INPUTS[@]}" --out /tmp/prov/zrs       2>"$OUT/zrs-fat.stderr"
"$ZRS" compile --all-supported "${INPUTS[@]}" --emit-style zic-slim --out /tmp/prov/zrs-slim 2>"$OUT/zrs-slim.stderr"
echo "ref files=$(find /tmp/prov/ref -type f|wc -l) zrs-fat=$(find /tmp/prov/zrs -type f|wc -l) zrs-slim=$(find /tmp/prov/zrs-slim -type f|wc -l)"

# 3. The 40 diversity-selected zones (name <TAB> category-id <TAB> selection-reason).
ZONES=$(cat <<'EOF'
America/New_York	1,6	pre-1883 LMT + many transitions + ~2^31 boundary
Europe/London	1,3,10	pre-1847 LMT + WW2 double summer time + far-future footer
Europe/Dublin	2	negative DST (winter-time IST/GMT inversion)
Europe/Prague	2	inline negative SAVE (1 -1 GMT, law 7)
Africa/Windhoek	2,8	Namibia winter-time / DST-sign churn
Europe/Paris	3,1	1940s double summer time under occupation
Asia/Kolkata	4,9	+5:30 non-hour offset + renamed (Calcutta)
Asia/Kathmandu	4	+5:45 sub-hour offset
Australia/Eucla	4	+8:45 non-hour offset + DST
Asia/Tehran	4,8	+3:30 offset + Iran DST (now abolished)
Asia/Yangon	4	+6:30 offset
Asia/Singapore	5,4	odd historical offsets: LMT +6:55:25 -> +7:20 -> +7:30 -> +8:00
Africa/Monrovia	5,8	Liberia -0:44:30 until 1972 (odd sub-minute, late switch)
Asia/Jerusalem	6,8	many transitions, complex modern DST
America/Santiago	6,10	southern-hemisphere DST + v3 footer (dayshift)
Pacific/Honolulu	7	sparse transitions, no modern DST
America/Phoenix	7	Arizona no-DST, sparse
Asia/Gaza	8,10,12	Ramadan rules, v3 footer, late churn (law 10)
Africa/Casablanca	8,10,12	Morocco Ramadan DST, unusual footer, late churn
Pacific/Fiji	8	late political DST on/off churn
Asia/Damascus	8	Syria DST churn (DST dropped 2022)
Europe/Volgograd	8	Russia MSK offset churn (recent)
Europe/Kyiv	9	renamed (Kiev), political
America/Argentina/Buenos_Aires	9	sub-zoned/renamed, provincial churn
Asia/Ho_Chi_Minh	9	renamed (Saigon)
Antarctica/Troll	10,12	UTC<->+02 (CEST) — unusual footer / reader stress
Pacific/Kiritimati	12	+14 maximum offset (reader stress)
Asia/Tokyo	7	stable control zone (well-behaved)
Africa/Juba	14,8	South Sudan 2021 offset change (underrepresented)
Pacific/Bougainville	14,15	2014 +10->+11 change (underrepresented)
America/Punta_Arenas	14	split from Santiago 2017 (underrepresented)
Asia/Atyrau	14,8	Kazakhstan offset change (underrepresented)
America/Vancouver	15,8	the real 2026a->2026b release-diff zone
Antarctica/Casey	8,14	flips +08/+11 (churn, underrepresented)
Pacific/Apia	8	Samoa 2011 dateline skip (lost a calendar day)
Asia/Pyongyang	8	2015 +8:30 then 2018 back to +9
Europe/Lisbon	10	slim/fat timecnt residual zone (structural edge)
America/Adak	1,10	abbreviation suffix-sharing (HST in AHST), Aleutian
US/Eastern	9,11	backward Link -> America/New_York (linked identity)
Etc/UTC	7,13	fixed sparse; + right/ leap profile witness (see reader-compat.2)
EOF
)

# 4. Per-zone witness rows.
TSV="$OUT/prov.tsv"
ZD="$OUT/zdump"; rm -rf "$ZD"; mkdir -p "$ZD"
printf 'zone\tsource_file\tref_sha256\tzrs_fat_sha256\tzrs_slim_sha256\tbehaviour_zdump\tstructural_slim_vs_ref\tcategories\tselection_reason\n' > "$TSV"
findings=0; n=0
sha() { [ -f "$1" ] && sha256sum "$1" | cut -c1-16 || echo "MISSING"; }

while IFS=$'\t' read -r zone cats reason; do
  [ -z "$zone" ] && continue
  n=$((n+1))
  # source file: where the zone (or, for a Link, the link line) is defined.
  sf=$(grep -lE "^(Zone[[:space:]]+${zone//\//\\/}[[:space:]]|Link[[:space:]].*[[:space:]]${zone//\//\\/}([[:space:]]|\$))" $REGIONS 2>/dev/null | head -1)
  [ -z "$sf" ] && sf="(derived)"
  rf=/tmp/prov/ref/$zone; zf=/tmp/prov/zrs/$zone; zs=/tmp/prov/zrs-slim/$zone
  rsha=$(sha "$rf"); zfsha=$(sha "$zf"); zssha=$(sha "$zs")
  # behaviour: zdump over the horizon, normalize the leading path column.
  mkdir -p "$(dirname "$ZD/$zone.ref.zdump")"
  zdump -v -c "$HORIZON" "$rf" 2>/dev/null | sed "s#^$rf#Z#" > "$ZD/$zone.ref.zdump"
  zdump -v -c "$HORIZON" "$zf" 2>/dev/null | sed "s#^$zf#Z#" > "$ZD/$zone.zrs.zdump"
  if diff -q "$ZD/$zone.ref.zdump" "$ZD/$zone.zrs.zdump" >/dev/null 2>&1 && [ -s "$ZD/$zone.ref.zdump" ]; then beh=MATCH; else beh=DIFF; fi
  # structural: zic-rs slim bytes vs reference slim bytes.
  if [ -f "$rf" ] && [ -f "$zs" ] && cmp -s "$rf" "$zs"; then st=BYTE_MATCH; else st=DIFF; fi
  [ "$beh" = "DIFF" ] && { echo "  FINDING: behaviour DIFF for $zone"; findings=$((findings+1)); }
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$zone" "$sf" "$rsha" "$zfsha" "$zssha" "$beh" "$st" "$cats" "$reason" >> "$TSV"
done <<< "$ZONES"

echo "=== zones=$n  behaviour_findings=$findings ==="
echo "=== behaviour verdict tally ==="; cut -f6 "$TSV" | tail -n +2 | sort | uniq -c
echo "=== structural(slim) verdict tally ==="; cut -f7 "$TSV" | tail -n +2 | sort | uniq -c
echo "TSV: $TSV"
