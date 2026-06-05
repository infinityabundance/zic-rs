#!/usr/bin/env bash
# panic-analysis — reproducible static panic-surface census over src/ (vs docs/panic-policy.md).
# A real, re-runnable scan (not a hand-written assertion). Emits counts + the unwrap/expect ledger.
set -u
cd "$(dirname "$0")/../.."   # repo root
echo "panic-analysis scan — $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "host: $(uname -srm)  rustc: $(rustc --version 2>/dev/null)"
echo "scope: src/ tree"
echo
prim() { printf '%-16s %s\n' "$1" "$(grep -rnE "$2" src/ 2>/dev/null | grep -vE '//|test' | wc -l)"; }
echo "## Prohibited primitives (must be 0 on every path)"
prim "panic!"        '\bpanic!\s*\('
prim "todo!"         '\btodo!\s*\('
prim "unimplemented!" '\bunimplemented!\s*\('
echo
echo "## Allowed-with-justification primitives (internal invariants)"
printf '%-16s %s\n' "unreachable!" "$(grep -rnE '\bunreachable!\s*\(' src/ | grep -v '//' | wc -l)"
printf '%-16s %s\n' "unwrap()" "$(grep -rnE '\.unwrap\(\)' src/ | wc -l)"
printf '%-16s %s\n' "expect()" "$(grep -rnE '\.expect\(' src/ | wc -l)"
echo
echo "## Backstops"
printf 'forbid(unsafe_code): %s\n' "$(grep -rl '#!\[forbid(unsafe_code)\]' src/lib.rs >/dev/null 2>&1 && echo present || echo MISSING)"
printf 'overflow-checks (release): %s\n' "$(grep -A3 '\[profile.release\]' Cargo.toml | grep -c 'overflow-checks = true')"
echo
echo "## unwrap/expect ledger (every site — for the STATIC-REVIEW.1 3-bucket classification)"
grep -rnE '\.unwrap\(\)|\.expect\(' src/ | sed 's/^/  /'
