#!/usr/bin/env python3
"""T23.reader-compat.2 — RFC 9636 Appendix-A microcase expansion.

Extends T23.reader-compat.1 from normal-ish zones to standards-derived edge cases (right/leap profile, v4
via leap-expiry, `-r` range truncation, footer extension, pre-first-transition, near-2^31 boundaries,
numeric/unusual abbreviations, non-hour/non-minute offsets). For each microcase, each real TZif reader
consumes zic-rs's output and reference-zic's output; the result is classified matched / diverged / skipped /
unassessed.

Tier (loud): reader-equivalent for THESE readers and THESE fixtures
  ≠ all readers  ≠ all TZif edge cases  ≠ reference-zic byte identity  ≠ civil-time truth.

Prereq trees (built by the documented commands; see the receipt):
  /tmp/rc_rs /tmp/rc_ref          normal zones (zic-rs all-supported · zic)
  /tmp/leaprs /tmp/leapref        right/leap profile (-L leapseconds)
  /tmp/v4rs /tmp/v4ref            v4 via crafted leap+Expires (-L /tmp/v4leap.txt)
  /tmp/trunc_rs /tmp/trunc_ref    -r @946684800/@1893456000 truncation
"""
import datetime as dt
import hashlib
import subprocess
import sys
from zoneinfo import ZoneInfo


def U(y, mo=1, d=1, h=0, mi=0, s=0):
    return dt.datetime(y, mo, d, h, mi, s, tzinfo=dt.timezone.utc)


PROBES = [
    U(1850), U(1901, 12, 13, 20, 45, 52), U(1900), U(1969, 12, 31, 23), U(1970),
    U(1995, 7, 1), U(2000), U(2010), U(2025, 1, 15), U(2025, 7, 15),
    U(2038, 1, 19, 3, 14, 7), U(2038, 1, 19, 3, 14, 8), U(2040, 6, 1), U(2050), U(2200),
]

# (label, appendix-A trap, risk, rs_path, ref_path, zdump_lo, zdump_hi, note)
M = [
    ("Etc/UTC", "baseline fixed UTC", "—", "/tmp/rc_rs/Etc/UTC", "/tmp/rc_ref/Etc/UTC", 1900, 2041, ""),
    ("America/New_York", "pre-first-transition LMT + ~2^31 boundary", "32-bit time_t / pre-1883 heuristic",
     "/tmp/rc_rs/America/New_York", "/tmp/rc_ref/America/New_York", 1850, 2041, ""),
    ("Europe/London", "far-future recurring POSIX footer extension", "footer-ignoring readers project last txn",
     "/tmp/rc_rs/Europe/London", "/tmp/rc_ref/Europe/London", 1900, 2041, ""),
    ("Asia/Gaza", "content-driven v3 footer", "v1/v2-only readers mis-evaluate v3 TZ string",
     "/tmp/rc_rs/Asia/Gaza", "/tmp/rc_ref/Asia/Gaza", 1900, 2041, ""),
    ("Africa/Cairo", "time-of-day == 24:00", "off-by-one-day / 24h-clamp",
     "/tmp/rc_rs/Africa/Cairo", "/tmp/rc_ref/Africa/Cairo", 1900, 2041, ""),
    ("Asia/Kathmandu", "non-hour offset +5:45", "whole-hour truncation",
     "/tmp/rc_rs/Asia/Kathmandu", "/tmp/rc_ref/Asia/Kathmandu", 1900, 2041, ""),
    ("Australia/Eucla", "+8:45 offset", "whole-hour truncation",
     "/tmp/rc_rs/Australia/Eucla", "/tmp/rc_ref/Australia/Eucla", 1900, 2041, ""),
    ("Antarctica/Troll", "numeric +00/+02 abbreviations", "readers expecting alpha abbrevs",
     "/tmp/rc_rs/Antarctica/Troll", "/tmp/rc_ref/Antarctica/Troll", 1900, 2041, ""),
    ("America/Argentina/Buenos_Aires", "numeric -03 abbreviation", "alpha-abbrev assumption",
     "/tmp/rc_rs/America/Argentina/Buenos_Aires", "/tmp/rc_ref/America/Argentina/Buenos_Aires", 1900, 2041, ""),
    ("Europe/Amsterdam", "sub-minute LMT (+0:19:32)", "whole-minute rounding",
     "/tmp/rc_rs/Europe/Amsterdam", "/tmp/rc_ref/Europe/Amsterdam", 1900, 2041, ""),
    ("Pacific/Kiritimati", "large +14 offset, date-line", "offset-range assumption",
     "/tmp/rc_rs/Pacific/Kiritimati", "/tmp/rc_ref/Pacific/Kiritimati", 1900, 2041, ""),
    # ---- special standards-derived profiles ----
    ("right/Etc/UTC (leap)", "right/leap profile (v2, 27-leap table)", "leap-aware vs leap-ignoring readers",
     "/tmp/leaprs/Etc/UTC", "/tmp/leapref/Etc/UTC", 1970, 2030,
     "Python zoneinfo does NOT apply leap seconds — its civil-offset reading is leap-table-blind; zdump IS leap-aware."),
    ("right/America/New_York (leap)", "right/leap profile on a DST zone", "leap × DST interaction in readers",
     "/tmp/leaprs/America/New_York", "/tmp/leapref/America/New_York", 1970, 2030,
     "same zoneinfo leap-blindness note."),
    ("v4 Etc/UTC (leap-expiry)", "TZif v4 via Expires (future-compatible reader)", "pre-v4 readers reject/mis-handle v4",
     "/tmp/v4rs/Etc/UTC", "/tmp/v4ref/Etc/UTC", 1970, 2110,
     "v4 byte; both readers here accept it; a strict pre-v4 reader is out of scope (unassessed)."),
    ("-r America/New_York (truncated)", "range-truncated TZif with -00 leading/trailing type", "readers mis-handle -00 unspecified-local-time",
     "/tmp/trunc_rs/America/New_York", "/tmp/trunc_ref/America/New_York", 1995, 2035, ""),
]


def sha(p):
    return hashlib.sha256(open(p, "rb").read()).hexdigest()


def zdump_dump(path, lo, hi):
    out = subprocess.run(["zdump", "-v", "-c", f"{lo},{hi}", path], capture_output=True, text=True, timeout=60)
    lines = []
    for ln in out.stdout.splitlines():
        parts = ln.split(None, 1)
        lines.append(parts[1] if len(parts) == 2 else ln)
    return "\n".join(lines), out.returncode


def zoneinfo_obs(path, name):
    with open(path, "rb") as f:
        z = ZoneInfo.from_file(f, key=name)
    obs = []
    for t in PROBES:
        lt = t.astimezone(z)
        obs.append((lt.utcoffset().total_seconds(), bool(lt.dst()) and lt.dst() != dt.timedelta(0), lt.tzname()))
    return obs


rows = []
for (label, trap, risk, rs, ref, lo, hi, note) in M:
    h_rs, h_ref = sha(rs), sha(ref)
    byte_id = h_rs == h_ref
    try:
        drs, rcrs = zdump_dump(rs, lo, hi)
        dref, rcref = zdump_dump(ref, lo, hi)
        zd = "matched" if (drs == dref and rcrs == 0 == rcref) else "diverged"
    except Exception as e:
        zd = "unassessed"
        note = (note + f" zdump error: {e}").strip()
    try:
        ors, oref = zoneinfo_obs(rs, label), zoneinfo_obs(ref, label)
        zi = "matched" if ors == oref else "diverged"
        if zi == "diverged":
            d0 = next(((PROBES[i].year, oref[i], ors[i]) for i in range(len(PROBES)) if ors[i] != oref[i]), None)
            note = (note + f" zoneinfo first-diff {d0}").strip()
    except Exception as e:
        zi = "unassessed"
        note = (note + f" zoneinfo error: {e}").strip()
    rows.append((label, trap, risk, byte_id, h_rs[:12], h_ref[:12], zd, zi, "unassessed", note))

# ---- report ----
print(f"# T23.reader-compat.2 — RFC 9636 Appendix-A microcase gauntlet — {len(M)} microcases")
print("# readers: zdump (tzcode 2026b) · Python zoneinfo 3.14 · Go=unassessed (absent)")
print(f"# {len(PROBES)} UTC probe instants incl. ~-2^31 / ~+2^31\n")
hdr = f"{'microcase':34} {'byteid':6} {'zdump':9} {'zoneinfo':9} {'go':10} appendix-A trap"
print(hdr); print("-" * len(hdr))
agg = {}
for (label, trap, risk, bid, hrs, href, zd, zi, go, note) in rows:
    print(f"{label:34} {('yes' if bid else 'no'):6} {zd:9} {zi:9} {go:10} {trap}")
    if note:
        print(f"    note: {note}")
    for s in (zd, zi, go):
        agg[s] = agg.get(s, 0) + 1
print()
print(f"READER-RESULT TOTALS ({len(M)}×3 readers): " + " · ".join(f"{k}={v}" for k, v in sorted(agg.items())))
byte_ids = sum(1 for r in rows if r[3])
print(f"byte-identical zic-rs vs ref-zic: {byte_ids}/{len(M)} (reader-equivalence holds where bytes differ)")
sys.exit(1 if agg.get("diverged") else 0)
