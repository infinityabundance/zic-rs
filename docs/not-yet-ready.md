# Not yet ready (T19)

> **This document is armor, not weakness.** It states, loudly and exactly, what zic-rs is **not ready
> for** — so no reader, packager, or reviewer can quietly promote a bounded claim into an unbounded one
> (the project's single biggest standing risk: *claim expansion under pressure*). Every line here is a
> deliberate refusal with a pointer to where the bound is enforced.

## Not ready as…

- **A default / primary system `zic` replacement.** Behaviour parity is exactly **CORE.1** (341 canonical
  zones, 2026b, 1900..2040) — not all releases, not all operational modes. Adoption is **staged**; see
  `docs/replacement-readiness-ladder.md` (zic-rs is at validator/report/compiler-candidate level, **not**
  default-replacement). What breaks if you alias `zic`→`zic-rs` today is enumerated in
  `docs/drop-in-compatibility-contract.md`.
- **A civil-time authority.** zic-rs compiles **admitted** IANA tzdb source; it does not curate data,
  settle legal time, or predict future law. IANA/CLDR own that (`RISK.TIME.1`, RFC 6557/BCP 175).
- **A runtime `localtime`/`mktime` library or a `tzselect`/CLDR replacement.** It produces and verifies
  TZif; it does not interpret a named zone at call time, localize display names, or own `Etc/Unknown`
  (those are the consumers' + CLDR's territory).
- **A signed attestation.** Reports are `unsigned_local_report` — evidence, not certification (`RISK.REPORT.1`).
- **Crash-atomic for a whole install tree.** Per-file durable publish is provided on Unix; a crash
  *mid-run* can leave a partial tree (`docs/install-materialization-contract.md`, `RISK.INSTALL.1`).
- **Hardened against a concurrent parent-component symlink-swap race.** Leaf TOCTOU is closed; the
  parent-component race needs `openat`/`O_NOFOLLOW` (forbidden without `unsafe`/a dep) →
  `RequiresOpenatStyleHardening` (`RISK.PATH.1`).
- **Exhaustively fuzzed.** A **bounded** smoke ran (T23.cargo-fuzz.1/.2: 9 targets × 25 s — found + fixed 3
  panic bugs F1–F3, re-ran 9/9 clean). A coverage-saturating (e.g. 24 h) campaign is **not** done;
  "bounded smoke clean" ≠ "no panic exists".
- **Externally audited / formally verified.** The `docs/audit-readiness.md` packet + the planned
  `audits/` suite (T23) prepare for it; no external audit has been performed.
- **Schema-instance-validated in the core.** The core carries no JSON-Schema validator dependency;
  `tests/schemas.rs` guards the registry/drift only — full instance validation is a future audit-suite task.
- **Proven across unadmitted tzdb releases.** Only **2026b** is admitted. A 2026b claim is never
  generalized to another release (the release-admission matrix).
- **A vendor-family theorem.** A vendor-oracle receipt admits **one measured ecology**, not a family-wide
  guarantee (`RISK.VENDOR.1`).
- **Performance-characterised.** No performance/resource ledger exists yet (T22); zic-rs claims no speed
  envelope, only that real tzdb input sits inside the `limits::ResourceLimits` caps.
- **Packaged / adoption-proven in distros.** No packaging gauntlet yet (T21); zic-rs is repo-evaluable, not
  package-evaluated.
- **Knowledge-archived.** The T18 knowledge index records canonical URLs; the Wayback/archive.today
  capture pass is partial — **S1–S6 captured** (operator pass, 2026-06-02), S10/S13 still `pending_capture`,
  copyrighted rows link-only.
- **A calendar / distribution / localization / geospatial / precision-time surface.** The upstream
  `tz-link` ecology map (S13) shows the world around tzdb; zic-rs is deliberately a *TZif compiler*, not any
  of its neighbours. zic-rs does **not** implement or claim: **TZDIST / CalDAV / `VTIMEZONE` / xCal / jCal**
  (calendar distribution); **CLDR localization / display-name translation / ICU-, Java-, or Windows-registry
  output formats**; **latitude/longitude → zone-id geospatial lookup** (it compiles a named zone, it does not
  decide which zone a coordinate is in); **NTP / PTP / leap-smear / precision-clock synchronization or
  post-2035 leap policy** (it compiles admitted leap *evidence* where given; it does not define UTC/TAI). Each
  is a separate ecosystem owner (`docs/drop-in-compatibility-contract.md` "Non-claims"; `RISK.TIME.1`/`RISK.ADOPT.1`).

## How to read this

If you need any of the above, the honest answer is **"not yet — here is the bound, the owning milestone,
and the evidence that would have to land first."** That is the same posture as the success surfaces: a
refusal is a typed, located claim, not a silence. Cross-reference: `docs/risk-register.md` (per-risk
status + non-claim), `support-report`'s `negative_capabilities` (each with its `enforced_by` guard), and
`TRUST.md` §6 (the consolidated non-claim list).
