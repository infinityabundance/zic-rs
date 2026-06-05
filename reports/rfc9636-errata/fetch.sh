#!/usr/bin/env bash
# RFC9636-ERRATA.1 — reproduce the errata check.
# The RFC Editor publishes the AUTHORITATIVE machine-readable errata feed at /errata.json
# (the HTML search page at errata.rfc-editor.org is a JS UI that renders no server-side records).
# This fetches the feed and filters for RFC 9636, with a sanity check against two known-errata RFCs.
set -u
OUT=${OUT:-/tmp/errata_all.json}
curl -sSL --max-time 90 -A "Mozilla/5.0" "https://www.rfc-editor.org/errata.json" -o "$OUT" || { echo "fetch failed"; exit 2; }
echo "feed sha256: $(sha256sum "$OUT" | cut -d' ' -f1)"
python3 - "$OUT" <<'PY'
import json,sys
d=json.load(open(sys.argv[1]))
def n(t): return sum(1 for e in d if e.get("doc-id")==t)
print(f"total errata in feed : {len(d)}")
print(f"RFC 8259 (sanity)    : {n('RFC8259')}  (expect >0)")
print(f"RFC 8536 (predecessor): {n('RFC8536')}  (expect 4)")
print(f"RFC 9636 (TZif)      : {n('RFC9636')}  (expect 0)")
PY
