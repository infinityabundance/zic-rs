#!/bin/bash
# Runs INSIDE archlinux:latest under `podman --network=none`. Drives zic-rs through a PKGBUILD/tzdata-build
# shape and compares to a reference zic. Mounts (ro): zic-rs binary, reference zic/zdump, tzdata.zi source.
set -u
echo "## container userland: $(grep PRETTY /etc/os-release | cut -d= -f2)"
echo "## zic-rs runs in Arch userland?"; /usr/local/bin/zic-rs --version 2>&1 | head -1; echo "   rs --version exit=$?"
echo "## reference zic (mounted host Arch-family tzcode):"; zic --version 2>&1 | head -1
echo "## PKGBUILD-style stage: compile system tzdata.zi -> staged zoneinfo tree"
/usr/local/bin/zic-rs compile --all-supported --input /src/tzdata.zi --out /out/rs 2>/dev/null; echo "   zic-rs compile exit=$?"
zic -d /out/ref /src/tzdata.zi 2>/dev/null; echo "   reference zic -d exit=$?"
echo "## file-set: rs=$(find /out/rs -type f | wc -l) ref=$(find /out/ref -type f | wc -l)"
echo "## slim byte parity (zic-rs -b slim vs reference slim), 4 zones:"
/usr/local/bin/zic-rs compile -b slim --all-supported --input /src/tzdata.zi --out /out/rsslim 2>/dev/null
m=0; d=0; for z in America/New_York Europe/London Asia/Gaza Etc/UTC; do cmp -s /out/rsslim/$z /out/ref/$z && m=$((m+1)) || d=$((d+1)); done
echo "   slim match=$m diff=$d"
echo "## bundle_hash (zic-rs size-report over the staged tree):"
/usr/local/bin/zic-rs size-report --out /out/rs --format json 2>/dev/null | grep -oE '"bundle_hash": "[0-9a-f]{16}' | head -1
echo "## argv/error matrix in-container:"
/usr/local/bin/zic-rs --bogusflag 2>/dev/null; echo "   invalid-flag rs-exit=$?"
zic --bogusflag 2>/dev/null; echo "   invalid-flag ref-exit=$?"
/usr/local/bin/zic-rs compile --input /nonexistent.zi --out /out/x 2>/dev/null; echo "   missing-src rs-exit=$?"
zic -d /out/y /nonexistent.zi 2>/dev/null; echo "   missing-src ref-exit=$?"
echo "## DONE"
