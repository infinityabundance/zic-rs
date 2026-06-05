# Reader-compatibility gauntlet — receipt (T23.reader-compat.2, RFC 9636 Appendix-A expansion)

- **Date:** 2026-06-03 · **Host:** Linux 7.0.9-1-cachyos x86_64
- **Run:** `python3 reports/reader-compat/gauntlet-appendix-a.py` — **exit 0 after the fix** (the first run
  was exit 1: it found 1 divergence, now fixed; see THE FINDING + FIX)
- **Readers:** `zdump (tzcode) 2026b-dirty` · Python `zoneinfo` 3.14.5 · Go = **unassessed** (absent — no silent upgrade).
- **Method:** each microcase is compiled by **both** zic-rs and reference `zic` from the admitted
  `tzdata.zi` 2026b (special profiles via `-L leapseconds`, a crafted `-L` leap source with an active
  `Expires` → **v4**, and `-r @946684800/@1893456000` truncation). Each reader consumes **both** files;
  a result is `matched`/`diverged`/`skipped`/`unassessed`. 15 UTC probe instants incl. ~−2³¹ / ~+2³¹.
- **Tier (loud):** reader-equivalent for **these readers and these fixtures** ≠ all readers ≠ all TZif edge
  cases ≠ reference-`zic` byte identity ≠ civil-time truth.

## Result

**15 Appendix-A microcases × {zdump, zoneinfo} = 30 reader-results: 30 matched · 0 diverged · (Go: 15
unassessed)** — after the fix below (the first run had 1 divergence). byte-identical zic-rs vs ref-zic: 6/15
(reader-equivalence holds where bytes differ).

| microcase (Appendix-A trap) | byte-id | zdump | zoneinfo |
|---|---|---|---|
| Etc/UTC (baseline) | yes | matched | matched |
| America/New_York (pre-first-transition LMT + ~2³¹) | no | matched | matched |
| Europe/London (far-future footer extension) | no | matched | matched |
| Asia/Gaza (v3 footer) | no | matched | matched |
| Africa/Cairo (24:00 time-of-day) | no | matched | matched |
| Asia/Kathmandu (+5:45 non-hour) | yes | matched | matched |
| Australia/Eucla (+8:45) | no | matched | matched |
| Antarctica/Troll (numeric +00/+02) | yes | matched | matched |
| America/Argentina/Buenos_Aires (numeric -03) | no | matched | matched |
| Europe/Amsterdam (sub-minute LMT) | no | matched | matched |
| Pacific/Kiritimati (+14) | yes | matched | matched |
| **right/Etc/UTC** (leap profile, v2, 27-leap table) | yes | matched | matched¹ |
| **right/America/New_York** (leap profile on a DST zone) | no | **matched** (after fix) | matched¹ |
| **v4 Etc/UTC** (leap-expiry → TZif v4) | yes | matched | matched² |
| **-r America/New_York** (range-truncated, `-00` type) | no | matched | matched |

¹ Python `zoneinfo` does **not** apply leap seconds — its civil-offset reading is leap-table-blind, so it
matched both files; the leap-specific axis is therefore *not exercised* by this reader (effectively
`skipped` for leap semantics). `zdump` **is** leap-aware and is the reader that found the divergence.
² Both readers accept the v4 byte here; a strict pre-v4 reader is out of scope (`unassessed`).

## THE FINDING + FIX — `right/` (leap) profile transition encoding for zones with transitions

`zdump` on **`right/America/New_York`** diverges between zic-rs and reference `zic`:

```text
reference:  Sun Oct 29 06:00:00 1972 UT = 01:59:59 1972 EDT   (transition time advanced by the leap correction)
zic-rs:     Sun Oct 29 05:59:59 1972 UT = 01:59:58 1972 EDT   (transition time left at the POSIX value)
```

The gap **grows with the accumulated leap count** (~1 s in 1972, ~2 s 1973, ~3 s 1974, …). Diagnosis:

- In a `right/` (leap-aware, TAI-based) TZif, reference `zic` **advances each transition time by the
  accumulated leap correction at that instant**; **zic-rs emits the leap *table* correctly but leaves the
  zone's transition times at their POSIX (UTC) values** — so a leap-aware reader (`zdump`) sees every
  transition drift behind reference by the running leap count.
- **`right/Etc/UTC` matched** because it has **no transitions** to adjust — which is exactly why **T11**
  (which verified only `right/UTC`) never caught this. The Appendix-A expansion is what surfaced it.
- **CORE.1 / the default profile are UNAFFECTED:** POSIX `America/New_York` (no `-L`) is behaviour-identical
  to reference (verified in this run); the defect is isolated to the **opt-in `right/` profile** on zones
  that have transitions.

**Finding fixed (same arc):** `apply_leaps` now leap-adjusts the zone's transition instants for the `right/`
profile — each transition is shifted by the cumulative leap correction effective there (matching reference
`zic`'s TAI-based `right/` encoding); `right/`-only, so POSIX/default output is byte-unchanged. **Verified:**
`right/{America/New_York, Europe/London, Etc/UTC}` `zdump`-match reference · POSIX New_York identical · CORE.1
341/0/0 · the gauntlet re-ran **30/30** · **+1 regression test**
(`right_profile_shifts_transitions_by_cumulative_leap_correction`, a transition-bearing witness — the case
T11's `right/UTC` lacked). **Scope / non-claims:** `right/` profile only; no change to POSIX/default output;
no universal leap-profile theorem beyond the tested fixtures; no civil-time-truth claim; no
TZDIST/truncated-profile claim. Tracked: `RISK.LEAP.1` + `audits/claim-boundary-map.md`.

## Non-claims

- Reader-equivalent for **these readers, these fixtures, these instants, this host** — not all readers, not
  all TZif edge cases, not reference-`zic` byte identity, not civil-time truth.
- Go = `unassessed`, not a pass. Python `zoneinfo` does not exercise leap-second semantics.
- This run **found AND fixed a real divergence** (no paper-over): the `right/`-profile transition-encoding
  defect was found here, fixed in `apply_leaps`, and re-verified 30/30 with a regression test; it was always
  an opt-in-profile gap, never a CORE.1 regression.
