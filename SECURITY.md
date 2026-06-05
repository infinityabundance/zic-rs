# Security policy

> **zic-rs treats *claim-boundary* bugs as security/reliability defects** when they could cause an
> operator to **trust an unproven timezone artifact**. The dangerous failure here is not (first) a Rust
> memory bug — it is a *plausible-but-wrong* TZif that loads fine and is wrong at a civil-time boundary,
> or a report/claim that says more than it proved. The full map is [`docs/risk-register.md`](docs/risk-register.md);
the rewrite-safety argument is [`docs/security-rewrite-evaluation.md`](docs/security-rewrite-evaluation.md)
and the per-audience reading is [`docs/security-personas.md`](docs/security-personas.md).

## Reporting

Report suspected issues privately to the maintainers (see `CONTRIBUTING.md` / repository contacts) rather
than opening a public issue, if the issue could let an operator trust a wrong artifact or escape the output
directory. Include: the input (or a minimised fixture), the command, the observed vs expected behaviour,
and the zic-rs version (`zic-rs --version`) + `zic-rs doctor` output. We aim to acknowledge promptly and to
fix with a regression test in the same change (the project's same-batch-seal discipline).

## What counts as a security / reliability / claim-boundary defect

zic-rs has a broader-than-usual notion of "security issue" because its job is to be *trustworthy about
time*. Any of the following is in scope:

- **Panic / crash on hostile or malformed input** (source `.zi`, a TZif byte stream, a vendor receipt) —
  the policy is *no panic on untrusted input; it becomes a typed `Err`* ([`docs/panic-policy.md`](docs/panic-policy.md)).
- **Path traversal / unsafe materialization** — any write outside `--out`, write-through of a pre-planted
  symlink/file, or a name escaping the root ([`docs/install-materialization-contract.md`](docs/install-materialization-contract.md), `ZIC008`).
- **Resource exhaustion** — an input that drives unbounded memory/time past the `limits::ResourceLimits`
  caps, or a count/size/offset that overflows or pre-allocates from an untrusted value
  (`reports/t17-count-arithmetic-verdict.md`).
- **A wrong TZif emitted with a valid-looking report** — structural validity or a clean report that masks
  a semantic divergence (TZif structural validity ≠ semantic parity).
- **`unknown` rendered as `unchanged` / "no change"** — e.g. `release-diff` reporting "same" when the
  behaviour oracle was absent (it must be `behaviour_unassessed`).
- **Oracle-identity confusion** — a report's verdict computed against the wrong bytes (host zoneinfo
  instead of the emitted file) or an unidentified reference tool.
- **Report-as-attestation confusion** — treating an `unsigned_local_report` as a signed certification.
- **Source-provenance / copyright mishandling** — admitting unverified source as pristine, or (future
  T18) republishing copyrighted material.
- **Claim overstatement** — any place the project asserts more than its typed evidence proves (a missing
  or wrong non-claim). *This is a defect class here, not a documentation nicety.*

Plus the conventional categories: memory safety (mitigated structurally — `#![forbid(unsafe_code)]`),
integer overflow (`overflow-checks` + checked arithmetic on untrusted counts), and supply-chain integrity.

## Not in scope (by design / explicit non-claim)

- **Civil-time correctness of the data itself** — zic-rs compiles admitted IANA tzdb source; it does not
  curate or define civil time (that is IANA/CLDR; see `docs/tzdb-governance.md`).
- **Whole-tree crash-atomic install** and the **concurrent parent-component symlink-swap race** — named,
  bounded non-claims (`RISK.INSTALL.1` / `RISK.PATH.1`); per-file durable publish *is* provided on Unix.
- **A report being a signed attestation** — reports are `unsigned_local_report`.
- Vendor behaviour beyond an **admitted receipt** (a receipt is one measured ecology, not a family theorem).

These are tracked refusals, not gaps — see the risk register's status + non-claim columns.

## Supported versions

Pre-1.0: the latest `main` is the supported surface. Behaviour claims are **release-scoped** to the
admitted tzdb release named in the report (currently `2026b`); historical claims are not auto-forward-ported.
Report/CLI compatibility follows [`docs/schema-compatibility-policy.md`](docs/schema-compatibility-policy.md)
and [`docs/cli-compatibility-policy.md`](docs/cli-compatibility-policy.md).

## Disclosure

We prefer coordinated disclosure: a fix + regression test land together, the risk register's relevant row
moves forward (with any residual stated as a non-claim), and the change is noted in `CHANGELOG.md`.
