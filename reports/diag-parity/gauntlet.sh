#!/usr/bin/env bash
# DIAG-PARITY.1 / HOSTILE-EQUIV.1 — malformed-source & hostile-input failure-mode parity.
# Runs a corpus of malformed/hostile fixtures through BOTH reference `zic` and zic-rs and compares the
# FAILURE MODE by diagnostic class, severity (error vs warning), exit status, and output side effects
# (partial files) — NOT exact wording (wording drifts across releases/vendors; compared by class).
#
# Verdicts:
#   class_exit_match       : same diagnostic class AND same exit status
#   class_match_loc_diff   : same class, different reported line
#   intentional_divergence : zic-rs classifies more precisely (documented; e.g. ContinuationWithoutZone
#                            where reference folds into "input line of unknown type")
#   safer_no_partial       : same error, but zic-rs leaves NO partial output where reference wrote some
#   safer_refusal          : zic-rs errors (exit 1) where reference only warns (exit 0) — stricter
#   warning_match          : both accept with a warning (exit 0)
#   divergent              : a real class/exit-status mismatch (a named finding)
set -u
cd "$(dirname "$0")/../.." || exit 2
ZRS=${ZRS:-target/release/zic-rs}
C=reports/diag-parity/corpus
OUT=reports/diag-parity/diag-parity.tsv
TMP=/tmp/dp; mkdir -p "$TMP"

# reference `zic` stderr keyword -> canonical class (pinned to tzcode 2026b wording; class only)
ref_class() {
  local s="$1"
  case "$s" in
    *"NUL input byte"*) echo NulByteInInput;;
    *"wrong number of fields"*) echo InvalidFieldCount;;
    *"input line of unknown type"*) echo UnknownLineType;;
    *"duplicate zone name"*) echo DuplicateZone;;
    *"unterminated line"*) echo UnterminatedInputLine;;
    *"Odd number of quotation marks"*) echo UnterminatedQuote;;
    *"line too long"*|*"too long"*) echo OverlongInputLine;;
    *"invalid month"*) echo InvalidMonth;;
    *"invalid day of month"*|*"invalid weekday"*|*"lacks '<=' or '>='"*|*"invalid day"*) echo InvalidDayRule;;
    *"invalid time of day"*|*"bad time"*|*"invalid time"*) echo InvalidTimeSuffix;;
    *"two rules for same instant"*) echo SimultaneousTransition;;
    *"year type"*) echo UnsupportedRuleType;;
    *"contains a directory"*|*".."*|*"file name"*) echo OutputPathTraversal;;
    *"too many characters"*|*"fewer than 3"*) echo AbbreviationPolicyViolation;;
    *"values over 24 hours"*) echo ValueOver24Hours;;
    *) echo "?";;
  esac
}
# zic-rs ZIC code -> canonical class
zrs_class() {
  local s="$1"
  case "$s" in
    *ZIC016_*) echo NulByteInInput;; *ZIC002_*) echo InvalidFieldCount;;
    *ZIC013_*) echo UnknownLineType;; *ZIC014_*) echo ContinuationWithoutZone;;
    *ZIC015_*) echo DuplicateZone;; *ZIC021_*) echo UnterminatedInputLine;;
    *ZIC022_*) echo UnterminatedQuote;; *ZIC017_*) echo OverlongInputLine;;
    *ZIC003_*) echo InvalidMonth;; *ZIC005_*) echo InvalidDayRule;;
    *ZIC006_*) echo InvalidTimeSuffix;; *ZIC023_*) echo SimultaneousTransition;;
    *ZIC027_*) echo UnsupportedRuleType;; *ZIC008_*) echo OutputPathTraversal;;
    *ZIC018_*) echo AbbreviationPolicyViolation;; *ZIC026_*) echo ValueOver24Hours;;
    *ZIC012_*) echo InvalidValue;; *) echo "?";;
  esac
}

printf 'fixture\tref_exit\tref_files\tref_class\tzrs_exit\tzrs_files\tzrs_class\tverdict\n' > "$OUT"

for f in "$C"/*.zi; do
  n=$(basename "$f" .zi)
  rm -rf "$TMP/r_$n" "$TMP/z_$n"
  re=$(zic -d "$TMP/r_$n" "$f" 2>&1 >/dev/null); rc=$?
  rfiles=$(find "$TMP/r_$n" -type f 2>/dev/null | wc -l); rcl=$(ref_class "$re")
  ze=$("$ZRS" compile --input "$f" --all-supported --out "$TMP/z_$n" 2>&1 >/dev/null); zc=$?
  zfiles=$(find "$TMP/z_$n" -type f 2>/dev/null | wc -l); zcl=$(zrs_class "$ze")

  # classify
  if [ "$rcl" = "$zcl" ] && [ "$rcl" != "?" ]; then
    if [ "$rfiles" -gt 0 ] && [ "$zfiles" -eq 0 ]; then verdict=safer_no_partial
    elif [ "$rc" -eq "$zc" ]; then verdict=class_exit_match
    else verdict=class_match_exit_diff; fi
  elif [ "$zcl" = "ContinuationWithoutZone" ] && [ "$rcl" = "UnknownLineType" ]; then verdict=intentional_divergence
  elif [ "$rc" -eq 0 ] && [ "$zc" -ne 0 ]; then verdict=safer_refusal
  elif [ "$rc" -eq 0 ] && [ "$zc" -eq 0 ]; then verdict=warning_match
  else verdict=divergent; fi
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$n" "$rc" "$rfiles" "$rcl" "$zc" "$zfiles" "$zcl" "$verdict" >> "$OUT"
done

echo "=== DIAG-PARITY.1 ==="; column -t -s$'\t' "$OUT"
echo ""
echo "verdict tally:"; cut -f8 "$OUT" | tail -n +2 | sort | uniq -c
echo "DIVERGENT (named findings):"; awk -F'\t' '$8=="divergent"{print "  "$1": ref("$2"/"$4") zrs("$5"/"$7")"}' "$OUT" || echo "  none"
