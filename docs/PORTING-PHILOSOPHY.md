# Porting Philosophy

> **This Rust port does not ask reviewers to trust the rewrite.** It gives them typed claims, admitted
> references, hostile-input evidence, known divergences, negative capabilities, and reproducible reports, so
> they can inspect the boundary of trust themselves. Memory safety is *one row of the threat table, not the
> argument.* (The method, generalized, is written up in `paper/reference-admitted-rust-ports.tex`.)

## 1. The precision ladder (every claim must name its rung)

The project is in the **precision phase**: the evidence skeleton exists; the job now is to prevent subtle
**category mistakes**. Each `≠` is a rung a claim must not silently climb. The authoritative copy lives atop
the anti-drift governor `audits/claim-boundary-map.md`; it is restated here as the philosophy front door.

```text
memory-safe accepted        ≠  format-valid (RFC 9636)            # parse is lenient; rfc9636::validate is strict (T23.kani.3f.enforce)
format-valid                ≠  semantically equivalent            # tzif-validate ≠ semantic-report
semantically equivalent     ≠  civil-time / domain truth          # IANA/CLDR own that (RISK.TIME.1)
admitted source release     ≠  vendor distribution policy         # only 2026b admitted; a distro ships what it ships
one vendor receipt          ≠  a vendor-family theorem            # 17 ecology rows, each one ecology (RISK.VENDOR.1)
an audit tool ran           ≠  global safety                      # a receipt is one dimension, not a verdict
a bounded proof             ≠  a universal proof                  # 10 reduced-surface Kani helpers, not the whole parser
an archive copy             ≠  authority                          # canonical_url = authority; archive_url = witness (T18)
byte-identical output       ≠  reader-equivalent behaviour        # T23.reader-compat: 4/12 byte-identical, all read-equal
a proven predicate          ≠  an enforced check                  # closed for the 4 RFC predicates (T23.kani.3f.enforce)
container/VM recipe ran     ≠  distro package acceptance          # T23.drop-in-gauntlet: ran ≠ apk/PKGBUILD/abuild acceptance
```

Every receipt answers one question — ***what lie does this prevent?*** — and refuses the next rung.

## 2. The four authorities (never conflated)

A claim is only as good as *which authority* backs it (`TRUST.md` §6a):

| Authority | Owner | zic-rs's relationship |
|---|---|---|
| Civil-time policy | governments · the IANA tz process | **deferred** — zic-rs does not set or judge civil time |
| Release-artifact | admitted tzdb tarballs · signatures · hashes | **admitted, recorded** — fetch + verify-sig + hash-pin (only 2026b; the real 2026a→2026b release-diff admitted 2026a too) |
| Compiler-behaviour | reference `zic`/`zdump` (the oracle) + zic-rs's typed reports | **measured** — this is zic-rs's lane (CORE.1, the `*-v1` reports) |
| Distribution | distro packages · the vendor-oracle lab | **observed per receipt** — admitted receipts, never a family theorem |

## 3. The doctrine, in one paragraph

zic-rs is a **reference-admitted Rust TZif compiler candidate**: it compiles IANA tzdb source → binary TZif
(RFC 9636) and checks itself against reference `zic`/`zdump`. It earns each claim through admitted sources,
typed contracts, oracle checks, machine-readable reports, bounded proofs, and explicit non-claims. The live
behaviour claim is never inflated: *behaviour-matches reference `zic`/`zdump` for all 341 canonical zones in
`tzdata.zi` 2026b over `1900..2040` (CORE.1).* It is **not** a universal `/usr/sbin/zic` drop-in (by design:
a sub-command CLI, not bare-flag argv), and it says so as loudly as it states its strengths.

## 4. What the evidence court already contains (cite, don't restate)

- **Behaviour:** CORE.1 sweep 341/0/0 · `semantic-report` (`zdump` witnesses).
- **Standards:** `tzif-validate` (RFC 9636, 5 typed verdicts) + `T23.kani.3f.enforce` (4 proven RFC predicates
  *enforced* in `rfc9636::validate`; `memory-safe ≠ format-valid`).
- **Formal:** 10 verified **reduced-surface** Kani helper proofs (count/cursor/type-index/abbr-slice +
  designation-RFC-validity/`isdst`/indicator-pairing/`utoff`) — *sharp invariants, not broad parser vibes*
  (`audits/kani`).
- **Hostile input:** pathology ledger · `ZIC001`–`ZIC026` · `panic-analysis` (0 panics) · `miri` (0 UB).
- **Ecology:** the vendor-oracle lab (17 rows / 19 receipts; two `zic` lineages; the glibc one
  version-stratified) — *core admits receipts, never runs VMs*.
- **Readers:** `T23.reader-compat.1/.2` — real readers consume the output; **found and fixed** the
  `right/`-leap transition defect; *byte-identical ≠ reader-equivalent*.
- **Releases:** `T23.release-diff-real.1` — real 2026a→2026b delta (`America/Vancouver`), independently
  confirmed.
- **Deployment:** `T23.drop-in-gauntlet` — Arch (glibc container) + Alpine (musl container **and a real
  QEMU VM**) package-build recipes; the staged-tree `bundle_hash` is **byte-identical across libc,
  container/VM, and kernel** (cross-environment determinism).
- **Supply chain (honest):** `cargo-audit` clean · `cargo-vet` 4 first-party / 23 trusted-import / **39
  exempted UNAUDITED** (counted separately, forever) · SBOM/SLSA/signing **planned, not present**.

## 5. Companion documents

- `docs/PORTING-DECISION-LEDGER.md` — the design choices, and the tempting shortcuts **declined**.
- `docs/DIVERGENCE-REGISTER.md` — every known difference from reference `zic`, owned and bounded.
- `audits/claim-boundary-map.md` — the anti-drift governor (boundary · evidence · gap · next-receipt).
- `TRUST.md` — the front door into the evidence court · `docs/not-yet-ready.md` — the loud refusal surface.
- `paper/reference-admitted-rust-ports.tex` — the method generalized for other legacy-infrastructure ports.
