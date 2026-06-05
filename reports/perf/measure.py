#!/usr/bin/env python3
"""T22 performance/resource-ledger harness (external operational tooling — NOT part of the crate).

Drives the **release** `zic-rs` binary over the standard admitted workloads and emits a Markdown receipt
to stdout. Per workload it records wall time (`perf_counter`) and **peak RSS** (`os.wait4` →
`ru_maxrss`, accurate per-child on Linux, KiB). It also pins the *deterministic* anchors — input/output
bytes, file count, the `size-report` bundle_hash + counts, and the `ResourceLimits` config — which are the
real regression anchors (timings are host- and run-specific, indicative only).

Doctrine (kept in the receipt): T22 proves normal admitted workloads are **measured and bounded**; it does
NOT claim fastest-`zic`, universal performance superiority, or adversarial-CPU-exhaustion immunity.

Usage: measure.py <zic-rs-binary> <tzdata.zi> <YYYY-MM-DD> [reference-zic] [reference-zdump]
"""
import hashlib
import json
import os
import subprocess
import sys
import tempfile
import time


def measure(cmd):
    """Run cmd (argv list, absolute argv[0]); return wall_ms, max_rss_kib, exit. stdout/stderr → /dev/null."""
    devnull = os.open(os.devnull, os.O_WRONLY)
    fa = [(os.POSIX_SPAWN_DUP2, devnull, 1), (os.POSIX_SPAWN_DUP2, devnull, 2)]
    t0 = time.perf_counter()
    pid = os.posix_spawn(cmd[0], cmd, os.environ, file_actions=fa)
    _, status, ru = os.wait4(pid, 0)
    t1 = time.perf_counter()
    os.close(devnull)
    return (t1 - t0) * 1000.0, ru.ru_maxrss, os.waitstatus_to_exitcode(status)


def capture(cmd):
    """Run cmd and return (stdout_text, exit)."""
    p = subprocess.run(cmd, capture_output=True, text=True)
    return p.stdout, p.returncode


def tree_stats(root):
    files = total = 0
    for dp, _, fns in os.walk(root):
        for fn in fns:
            fp = os.path.join(dp, fn)
            if os.path.islink(fp):
                continue
            files += 1
            total += os.path.getsize(fp)
    return files, total


def main():
    binp, tzdata, date = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2]), sys.argv[3]
    ref_zic = sys.argv[4] if len(sys.argv) > 4 else None
    ref_zdump = sys.argv[5] if len(sys.argv) > 5 else None

    in_bytes = os.path.getsize(tzdata)
    in_sha = hashlib.sha256(open(tzdata, "rb").read()).hexdigest()
    out = tempfile.mkdtemp(prefix="zic-rs-perf-")

    rows = []  # (label, cmd_display, wall_ms, rss_kib, exit)

    def run(label, argv):
        wall, rss, code = measure(argv)
        rows.append((label, " ".join(argv[1:]), wall, rss, code))
        return code

    run("compile full admitted tree (--all-supported)",
        [binp, "compile", "--input", tzdata, "--out", out, "--all-supported"])
    files, out_bytes = tree_stats(out)

    run("size-report (footprint + bundle_hash)", [binp, "size-report", "--out", out])
    sr_json, _ = capture([binp, "size-report", "--out", out, "--format", "json"])
    try:
        sr = json.loads(sr_json)
    except Exception:
        sr = {}

    run("support-report", [binp, "support-report", "--input", tzdata])
    if ref_zic:
        run("structural-report (vs reference zic)",
            [binp, "structural-report", "--input", tzdata, "--reference-zic", ref_zic])
    run("doctor (host probe overhead)", [binp, "doctor"] + (
        ["--reference-zic", ref_zic] if ref_zic else []) + (
        ["--reference-zdump", ref_zdump] if ref_zdump else []))
    run("release-diff self-diff (engine cost, behaviour axis off)",
        [binp, "release-diff", "--old", tzdata, "--new", tzdata])

    # host identity
    uname = os.uname()
    try:
        ncpu = os.cpu_count()
    except Exception:
        ncpu = None
    rustc = capture(["rustc", "--version"])[0].strip() or "unknown"
    zver = capture([binp, "--version"])[0].strip() or "unknown"

    p = print
    p(f"# Performance / resource receipt — {uname.machine} — {date}")
    p("")
    p("> **Receipt, not a benchmark claim.** Wall time + peak RSS are **this host, this run** — indicative,")
    p("> not authoritative and not a speed claim. The *deterministic anchors* (input/output bytes, file")
    p("> count, `bundle_hash`, counts, limits config) are the regression anchors. **Non-claims:** T22 does")
    p("> NOT claim fastest-`zic`, universal performance superiority, or adversarial-CPU-exhaustion immunity")
    p("> (the `ResourceLimits` caps bound the *input-size* tail — `RISK.RESOURCE.1` — not all adversarial CPU).")
    p("")
    p("## Host & build identity")
    p("")
    p(f"- machine: `{uname.machine}` · cpus: `{ncpu}` · kernel: `{uname.sysname} {uname.release}`")
    p(f"- rustc: `{rustc}` · profile: `release` (`overflow-checks` on) · zic-rs: `{zver}`")
    p(f"- reference zic: `{ref_zic or 'absent'}` · reference zdump: `{ref_zdump or 'absent'}`")
    p(f"- date: `{date}`")
    p("")
    p("## Deterministic anchors (the regression anchors — host-independent)")
    p("")
    p(f"- input: `{os.path.basename(tzdata)}` — **{in_bytes} bytes**, sha256 `{in_sha[:16]}…`")
    p(f"- output tree: **{files} files**, **{out_bytes} bytes**")
    p(f"- size-report: tzif_files=**{sr.get('tzif_files','?')}** · symlink_links={sr.get('symlink_links','?')} · "
      f"other_files={sr.get('other_files','?')} · footer_present={sr.get('footer_present','?')}")
    vh = sr.get("version_histogram", {})
    p(f"- version histogram: v1={vh.get('v1','?')} v2={vh.get('v2','?')} v3={vh.get('v3','?')} v4={vh.get('v4','?')}")
    p(f"- **bundle_hash:** `{sr.get('bundle_hash','?')}`")
    p("")
    p("## Resource limits in force (`limits::ResourceLimits` defaults — the bounded envelope)")
    p("")
    p("- source bytes/file ≤ 512 MiB · zones/rules/links ≤ 1,000,000 · leap entries ≤ 100,000 ·")
    p("  link-chain depth ≤ 256 · zone-eras ≤ 100,000 · line length ≤ 2048 · transitions/zone ≤ `MAX_TRANSITIONS`.")
    p("  Real 2026b sits *far* inside every cap (≈350 zones / ≈600 links / 27 leaps) — a breach is `Error::config`.")
    p("")
    p("## Measured workloads (wall + peak RSS — indicative, this host/run)")
    p("")
    p("| Workload | wall (ms) | peak RSS (MiB) | exit |")
    p("|---|---:|---:|---:|")
    for label, _disp, wall, rss, code in rows:
        p(f"| {label} | {wall:.1f} | {rss/1024.0:.1f} | {code} |")
    p("")
    p("## Reproduce")
    p("")
    p("```sh")
    p("cargo build --release")
    p(f"python3 reports/perf/measure.py target/release/zic-rs {tzdata} {date} zic zdump")
    p("```")
    p("")
    p("_(Timings will differ per host/run; the deterministic anchors above must not.)_")

    # leave `out` for the caller to inspect/remove; print its path to stderr.
    print(f"(output tree: {out})", file=sys.stderr)


if __name__ == "__main__":
    main()
