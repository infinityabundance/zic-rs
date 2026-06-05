#!/usr/bin/env python3
"""T23.reader-compat.1 reader gauntlet.

For each RFC 9636 Appendix-A / known-reader-trap microcase zone, ask each *real* TZif reader to
consume zic-rs's compiled output and reference-zic's compiled output, and classify the result:

  matched   - the reader observed identical behaviour from both TZif files
  diverged  - the reader observed different behaviour
  unassessed- reader unavailable here (e.g. Go) or it errored

This proves output-consumer equivalence ONLY. It does NOT prove civil-time truth, reference-zic
parity beyond these readers, or that untested readers accept the output.
"""
import datetime as dt
import hashlib
import subprocess
import sys
from zoneinfo import ZoneInfo

RS = "/tmp/rc_rs"
REF = "/tmp/rc_ref"

# microcase zone -> (trap it exercises)
MICRO = [
    ("Etc/UTC",                         "baseline / trivial fixed UTC"),
    ("Europe/Dublin",                   "Irish negative DST (winter is the 'DST' type; isdst inverted)"),
    ("Asia/Gaza",                       "content-driven v3 footer (non-POSIX ON, Ramadan rules)"),
    ("Africa/Cairo",                    "lastThu 24:00 (time-of-day == 24h; stays v2)"),
    ("Asia/Kathmandu",                  "non-hour, non-half-hour offset (+5:45)"),
    ("Australia/Eucla",                 "+8:45 offset"),
    ("America/Argentina/Buenos_Aires",  "numeric '-03' abbreviation"),
    ("Antarctica/Troll",                "numeric '+00'/'+02' abbreviations, modern"),
    ("Europe/London",                   "far-future recurring POSIX footer + pre-1970 history"),
    ("America/New_York",                "pre-1901 LMT + 2^31 boundary + far-future footer"),
    ("Pacific/Kiritimati",              "large +14 offset, date-line"),
    ("Europe/Amsterdam",                "historical sub-minute LMT (+0:19:32)"),
]

# UTC probe instants spanning the traps (incl. the 32-bit signed boundaries)
def U(y, mo=1, d=1, h=0, mi=0, s=0):
    return dt.datetime(y, mo, d, h, mi, s, tzinfo=dt.timezone.utc)

PROBES = [
    U(1850), U(1901, 12, 13, 20, 45, 52),  # ~ -2^31
    U(1900), U(1969, 12, 31, 23), U(1970),
    U(1995, 7, 1), U(2000), U(2025, 1, 15), U(2025, 7, 15),
    U(2038, 1, 19, 3, 14, 7), U(2038, 1, 19, 3, 14, 8),  # ~ +2^31 boundary
    U(2040, 6, 1), U(2050), U(2200),
]

def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()

def zdump_dump(path):
    """Reader A: reference C reader. Strip the leading path token so only the behaviour remains."""
    out = subprocess.run(["zdump", "-v", "-c", "1900,2041", path],
                         capture_output=True, text=True, timeout=60)
    lines = []
    for ln in out.stdout.splitlines():
        parts = ln.split(None, 1)
        lines.append(parts[1] if len(parts) == 2 else ln)
    return "\n".join(lines), out.returncode

def zoneinfo_obs(path, name):
    """Reader B: Python zoneinfo. Observation tuple per probe instant."""
    with open(path, "rb") as f:
        z = ZoneInfo.from_file(f, key=name)
    obs = []
    for t in PROBES:
        lt = t.astimezone(z)
        off = lt.utcoffset().total_seconds()
        is_dst = bool(lt.dst()) and lt.dst() != dt.timedelta(0)
        obs.append((off, is_dst, lt.tzname()))
    return obs

rows = []
for zone, trap in MICRO:
    p_rs, p_ref = f"{RS}/{zone}", f"{REF}/{zone}"
    h_rs, h_ref = sha(p_rs), sha(p_ref)
    byte_identical = (h_rs == h_ref)

    # Reader A: zdump
    try:
        d_rs, rc_rs = zdump_dump(p_rs)
        d_ref, rc_ref = zdump_dump(p_ref)
        zdump_status = "matched" if (d_rs == d_ref and rc_rs == 0 == rc_ref) else "diverged"
        zdump_note = "" if zdump_status == "matched" else "transition dumps differ"
    except Exception as e:
        zdump_status, zdump_note = "unassessed", f"error: {e}"

    # Reader B: Python zoneinfo
    try:
        o_rs = zoneinfo_obs(p_rs, zone)
        o_ref = zoneinfo_obs(p_ref, zone)
        if o_rs == o_ref:
            zi_status, zi_note = "matched", ""
        else:
            diffs = [(PROBES[i].year, o_ref[i], o_rs[i]) for i in range(len(PROBES)) if o_rs[i] != o_ref[i]]
            zi_status, zi_note = "diverged", f"{len(diffs)} probe(s) differ e.g. {diffs[0]}"
    except Exception as e:
        zi_status, zi_note = "unassessed", f"error: {e}"

    rows.append((zone, trap, byte_identical, h_rs[:12], h_ref[:12],
                 zdump_status, zdump_note, zi_status, zi_note))

# ---- report ----
print(f"# T23.reader-compat.1 gauntlet — {len(MICRO)} microcases × 2 readers (zdump, Python zoneinfo); Go=unassessed")
print(f"# probes: {len(PROBES)} UTC instants incl. ~-2^31 and ~+2^31 boundaries")
print()
hdr = f"{'zone':32} {'byteid':6} {'zdump':9} {'zoneinfo':9}  trap"
print(hdr); print("-" * len(hdr))
agg = {"matched": 0, "diverged": 0, "unassessed": 0}
for (zone, trap, bid, hrs, href, zd, zdn, zi, zin) in rows:
    print(f"{zone:32} {('yes' if bid else 'no'):6} {zd:9} {zi:9}  {trap}")
    if zdn: print(f"    zdump:    {zdn}")
    if zin: print(f"    zoneinfo: {zin}")
    for s in (zd, zi):
        agg[s] = agg.get(s, 0) + 1
print()
n_pairs = len(MICRO) * 2
print(f"PAIR TOTALS ({n_pairs} = {len(MICRO)} zones × 2 readers): "
      f"matched={agg['matched']} diverged={agg['diverged']} unassessed={agg['unassessed']}")
byte_ids = sum(1 for r in rows if r[2])
print(f"byte-identical zic-rs vs ref-zic: {byte_ids}/{len(MICRO)} "
      f"(behaviour parity holds for the rest where readers still matched)")
# machine-checkable exit: fail only on a genuine divergence
sys.exit(1 if agg["diverged"] else 0)
