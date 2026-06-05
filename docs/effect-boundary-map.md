# Effect-boundary map (T19)

> Which parts of zic-rs are **pure** and which **touch the host** (filesystem, external process, env) —
> classified per module, so a security/audit reader (and the T20 personas) can see exactly where the
> trust boundaries are. Verified by grepping `std::fs` / `std::process::Command` / `std::env` across
> `src/` (this is an *audit-confirmed* map of already-true facts, not an aspiration); pairs with
> `docs/platform-portability.md`, `docs/panic-policy.md`, and `docs/risk-register.md`.

## Effect classes

`PureModel` · `BoundedParser` (pure over untrusted bytes) · `FilesystemRead` · `FilesystemWrite` ·
`ExternalProcess` · `HostProbe` · `HostEnvRead` · `ReportRendering` · `LabOnly`.

> **The load-bearing fact:** the **production compile path is pure** — `source bytes → parse → model →
> compile → TZif bytes` reads no file, runs no process, and reads no env. Host effects are confined to (a)
> the explicit `--out` **materialization** layer, (b) the **oracle/conformance** layer (running reference
> `zic`/`zdump`), (c) the **doctor** host probe, and (d) the **report-layer** file reads (hashing emitted
> output, reading inputs to summarise). Each is named below.

## Module map

| Module | Effect class | Claim surface | Host dependency | Panic / resource posture |
|---|---|---|---|---|
| `src/source/{lexer,parser,names,records,leap}` | **BoundedParser** | source → `Database` | **none** (pure over `&[u8]`) | typed `Err`, never panic (T14); `MAX_LINE_LEN` cap |
| `src/model/` | **PureModel** | the record/type model | none | pure |
| `src/compile/` (transitions, leap, posix_footer, …) | **PureModel** | `Database` → `TzifData` | none | typed `Err`; `MAX_TRANSITIONS`; `ZIC023` simultaneous-transition guard |
| `src/compile/plan.rs` | **FilesystemWrite** | the install/materialization run | writes under `--out` only | T9.3 no-partial-install; T17.4 durable publish |
| `src/tzif/` (writer · `validate::parse` · `rfc9636::validate(bytes)`) | **PureModel** / **BoundedParser** | TZif emit + read | none (the pure paths) | bounds-safe `parse` (T17.1a/T17.5), never panics on bytes |
| `src/tzif/rfc9636.rs` (multi-zone *report*) | **ReportRendering** + **FilesystemRead** | `tzif-validate` report | reads input files to validate | report path; `validate(bytes)` itself is pure |
| `src/fs/output_tree.rs`, `src/fs/atomic_write.rs` | **FilesystemWrite** | materialization choke point | writes/symlinks under `--out` (`cfg(unix)` gates) | `ZIC008` traversal reject; exclusive-create; durable publish (T17.4) |
| `src/lib.rs` (`load_database`, `collect_source_files`) | **FilesystemRead** | reading caller-named source files | reads `--input` paths only (never implicit `/usr/share/zoneinfo`) | `limits::ResourceLimits` caps (T17.1b) |
| `src/compare/reference_zic.rs`, `src/compare/zdump.rs`, `src/compare/mod.rs` | **ExternalProcess** + **FilesystemRead/Write** (scratch) | the oracle (`zic`/`zdump`) | runs reference tools into a temp dir; **never on the compile path** | oracle-only; absence → `OracleMode::Unavailable`, not a silent pass |
| `src/semantic_witness.rs` | **ExternalProcess** + **ReportRendering** + **HostEnvRead** | `semantic-report` (`zdump` witnesses) | runs `zdump`; records oracle env (`TZ`/`LC_ALL`) | absence visible; deterministic witnesses |
| `src/doctor.rs` | **HostProbe** + **ExternalProcess** + **FilesystemRead** | `doctor` report | probes `zic`/`zdump` `--version` + hashes them; optional `--tzdata` read | always exit 0; typed `ToolVersionStatus`/`HashReadStatus` (T17.3) |
| `src/report.rs`, `src/structural.rs`, `src/aux_tables.rs`, `src/release_diff.rs`, `src/manifest.rs` | **ReportRendering** (+ scoped **FilesystemRead**) | the public reports + manifest | read emitted output / inputs to summarise | deterministic JSON; typed enums (T17.2); `release-diff` `OracleFailureScope` (T17.3) |
| `src/diagnostics.rs`, `src/error.rs`, `src/hash.rs`, `src/json.rs`, `src/limits.rs` | **PureModel** | diagnostics · errors · in-house SHA-256 · JSON escaper · caps | none | pure |
| `src/cli.rs`, `src/main.rs` | thin shell (**FilesystemRead/Write** + **HostEnvRead** via the above) | arg parsing → library | delegates; the 0/1/2 exit contract | `docs/cli-compatibility-policy.md` |
| `../zic-rs-vendor-oracle-lab/`, `fuzz/` | **LabOnly** | external evidence generation / fuzzing | runs VMs/containers / nightly+libFuzzer | **never in the core crate / never on the default gate** |

## What this buys the reader

- **The compile path needs no host tool.** Production `compile` (source bytes → TZif bytes / staged
  `--out`) requires neither reference `zic`/`zdump` nor network nor host tzdata — only the *conformance*
  path (`compare`/`semantic-report`/`structural-report` with `--reference-*`) does. (`docs/platform-portability.md`.)
- **No implicit host reads.** The library never silently reads `/usr/share/zoneinfo` or env; all input is
  caller-explicit (`--input`/`--out`/`--reference-*`/`--tzdata`).
- **`unsafe`-free, no `build.rs`.** The only platform-specific code is the `cfg(unix)` gates in
  `output_tree.rs` (symlink/`chmod`), each failing closed off-Unix.

## Non-claims

- This map states *which effects exist where*, not that every effect is hardened to every threat — the
  per-risk status is `docs/risk-register.md` (e.g. the parent-component TOCTOU residual on the
  `FilesystemWrite` boundary is still `RequiresOpenatStyleHardening`).
- `ReportRendering` modules read files **to summarise/hash already-produced artifacts** — that is not the
  compile path and is not a host dependency of compilation.
