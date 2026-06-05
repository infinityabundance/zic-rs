#!/usr/bin/env bash
# SHIM-CONTRACT.1 — conformance test for zic-shim-v1 (asserts the contract in SHIM-CONTRACT.md).
# Each check prints PASS/FAIL; the script exits non-zero on any failure.
set -u
cd "$(dirname "$0")/../.." || exit 2
ZICRS=${ZICRS:-$PWD/target/release/zic-rs}
SHIM="$PWD/reports/package-acceptance/zic-shim.sh"
PKG=${PKG:-/tmp/pkg}
T=/tmp/shim-test; rm -rf "$T"; mkdir -p "$T"
sh_zic() { ZICRS="$ZICRS" sh "$SHIM" "$@"; }
fails=0
ck() { if [ "$2" = "$3" ]; then echo "PASS  $1"; else echo "FAIL  $1 (got [$2] want [$3])"; fails=$((fails+1)); fi; }

# 1. accepted flags: -d + input builds a tree
sh_zic -d "$T/a" "$PKG/tzdata.zi" >/dev/null 2>&1
ck "accepts -d + input (builds tree)" "$([ "$(find "$T/a" -type f 2>/dev/null | wc -l)" -gt 100 ] && echo ok)" ok

# 2. -L leapseconds accepted (right tree builds)
sh_zic -d "$T/b" -L "$PKG/leapseconds" "$PKG/tzdata.zi" >/dev/null 2>&1
ck "accepts -L leapseconds" "$([ "$(find "$T/b" -type f 2>/dev/null | wc -l)" -gt 100 ] && echo ok)" ok

# 3. refused/ignored flags (-p, -y, unknown) do not break the build
sh_zic -d "$T/c" -p posixrules -y cmd --bogus "$PKG/tzdata.zi" >/dev/null 2>&1
ck "ignores -p/-y/unknown (build still succeeds)" "$?" 0

# 4. error propagation: bad input -> exit 1 (zic-rs status passes through)
printf 'Frobnicate X\n' > "$T/bad.zi"
sh_zic -d "$T/d" "$T/bad.zi" >/dev/null 2>&1
ck "propagates error exit (bad input -> 1)" "$?" 1

# 5. multiple input files: both are compiled (order preserved)
printf 'Zone Test/One 1:00 - O1\n' > "$T/one.zi"
printf 'Zone Test/Two 2:00 - T2\n' > "$T/two.zi"
sh_zic -d "$T/multi" --all-supported "$T/one.zi" "$T/two.zi" >/dev/null 2>&1
ck "multiple --input (both zones present)" "$([ -f "$T/multi/Test/One" ] && [ -f "$T/multi/Test/Two" ] && echo ok)" ok

# 6. -t safety preserved: absolute/outside -t is REFUSED (zic-rs ZIC008), relative accepted under --out
sh_zic -d "$T/abs/zoneinfo" -l Factory -t "$T/abs/etc/localtime" "$PKG/tzdata.zi" >/dev/null 2>&1
ck "refuses absolute -t (path safety not bypassed)" "$?" 1
sh_zic -d "$T/rel/zoneinfo" -l Factory -t localtime "$PKG/tzdata.zi" >/dev/null 2>&1
ck "accepts safe relative -t under --out" "$([ -e "$T/rel/zoneinfo/localtime" ] && echo ok)" ok

# 7. environment quarantine: output identical regardless of ambient TZ/LC_ALL (only ZICRS matters)
sh_zic -d "$T/env1" "$PKG/tzdata.zi" >/dev/null 2>&1
TZ=Pacific/Kiritimati LC_ALL=tr_TR.UTF-8 sh_zic -d "$T/env2" "$PKG/tzdata.zi" >/dev/null 2>&1
h1=$(cd "$T/env1" && find . -type f | sort | xargs sha256sum 2>/dev/null | sha256sum)
h2=$(cd "$T/env2" && find . -type f | sort | xargs sha256sum 2>/dev/null | sha256sum)
ck "ambient TZ/LC_ALL do not change output (env-quarantined)" "$h1" "$h2"

# 8. quoting: a path containing a space is passed verbatim (no word-split)
mkdir -p "$T/has space"; cp "$PKG/tzdata.zi" "$T/has space/d.zi"
sh_zic -d "$T/spaced" "$T/has space/d.zi" >/dev/null 2>&1
ck "path with spaces passed verbatim (no glob/word-split)" "$([ "$(find "$T/spaced" -type f 2>/dev/null | wc -l)" -gt 100 ] && echo ok)" ok

echo ""
if [ "$fails" -eq 0 ]; then echo "shim-test: OK — zic-shim-v1 conforms to SHIM-CONTRACT.md (8/8 checks)"; else echo "shim-test: $fails FAILED"; fi
exit "$fails"
