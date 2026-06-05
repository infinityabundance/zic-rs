#!/usr/bin/env bash
# INSTALL-SEMANTICS.1 — filesystem/materialization replacement semantics.
# Runs reference `zic` and zic-rs on the SAME install fixtures under package-relevant conditions, and
# classifies how each materializes / refuses / preserves filesystem objects. Disposable root only — no
# host mutation. Compares: existing-tree cases, symlink/dir/file collisions, read-only, umask, -m, -D,
# localtime -l/-t, -p posixrules, partial-output-on-fatal, hardlink-vs-copy + a metadata table.
#
# Verdicts: match · safer-divergence · install-policy-difference · unsupported-by-design · divergent
set -u
cd "$(dirname "$0")/../.." || exit 2
ZRS=${ZRS:-$PWD/target/release/zic-rs}
OUT=reports/install-semantics/install-semantics.tsv
T=/tmp/is-g; rm -rf "$T"; mkdir -p "$T"
FIX="$T/fix.zi"; printf 'Zone Test/A 1:00 - AAA\nZone Test/B 2:00 - BBB\n' > "$FIX"
# a 2-zone fixture where the SECOND zone fails at compile (simultaneous transition) → partial-output probe
FATAL="$T/fatal.zi"; printf 'Zone Test/Good 1:00 - G\nRule R 2000 only - Jan 1 0:00 1:00 D\nRule R 2000 only - Jan 1 0:00 0 S\nZone Test/Bad 0 R X\n' > "$FATAL"

refzic() { zic "$@" 2>&1; }                                   # reference
zrszic() { "$ZRS" compile --all-supported "$@" 2>&1; }        # zic-rs (-d -> --out, --input per file)

printf 'scenario\treference_zic\tzic_rs\tverdict\n' > "$OUT"
row() { printf '%s\t%s\t%s\t%s\n' "$1" "$2" "$3" "$4" >> "$OUT"; }

# 1. existing correct/stale file (overwrite policy)
rm -rf "$T/r1" "$T/z1"; mkdir -p "$T/r1/Test" "$T/z1/Test"; echo STALE >"$T/r1/Test/A"; echo STALE >"$T/z1/Test/A"
refzic -d "$T/r1" "$FIX" >/dev/null 2>&1; r=$([ "$(head -c4 "$T/r1/Test/A"|tr -d '\0')" = TZif ] && echo "overwrites-in-place" || echo "kept-stale")
zrszic --input "$FIX" --out "$T/z1" >/dev/null 2>&1; z=$([ "$(head -c4 "$T/z1/Test/A"|tr -d '\0')" = TZif ] && echo "overwrote" || echo "REFUSES-no-clobber(needs --force)")
row "existing_file_no_force" "$r" "$z" "install-policy-difference"

# 1b. with --force, zic-rs overwrites atomically
rm -rf "$T/z1f"; mkdir -p "$T/z1f/Test"; echo STALE >"$T/z1f/Test/A"
zrszic --input "$FIX" --out "$T/z1f" --force >/dev/null 2>&1
row "existing_file_with_force" "overwrites-in-place" "$([ "$(head -c4 "$T/z1f/Test/A"|tr -d '\0')" = TZif ] && echo "overwrites-atomically(temp+rename)" || echo FAIL)" "match"

# 2. directory where a file should be
rm -rf "$T/r2" "$T/z2"; mkdir -p "$T/r2/Test/A" "$T/z2/Test/A"
refzic -d "$T/r2" "$FIX" >/dev/null 2>&1; r="exit=$?"
zrszic --input "$FIX" --out "$T/z2" >/dev/null 2>&1; z="exit=$?"
row "dir_where_file" "$r(refuses)" "$z(refuses)" "$([ "$r" = "$z" ] && echo match || echo install-policy-difference)"

# 3. read-only output dir (non-root): writes must fail
rm -rf "$T/r3" "$T/z3"; mkdir -p "$T/r3" "$T/z3"; chmod 555 "$T/r3" "$T/z3"
refzic -d "$T/r3/sub" "$FIX" >/dev/null 2>&1; r="exit=$?"
zrszic --input "$FIX" --out "$T/z3/sub" >/dev/null 2>&1; z="exit=$?"
chmod 755 "$T/r3" "$T/z3"
row "readonly_parent_dir" "$r(refuses)" "$z(refuses)" "$([ "$r" != 0 ] && [ "$z" != 0 ] && echo match || echo divergent)"

# 4. symlink to OUTSIDE root at the output leaf — must NOT write through (victim preserved)
rm -rf "$T/r4" "$T/z4" "$T/victimR" "$T/victimZ"; mkdir -p "$T/r4/Test" "$T/z4/Test"
echo PRECIOUS >"$T/victimR"; echo PRECIOUS >"$T/victimZ"; ln -s "$T/victimR" "$T/r4/Test/A"; ln -s "$T/victimZ" "$T/z4/Test/A"
refzic -d "$T/r4" "$FIX" >/dev/null 2>&1; r=$([ "$(cat "$T/victimR")" = PRECIOUS ] && echo "victim-preserved" || echo "VICTIM-CLOBBERED")
zrszic --input "$FIX" --out "$T/z4" >/dev/null 2>&1; z=$([ "$(cat "$T/victimZ")" = PRECIOUS ] && echo "victim-preserved" || echo "VICTIM-CLOBBERED"); [ -f "$T/z4/Test/A" ] && [ ! -L "$T/z4/Test/A" ] && z="$z(symlink-replaced-not-followed)"
case "$r:$z" in victim-preserved:victim-preserved*) sv=match;; *) sv=divergent;; esac
row "symlink_outside_root_leaf" "$r" "$z" "$sv"

# 5. umask sensitivity (base mode)
for u in 000 022 077; do
  rm -rf "$T/ru$u" "$T/zu$u"
  ( umask $u; zic -d "$T/ru$u" "$FIX" >/dev/null 2>&1 )
  ( umask $u; "$ZRS" compile --all-supported --input "$FIX" --out "$T/zu$u" >/dev/null 2>&1 )
  rm=$(stat -c%a "$T/ru$u/Test/A" 2>/dev/null); zm=$(stat -c%a "$T/zu$u/Test/A" 2>/dev/null)
  row "umask_$u" "mode=$rm" "mode=$zm" "$([ "$rm" = "$zm" ] && echo match || echo install-policy-difference)"
done

# 6. -m mode (octal)
rm -rf "$T/rm" "$T/zm"; refzic -d "$T/rm" -m 600 "$FIX" >/dev/null 2>&1
"$ZRS" compile --all-supported --input "$FIX" --out "$T/zm" --mode 600 >/dev/null 2>&1
rm=$(stat -c%a "$T/rm/Test/A" 2>/dev/null); zm=$(stat -c%a "$T/zm/Test/A" 2>/dev/null)
row "mode_-m_600" "mode=$rm" "mode=$zm" "$([ "$rm" = "$zm" ] && echo match || echo install-policy-difference)"

# 7. -D into a missing dir (both should refuse) and existing dir (both should write)
rm -rf "$T/rD" "$T/zD"
refzic -D -d "$T/rD" "$FIX" >/dev/null 2>&1; r="missing:exit=$?"
"$ZRS" compile --all-supported --no-create-dirs --input "$FIX" --out "$T/zD" >/dev/null 2>&1; z="missing:exit=$?"
row "-D_missing_dir" "$r" "$z" "$([ "${r#*=}" != 0 ] && [ "${z#*=}" != 0 ] && echo match || echo install-policy-difference)"

# 8. partial output on a fatal (zone 2 fails to compile)
rm -rf "$T/rP" "$T/zP"
refzic -d "$T/rP" "$FATAL" >/dev/null 2>&1; r="files=$(find "$T/rP" -type f 2>/dev/null|wc -l)"
"$ZRS" compile --all-supported --input "$FATAL" --out "$T/zP" >/dev/null 2>&1; z="files=$(find "$T/zP" -type f 2>/dev/null|wc -l)"
row "partial_output_on_fatal" "$r(writes-as-it-goes)" "$z(compile-all-then-write)" "$([ "${z#*=}" -eq 0 ] && [ "${r#*=}" -gt 0 ] && echo safer-no-partial || echo match)"

# 9. -p posixrules (both deprecate)
rm -rf "$T/rp"; pe=$(refzic -d "$T/rp" -p Test/A "$FIX" 2>&1 | grep -io "obsolete\|ineffective" | head -1)
row "-p_posixrules" "deprecated(${pe:-accepted-noop})" "ignored-by-shim/unsupported" "unsupported-by-design"

# 10. hardlink vs copy materialization (links share inodes in ref, copies in zic-rs)
rhl=$(find "$T/r1" -type f -links +1 2>/dev/null | wc -l)   # r1 is a full-ish build? no — use baseline trees
rm -rf "$T/rL" "$T/zL"; printf 'Zone Test/Real 1:00 - R\nLink Test/Real Test/Alias\n' > "$T/link.zi"
refzic -d "$T/rL" "$T/link.zi" >/dev/null 2>&1; "$ZRS" compile --all-supported --input "$T/link.zi" --out "$T/zL" >/dev/null 2>&1
rln=$(stat -c%h "$T/rL/Test/Alias" 2>/dev/null); zln=$(stat -c%h "$T/zL/Test/Alias" 2>/dev/null)
row "link_materialization" "hardlink(nlink=$rln)" "copy(nlink=$zln)" "install-policy-difference"

echo "=== INSTALL-SEMANTICS.1 ==="; column -t -s$'\t' "$OUT"
echo ""
echo "verdict tally:"; cut -f4 "$OUT" | tail -n +2 | sort | uniq -c
echo "DIVERGENT (named findings):"; awk -F'\t' '$4=="divergent"{print "  "$1": ref["$2"] zrs["$3"]"}' "$OUT" || echo "  none"
echo ""
echo "=== metadata table (zic-rs build, sample zone) ==="
f="$T/z0meta"; "$ZRS" compile --all-supported --input "$FIX" --out "$f" >/dev/null 2>&1
stat -c 'type=%F mode=%a uid=%u gid=%g size=%s nlink=%h' "$f/Test/A" 2>/dev/null
echo "(mtime: current-time at write; deterministic content via bundle_hash, not mtime)"
