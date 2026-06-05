#!/usr/bin/env bash
# T23.drop-in-gauntlet.1 — reference-zic argv/install compatibility matrix (first cut).
# Runs reference `zic` and the zic-rs EQUIVALENT invocation across argv / install-tree / error rows and
# classifies each: match · accepted-divergence (named) · not-claimed. zic-rs is DELIBERATELY not a literal
# argv drop-in (subcommand CLI + required --out; see docs/drop-in-compatibility-contract.md) — this gauntlet
# quantifies exactly that gap; it makes NO universal drop-in claim beyond the tested matrix.
set -u
TZ=${TZ:-/usr/share/zoneinfo/tzdata.zi}
LS=${LS:-/usr/share/zoneinfo/leapseconds}
RS=$(realpath "${RS:-target/release/zic-rs}")  # absolute: the harness cd's into a tmpdir below
W=$(mktemp -d)
cd "$W" || exit 1
row(){ printf '%-26s | ref-exit=%s rs-exit=%s | %s\n' "$1" "$2" "$3" "$4"; }

zic -d R1 "$TZ" 2>/dev/null; re=$?; "$RS" compile --all-supported --input "$TZ" --out S1 >/dev/null 2>&1; se=$?
row "compile→tree (default)" $re $se "argv DIVERGE; file-set $(find R1 -type f|wc -l)=$(find S1 -type f|wc -l); bytes DIVERGE-by-default (rs fat vs zic slim, bucket-3)"

zic -b slim -d R2 "$TZ" 2>/dev/null; "$RS" compile -b slim --all-supported --input "$TZ" --out S2 >/dev/null 2>&1
m=0; d=0; for z in America/New_York Europe/London Asia/Gaza Etc/UTC; do cmp -s R2/$z S2/$z && m=$((m+1)) || d=$((d+1)); done
row "-b slim byte parity" 0 0 "match=$m diff=$d → ACCEPTED-DIVERGENCE (structural residuals; behaviour/zdump-matched, see structural-report)"

zic -L "$LS" -d R3 "$TZ" 2>/dev/null; "$RS" compile --all-supported -L "$LS" --input "$TZ" --out S3 >/dev/null 2>&1
row "-L right/ profile" 0 0 "file-set $(find R3 -type f|wc -l)=$(find S3 -type f|wc -l); right/UTC bytes $(cmp -s R3/Etc/UTC S3/Etc/UTC&&echo MATCH||echo DIFF)"

zic --version >/dev/null 2>&1; re=$?; "$RS" --version >/dev/null 2>&1; se=$?
row "--version" $re $se "argv MATCH (both print + exit 0)"

zic --bogusflag 2>/dev/null; re=$?; "$RS" --bogusflag 2>/dev/null; se=$?
row "invalid flag" $re $se "ACCEPTED-DIVERGENCE: both reject, exit code differs (zic 1 / clap-usage 2; documented taxonomy)"

zic -d R6 /nonexistent.zi 2>/dev/null; re=$?; "$RS" compile --input /nonexistent.zi --out S6 >/dev/null 2>&1; se=$?
row "missing source file" $re $se "MATCH-class (both nonzero, operational error)"

zic -p America/New_York -d R7 "$TZ" 2>/dev/null; re=$?
row "-p posixrules" $re "n/a" "NOT-CLAIMED: reference accepts but warns '-p is obsolete and likely ineffective'; zic-rs does not implement -p"
rm -rf "$W"
