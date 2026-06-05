# Platform portability — a Redox-style audit (zic-rs)

> **Status: audit-confirmed documentation front door — not a new milestone and not a behaviour
> change.** This is a reviewer-facing summary of *already-true* facts (a read-only audit of the crate),
> drafted early like [`differences-from-reference-zic.md`](differences-from-reference-zic.md); it adds
> no new claims beyond existing code/tests. Its formal home is the release-ecology milestone (**T16.4**);
> the cross-platform CI + tests live in **T17**, the container/embedded bundles in **T21**.
>
> **Framing:** this is a *Redox-style portability audit* — "knowing exactly which layer needs Unix and
> keeping the rest independent." It is **not** a claim of Redox support (see Non-claims).

## 1. Claim

**zic-rs has a platform-neutral compiler core and platform-specific install surfaces.** The core that
turns tzdata source into TZif bytes is pure computation and runs anywhere Rust runs; the OS-specific
behaviour is confined to the output/install layer and to the (optional) conformance oracle.

## 2. Platform-neutral core

The production pipeline is pure — **no filesystem, environment, or process access**:

```
explicit source bytes → parse → model → transitions → TZif bytes
```

Audit evidence: zero `std::fs` / `std::env` / `std::process` in `src/source/`, `src/model/`,
`src/compile/`, and `src/tzif/`. The library entry points `compile_zone[_styled]` /
`compile_zone_to_bytes[_styled]` (`src/lib.rs`) take a parsed `Database` and return `TzifData` / bytes
with no side effects. There is **no `build.rs`** and no build-time host dependency. Pinned by
`tests/portability.rs` (compile explicit source *bytes* → TZif *bytes* entirely in-process — no file,
no env, no spawn).

## 3. Host-dependent conformance tools (segregated)

The **conformance / oracle** surfaces — `compare` and `structural-report` — may shell out to reference
`zic`/`zdump` and use a temp directory (`std::process::Command`, `std::env::temp_dir()` in
`src/compare/` and `src/structural.rs`). These are **verification** tools, host-dependent **by design**.
**Production `compile` requires none of them** — no reference `zic`, no `zdump`, no host tzdata, no
network.

## 4. Platform-specific install surfaces

All OS-specific behaviour lives in the output layer (`src/fs/output_tree.rs`), behind **4
`#[cfg(unix)]` gates**, each with a `#[cfg(not(unix))]` arm that **fails closed** with a clear
diagnostic (never a silent no-op):

- **symlink** link mode (`std::os::unix::fs::symlink`) — Unix only.
- **chmod / `--mode`** (`PermissionsExt::set_permissions`) — Unix only.
- **ownership / `-u`** — deferred, Unix-only privileged install metadata (no flag yet).
- **localtime materialization** (`-l`/`-t`) — written only under `--out` (a safe relative name).
- **system paths** — never read implicitly; output goes only under the explicit `--out`.

## 5. Current behaviour matrix

Feature × platform (✅ works · ⚠ explicit-error / unsupported · ▷ deferred · ◇ host-dependent):

| Feature | Linux | macOS | BSD | Windows | Redox / other |
|---------|-------|-------|-----|---------|---------------|
| compile source → TZif **bytes** (library) | ✅ | ✅ | ✅ | ✅ | ✅ (target) |
| write regular TZif **files** (`--out`) | ✅ | ✅ | ✅ | ✅ | ✅ (target) |
| **copy** link mode (default) | ✅ | ✅ | ✅ | ✅ | ✅ (target) |
| **symlink** link mode | ✅ | ✅ | ✅ | ⚠ error | ⚠/unknown |
| `-m` / `--mode` (chmod) | ✅ | ✅ | ✅ | ⚠ error | ⚠ error |
| `-u` ownership | ▷ deferred (Unix-only) | ▷ | ▷ | ⚠ | ⚠ |
| `-l`/`-t` localtime (under `--out`) | ✅ | ✅ | ✅ | ✅ | ✅ |
| reference-`zic`/`zdump` oracle (`compare`/`structural-report`) | ◇ | ◇ | ◇ | ◇ | ◇ |

**Portable baseline** (most likely to work everywhere, incl. containers/embedded/non-Unix): *copy
links · regular files · no chmod/chown · explicit `--out` · no implicit system install.*

## 6. Non-claims

- **No Redox support claim.** This is a portability *audit*, not a supported target.
- **No Windows install-parity claim.** Symlink/chmod/ownership are Unix-only and fail closed elsewhere.
- **No universal filesystem-parity claim.** Install surfaces are platform-specific by design.

## 7. T16 / T17 / T21 handoff

- **T16** (release ecology): this doc + the host-assumption inventory live here.
- **T17** (reliability): cross-target `cargo check` (`x86_64-unknown-linux-musl`, Windows, macOS;
  `wasm32-wasip1` library-only if feasible) to prove the core builds platform-independently while
  install surfaces stay `cfg`-gated; the no-host-dependence test.
- **T21** (embedded/container): the copy-mode portable baseline for minimal, no-host-contamination
  bundles.

> **Doctrine:** the compiler core is portable; install features are platform-specific; unsupported
> install features fail closed or are unavailable; reference-`zic` verification is **not** required for
> production compilation; the default safe path is *explicit-source → TZif bytes / staged output*, not
> system installation.
