#!/bin/sh
# Runs INSIDE a musl/Alpine userland (python:3.12-alpine) under `podman --network=none`.
# ABI-honest: the MUSL-built static zic-rs is the binary under test; the glibc binary is run only to
# DEMONSTRATE the ABI boundary (its failure is an expected ABI mismatch, NOT a zic-rs parity failure).
set -u
echo "## userland: $(grep PRETTY /etc/os-release | cut -d= -f2)"
echo "## --- ABI boundary demo: glibc binary in musl userland (expected to FAIL, not a parity result) ---"
/usr/local/bin/zic-rs-glibc --version >/dev/null 2>&1; echo "   glibc-binary exit=$? (nonzero/exec-fail = expected ABI mismatch)"
echo "## --- the test: MUSL-built static zic-rs in musl userland ---"
ldd /usr/local/bin/zic-rs 2>&1 | head -1
/usr/local/bin/zic-rs --version 2>&1 | head -1; echo "   musl zic-rs --version exit=$?"
/usr/local/bin/zic-rs compile --all-supported --input /src/tzdata.zi --out /out/rs 2>/dev/null; echo "   compile exit=$?"
echo "## file-set: $(find /out/rs -type f | wc -l)"
echo "## bundle_hash:"; /usr/local/bin/zic-rs size-report --out /out/rs --format json 2>/dev/null | grep -oE '"bundle_hash": "[0-9a-f]{16}'
echo "## argv/error matrix (musl):"
/usr/local/bin/zic-rs --bogusflag 2>/dev/null; echo "   invalid-flag exit=$?"
/usr/local/bin/zic-rs compile --input /nonexistent.zi --out /out/x 2>/dev/null; echo "   missing-src exit=$?"
echo "## DONE"
