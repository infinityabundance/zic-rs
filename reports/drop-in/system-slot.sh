#!/bin/bash
# T23.drop-in-gauntlet.system-slot.1 — run zic-rs AND reference zic through a system-install-shaped matrix
# and capture the REAL outcome of each row (exit codes + resulting tree state) for honest classification.
# Observational: it prints evidence; the receipt classifies match / accepted-divergence / not-claimed.
set +e
RS="$1"; ZIC="${2:-zic}"; ZDUMP="${3:-zdump}"; SRC="${4:-/usr/share/zoneinfo/tzdata.zi}"; LEAP="${5:-/usr/share/zoneinfo/leapseconds}"
W=$(mktemp -d); trap 'rm -rf "$W"' EXIT
hr(){ echo "----------------------------------------------------------------"; }
echo "## host: $(uname -sm)  zic=$($ZIC --version 2>&1|head -1)  rs=$($RS --version 2>&1|head -1)"
echo "## src=$(sha256sum "$SRC"|cut -c1-16)  leap=$(sha256sum "$LEAP"|cut -c1-16)"

hr; echo "ROW A — default posix/ profile (the behaviour contract)"
$ZIC -d "$W/A_ref" "$SRC" 2>"$W/A_ref.err"; echo "  ref_exit=$?  files=$(find "$W/A_ref" -type f -o -type l 2>/dev/null|wc -l)"
$RS compile --input "$SRC" --out "$W/A_rs" --all-supported >/dev/null 2>"$W/A_rs.err"; echo "  rs_exit=$?  files=$(find "$W/A_rs" -type f -o -type l 2>/dev/null|wc -l)"

hr; echo "ROW B — localtime install (-l America/New_York)"
$ZIC -d "$W/B_ref" -l America/New_York "$SRC" 2>/dev/null; echo "  ref_exit=$?  localtime? $([ -e "$W/B_ref/localtime" ] && echo yes || echo no)"
$RS compile --input "$SRC" --out "$W/B_rs" --all-supported -l America/New_York >/dev/null 2>/dev/null; echo "  rs_exit=$?  localtime? $([ -e "$W/B_rs/localtime" ] && echo yes || echo no)"
echo "  zdump-localtime-equal? $([ -e "$W/B_ref/localtime" ] && [ -e "$W/B_rs/localtime" ] && { diff <($ZDUMP -v -c 1970,2030 "$W/B_ref/localtime" 2>/dev/null|sed 's#.*localtime##') <($ZDUMP -v -c 1970,2030 "$W/B_rs/localtime" 2>/dev/null|sed 's#.*localtime##') >/dev/null 2>&1 && echo yes || echo no; } || echo n/a)"

hr; echo "ROW C — posixrules (-p America/New_York)"
$ZIC -d "$W/C_ref" -p America/New_York "$SRC" 2>"$W/C_ref.err"; echo "  ref_exit=$?  ref_msg=$(grep -i posix "$W/C_ref.err"|head -1)"
echo "  rs: no -p flag (deliberate non-capability)"

hr; echo "ROW D — right/ leap profile (-L leapseconds), zdump America/New_York"
$ZIC -d "$W/D_ref" -L "$LEAP" "$SRC" 2>/dev/null; rexit=$?
$RS compile --input "$SRC" --out "$W/D_rs" --all-supported -L "$LEAP" >/dev/null 2>/dev/null; sexit=$?
echo "  ref_exit=$rexit  rs_exit=$sexit"
echo "  zdump-NY-equal? $(diff <($ZDUMP -v -c 1970,2030 "$W/D_ref/America/New_York" 2>/dev/null|sed 's#.*New_York##') <($ZDUMP -v -c 1970,2030 "$W/D_rs/America/New_York" 2>/dev/null|sed 's#.*New_York##') >/dev/null 2>&1 && echo yes || echo no)"

hr; echo "ROW E — existing output tree, re-run overwrite"
$ZIC -d "$W/E_ref" "$SRC" 2>/dev/null; $ZIC -d "$W/E_ref" "$SRC" 2>"$W/E_ref2.err"; echo "  ref re-run exit=$?  (default overwrite)"
$RS compile --input "$SRC" --out "$W/E_rs" --all-supported >/dev/null 2>/dev/null
$RS compile --input "$SRC" --out "$W/E_rs" --all-supported >/dev/null 2>"$W/E_rs2.err"; echo "  rs re-run (no --force) exit=$?  (no-clobber)"
$RS compile --input "$SRC" --out "$W/E_rs" --all-supported --force >/dev/null 2>/dev/null; echo "  rs re-run (--force) exit=$?"

hr; echo "ROW F — preexisting REGULAR FILE at a zone leaf"
mkdir -p "$W/F_ref/Africa" "$W/F_rs/Africa"; echo BLOCKER > "$W/F_ref/Africa/Abidjan"; echo BLOCKER > "$W/F_rs/Africa/Abidjan"
$ZIC -d "$W/F_ref" "$SRC" 2>/dev/null; echo "  ref_exit=$?  Abidjan-still-BLOCKER? $(grep -q BLOCKER "$W/F_ref/Africa/Abidjan" 2>/dev/null && echo yes || echo no)"
$RS compile --input "$SRC" --out "$W/F_rs" --all-supported >/dev/null 2>/dev/null; echo "  rs_exit(no-force)=$?  Abidjan-still-BLOCKER? $(grep -q BLOCKER "$W/F_rs/Africa/Abidjan" 2>/dev/null && echo yes || echo no)"

hr; echo "ROW G — preexisting SYMLINK at a zone leaf (hostile write-through test)"
echo "SECRET" > "$W/victim"
mkdir -p "$W/G_ref/Etc" "$W/G_rs/Etc"; ln -s "$W/victim" "$W/G_ref/Etc/UTC"; ln -s "$W/victim" "$W/G_rs/Etc/UTC"
$ZIC -d "$W/G_ref" "$SRC" 2>/dev/null; echo "  ref_exit=$?  victim-still-SECRET? $(grep -q SECRET "$W/victim" && echo yes || echo NO-WROTE-THROUGH)  Etc/UTC-is-symlink? $([ -L "$W/G_ref/Etc/UTC" ] && echo yes || echo no)"
echo "SECRET" > "$W/victim"
$RS compile --input "$SRC" --out "$W/G_rs" --all-supported --force >/dev/null 2>/dev/null; echo "  rs_exit(--force)=$?  victim-still-SECRET? $(grep -q SECRET "$W/victim" && echo yes || echo NO-WROTE-THROUGH)  Etc/UTC-is-symlink? $([ -L "$W/G_rs/Etc/UTC" ] && echo yes || echo no)"

hr; echo "ROW H — permission denied destination"
mkdir -p "$W/H_ro"; chmod 555 "$W/H_ro"
$ZIC -d "$W/H_ro/sub" "$SRC" 2>/dev/null; echo "  ref_exit=$?  wrote? $([ -d "$W/H_ro/sub" ] && echo yes || echo no)"
$RS compile --input "$SRC" --out "$W/H_ro/sub" --all-supported >/dev/null 2>/dev/null; echo "  rs_exit=$?  wrote? $([ -d "$W/H_ro/sub" ] && echo yes || echo no)"
chmod 755 "$W/H_ro"

hr; echo "ROW I — bad destination: zone name path traversal"
printf 'Zone ../escape 0:00 - EVL\n' > "$W/evil.zi"
$ZIC -d "$W/I_ref" "$W/evil.zi" 2>"$W/I_ref.err"; echo "  ref_exit=$?  escaped-file-outside? $([ -e "$W/escape" ] && echo YES-ESCAPED || echo no)  ref_msg=$(head -1 "$W/I_ref.err")"
$RS compile --input "$W/evil.zi" --out "$W/I_rs" --all-supported >/dev/null 2>"$W/I_rs.err"; echo "  rs_exit=$?  escaped? $([ -e "$W/escape" ] && echo YES-ESCAPED || echo no)  rs_msg=$(grep -oE 'ZIC[0-9]+|traversal|escape|component' "$W/I_rs.err"|head -1)"

hr; echo "ROW J — partial-failure boundary (good zone then a fatal unknown line)"
printf 'Zone Good/One 0:00 - GMT\nBOGUSLINE x y z\n' > "$W/partial.zi"
$ZIC -d "$W/J_ref" "$W/partial.zi" 2>"$W/J_ref.err"; echo "  ref_exit=$?  Good/One-written? $([ -e "$W/J_ref/Good/One" ] && echo YES-PARTIAL || echo no)"
$RS compile --input "$W/partial.zi" --out "$W/J_rs" --all-supported >/dev/null 2>"$W/J_rs.err"; echo "  rs_exit=$?  Good/One-written? $([ -e "$W/J_rs/Good/One" ] && echo YES-PARTIAL || echo no-NO-PARTIAL)"
hr; echo "## DONE"
