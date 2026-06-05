# Audit-readiness packet (T19)

> **This is the packet, not a performed audit.** It is what an external security/correctness auditor needs
> to start *without* a kickoff call: the threat model, the surfaces to attack, the corpus to attack them
> with, the reproduction commands, and — stated up front — **what an audit of zic-rs may and may not
> conclude.** No external audit has been performed; this document makes one cheap to run. Pairs with
> `docs/risk-register.md` (the 15 claim-boundary risks), `docs/effect-boundary-map.md` (where the host is
> touched), `SECURITY.md` (what counts as a reportable defect), and the planned `audits/` suite (T23).

## 1. What to audit (the claim, exactly)

zic-rs **behaviour-matches reference `zic`/`zdump` for all 341 canonical zones in `tzdata.zi` 2026b over
1900..2040** (CORE.1), with structural / byte / diagnostic / operational parity kept as **separate axes**.
That sentence — not "a `zic` rewrite" — is the audit target. Everything below is in service of confirming
or refuting *it* and the typed non-claims around it.

## 2. Threat model (audit lens)

The project's thesis: **the dangerous failures are claim-boundary bugs first, memory bugs second** — a
plausible-but-wrong TZif that loads cleanly and is wrong at a civil-time boundary is worse than a loud
crash. So the audit's highest-value targets are:

| Threat | Where it lives | Already-claimed defence | Audit question |
|---|---|---|---|
| wrong offset/abbrev/footer (silent) | `compile/transitions`, `compile/posix_footer` | CORE.1 sweep vs `zdump` | does the sweep miss a zone class? |
| structurally-valid-but-wrong TZif | `tzif/` writer | 5 separate RFC-9636 verdicts | can a wrong file pass all five? |
| panic / OOM on hostile input | `tzif/validate::parse`, `source/` | bounds-guard + `CountArithmeticVerdict` + `ResourceLimits` | a fixture that panics or OOMs? |
| path traversal / clobber / TOCTOU | `fs/output_tree`, `fs/atomic_write` | `ZIC008` reject · exclusive-create · durable publish | a write-through or leaf-race? |
| report overclaim (report-as-truth) | `report`, `manifest`, `release_diff`, `doctor` | bounded statuses · typed unknowns · `unsigned_local_report` | a report that reads as more than it proves? |
| supply chain | `Cargo.toml` deps | `#![forbid(unsafe_code)]`, no `build.rs`, lean deps | a transitive `unsafe`/yank/advisory? |

## 3. Surfaces (ranked by danger)

1. **`tzif::validate::parse` + `tzif::rfc9636`** — the hostile-bytes front door (every count/offset checked; see `reports/t17-count-arithmetic-verdict.md`).
2. **`source/{lexer,parser,leap}`** — hostile-source front door (`ZIC001`–`ZIC026`; line/NUL/quote caps).
3. **`fs/output_tree` + `fs/atomic_write`** — the only `FilesystemWrite` boundary (`cfg(unix)` gated).
4. **`vendor_oracle::from_json`** — external-receipt ingestion (no-dep, fail-closed; admission recomputed).
5. **`compile/`** — semantic correctness (CORE.1's subject).
6. **report/manifest emitters** — claim-shape integrity (schemas in `schemas/`).

`docs/effect-boundary-map.md` is the per-module effect classification; an auditor should read it first to
know which surfaces touch the host at all.

## 4. Corpus to attack with

- **Canonical:** `tzdata.zi` 2026b (the CORE.1 subject) — provenance + hash in `reports/t12_5a2-reference-admission.md`.
- **Hostile source:** `tests/input_admissibility.rs`, `tests/pathology_ledger.rs`, `fixtures/` (range/leap/minimal).
- **Hostile TZif:** the hand-built OOB/overlarge-count fixtures in `src/tzif/validate.rs` tests + the `tzif_validate_bytes`/`vendor_oracle_json` fuzz targets' seed corpora (`fuzz/corpus/`).
- **Vendor receipts:** `../zic-rs-vendor-oracle-lab/receipts/` (19 receipts) — real platform `zic` diagnostics.
- **Fuzz harness:** `fuzz/` (9 targets) — **a bounded smoke RAN (T23.cargo-fuzz.1/.2, 25 s/target)**: found 3 panic bugs (F1–F3), fixed 3/3 with regression seeds, re-ran 9/9 clean (`audits/cargo-fuzz/receipts/RECEIPT-2026-06-04*.md`). A coverage-saturating (24 h) campaign remains an auditor/lab task — bounded smoke ≠ exhaustive.

## 5. Reproduction commands

```sh
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test   # all green (count: STATUS.md), zero unsafe
bash /tmp/t9sweep.sh                                                           # CORE.1 → 341/0/0
zic-rs structural-report --input /usr/share/zoneinfo/tzdata.zi --reference-zic zic
zic-rs tzif-validate --input <file.tzif>                                       # 5 typed verdicts
zic-rs doctor --format json                                                    # host probe, always exit 0
cd fuzz && cargo +nightly fuzz run tzif_validate_bytes                          # operator/lab: produce a run-receipt
```

The How-to-verify posture: **explicit paths only** — the commands never read system tzdb implicitly (the
`VerifierCommandPolicy` rule). An auditor reproduces against *named* inputs, not the host's zoneinfo.

## 6. What an audit MAY conclude

- Whether the CORE.1 sweep is sound and complete **for its declared scope** (341 zones · 2026b · 1900..2040).
- Whether any hostile input panics, OOMs, writes outside `--out`, or escapes via a leaf race.
- Whether a structurally-valid TZif can be semantically wrong without the five verdicts catching it.
- Whether any report reads as a stronger claim than its evidence supports.
- Whether the dependency/`unsafe`/`build.rs` posture holds.

## 7. What an audit may NOT conclude (out of scope by construction)

- That zic-rs is a **full `zic` replacement** — it is RRL-1→2 (`docs/replacement-readiness-ladder.md`); the audit target is CORE.1, not universal parity.
- That **other tzdb releases** behave correctly — only 2026b is admitted.
- That **other platforms** behave identically — vendor behaviour is per-receipt, not a family theorem.
- That a clean audit is a **signed attestation** — reports are `unsigned_local_report`; an audit of zic-rs is an audit, not a certification of every future build.
- That **civil-time correctness** is established — zic-rs compiles IANA source faithfully; legal-time truth is IANA/CLDR's (`RISK.TIME.1`).

## 8. Non-claims this packet itself carries

- This packet **prepares for** an audit; it is not one and asserts no audit result.
- The `audits/` suite (T23) names the tools (miri, kani, cargo-fuzz, cargo-vet, …) but is **receipt-bearing-when-built**, not yet run.
- Bounded fuzzing ran (T23.cargo-fuzz.1/.2) ≠ exhaustive fuzzing done (no coverage-saturating campaign yet).
