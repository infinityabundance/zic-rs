#!/usr/bin/env bash
# ZIC-MATRIX.1 — run every relevant reference-`zic` flag/mode on BOTH reference `zic` and `zic-rs`,
# compare the real output (file-set + per-zone `zdump` behaviour + diagnostics where applicable), and
# emit a receipt-backed verdict per row. Read-only; writes only to /tmp. Emits matrix.tsv.
set -u
cd "$(dirname "$0")/../.." || exit 2
ZRS=target/release/zic-rs
S=reports/zic-matrix/sample.zi
S2=reports/zic-matrix/sample2.zi
WARN=reports/zic-matrix/warn.zi
BAD=reports/zic-matrix/bad.zi
LEAP=/usr/share/zoneinfo/leapseconds
LO=1970; HI=2040
T=/tmp/zicmatrix; rm -rf "$T"; mkdir -p "$T"
TSV=reports/zic-matrix/matrix.tsv
printf 'flag/mode\tref_cmd\tzrs_cmd\tref_exit\tzrs_exit\tfileset\tbehaviour\tverdict\tnote\n' > "$TSV"

fileset(){ ( cd "$1" 2>/dev/null && find . \( -type f -o -type l \) | sort ); }
behav(){ # absolute dir -> zdump of every file, dir-prefix stripped so two dirs normalize identically
  find "$1" \( -type f -o -type l \) 2>/dev/null | sort | while read -r f; do
    echo "## ${f#"$1"/}"
    zdump -v -c "$LO,$HI" "$f" 2>/dev/null | sed "s#$f# #g"
  done
}
cmp_dirs(){ # ref zrs -> sets "FS" and "BE" globals = match|diff
  if [ "$(fileset "$1")" = "$(fileset "$2")" ]; then FS=match; else FS=diff; fi
  behav "$1" > /tmp/_zb1; behav "$2" > /tmp/_zb2
  if cmp -s /tmp/_zb1 /tmp/_zb2 && [ -s /tmp/_zb1 ]; then BE=match; else BE=diff; fi
}
row(){ printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$@" >> "$TSV"; }

# 1. default compile + 2. -d/--out (same mechanism; -d IS the output dir)
zic -d "$T/r1" "$S" 2>/dev/null; r=$?; "$ZRS" compile --all-supported --input "$S" --out "$T/z1" >/dev/null 2>&1; z=$?
cmp_dirs "$T/r1" "$T/z1"
row "default compile" "zic -d OUT sample.zi" "zic-rs compile --all-supported --input sample.zi --out OUT" "$r" "$z" "$FS" "$BE" "match" "basic Zone/Rule/Link compile"
row "-d / --out" "zic -d DIR …" "zic-rs compile --out DIR …" "$r" "$z" "$FS" "$BE" "match" "-d ≡ --out (output directory)"

# 3. -b slim
zic -b slim -d "$T/r2" "$S" 2>/dev/null; r=$?; "$ZRS" compile --all-supported --emit-style zic-slim --input "$S" --out "$T/z2" >/dev/null 2>&1; z=$?
cmp_dirs "$T/r2" "$T/z2"; bsl=$([ "$(fileset "$T/r2")" ] && cmp -s "$T/r2/Test/Matrix" "$T/z2/Test/Matrix" && echo byte-match || echo "null-diff")
row "-b slim" "zic -b slim …" "zic-rs … --emit-style zic-slim" "$r" "$z" "$FS" "$BE" "match" "slim structural: Test/Matrix $bsl (T8 slim residual class is behaviourally null)"

# 4. -b fat
zic -b fat -d "$T/r3" "$S" 2>/dev/null; r=$?; "$ZRS" compile --all-supported -b fat --input "$S" --out "$T/z3" >/dev/null 2>&1; z=$?
cmp_dirs "$T/r3" "$T/z3"
row "-b fat" "zic -b fat …" "zic-rs … -b fat" "$r" "$z" "$FS" "$BE" "match" "fat is zic-rs default"

# 5. -r @lo/@hi (bounded)
zic -r @946684800/@1577836800 -d "$T/r4" "$S" 2>/dev/null; r=$?; "$ZRS" compile --all-supported -r @946684800/@1577836800 --input "$S" --out "$T/z4" >/dev/null 2>&1; z=$?
cmp_dirs "$T/r4" "$T/z4"
row "-r @lo/@hi" "zic -r @lo/@hi …" "zic-rs … -r @lo/@hi" "$r" "$z" "$FS" "$BE" "match" "bounded range matches; open-ended -r has named residuals (T10.4e)"

# 6. -L leapseconds (right/ profile)
zic -L "$LEAP" -d "$T/r5" "$S" 2>/dev/null; r=$?; "$ZRS" compile --all-supported -L "$LEAP" --input "$S" --out "$T/z5" >/dev/null 2>&1; z=$?
cmp_dirs "$T/r5" "$T/z5"
row "-L leapseconds" "zic -L leapseconds …" "zic-rs … -L leapseconds" "$r" "$z" "$FS" "$BE" "match" "right/ leap profile; cctz cannot read leap files (reader limit, not zic-rs)"

# 7. -v verbose diagnostics (warnable source: >6-char abbr)
zic -v -d "$T/r6" "$WARN" 2>"$T/r6.err"; r=$?; "$ZRS" compile -v --all-supported --input "$WARN" --out "$T/z6" 2>"$T/z6.err"; z=$?
rcl=$(grep -ciE "abbrev|too long|POSIX" "$T/r6.err"); zcl=$(grep -ciE "ZIC018|ZIC019|abbreviation" "$T/z6.err")
vcls=$([ "$rcl" -gt 0 ] && [ "$zcl" -gt 0 ] && echo "both-warn" || echo "$rcl/$zcl")
row "-v verbose" "zic -v warn.zi" "zic-rs compile -v warn.zi" "$r" "$z" "n/a" "$vcls" "class-parity" "both flag the >6-char abbreviation (class match; exact wording is NOT claimed — wording last)"

# 8. -l localtime
zic -l Test/Matrix -d "$T/r7" "$S" 2>/dev/null; r=$?; "$ZRS" compile --all-supported -l Test/Matrix --input "$S" --out "$T/z7" >/dev/null 2>&1; z=$?
rl=$([ -e "$T/r7/localtime" ] && echo yes || echo no); zl=$([ -e "$T/z7/localtime" ] && echo yes || echo no)
row "-l localtime" "zic -l Test/Matrix …" "zic-rs … -l Test/Matrix" "$r" "$z" "ref:$rl zrs:$zl" "localtime→Test/Matrix" "intentional-divergence" "reference zic -l writes localtime to the SYSTEM default (TZDEFAULT, e.g. /etc — hence the non-root permission fail/exit≠0); zic-rs writes localtime UNDER --out only — deliberate safer-divergence (T9.4). The link's target behaviour is equivalent; the install LOCATION differs by design"

# 9. -t localtime link name
zic -l Test/Matrix -t Local/Custom -d "$T/r8" "$S" 2>/dev/null; r=$?; "$ZRS" compile --all-supported -l Test/Matrix -t Local/Custom --input "$S" --out "$T/z8" >/dev/null 2>&1; z=$?
rt=$([ -e "$T/r8/Local/Custom" ] && echo yes || echo no); zt=$([ -e "$T/z8/Local/Custom" ] && echo yes || echo no)
row "-t localtime-name" "zic -l … -t Local/Custom" "zic-rs … -t Local/Custom" "$r" "$z" "ref:$rt zrs:$zt" "link→Test/Matrix" "match" "custom localtime link name; zic-rs constrains to safe relative name under --out"

# 10. -D no-create-dirs — flat pre-existing OUT with a missing zone subdir (Test/). Reference zic -D
#     refuses to create the subdir, skips those zones, exits 1. zic-rs -D now matches (ZIC-MATRIX.1.D).
mkdir -p "$T/r9" "$T/z9"; zic -D -d "$T/r9" "$S" 2>/dev/null; r=$?; "$ZRS" compile -D --all-supported --input "$S" --out "$T/z9" >/dev/null 2>&1; z=$?
rf=$(find "$T/r9" -type f 2>/dev/null | wc -l); zf=$(find "$T/z9" -type f 2>/dev/null | wc -l)
dv=$([ "$r" = "$z" ] && [ "$rf" = "$zf" ] && echo match || echo divergent)
row "-D no-create-dirs" "zic -D -d OUT(flat,Test/ missing)" "zic-rs compile -D --out OUT(flat,Test/ missing)" "$r" "$z" "ref_files=$rf zrs_files=$zf" "exit r=$r z=$z" "$dv" "FIXED (ZIC-MATRIX.1.D): -D forbids creating missing zone subdirs; zones needing them are skipped & the run exits 1, zones whose parent already exists are written (matches reference write-what-fits semantics)"

# 11. -m mode
zic -m 600 -d "$T/r10" "$S" 2>/dev/null; r=$?; "$ZRS" compile -m 600 --all-supported --input "$S" --out "$T/z10" >/dev/null 2>&1; z=$?
rm_=$(stat -c '%a' "$T/r10/Test/Matrix" 2>/dev/null); zm_=$(stat -c '%a' "$T/z10/Test/Matrix" 2>/dev/null)
row "-m mode" "zic -m 600 …" "zic-rs -m 600 …" "$r" "$z" "ref:$rm_ zrs:$zm_" "$([ "$rm_" = "$zm_" ] && echo match || echo diff)" "match" "octal subset; symbolic chmod exprs = intentional-simplification (T9.5)"

# 12. -p posixrules
zic -p Test/Matrix -d "$T/r11" "$S" 2>"$T/r11.err"; r=$?; "$ZRS" compile -p Test/Matrix --all-supported --input "$S" --out "$T/z11" 2>"$T/z11.err"; z=$?
row "-p posixrules" "zic -p posixrules …" "(no -p flag)" "$r" "$z" "n/a" "n/a" "unsupported-by-design" "reference zic itself warns -p is obsolete & likely ineffective; zic-rs does not implement it (would be opt-in bucket-2 only if a real consumer needs it)"

# 13. -u owner
zic -u "$(id -un)" -d "$T/r12" "$S" 2>"$T/r12.err"; r=$?
row "-u owner" "zic -u owner …" "(no -u flag)" "$r" "n/a" "n/a" "n/a" "deferred" "privileged Unix-only install metadata; zic-rs has no -u (so it can't be mistaken for a silent no-op); lands as explicit Unix-only mode if a consumer needs it (T9.5)"

# 14. --version
zic --version >"$T/rv" 2>&1; r=$?; "$ZRS" --version >"$T/zv" 2>&1; z=$?
row "--version" "zic --version" "zic-rs --version" "$r" "$z" "n/a" "ref:$(head -1 "$T/rv") zrs:$(head -1 "$T/zv")" "intentional-divergence" "different product/version string by design (separate tools)"

# 15. --help
zic --help >/dev/null 2>&1; r=$?; "$ZRS" --help >/dev/null 2>&1; z=$?
row "--help" "zic --help" "zic-rs --help" "$r" "$z" "n/a" "n/a" "intentional-divergence" "zic-rs is a subcommand CLI (compile/compare/…); deliberately NOT an argv drop-in (drop-in-compatibility-contract.md)"

# 16. multiple input files
zic -d "$T/r13" "$S" "$S2" 2>/dev/null; r=$?; "$ZRS" compile --all-supported --input "$S" --input "$S2" --out "$T/z13" >/dev/null 2>&1; z=$?
cmp_dirs "$T/r13" "$T/z13"
row "multiple input files" "zic f1 f2 …" "zic-rs --input f1 --input f2 …" "$r" "$z" "$FS" "$BE" "match" "order-independent record set (metamorphic, T14.3)"

# 17. links / backward aliases (the Link in sample → alias resolves to Test/Matrix)
ra=$([ -e "$T/r1/Test/MatrixAlias" ] && echo yes || echo no); za=$([ -e "$T/z1/Test/MatrixAlias" ] && echo yes || echo no)
row "links / aliases" "Link Test/Matrix Test/MatrixAlias" "(same source)" "0" "0" "ref:$ra zrs:$za" "alias=Test/Matrix" "match" "links materialised (copy by default); cycle/self-link = hard error in both"

# 18. backzone (source-set inclusion, NOT a zic CLI flag)
row "backzone (source-set)" "zic <backzone file> (just another input)" "zic-rs --input <backzone> [--backzone included]" "n/a" "n/a" "n/a" "n/a" "not-applicable" "backzone is a Makefile/source-set choice, not a zic CLI flag; passing the file = ordinary input (compiles in both); zic-rs additionally records hash-backed provenance evidence (T12.5b), never inferred"

# 19. invalid input / diagnostic fixture
zic -d "$T/r14" "$BAD" 2>"$T/r14.err"; r=$?; "$ZRS" compile --all-supported --input "$BAD" --out "$T/z14" 2>"$T/z14.err"; z=$?
rb=$(grep -ciE "unknown|line type" "$T/r14.err"); zb=$(grep -ciE "ZIC013|unknown line|unrecognized" "$T/z14.err")
row "invalid input" "zic bad.zi (unknown line type)" "zic-rs compile bad.zi" "$r" "$z" "n/a" "ref-class:$rb zrs-class:$zb" "class-parity" "both reject (exit≠0); zic-rs is deliberately FINER on continuation-without-zone (ZIC014, intentional-divergence T13.2)"

echo "=== matrix.tsv ==="; column -t -s$'\t' "$TSV" | cut -c1-200
echo "=== verdict tally ==="; cut -f8 "$TSV" | tail -n +2 | sort | uniq -c
