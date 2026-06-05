# Hostile-output-tree (TOCTOU) boundary (T14.6)

> **Honest trust boundary.** T9 guarantees *no partial install* under **normal** filesystem errors.
> T14.6 classifies what happens when the output tree is **hostile / pre-mutated**, pins the cases that
> are deterministically testable with std-only, `#![forbid(unsafe_code)]` primitives, and states plainly
> what is **not** claimed. Executable witness: `tests/hostile_output_tree.rs`.
>
> **The one-sentence claim:** *zic-rs fail-closes the covered hostile-output cases (pre-existing
> file/symlink/dir at the leaf is never written through) and preserves no-partial-install under normal
> filesystem errors, but full concurrent hostile-tree TOCTOU resistance is **not** claimed unless
> fd-relative (`openat`/`O_NOFOLLOW`) materialization is implemented and tested.*

## Why the covered cases hold — the materialization primitives

From `src/fs/atomic_write.rs` + `src/fs/output_tree.rs`:

* **Default publish = `hard_link(tmp → target)`** — an *atomic exclusive create*: it fails `EEXIST` if
  the leaf already exists, **including when the leaf is a symlink, without following it**. So a planted
  file/symlink/dir at the output leaf fails closed and is never written *through*.
* **Temp file = `create_new(true)`** (`O_EXCL`) — a planted symlink/file at the temp path also fails
  closed; a crash-leftover temp surfaces as an error, never a silent truncate.
* **`--force` publish = `rename(tmp → target)`** — atomically *replaces* whatever is at the leaf
  (including a symlink) with the freshly-compiled regular file; it does not follow the symlink, so the
  victim is still not written through.
* **Compile-all-to-memory then materialise** (T9.3) — a fatal in any selected zone aborts before the
  write phase, so a hostile/normal failure leaves no partial tree.

## Status vocabulary (typed-or-planned-typed)

`Covered` (demonstrated by a test) · `FailClosed` (errors before any write-through) · `PlatformDependent`
· `NotClaimed` · `RequiresOpenatStyleHardening` (a race only closable with fd-relative primitives).

## The ledger

| Hostile condition | zic-rs behaviour | Layer | Status | Test |
|-------------------|------------------|-------|--------|------|
| Pre-existing **file** at the leaf (no `--force`) | `EEXIST` → fail closed; file untouched | operational | `Covered`/`FailClosed` | `preexisting_file_at_leaf_is_fail_closed_no_clobber` |
| Pre-existing **symlink** at the leaf (no `--force`) | `hard_link` `EEXIST` → fail closed; **victim not written through**; symlink left intact | operational | `Covered`/`FailClosed` | `preexisting_symlink_at_leaf_is_not_followed_through` (unix) |
| Pre-existing symlink at the leaf **with `--force`** | `rename` **replaces** the symlink with the compiled file; **victim not written through** | operational | `Covered`/`FailClosed` | `force_replaces_symlink_without_writing_through_it` (unix) |
| **Directory** occupying the leaf path | can't `hard_link`/`rename` over a dir → fail closed | operational | `Covered`/`FailClosed` | `target_leaf_is_a_directory_is_fail_closed` |
| A **file** where a parent dir component is needed | `create_dir_all` errors → fail closed, no write | operational | `Covered`/`FailClosed` | `parent_component_is_a_file_is_fail_closed` |
| Pre-existing temp-name collision / planted temp symlink | `create_new` (`O_EXCL`) → fail closed | operational | `Covered` (by construction) | — (primitive-level) |
| No-partial-install under **normal** fs errors (fatal zone, unreadable input, …) | compile-all-to-memory → abort before write phase | operational | `Covered` | `clean_*` baseline + T9.3 `*_leaves_no_partial_install` |
| **Parent-component symlink swap *mid-run*** (between `create_dir_all`/validate and write) | `create_dir_all`/`open` resolve parent symlinks at syscall time; std cannot close this race portably | operational | **`NotClaimed` / `RequiresOpenatStyleHardening`** | — (not faked) |
| **Leaf swapped to a symlink *during* the write window** (true concurrent race) | the `hard_link`/`rename` is atomic at the leaf, but the *parent path* resolution is not fd-pinned | operational | **`NotClaimed` / `RequiresOpenatStyleHardening`** | — |
| **Permissions removed mid-run** | surfaces as a normal `io` error → fail closed (no partial); exact timing is OS-scheduling-dependent | operational | `PlatformDependent` (normal-error safety `Covered`) | — |
| `--link-mode symlink` exists→remove→create window | the default `Copy` mode uses the race-free `hard_link` path; the opt-in symlink mode has a small check-then-act window | operational | `NotClaimed` for the symlink-mode window (default mode `Covered`) | — |

## What T14.6 does NOT claim (the trust boundary, stated plainly)

* **Not** "secure against a malicious output directory under concurrent mutation." The covered cases are
  *pre-mutated tree* (deterministic) and *atomic leaf publish*; a genuine *race* on a **parent path
  component** mid-run is not closed, because std's `create_dir_all`/`OpenOptions`/`rename` resolve parent
  symlinks at syscall time and zic-rs does not (yet) materialise fd-relative (`openat` + `O_NOFOLLOW`,
  walking the tree by file descriptor). That hardening would need either `unsafe` raw-fd syscalls
  (forbidden by `#![forbid(unsafe_code)]`) or a vetted dependency (`cap-std`/`openat`) — a deliberate
  future decision, not a silent gap.
* **Not** a claim of resistance on non-Unix targets — symlink semantics there are `PlatformDependent`
  (cross-link `docs/platform-portability.md`); the symlink tests are `#[cfg(unix)]`.
* The default **Copy** link mode is the race-safer baseline (hard_link `O_EXCL`); the opt-in **symlink**
  link mode's exists→remove→create window is explicitly `NotClaimed`.

## Acceptance (T14.6)

> Met: the hostile-output-tree boundary is recorded (this ledger, typed statuses) and tested where
> feasible — pre-existing file/symlink/dir at the leaf and a file blocking a parent dir all fail closed
> (the symlink victim is never written through, with or without `--force`); the T9 no-partial-install
> guarantee under normal errors is preserved; and the genuine concurrent-race cases are honestly marked
> `NotClaimed`/`RequiresOpenatStyleHardening`, not faked. Future fd-relative hardening is named, deferred
> to **T17/T20** (reliability/security), and gated behind the `unsafe`/dependency decision.

## Non-claims (T14.6)

* No full concurrent-TOCTOU resistance (parent-component swap mid-run) — deferred, `RequiresOpenatStyleHardening`.
* No non-Unix output-safety parity (symlink/perms semantics platform-dependent).
* No new dependency or `unsafe` added in T14.6 — it classifies + tests the *current* std-only behaviour
  honestly; the `openat`-style upgrade is a separate, named future step.
