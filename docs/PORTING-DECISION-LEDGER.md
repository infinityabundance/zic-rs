# Porting Decision Ledger

> The load-bearing design choices of the port — and, deliberately, the **tempting shortcuts that were
> declined**. Careful planning is legible not from what was built but from what was *refused*: each row
> rejected an easier option that would have produced a false or inflated claim. Pairs with
> `docs/PORTING-PHILOSOPHY.md` (the why) and `docs/DIVERGENCE-REGISTER.md` (the resulting differences).

| Decision | Options considered | Chosen | Why (lie it prevents) | Risk accepted | Evidence |
|---|---|---|---|---|---|
| `time_t` model of the reference | infer-from-host · unknown-unless-measured · oracle-measured | **unknown unless measured** | a host 64-bit `time_t` is not the reference's → don't fake portability | lower apparent completeness | `ReferenceBuildProfile` (T16.2), `BuildAxisEvidence::InferredForbidden` |
| Vendor/QEMU lab location | in-core-repo · external-lab-admitted · ignore ecology | **external lab; core admits receipts only** | core stays small/deterministic; "we ran a VM in the compiler repo" is not a compiler claim | harder reproduction (lab is separate) | `vendor-oracle-receipt-v1` + `from_json` admit (T16.5); non-claim `does_not_ship_or_operate_vendor_qemu_labs_in_core_repo` |
| `zone.tab` role | compile-input · semantic-witness · auxiliary-table | **auxiliary table (policy/index)** | `zone.tab` is geography, not Rule/Zone/Link → category error if compiled | needs a separate validator | `aux-table-validate` / `ZoneTableKind` (T16.4) |
| Table-diagnostic code space | reuse `ZIC###` · separate space | **separate** | a table-structural finding is not a `zic`-grammar diagnostic | extra schema surface | `aux-table-validation-v1` (T16.4) |
| Kani harness shape | whole-parser entrypoint · reduced-surface helpers | **reduced-surface helpers only** | a non-convergent oversized harness is a *harness-design failure*, not a code verdict (`negative lying is overclaim`) | no single "the parser is proven" headline | 10 verified helper proofs; broad harnesses discarded (`audits/kani`, T23.kani.3f) |
| Proven RFC predicate vs enforcement | prove only · prove + enforce | **prove, then enforce in `rfc9636::validate`** | a proven predicate is not an enforced check | validator behaviour change (tested) | T23.kani.3f.enforce (7 format-validity tests) |
| Unsupported source syntax | approximate output · fail closed | **fail closed (`ZIC###`)** | approximate timezone output is a silent wrong-answer | rejects some inputs `zic` tolerates → bucket-3 | `ZIC001`/diagnostic contract; `not-yet-ready.md` |
| Default emit style | slim (byte-match `zic`) · fat (behaviour-matched) | **fat default; slim explicit (`-b slim`)** | byte parity claimed everywhere would be false (structural residuals exist) | default bytes differ from `zic` slim (labelled bucket-3) | `differences-from-reference-zic.md`; structural-report |
| CLI shape | bare-flag argv drop-in · sub-command + required `--out` | **sub-command + required `--out`** | a silent system-`zic` alias would surprise scripts / install unsafely | not a literal argv drop-in (by design) | `drop-in-compatibility-contract.md`; `T23.drop-in-gauntlet.1` |
| Usage-error exit code | match `zic` (1) · clap convention (2) | **clap usage = 2; operational = 1** | conflating usage vs operational failure hides the distinction | exit-code divergence vs `zic` (documented) | `cli-compatibility-policy.md`; drop-in matrix |
| `cargo-vet` exemptions | rubber-stamp all deps · audit a small honest tier | **20 first-party + trusted-import + 23 exempted UNAUDITED** (42 fully · 1 partial · 23 exempted; exempted 39→36→33→31→26→23) | "vet succeeded" ≠ "reviewed"; never imply review of unsafe/platform leaves; admit only what is fully understood, record host-vs-all-target reachability | supply-chain maturity gap stays visible | `audits/cargo-vet` (T23.cargo-vet.8; large syscall/serde/clap/proc-macro tiers deferred) |
| musl/Alpine recipe | mount the glibc binary in musl (fail=defect?) · musl-built · explicitly classify | **musl-built static + ABI boundary demonstrated** | a glibc-binary exec-fail in musl is an ABI mismatch, **not** a parity defect | host-cross-built ≠ Alpine-native (a further rung) | `T23.drop-in-gauntlet.3.alpine` (container + QEMU VM) |
| Admitted source bytes | vendor tarballs into the repo · hash-pin + signature only | **fetch + verify-sig + hash-pin; bytes ephemeral** | reproducibility without a copyrighted/heavy in-repo mirror | bytes must be re-fetched | T12.5a.2; `T23.release-diff-real.1` (GPG-verified 2026a/2026b) |
| Milestone numbering | parallel ladders (e.g. `KANI.N`/`VET.N`) · one contiguous T-sequence | **one T0→T23; sub-work subordinate (`T<n>.<thing>.<k>`)** | a second numbering namespace implies a second authority structure | rigid numbering discipline | the plan ladder; `[[working-standards]]` |
| Archive vs authority | promote archive copy to source · canonical=authority, archive=witness | **archive is a preservation witness only** | an archived copy is not the authoritative source | none (purely honesty) | T18 knowledge index; `claim-source-map.md` |
| Install durability | claim whole-tree atomicity · per-file durable + named residual | **per-file crash-durable (Unix); whole-tree refused** | a whole-tree-atomic claim would be false | named residual (no tree transaction) | `install-materialization-contract.md` (T17.4); `RISK.INSTALL.1` |

## How to read this ledger

Every row is a place the project could have looked *more* complete by taking the easier option, and chose
the honest one instead. The accepted-risk column is the cost of that honesty; the evidence column is where to
verify the choice was actually implemented, not merely asserted. New major decisions append a row here in the
same batch as the code/receipt that makes them real.
