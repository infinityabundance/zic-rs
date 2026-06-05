#!/usr/bin/env bash
cd /tmp/sv/tzdb-2026b || exit 2
TDATA="africa antarctica asia australasia europe northamerica southamerica etcetera factory backward"
mkdir -p /tmp/sv/zi
for form in main vanguard rearguard; do
  awk -v DATAFORM=$form -f ziguard.awk $TDATA > /tmp/sv/zi/$form.zi 2>/tmp/sv/zi/$form.err
  echo "$form.zi: $(wc -l < /tmp/sv/zi/$form.zi) lines, sha256 $(sha256sum /tmp/sv/zi/$form.zi | cut -c1-16)"
done
# backzone-included (PACKRATDATA=backzone) in main form
awk -v DATAFORM=main -f ziguard.awk $TDATA backzone > /tmp/sv/zi/main-backzone.zi 2>/dev/null
echo "main-backzone.zi: $(wc -l < /tmp/sv/zi/main-backzone.zi) lines, sha256 $(sha256sum /tmp/sv/zi/main-backzone.zi | cut -c1-16)"
