#!/usr/bin/env bash
# Regenerate the pinned expected/ TZif blobs from reference `zic`.
#
# These blobs are the byte-parity oracle (see fixtures/MANIFEST.toml). Re-run this only when
# intentionally re-pinning to a new reference `zic`/tzdata version, and record the new
# version in MANIFEST.toml. Requires `zic` on PATH.
set -euo pipefail
cd "$(dirname "$0")/.."

ref="$(mktemp -d)"
trap 'rm -rf "$ref"' EXIT

echo "reference: $(zic --version)"
zic -d "$ref" fixtures/minimal/utc.zi fixtures/minimal/fixed.zi

cp "$ref/Etc/UTC"    fixtures/expected/Etc_UTC.tzif
cp "$ref/Test/Fixed" fixtures/expected/Test_Fixed.tzif
echo "regenerated fixtures/expected/{Etc_UTC,Test_Fixed}.tzif"
