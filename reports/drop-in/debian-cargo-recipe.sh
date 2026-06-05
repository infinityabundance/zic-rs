#!/bin/sh
# Tier-3: Debian's own Rust toolchain builds zic-rs FROM the crates.io source crate. Tries the Debian
# packaging tool debcargo first; falls back to `cargo install` from crate source. Runs in debian:trixie.
set -e
echo "## debian: $(grep PRETTY_NAME /etc/os-release | cut -d= -f2)"
export DEBIAN_FRONTEND=noninteractive
echo "## apt install cargo + build-essential ..."
apt-get update -qq >/dev/null 2>&1
apt-get install -y -qq cargo build-essential ca-certificates >/dev/null 2>&1
echo "## toolchain identity:"
echo "   rustc=$(rustc --version 2>&1)"
echo "   cargo=$(cargo --version 2>&1)"
echo "   dpkg cargo=$(dpkg-query -W -f='${Package} ${Version}\n' cargo 2>/dev/null)"
echo "   glibc=$(ldd --version 2>&1 | head -1)"
echo "## building zic-rs 0.1.0 from the crates.io SOURCE crate (Debian cargo, --locked) ..."
cargo install zic-rs --version 0.1.0 --locked --root /opt/zic >/tmp/build.log 2>&1 || { echo "BUILD FAILED"; tail -20 /tmp/build.log; exit 1; }
BIN=/opt/zic/bin/zic-rs
echo "## ecosystem-built binary:"
echo "   $($BIN --version 2>&1 | head -1)"
echo "   binary_sha256=$(sha256sum $BIN | cut -d' ' -f1)"
echo "   runtime_abi: $(ldd $BIN 2>&1 | head -1)"
echo "## drop-in matrix with the Debian-built binary:"
$BIN compile --all-supported --input /src/tzdata.zi --out /out >/dev/null 2>&1; echo "   compile_exit=$?"
echo "   files=$(find /out -type f | wc -l)"
echo "   bundle_hash: $($BIN size-report --out /out --format json 2>/dev/null | grep -oE '\"bundle_hash\": \"[0-9a-f]{16}')"
echo "## tzdata source sha: $(sha256sum /src/tzdata.zi | cut -c1-16)"
echo "## DONE"
