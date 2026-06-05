#!/usr/bin/env bash
# RELEASE-METADATA.1 — tzdb release-identity contradiction receipt.
# For every cached tzdata release, derive the release identity from MULTIPLE upstream metadata surfaces
# and record agreements/contradictions as provenance facts — never silently trusting a single label.
#
# Surfaces compared (per release):
#   filename_id  : the IANA index filename version (tzdata<X>.tar.gz -> X)        [authoritative tag]
#   version_file : the `version` file inside the archive                          [authoritative tag]
#   news_top     : the top "Release <X>" heading in the NEWS changelog            [changelog surface]
#   sig          : OpenPGP signature status (from verified.tsv, GOODSIG = Eggert) [provenance]
#   bundle       : does a tzdb-<X>.tar.lz complete bundle exist alongside the tzdata/tzcode pair? [index]
#
# Verdicts (a contradiction is recorded, not resolved or hidden):
#   consistent                    : every PRESENT surface gives the same id
#   minor_metadata_contradiction  : the two authoritative tags (filename == version_file) AGREE, but the
#                                   NEWS changelog heading lags/differs (the documented code-fix/late-data
#                                   pattern — e.g. 2026b: version 2026b, NEWS top 2026a)
#   major_metadata_contradiction  : the two authoritative tags DISAGREE (filename != version_file)
#   legacy_unstructured           : pre-structured-metadata era — only the filename tag exists (no
#                                   `version` file, no NEWS heading)
#   unavailable                   : the cached source is missing
set -u
cd "$(dirname "$0")/../.." || exit 2
CACHE=${CACHE:-/tmp/data-gauntlet}
IDX=reports/release-all/index.tsv
VER=reports/release-all/verified.tsv
OUT=reports/release-metadata/release-metadata.tsv

# sig + bundle lookups from the existing RELEASE-ALL evidence (index + verified.tsv)
declare -A SIG BUNDLE
if [ -f "$VER" ]; then while IFS=$'\t' read -r art typ sha sig; do SIG["$art"]="$sig"; done < <(tail -n +2 "$VER"); fi
# a release has a complete bundle iff tzdb-<v>.tar.lz appears in the index
while IFS=$'\t' read -r fn typ ver _; do [ "$typ" = "tzdb_complete_bundle" ] && BUNDLE["$ver"]=1; done < <(tail -n +2 "$IDX")

printf 'release_id\tfilename_id\tversion_file\tnews_top\tsig\tbundle\tverdict\tnote\n' > "$OUT"

# every cached tzdata release = the data-gauntlet source extractions (the same 276 as RELEASE-ALL.DATA.1)
for S in "$CACHE"/s_*; do
  rid=${S##*/s_}
  arc="tzdata$rid.tar.gz"; [ -f "$CACHE/$arc" ] || arc="tzdata$rid.tar.Z"
  sig=${SIG[$arc]:-unverified}
  bundle=$([ -n "${BUNDLE[$rid]:-}" ] && echo bundle+pair || echo pair_only)
  if [ ! -d "$S" ]; then
    printf '%s\t%s\t-\t-\t%s\t%s\tunavailable\tno-cached-source\n' "$rid" "$rid" "$sig" "$bundle" >> "$OUT"; continue
  fi
  vf=$([ -f "$S/version" ] && tr -d '\n' < "$S/version" || echo "-")
  nt=$(grep -m1 -oE '^Release[[:space:]]+[0-9]+[a-z]*' "$S/NEWS" 2>/dev/null | awk '{print $2}'); nt=${nt:--}

  note=-
  if [ "$vf" = "-" ] && [ "$nt" = "-" ]; then
    verdict=legacy_unstructured; note="only filename tag (no version file, no NEWS heading)"
  elif [ "$vf" != "-" ] && [ "$vf" != "$rid" ]; then
    verdict=major_metadata_contradiction; note="version file '$vf' != filename '$rid'"
  elif [ "$nt" != "-" ] && [ "$nt" != "$rid" ]; then
    verdict=minor_metadata_contradiction; note="NEWS heading '$nt' lags the version tag '$rid' (changelog surface)"
  else
    verdict=consistent; note="all present surfaces agree on '$rid'"
  fi
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$rid" "$rid" "$vf" "$nt" "$sig" "$bundle" "$verdict" "$note" >> "$OUT"
done

echo "=== RELEASE-METADATA.1: $(($(wc -l < "$OUT")-1)) releases ==="
echo "verdict tally:"; cut -f7 "$OUT" | tail -n +2 | sort | uniq -c
echo "minor contradictions (NEWS lag — authoritative tags still agree):"
awk -F'\t' '$7=="minor_metadata_contradiction"{print "  "$1": version="$3" news="$4}' "$OUT" | head
echo "MAJOR contradictions (authoritative tags disagree):"
awk -F'\t' '$7=="major_metadata_contradiction"{print "  "$1": "$8}' "$OUT" || echo "  none"
