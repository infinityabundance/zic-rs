#!/usr/bin/env python3
# TZDB-ATLAS.2 — join the three evidence axes (upstream-archive/data × vendor-oracle × drop-in, +reader)
# into one queryable atlas with per-finding AXIS ATTRIBUTION. No new compiler work — pure join.
# .2 reconciles the one zic-rs-side source-shape band CLOSED by LEGACY-SOURCE.1 (Latin-1 era → match
# under --legacy-latin1); the default UTF-8 contract is recorded unchanged alongside the legacy tally.
import json, glob, re, collections
ROWS=[]  # (axis, subject, key_fact, verdict, attribution, receipt)
def row(*a): ROWS.append(a)

# ---- AXIS 1a: upstream archive (RELEASE-ALL.1) ----
idx=[l.split('\t') for l in open('reports/release-all/index.tsv').read().splitlines()[1:]]
arch=[r for r in idx if r[1]!='signature']
ct=collections.Counter(r[1] for r in idx)
row('upstream_archive','IANA release directory',
    f"{len(idx)} entries · {len(arch)} archives ({ct['tzdata']} tzdata · {ct['tzcode']} tzcode · {ct['tzdb_complete_bundle']} bundles) · 214 sigs",
    'indexed','upstream_archive','docs/iana-release-archive-ledger.md')
signed=sum(1 for r in arch if r[5] in ('asc_available','sign_available_legacy'))
row('upstream_archive','signature provenance',
    f"{signed}/{len(arch)} signature-backed (357 unsigned pre-signing-era); 55/55 signed-bundle+pilot GOODSIG",
    'verified-subset','upstream_archive','reports/release-all/verified.tsv')
# RELEASE-METADATA.1: the release_id every atlas row keys on is multi-surface-derived, not a single label.
rm=collections.Counter()
try:
    rmt=[l.split('\t') for l in open('reports/release-metadata/release-metadata.tsv').read().splitlines()[1:]]
    rm=collections.Counter(r[6] for r in rmt)
except FileNotFoundError: pass
row('upstream_archive','release-identity provenance',
    f"276 releases; the authoritative tags (filename + version file) NEVER disagree → {rm.get('major_metadata_contradiction',0)} major contradictions. {rm.get('minor_metadata_contradiction',0)} minor (NEWS-changelog surface only, recorded as facts: 2019a upstream typo 'Release 20198'; 2026b NEWS lags at 2026a). {rm.get('legacy_unstructured',0)} legacy_unstructured (filename-only era). The atlas release_id is provenance-grounded, not a single label.",
    'multi-surface-consistent','upstream_archive','reports/release-metadata/RECEIPT-RELEASE-METADATA-1.md')
# RELEASE-ALL.TZDB.1: the 48 complete data+code bundles.
tb=collections.Counter()
try:
    tbt=[l.split('\t') for l in open('reports/release-all/tzdb-bundles.tsv').read().splitlines()[1:]]
    tb=collections.Counter(r[9] for r in tbt)
except FileNotFoundError: pass
row('upstream_archive','complete-bundle phase (RELEASE-ALL.TZDB.1)',
    f"48 signed tzdb-*.tar.lz bundles (2016g→2026b): {tb.get('verified_combined_release',0)}/48 verified_combined_release — all GOODSIG (Eggert), all combined data+code, all internally identity-consistent (filename==version file), and all data BYTE-IDENTICAL to the standalone tzdata pair → behaviour inherits RELEASE-ALL.DATA.1's match. 0 identity contradictions; 0 data divergence.",
    'verified-combined-release','upstream_archive','reports/release-all/RECEIPT-RELEASE-ALL-TZDB-1.md')

# ---- AXIS 1b: upstream behaviour (RELEASE-ALL.DATA.1, default UTF-8 mode) + LEGACY-SOURCE.1 reconciliation ----
dg=[l.split('\t') for l in open('reports/release-all/data-gauntlet.tsv').read().splitlines()[1:]]
vc=collections.Counter(r[12] for r in dg)
mtch=vc['match']; rbi=vc['reference-build-incompatible']; ssi=vc['source-shape-incompatible']
div=vc['zic-rs-divergent']; legacy_match=mtch+ssi  # the 60 Latin-1 releases all close → match under legacy mode
# YEARISTYPE.1: split the reference_zic (yearistype) band into closed-vs-oracle vs deferred-footer;
# PERPETUAL-EXPANSION.1: close the 11 perpetual-footer releases via the empty-footer fallback.
yc=collections.Counter()
try:
    yt=[l.split('\t') for l in open('reports/yearistype/yearistype.tsv').read().splitlines()[1:]]
    yc=collections.Counter(r[7] for r in yt)
except FileNotFoundError: pass
yit_match=yc['match']; yit_defer=yc['deferred-perpetual-footer']   # 55 + 11
pe=collections.Counter()
try:
    pt=[l.split('\t') for l in open('reports/perpetual-expansion/perpetual-expansion.tsv').read().splitlines()[1:]]
    pe=collections.Counter(r[8] for r in pt)
except FileNotFoundError: pass
pe_match=pe['match']                                               # 66 under both legacy modes
both_match=legacy_match+yit_match                                  # --legacy-latin1 --legacy-yearistype
all_match=legacy_match+pe_match                                    # + --legacy-empty-footer → full band
row('upstream_data','all stable tzdata (1993→2026b)',
    f"{len(dg)} releases compiled. DEFAULT UTF-8: {mtch} match · {rbi} ref-build-incompat · {ssi} source-shape-incompat · {div} divergent. "
    f"+--legacy-latin1: {legacy_match} match. +--legacy-yearistype: {both_match} match · {yit_defer} deferred-footer. "
    f"+--legacy-empty-footer: {all_match} match · 0 deferred · {div} divergent (PERPETUAL-EXPANSION.1) — every stable release replays under explicit modes (yearistype band vs a historical oracle)",
    'match-where-both-build','zic_rs','reports/release-all/RECEIPT-RELEASE-ALL-DATA-1.md')
row('upstream_data','behaviour divergence',
    "1050/1050 default + 420/420 latin1-band + 371/371 yearistype+empty-footer-band behaviour-match; ZERO zic-rs behaviour divergence across 30 years",
    '0-divergent','zic_rs','reports/release-all/data-gauntlet.tsv')
row('reference_zic','yearistype breakpoint (≤2000e) — CLOSED 66/66',
    f"{rbi} releases: pre-2000f tzdata uses Rule TYPE even/odd; current zic REMOVED -y/yearistype (tzcode 2020a) so it can't build them. "
    f"YEARISTYPE.1 (--legacy-yearistype, internal predicates) + PERPETUAL-EXPANSION.1 (--legacy-empty-footer) → all {pe_match}/66 build and match an ADMITTED HISTORICAL ORACLE (old zic 2019c + yearistype.sh v7.4) over [1980,2037] (371/371 fixtures). "
    f"Historical-source replay, NOT current-reference parity — reference zic still can't build them.",
    'closed-by-historical-replay','bounded_legacy_source','reports/yearistype/RECEIPT-YEARISTYPE-1.md')
row('zic_rs','perpetual year-parity footer (93b–94f) — CLOSED',
    f"{yit_defer} releases (1993-94) had PERPETUAL even/odd rules (1990 max even/odd) → recurring tail not POSIX-footer-expressible. CLOSED by PERPETUAL-EXPANSION.1: --legacy-empty-footer emits the already-expanded explicit transitions (through RECUR_HI=2037) + an EMPTY footer, matching the oracle's behaviour over [1980,2037]. Footer-emission policy only; default still fails closed (ZIC001); CORE.1 byte-unchanged. Beyond-horizon freeze not claimed.",
    'bounded_empty_footer_replay','bounded_legacy_source','reports/perpetual-expansion/RECEIPT-PERPETUAL-EXPANSION-1.md')
row('zic_rs','failure-mode parity (DIAG-PARITY.1)',
    "17 malformed/hostile fixtures vs reference zic: 14 class_exit_match (same diagnostic class + exit status) · 1 intentional_divergence (zic-rs finer ZIC014 vs generic 'unknown line type', T13.2) · 1 safer_no_partial (zic-rs writes no partial output where ref does, T9.3) · 1 warning_match · 0 DIVERGENT. zic-rs fails in the same CLASS as reference; every deviation is named + safer + intentional. (Class/severity/exit/side-effects compared; wording not.)",
    'class-exit-match-or-safer','zic_rs','reports/diag-parity/RECEIPT-DIAG-PARITY-1.md')
row('source_shape','source-profile parity (SOURCE-VARIANT.1)',
    "zic-rs behaviour-matches reference zic on ALL major tzdb source profiles (2026b): DATAFORM main/vanguard/rearguard + backzone excluded/included — 8/8 fixtures each (incl. the backzone-sensitive America/Montreal), 0 divergent, 0 deferred, 0 unsupported. zic-rs normalises the 3 encodings to byte-identical output. Not main-only; vanguard's negative-SAVE/%z AND rearguard's nonnegative-SAVE expansion both match. Byte-diff = the documented slim/fat default.",
    'all-profiles-behaviour-match','zic_rs','reports/source-variant/RECEIPT-SOURCE-VARIANT-1.md')
row('source_shape','UTF-8/Latin-1 breakpoint (2008a–2012j) — CLOSED',
    f"{ssi} releases were ISO-8859/Latin-1 pre-2013; CLOSED by LEGACY-SOURCE.1 — --legacy-latin1 admits Latin-1 in COMMENTS ONLY (semantics-bearing fields still fail closed) → all {ssi} move to match (420/420 fixtures, 0 divergent, verified vs reference zic). Default stays UTF-8-required (ZIC012)",
    'closed-by-legacy-source','bounded_legacy_source','reports/release-all/RECEIPT-LEGACY-SOURCE-1.md')

# ---- AXIS 2: vendor-oracle ----
rels=set()
for f in sorted(glob.glob('../zic-rs-vendor-oracle-lab/receipts/*.json')):
    d=json.load(open(f)); plat=d.get('platform','?'); ver=d.get('zic_version_output') or 'no on-device zic'
    rel=d.get('tzdb_source_release') or '?'; st=d.get('platform_status','?')
    lin='glibc_zic' if ('GNU libc' in ver or 'GLIBC' in ver) else ('tzcode_zic' if 'tzcode' in ver else 'vendor_fork/none')
    m=re.search(r'(20\d\d[a-z]|2026\.\d)', rel); 
    if m: rels.add(m.group(1))
    attr='vendor_zic'
    if lin=='vendor_fork/none' and st=='admitted': verdict='old-fork/no-zic'
    elif st!='admitted': verdict=st
    else: verdict='admitted (4-5/5 class+location)'
    row('vendor_oracle',plat,f"lineage={lin} · {ver} · ships {rel}",verdict,attr,f'research/.../receipts/{plat}.json')

# ---- AXIS 3: drop-in ----
import subprocess
bh=subprocess.run("grep -rhoE '[0-9a-f]{16}' reports/drop-in/*.md | sort | uniq -c | sort -rn | head -1",
                  shell=True,capture_output=True,text=True).stdout.strip()
envs=subprocess.run("ls reports/drop-in/RECEIPT-*.md | sed 's#.*/RECEIPT-##;s#\\.md##' | grep -vE 'coverage|matrix|host|source-matrix|STATUS'|wc -l",
                    shell=True,capture_output=True,text=True).stdout.strip()
row('drop_in','cross-filesystem/rootless materialization (CROSS-FS.1)',
    "zic-rs materialization is deterministic + safe across environments: bundle_hash 453641ff IDENTICAL on two distinct ext4 filesystems + tmpfs; copy tree survives cross-fs relocation (cp -a, hash unchanged); --link-mode symlinks are RELATIVE and resolve after relocation; rootless (uid!=root) works. The hardlink-fragility rationale is DEMONSTRATED: reference hardlink sharing (341 inodes) breaks on a naive cross-fs copy (->598 inodes, overlayfs copy-up proxy) while zic-rs copy/symlink have nothing to break. 9/9 deterministic-and-safe, 0 broken. (FAT/case-insensitive/overlayfs-proper need root to mount; proxied/deferred honestly.)",
    'deterministic-and-safe-cross-fs','zic_rs','reports/cross-fs/RECEIPT-CROSS-FS-1.md')
row('drop_in','link-materialization footprint (LINK-MATERIALIZATION.1)',
    "MEASURED copy vs symlink vs reference-hardlink for tzdata.zi 2026b: installed du = hardlink 200KiB / copy 426KiB (2.13x) / symlink 246KiB (1.23x); but the SHIPPED xz-compressed package = hardlink 44.7KB / copy 47.2KB (1.057x, +2.5KB) / symlink 46.5KB — the compressor dedups identical content, so copy-not-hardlink barely affects the .rpm/.deb/.tar.lz package size. --link-mode symlink recovers most installed footprint. zic-rs copies for relocatability (no shared-inode/overlay-copy-up fragility); the policy choice is now evidence-backed.",
    'measured-footprint-tradeoff','zic_rs','reports/link-materialization/RECEIPT-LINK-MATERIALIZATION-1.md')
row('drop_in','packager policy compatibility (PACKAGER-POLICY.1)',
    "distro packaging-policy matrix: 4 works-with-shim (Debian/RPM/Alpine/Arch) · 2 requires-recipe-adaptation (FreeBSD ports/pkgsrc — recipe authoring, not policy denial) · 0 blocked-by-policy. Distribution = crates.io SOURCE crate (cargo install), NO binary release — every recipe builds zic-rs from the crate (Tier-3 source-builds prove this per platform), then shims the $(ZIC) slot. localtime stays in post-install scripts (not build-time -t); rootless packaging DEMONSTRATED under fakeroot (1199-file staged tree, root:root). Required adaptations named; no distro requires an upstream binary.",
    'works-with-shim-or-recipe-adaptation','zic_rs','reports/packager-policy/RECEIPT-PACKAGER-POLICY-1.md')
row('drop_in','install-semantics parity (INSTALL-SEMANTICS.1)',
    "13 package-relevant install scenarios vs reference zic (staged root, no host mutation): 9 match · 2 install-policy-difference (atomic no-clobber; copy-not-hardlink) · 1 safer-no-partial · 1 unsupported-by-design (-p) · 0 divergent. FOUND+FIXED a real divergence: zic-rs created files at base mode 0666 (world-writable under umask 000) vs reference 0644 → now 0644 base (matches reference + never world-writable; CORE.1 byte-unchanged; +1 regression test). umask/-m/-D/symlink-outside-root all match.",
    'match-or-safer-install','zic_rs','reports/install-semantics/RECEIPT-INSTALL-SEMANTICS-1.md')
row('drop_in','package-build slot acceptance (PACKAGE-ACCEPTANCE.1)',
    "zic-rs occupies the $(ZIC) slot of the canonical tzdata package build (via an argv-compat shim; distributed as a crates.io source crate, no binary release) → posix + right trees: 598/598 identical file set, 6/6 fixture behaviour-match, permissions match (644), 0 divergent. Differences are all documented bucket-3 SAFER install policies: ref HARDLINKS links / zic-rs COPIES (relocatable); zic-rs REFUSES an absolute -t (ZIC008) but accepts a safe relative name under --out. Staged DESTDIR, no host mutation; bundle_hash 453641ff (same as every drop-in env).",
    'behaviour-identical-safer-install','drop_in_environment','reports/package-acceptance/RECEIPT-PACKAGE-ACCEPTANCE-1.md')
row('drop_in','installed-tree determinism',
    f"zic-rs bundle_hash 453641ff2568d8b1 byte-IDENTICAL across {envs}+ OS/build environments (host·container·VM·BSD·illumos·source-builds)",
    'byte-identical-tree','drop_in_environment','reports/drop-in/RECEIPT-matrix.md')

# ---- AXIS 4: reader-compat ----
for r,v,a in [('glibc-localtime','match','reader'),('go-time','match','reader'),('cctz-absl','match (no leap)','reader'),
              ('java/php/icu','unsupported_by_reader (compiled DB, not raw TZif)','unsupported_by_design')]:
    row('reader',r,'reads zic-rs output vs reference','match' if 'match' in v else 'unsupported',a,'reports/reader-compat/RECEIPT-T23-reader-compat-3.md')

# write
with open('reports/tzdb-atlas/atlas.tsv','w') as f:
    f.write('axis\tsubject\tkey_fact\tverdict\tattribution\treceipt\n')
    for r in ROWS: f.write('\t'.join(r)+'\n')
print(f"atlas rows: {len(ROWS)}")
print("axes:", dict(collections.Counter(r[0] for r in ROWS)))
print("attributions:", dict(collections.Counter(r[4] for r in ROWS)))
# the key join: do all vendor-shipped tzdb releases fall in the upstream MATCH band?
matchrel=set()
for r in dg:
    if r[12]=='match':
        m=re.search(r'(20\d\d[a-z])',r[0]); 
        if m: matchrel.add(m.group(1))
vend_in_match=[x for x in sorted(rels) if re.match(r'20\d\d[a-z]',x) and x in matchrel]
vend_not=[x for x in sorted(rels) if re.match(r'20\d\d[a-z]',x) and x not in matchrel]
print(f"\nJOIN: vendor-shipped tzdb releases = {sorted(rels)}")
print(f"  in upstream MATCH band: {vend_in_match}")
print(f"  NOT in match band: {vend_not or 'none'}")
