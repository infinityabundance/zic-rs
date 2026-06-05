#!/usr/bin/env bash
# doc-staleness-check.sh (DOC-CURRENCY.1, strengthened in DOC-CURRENCY.2) — guard the FRONT-DOOR docs
# against status drift.
#
# DOC-CURRENCY.1 only grepped for *known-old* phrases, so when a campaign minted NEW stale values
# (e.g. cargo-vet.4 -> .5 made "36 exempted" / "29 fully" / "cargo-vet.4" / "pass4" stale) the gate
# missed them. DOC-CURRENCY.2 fixes that: the maintainer keeps a single CURRENT-STATE FACTS block
# below, and the gate (a) flags PRIOR values of those facts when they appear as a live front-door
# claim, and (b) asserts the live authority (STATUS.md) actually states the CURRENT facts.
#
# A stale hit is excused only when the SAME line carries a reconciliation marker (it names the current
# value, a progression like "39->36->33", "historical"/"at seal"/"later", or "STATUS.md"). Historical
# receipts (reports/t*-close-receipt.md, audits/*/receipts/) and legit T18-archival `pending_capture`
# are OUT OF SCOPE by design.
#
# Exit 0 = front door is current. Exit 1 = drift (printed). Run from repo root.
set -u
cd "$(dirname "$0")/.." || exit 2

# ============================ CURRENT-STATE FACTS ============================
# Update these in the SAME batch as any campaign that changes them. This is the single source the
# gate reasons from — change a number here and the gate will demand the docs agree.
CUR_CAMPAIGN='AUDIT-SUITE-RUN.1'   # the latest campaign (STATUS.md "Last updated" must name it)
CUR_TESTS=520
CUR_VET_FULLY=42
CUR_VET_EXEMPTED=23
CUR_VET_PASS=8
CUR_VET_RECEIPT='RECEIPT-2026-06-05-pass8.md'
CUR_KANI=10
# ===========================================================================

# PRIOR (now-stale) values that must NOT appear as a LIVE front-door claim (regex<TAB>label):
STALE_RULES=(
  '\b(480|485|497|503) (default )?(tests|green)	stale test count'
  '\b(43|66|36|33|31|26) (still |honestly )?(deps?|dependencies|exempt|UNAUD)	stale cargo-vet exempted count'
  '\b(22|29|39) fully	stale cargo-vet fully-audited count'
  'cargo-vet\.(1|2|3|4)\b	cargo-vet pass < current as live state'
  'RECEIPT-2026-06-0[0-9]-pass[1-7]	stale cargo-vet receipt as current'
  '\b6 (bounded|reduced|Kani|formal)	stale Kani proof count'
  'scaffold.only	fuzz scaffold-only (superseded by T23.cargo-fuzz)'
  'no fuzz-run claim	fuzz no-run (superseded)'
)
# A stale hit on a line ALSO matching this regex is reconciled (historical / progression / current value):
MARK='historical|at seal|snapshot|later|progression|→|->|STATUS\.md|cargo-vet\.[567]|RECEIPT-2026-06-05-pass[567]|\b32\b|\b33\b|\b10 (bounded|formal)|503|T23\.cargo-fuzz'

# Front-door files a reviewer reaches first (summary surfaces, not deep receipts):
FRONTDOOR=(
  README.md TRUST.md STATUS.md docs/REVIEW-IN-10-MINUTES.md
  audits/README.md audits/index.html
  docs/audit-readiness.md docs/security-rewrite-evaluation.md docs/not-yet-ready.md
  docs/security-personas.md docs/misuse-resistance-ledger.md docs/reviewer-orientation.md
  docs/replacement-readiness-ladder.md docs/PORTING-DECISION-LEDGER.md
)

fail=0

# (a) stale-value detection across the front door
for f in "${FRONTDOOR[@]}"; do
  [ -f "$f" ] || { echo "MISSING front-door file: $f"; fail=1; continue; }
  for rule in "${STALE_RULES[@]}"; do
    pat="${rule%%	*}"; label="${rule##*	}"
    while IFS=: read -r ln text; do
      [ -z "${ln:-}" ] && continue
      printf '%s' "$text" | grep -qiE "$MARK" && continue
      echo "DRIFT  $f:$ln  [$label]  /$pat/ without reconciliation"
      echo "       > $(printf '%s' "$text" | sed 's/^[[:space:]]*//' | cut -c1-110)"
      fail=1
    done < <(grep -niE "$pat" "$f" 2>/dev/null)
  done
done

# (b) positive assertions: the LIVE authority STATUS.md must STATE the current facts
assert_status() {  # regex, human-label
  if ! grep -qE "$1" STATUS.md; then
    echo "MISSING-FACT  STATUS.md does not state current $2 (expected /$1/)"; fail=1
  fi
}
assert_status "$CUR_VET_FULLY fully" "cargo-vet fully-audited ($CUR_VET_FULLY)"
assert_status "$CUR_VET_EXEMPTED (dependencies|exempt|still)" "cargo-vet exempted ($CUR_VET_EXEMPTED)"
assert_status "$CUR_VET_RECEIPT" "cargo-vet receipt ($CUR_VET_RECEIPT)"
assert_status "\b$CUR_TESTS tests" "test count ($CUR_TESTS)"
# STATUS.md "Last updated" line must name the latest campaign
if ! grep -E "Last updated" STATUS.md | grep -qF "$CUR_CAMPAIGN"; then
  echo "STALE-STAMP  STATUS.md 'Last updated' does not name the latest campaign ($CUR_CAMPAIGN)"; fail=1
fi

if [ "$fail" -eq 0 ]; then
  echo "doc-staleness-check: OK — front-door docs are current and STATUS.md states the facts."
else
  echo "doc-staleness-check: DRIFT (above). Fix the doc, add a reconciliation marker, or update the CURRENT-STATE FACTS block."
fi
exit "$fail"
