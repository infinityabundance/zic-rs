#!/bin/bash
# Runs INSIDE registry.suse.com/bci/bci-base (SLES 15-SP7, glibc 2.38) under `podman --network=none`.
# Enterprise RPM/SUSE commercial-support ecology. ABI-honest: the host glibc PIE (needs GLIBC_2.39) is
# EXPECTED to fail on glibc 2.38 -> recorded as a version/ABI mismatch, NOT a parity failure; the
# musl-static zic-rs is the binary under test. Reference = the BCI's OWN /usr/sbin/zic (SLES tzcode,
# the `timezone` RPM lineage from T16.5b.16). Mounts (ro): both zic-rs binaries + the host tzdata.zi.
set -u
echo "## container userland: $(grep PRETTY_NAME /etc/os-release | cut -d= -f2)"
echo "## glibc: $(ldd --version 2>&1 | head -1)  rpm: $(rpm -q glibc 2>/dev/null)"
echo "## --- ABI honesty: host glibc PIE (built against GLIBC_2.39) in glibc 2.38 ---"
/usr/local/bin/zic-rs-glibc --version 2>&1 | head -1; echo "   glibc-PIE --version exit=$? (expected nonzero: GLIBC_2.39 absent)"
echo "## --- binary under test: host-cross-built musl-static zic-rs ---"
/usr/local/bin/zic-rs-musl --version 2>&1 | head -1; echo "   musl --version exit=$?"
echo "## reference zic (BCI's own /usr/sbin/zic, SLES `timezone` RPM):"; zic --version 2>&1 | head -1
echo "   reference owner: $(rpm -qf /usr/sbin/zic 2>/dev/null)"
echo "## RPM-build-style stage: compile the (mounted) tzdata.zi -> staged zoneinfo tree"
/usr/local/bin/zic-rs-musl compile --all-supported --input /src/tzdata.zi --out /out/rs 2>/dev/null; echo "   zic-rs compile exit=$?"
zic -d /out/ref /src/tzdata.zi 2>/dev/null; echo "   reference zic -d exit=$?"
echo "## file-set: rs=$(find /out/rs -type f | wc -l) ref=$(find /out/ref -type f | wc -l)"
echo "## slim byte parity (zic-rs -b slim vs reference slim), 4 zones:"
/usr/local/bin/zic-rs-musl compile -b slim --all-supported --input /src/tzdata.zi --out /out/rsslim 2>/dev/null
m=0; d=0; for z in America/New_York Europe/London Asia/Gaza Etc/UTC; do cmp -s /out/rsslim/$z /out/ref/$z && m=$((m+1)) || d=$((d+1)); done
echo "   slim match=$m diff=$d"
echo "## bundle_hash (zic-rs size-report over the staged tree):"
/usr/local/bin/zic-rs-musl size-report --out /out/rs --format json 2>/dev/null | grep -oE '"bundle_hash": "[0-9a-f]{16}' | head -1
echo "## argv/error matrix in-container:"
/usr/local/bin/zic-rs-musl --bogusflag 2>/dev/null; echo "   invalid-flag rs-exit=$?"
zic --bogusflag 2>/dev/null; echo "   invalid-flag ref-exit=$?"
/usr/local/bin/zic-rs-musl compile --input /nonexistent.zi --out /out/x 2>/dev/null; echo "   missing-src rs-exit=$?"
zic -d /out/y /nonexistent.zi 2>/dev/null; echo "   missing-src ref-exit=$?"
echo "## DONE"
