# cargo-vet receipt — 2026-06-02 — T23.cargo-vet.2 (trusted-audit imports)

- **Tool:** `cargo-vet`. **Host:** x86_64, Linux. **zic-rs:** 0.1.0.
- **Commands:** `cargo vet import mozilla <url>` · `… google <url>` · `… bytecodealliance <url>` → `cargo vet` → `cargo vet prune`. **Exit:** 0.
- **Imports pinned in** `supply-chain/imports.lock` (623 lines; the imported audit entries are pinned for reproducibility — not re-fetched on each run).

## Result — exemptions shrunk by delegated trust; before/after recorded

| State | Fully audited (via import) | Partially audited | Exempted (UNAUDITED) |
|---|---:|---:|---:|
| **T23.cargo-vet.1** (init only) | 0 | 0 | **66** |
| **T23.cargo-vet.2** (after imports + prune) | **22** | **1** | **43** |

**23 of the 66 third-party dependencies now carry trusted third-party review** (the exact crate+version
entries published by the imported sources); the remaining **43 stay explicitly `exemptions` = UNAUDITED.**
`cargo vet` succeeds with this mix (22 fully audited · 1 partial · 43 exempted), and `prune` removed the
now-unneeded exemptions so `config.toml` reflects reality.

## Imported trusted sources (the trust decisions — scoped + evidenced)

| Import | Source (pinned in `imports.lock`) | What it means |
|---|---|---|
| `mozilla` | `github.com/mozilla/supply-chain/main/audits.toml` | delegated trust in Mozilla's published reviews |
| `google` | `github.com/google/supply-chain/main/audits.toml` | delegated trust in Google's published reviews |
| `bytecodealliance` | `github.com/bytecodealliance/wasmtime/.../audits.toml` | delegated trust in the Bytecode Alliance's reviews |

## Boundary · non-claims (read this — it is the whole point)

- **These are DELEGATED-trust imports, not first-party zic-rs audits.** "22 fully audited" means *Mozilla /
  Google / Bytecode-Alliance reviewers audited those exact crate versions and we chose to trust their
  published reviews* — **zic-rs did not itself read that source.** The trust decision is *trust in those
  three review organizations*, scoped to exactly the entries in their pinned `audits.toml`.
- **No dependency was silently promoted.** The 43 still-exempted crates remain **explicitly UNAUDITED**;
  nothing is marked trusted without an import entry or (future) a first-party audit.
- **Vetted ≠ bug-free.** cargo-vet records *review*, not correctness.
- **T23.cargo-vet.3 (next, on demand):** first-party audits of the highest-value still-exempted crates (the ones
  closest to the trust boundary — e.g. `tempfile`'s `rustix`/`getrandom` path) → shrink the 43 further, each
  with a real `audits.toml` entry + scope; and/or add more reputable imports where appropriate.
- **Status:** ◐ **RUN (T23.cargo-vet.2) — 23/66 deps now audited via 3 pinned trusted imports; 43 still exempted/unaudited.** No first-party audit performed; no "supply chain verified" claim.
- **Cross-reference:** `RECEIPT-2026-06-02.md` (T23.cargo-vet.1) · `docs/security-rewrite-evaluation.md` §D · `docs/maintenance-policy.md` §5.
