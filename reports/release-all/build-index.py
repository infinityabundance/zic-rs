#!/usr/bin/env python3
# RELEASE-ALL.1 — scrape data.iana.org/time-zones/releases/ and emit the complete classified index.
# Usage: curl -sSL https://data.iana.org/time-zones/releases/ -o /tmp/iana-index.html; python3 build-index.py
import re, sys
html=open(sys.argv[1] if len(sys.argv)>1 else '/tmp/iana-index.html').read()
rows=re.findall(r'<a href="([^"?/][^"]*)">[^<]*</a>\s*([0-9]{4}-[0-9]{2}-[0-9]{2} [0-9:]+)?\s*([0-9.]+[KMG]?|-)?', html)
entries={n:(d or '', s or '') for n,d,s in rows if not n.startswith('http') and n not in ('..','/')}
names=set(entries)
def classify(n):
    if n.endswith('.asc'): return 'signature'
    if n.endswith('.sign'): return 'signature'
    if re.match(r'tzdb-.*\.tar\.lz$', n): return 'tzdb_complete_bundle'
    if re.match(r'tz(32|64)?code', n): return 'tzcode'
    if re.match(r'tzdata', n): return 'tzdata_beta' if 'beta' in n else 'tzdata'
    return 'other'
def relid(n):
    m=re.search(r'((?:19|20)\d\d[a-z]?)', n); return m.group(1) if m else ('beta' if 'beta' in n else '')
def sigstat(n):
    if n+'.asc' in names: return 'asc_available'
    if n+'.sign' in names: return 'sign_available_legacy'
    return 'no_signature_available'
with open('reports/release-all/index.tsv','w') as f:
    f.write('filename\ttype\trelease_id\tsize\tdate\tsig_availability\turl\n')
    for n in sorted(entries):
        t=classify(n); d,s=entries[n]
        sa='' if t=='signature' else sigstat(n)
        f.write(f'{n}\t{t}\t{relid(n)}\t{s}\t{d}\t{sa}\thttps://data.iana.org/time-zones/releases/{n}\n')
print('indexed', len(entries), 'entries')
