# `schemas/` — published JSON Schemas for zic-rs's machine-readable surfaces (T17.7)

One JSON Schema (Draft 2020-12) per public `schema` id. These are the **published contract documents** for
the JSON that zic-rs emits (reports + the compile manifest/alias-map) and for the one JSON it **ingests**
(the vendor-oracle receipt). They are governed by [`../docs/schema-compatibility-policy.md`](../docs/schema-compatibility-policy.md).

## The registry (10 surfaces)

| Schema file | Emitted/ingested by |
|---|---|
| `zic-rs-compile-manifest-v8.schema.json` | `compile --manifest` |
| `zic-rs-alias-map-v1.schema.json` | `compile --alias-map` |
| `zic-rs-support-report-v4.schema.json` | `support-report` |
| `zic-rs-structural-report-v3.schema.json` | `structural-report` |
| `zic-rs-semantic-report-v1.schema.json` | `semantic-report` |
| `zic-rs-tzif-validation-v1.schema.json` | `tzif-validate` |
| `zic-rs-aux-table-validation-v1.schema.json` | `aux-table-validate` |
| `zic-rs-release-diff-v1.schema.json` | `release-diff` |
| `zic-rs-doctor-v2.schema.json` | `doctor` |
| `vendor-oracle-receipt-v1.schema.json` | external lab → `vendor-oracle-admit` (interchange) |

## Fidelity scope (honest)

Each schema pins the **schema identity** (the `schema` const) and the **top-level object shape**. It does
**not** yet exhaustively constrain every nested field — the reports are hand-rolled deterministic JSON, and
over-specifying un-validated nested shapes would risk drift between schema and emitter. Field-level
tightening, and **full instance-validation** (emit a report → validate the bytes against the schema with a
JSON-Schema validator), are a tracked **audit-suite (T23) / T20** item: the core crate carries **no
validator dependency** (the no-new-deps posture), so instance validation runs in the external audit tooling,
not the default gate.

## What *is* gated now (dep-free)

[`../tests/schemas.rs`](../tests/schemas.rs) is the **registry / drift guard** (no validator needed):
- **completeness** — every emitted/ingested `schema` id has a published schema file here;
- **no orphans** — every `schemas/*.schema.json` maps to a real emitter;
- **drift** — each id literal is still present in its emitter source, so bumping an emitter (e.g.
  `…-v4`→`…-v5`) without updating the registry + schema file **fails the test**;
- each schema file declares its own id.

This turns "the schemas exist and match the code" into a test, while keeping full instance-validation as an
explicit future audit step rather than a faked-now claim.
