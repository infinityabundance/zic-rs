#!/usr/bin/env bash
# Run the oracle (`zic-rs compare`) for the minimal fixtures against system `zic`.
# A convenience wrapper around the `compare` subcommand; exits non-zero on any mismatch.
set -euo pipefail
cd "$(dirname "$0")/.."

cargo build --quiet
bin=target/debug/zic-rs

status=0
for spec in "fixtures/minimal:Etc/UTC" "fixtures/minimal:Test/Fixed"; do
  input="${spec%%:*}"
  zone="${spec##*:}"
  if ! "$bin" compare --input "$input" --zone "$zone" --reference-zic zic; then
    status=1
  fi
done
exit "$status"
