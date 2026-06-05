#!/usr/bin/env bash
# LONG-FUZZ-HARNESS.1 — reproducible long fuzz burn-in runner for an equipped operator/lab.
#
# This is the CAMPAIGN MACHINERY, not a claim that a burn-in ran. It runs the 9 cargo-fuzz targets for a
# declared per-target duration, records the full toolchain/host/seed/git provenance, detects crash
# artifacts, and writes a receipt under audits/cargo-fuzz/receipts/RECEIPT-LONG-FUZZ-<UTC>.md using the
# fixed vocabulary { clean | crash_found | inconclusive_environment | interrupted | not_run }.
#
# It does NOT claim coverage saturation: a clean run means only "no crash found within the stated
# target/time/seed configuration". The real 24h-class campaign is an operator/lab task; this harness is
# what makes it reproducible.
#
# Modes (combine freely; precedence noted):
#   --smoke                  bounded sanity pass (per-target default 60s unless --per-target-minutes given)
#   --per-target-minutes N   explicit per-target wall budget (minutes) — WINS over --campaign-hours
#   --campaign-hours N       total wall budget (hours), split evenly across the selected targets
#   --target NAME            restrict to one target (repeatable); default = all 9
#   --help
#
# Requires: a nightly toolchain + cargo-fuzz + libFuzzer (the cargo-fuzz default). On a host without them,
# the harness still writes a receipt with status not_run / inconclusive_environment — it never fakes a run.
set -u

FUZZ_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$FUZZ_DIR/.." && pwd)"
RECEIPTS="$REPO/audits/cargo-fuzz/receipts"
NIGHTLY="${ZICRS_FUZZ_TOOLCHAIN:-nightly}"

ALL_TARGETS=(tzif_validate_bytes vendor_oracle_json manifest_json source_lexer zone_rule_link_parser \
             posix_footer aux_table_validator release_diff_tree path_materialization_model)

# ---- arg parse ----
MODE="manual"; PER_TARGET_MIN=""; CAMPAIGN_HOURS=""; SELECTED=()
while [ $# -gt 0 ]; do
  case "$1" in
    --smoke)              MODE="smoke"; shift ;;
    --per-target-minutes) PER_TARGET_MIN="$2"; shift 2 ;;
    --campaign-hours)     CAMPAIGN_HOURS="$2"; shift 2 ;;
    --target)             SELECTED+=("$2"); shift 2 ;;
    --help|-h)            sed -n '2,30p' "$0"; exit 0 ;;
    *) echo "unknown arg: $1 (try --help)" >&2; exit 2 ;;
  esac
done
[ ${#SELECTED[@]} -eq 0 ] && SELECTED=("${ALL_TARGETS[@]}")
NT=${#SELECTED[@]}

# ---- per-target duration (seconds); precedence: --per-target-minutes > --campaign-hours > --smoke > default ----
if [ -n "$PER_TARGET_MIN" ]; then
  PER_TARGET_SEC=$(( PER_TARGET_MIN * 60 ))
elif [ -n "$CAMPAIGN_HOURS" ]; then
  PER_TARGET_SEC=$(( (CAMPAIGN_HOURS * 3600) / NT ))
elif [ "$MODE" = "smoke" ]; then
  PER_TARGET_SEC=60
else
  PER_TARGET_SEC=300   # default bounded run if no mode given
fi

# ---- provenance capture (recorded by the runner, never hand-written) ----
START_UTC="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
RECEIPT="$RECEIPTS/RECEIPT-LONG-FUZZ-$STAMP.md"
GIT_COMMIT="$(cd "$REPO" && git rev-parse --short HEAD 2>/dev/null || echo 'not-a-git-repo')"
HOST_OS="$(uname -srm)"; HOST_KERNEL="$(uname -r)"
HOST_CPU="$(grep -m1 'model name' /proc/cpuinfo 2>/dev/null | cut -d: -f2- | sed 's/^ //' || echo unknown)"
NCPU="$(nproc 2>/dev/null || echo '?')"
RUSTC_VER="$(rustc +"$NIGHTLY" --version 2>/dev/null || rustc --version 2>/dev/null || echo absent)"
CARGO_FUZZ_VER="$(cargo +"$NIGHTLY" fuzz --version 2>/dev/null || cargo fuzz --version 2>/dev/null || echo absent)"
# libFuzzer is bundled in the rustc sanitizer runtime; surface its presence honestly.
LIBFUZZER_VER="bundled with rustc sanitizer runtime ($RUSTC_VER)"
CMDLINE="$0 $*"

mkdir -p "$RECEIPTS"

seed_hash() {  # sha256 over the sorted per-seed-file hashes = the input set identity
  local d="$REPO/fuzz/corpus/$1"
  [ -d "$d" ] || { echo "no-corpus"; return; }
  find "$d" -type f -exec sha256sum {} \; 2>/dev/null | awk '{print $1}' | sort | sha256sum | cut -d' ' -f1
}
seed_count() { local d="$REPO/fuzz/corpus/$1"; [ -d "$d" ] && find "$d" -type f 2>/dev/null | wc -l | tr -d ' ' || echo 0; }

# ---- interrupted handling: write a partial receipt and exit ----
INTERRUPTED=0
trap 'INTERRUPTED=1' INT TERM

declare -a ROW_TARGET ROW_VERDICT ROW_SEEDS ROW_SEEDHASH ROW_DUR ROW_CRASHES ROW_EXIT
overall="not_run"

run_target() {
  local t="$1" adir="$REPO/fuzz/artifacts/$t" before after newc rc
  mkdir -p "$adir"
  before="$(find "$adir" -name 'crash-*' -o -name 'oom-*' -o -name 'timeout-*' 2>/dev/null | sort)"
  ( cd "$FUZZ_DIR" && cargo +"$NIGHTLY" fuzz run "$t" -- -max_total_time="$PER_TARGET_SEC" -print_final_stats=1 ) \
    >"$RECEIPTS/.long-fuzz-$t-$STAMP.log" 2>&1
  rc=$?
  after="$(find "$adir" -name 'crash-*' -o -name 'oom-*' -o -name 'timeout-*' 2>/dev/null | sort)"
  newc="$(comm -13 <(printf '%s\n' "$before") <(printf '%s\n' "$after") | grep -c . )"
  ROW_TARGET+=("$t"); ROW_SEEDS+=("$(seed_count "$t")"); ROW_SEEDHASH+=("$(seed_hash "$t")")
  ROW_DUR+=("${PER_TARGET_SEC}s"); ROW_CRASHES+=("$newc"); ROW_EXIT+=("$rc")
  if [ "$newc" -gt 0 ]; then ROW_VERDICT+=("crash_found")
  elif [ "$rc" -eq 0 ]; then ROW_VERDICT+=("clean")
  else ROW_VERDICT+=("inconclusive_environment"); fi
}

# ---- preflight: toolchain present? ----
if [ "$CARGO_FUZZ_VER" = "absent" ] || [ "$RUSTC_VER" = "absent" ]; then
  overall="not_run"
else
  for t in "${SELECTED[@]}"; do
    [ "$INTERRUPTED" -eq 1 ] && break
    run_target "$t"
  done
  # aggregate verdict
  if [ "$INTERRUPTED" -eq 1 ]; then overall="interrupted"
  elif printf '%s\n' "${ROW_VERDICT[@]:-}" | grep -q crash_found; then overall="crash_found"
  elif [ ${#ROW_VERDICT[@]} -eq 0 ]; then overall="not_run"
  elif printf '%s\n' "${ROW_VERDICT[@]}" | grep -q inconclusive_environment; then overall="inconclusive_environment"
  else overall="clean"; fi
fi
END_UTC="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# ---- write receipt ----
{
echo "# RECEIPT — LONG-FUZZ burn-in run — $START_UTC"
echo
echo "> **Overall: \`$overall\`.** Vocabulary: clean | crash_found | inconclusive_environment | interrupted | not_run."
echo "> **This run does NOT claim coverage saturation.** A \`clean\` result means only that no crash was found"
echo "> within the stated target / time / seed configuration below. Produced by \`fuzz/run-long-burnin.sh\`."
echo
echo "## Provenance (recorded by the runner)"
echo
echo '```text'
echo "command_line:        $CMDLINE"
echo "mode:                ${MODE}${PER_TARGET_MIN:+ (per-target-minutes=$PER_TARGET_MIN)}${CAMPAIGN_HOURS:+ (campaign-hours=$CAMPAIGN_HOURS)}"
echo "per_target_duration: ${PER_TARGET_SEC}s  ($NT targets)"
echo "start_time:          $START_UTC"
echo "end_time:            $END_UTC"
echo "git_commit:          $GIT_COMMIT"
echo "rustc:               $RUSTC_VER"
echo "cargo_fuzz:          $CARGO_FUZZ_VER"
echo "libfuzzer:           $LIBFUZZER_VER"
echo "host_os:             $HOST_OS"
echo "host_kernel:         $HOST_KERNEL"
echo "host_cpu:            $HOST_CPU ($NCPU cpus)"
echo "crash_artifact_dir:  fuzz/artifacts/<target>/   (crash-*/oom-*/timeout-* land here)"
echo "target_list:         ${SELECTED[*]}"
echo '```'
echo
echo "## Per-target result"
echo
if [ ${#ROW_TARGET[@]} -eq 0 ]; then
  echo "_No target executed (toolchain absent / interrupted before first target). Status: \`$overall\`._"
else
  echo "| target | verdict | seeds | seed_corpus_hash | duration | new_crashes | exit |"
  echo "|---|---|--:|---|--:|--:|--:|"
  for i in "${!ROW_TARGET[@]}"; do
    printf '| %s | %s | %s | `%s` | %s | %s | %s |\n' \
      "${ROW_TARGET[$i]}" "${ROW_VERDICT[$i]}" "${ROW_SEEDS[$i]}" "${ROW_SEEDHASH[$i]:0:16}…" \
      "${ROW_DUR[$i]}" "${ROW_CRASHES[$i]}" "${ROW_EXIT[$i]}"
  done
fi
echo
echo "## Non-claims"
echo
echo "- **Not a saturation campaign.** Coverage is bounded by the per-target time above; un-run code paths are not exercised."
echo "- A \`clean\` verdict is evidence of no crash in *this* configuration, **not** a proof of crash-freedom."
echo "- \`inconclusive_environment\` = a target's libFuzzer process exited non-zero with no new artifact (build/host issue), **not** a zic-rs finding."
echo "- A real 24h-class burn-in (this harness, large \`--campaign-hours\`) on a dedicated host is the future operator/lab step."
echo
echo "## On a crash"
echo
echo "Minimize the artifact (\`cargo +$NIGHTLY fuzz tmin <target> <artifact>\`), add it as a regression test in"
echo "the **main** crate (\`tests/fuzz_regressions.rs\`), land the fix with that test, and record both here."
} > "$RECEIPT"

echo "LONG-FUZZ: overall=$overall  receipt=$RECEIPT"
[ "$overall" = "crash_found" ] && exit 1 || exit 0
