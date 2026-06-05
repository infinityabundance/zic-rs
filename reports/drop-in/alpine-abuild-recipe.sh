#!/bin/sh
# Tier-3 ecosystem source-build: Alpine's OWN Rust toolchain (apk `rust`/`cargo`) builds zic-rs FROM THE
# crates.io source crate, producing a distro-native musl binary that then runs the drop-in matrix.
# Runs INSIDE alpine:latest WITH network (a source-build must fetch the crate + its crates.io deps).
set -e
echo "## alpine: $(grep PRETTY_NAME /etc/os-release | cut -d= -f2)"
echo "## installing the Alpine Rust toolchain (apk rust cargo) ..."
apk add --no-cache rust cargo >/dev/null 2>&1
echo "## toolchain identity:"
echo "   rustc=$(rustc --version 2>&1)"
echo "   cargo=$(cargo --version 2>&1)"
echo "   apk rust=$(apk list --installed 2>/dev/null | grep -E '^rust-' | head -1)"
echo "   apk cargo=$(apk list --installed 2>/dev/null | grep -E '^cargo-' | head -1)"
echo "   musl: $(ls /lib/ld-musl-* 2>/dev/null | head -1)"
echo "## building zic-rs 0.1.0 from the crates.io SOURCE crate (Alpine cargo, musl-native, --locked) ..."
cargo install zic-rs --version 0.1.0 --locked --root /opt/zic >/tmp/build.log 2>&1 || { echo "BUILD FAILED"; tail -20 /tmp/build.log; exit 1; }
BIN=/opt/zic/bin/zic-rs
echo "## ecosystem-built binary:"
echo "   $($BIN --version 2>&1 | head -1)"
echo "   binary_sha256=$(sha256sum $BIN | cut -d' ' -f1)"
echo "   runtime_abi: $(ldd $BIN 2>&1 | head -1 || echo 'static-musl')"
echo "## drop-in matrix with the ecosystem-built binary:"
$BIN compile --all-supported --input /src/tzdata.zi --out /out >/dev/null 2>&1; echo "   compile_exit=$?"
echo "   files=$(find /out -type f | wc -l)"
echo "   bundle_hash: $($BIN size-report --out /out --format json 2>/dev/null | grep -oE '\"bundle_hash\": \"[0-9a-f]{16}')"
echo "## tzdata source sha: $(sha256sum /src/tzdata.zi | cut -c1-16)"
echo "## DONE"
