# RECEIPT — RFC9636-ERRATA.1 (RFC 9636 errata & conformance-trust check) — 2026-06-05

> **Claim wording (binding):** *RFC9636-ERRATA.1 records the published-errata state of RFC 9636 (the TZif
> format standard zic-rs's output and `rfc9636.rs` validator claim conformance to) at a pinned date, and
> classifies each erratum's relevance to zic-rs. An errata-clean check is a **standards-currency** snapshot —
> it does **not** prove zic-rs is bug-free, and it is **not** a conformance proof; it confirms the normative
> text zic-rs targets has no outstanding corrections that would change what "conformant" means.*

## What was checked (authoritative source, pinned)

The RFC Editor's **machine-readable** errata feed (the canonical source — the
`errata.rfc-editor.org/search` page is a JS UI that renders no server-side records):

| | |
|---|---|
| source | `https://www.rfc-editor.org/errata.json` |
| fetched | 2026-06-05 |
| feed sha256 | `cc92d64d201e5775ffbd6a82c31ed9ef0bd8346e94d55beab0f84c582f1f4f28` |
| feed size / total errata | 11,508,091 B / **7,935** errata (all RFCs) |
| filter sanity | RFC 8259 → 15 · RFC 8536 → 4 (both non-zero ⇒ the `doc-id == "RFC9636"` filter is sound) |

Reproduce: `bash reports/rfc9636-errata/fetch.sh`. Pinned snapshot: `reports/rfc9636-errata/errata-snapshot.json`.

## Result

> **RFC 9636 (The Time Zone Information Format, TZif): 0 reported errata** as of 2026-06-05.

The normative format text zic-rs implements has **no outstanding corrections**.

## Lineage context — the predecessor RFC 8536 (which RFC 9636 obsoletes)

RFC 8536 carries **4** errata. Read in full; **none touches the normative §3 format rules zic-rs implements** —
all are example/illustration corrections, and RFC 9636 (the corrected successor) folds them in:

| EID | status | §section | what it corrects | relevant to zic-rs? |
|---|---|---|---|---|
| 7681 | Held for Document Update | **B.2** (example) | a UT/local indicator field value in the worked byte-table example | No — example typo, not a rule |
| 6426 | Held for Document Update | **B.3** (example) | example header table inconsistent with its own octets; *note restates §3.1 "typecnt and charcnt MUST NOT be zero"* | No (example), but the **underlying rule is already enforced + Kani-proven** in zic-rs (`charcnt != 0`, `typecnt >= 1`) |
| 6435 | Verified | **5.2** (TZDIST HTTP example) | `application/json` should carry no `charset` parameter | No — a TZDIST-protocol example, **not TZif format**, and out of zic-rs scope |
| 6757 | Held for Document Update | **B.3** (example) | big-endian encoding typos in the worked example's `isutcnt`/`isstdcnt` octets | No — example typo, not a rule |

**Finding:** every RFC 8536 erratum is an **appendix/example** correction (B.2/B.3) or a TZDIST protocol-example
nit (§5.2); not one changes the normative byte-format definition (§3) that `src/tzif/rfc9636.rs` validates.
The single erratum that even *references* a format rule (EID 6426 → "typecnt/charcnt MUST NOT be zero") names a
rule zic-rs **already enforces** (and proves: T23.kani.3f — `charcnt != 0`).

## zic-rs's RFC 9636 reliance surface (what the 0-errata result protects)

- **Output contract:** zic-rs emits TZif per RFC 9636 (S1 in the knowledge index; `RISK.TZIF.1`).
- **Validator:** `src/tzif/rfc9636.rs` checks the §3.2 byte-format invariants — `typecnt >= 1`, transition
  `type_index < typecnt`, indicator counts ∈ {0, typecnt}, `isut == 1 ⇒ isstd == 1`, designation index
  `< charcnt` with `charcnt != 0` and a NUL terminator, `utoff != i32::MIN`, strictly-ascending transitions.
  Four of these are **Kani-proven** (T23.kani.3f) and enforced as `Violation`s.

Because RFC 9636 has no errata, these checks are validating against the **current, uncorrected** normative text —
there is no pending correction that would move the conformance target.

## Non-claims

- **Errata-clean ≠ bug-free.** This says the *standard* has no outstanding corrections; it says nothing about
  zic-rs's own correctness (that is CORE.1 / the validator / the audit board).
- **Not a conformance proof.** zic-rs is a measured implementation; RFC 9636 is the format authority. This
  receipt confirms standards *currency*, not full conformance.
- **Point-in-time snapshot.** Valid as of 2026-06-05 against the pinned feed hash; re-run `fetch.sh` at each
  release (and on any RFC 9636 erratum) — a new erratum is a signal to re-assess the validator, not silently
  ignore it. (A standing `RISK.TZIF.1` / maintenance-policy item.)
- Scope is RFC 9636 (TZif). TZDIST (RFC 8536 §5 / RFC 7808) is out of zic-rs scope.

## Gate

Docs/report only — no `src/` change; CORE.1 341/0/0 unaffected; doc-staleness green. Cross-linked from the
knowledge index (S1) + `docs/claim-source-map.md` + `RISK.TZIF.1`.
