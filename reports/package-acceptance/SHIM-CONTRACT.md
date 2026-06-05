# SHIM-CONTRACT.1 — `zic-shim-v1` compatibility contract & threat model — 2026-06-05

> `zic-shim.sh` is the bridge that lets zic-rs occupy the `$(ZIC)` slot of a distro tzdata package build
> **without editing the package Makefile** (zic-rs is deliberately not a literal argv drop-in — subcommand
> CLI, `--out` not `-d`). Because PACKAGE-ACCEPTANCE.1 relies on it, the shim is a **first-class compatibility
> surface** with its own versioned contract, conformance test (`shim-test.sh`), and threat model — not test
> glue. **It adds no capability zic-rs does not already have**; it only re-shapes argv.

## Version

`zic-shim-v1`. The contract below is stable within v1; a breaking change bumps the version (and the header).

## Accepted flags (translated to zic-rs)

| reference `zic` | → zic-rs | notes |
|---|---|---|
| `-d <dir>` | `--out <dir>` | the output root (required for a build) |
| `-L <leapfile>` | `--leapseconds <leapfile>` | the `right/` leap profile |
| `-l <zone>` | `--localtime <zone>` | localtime target zone |
| `-t <name>` | `--localtime-name <name>` | localtime link name (zic-rs constrains it — see below) |
| `-b <slim\|fat>` | `--bloat <slim\|fat>` | emission style |
| `-m <mode>` | `--mode <mode>` | octal file mode |
| `-D` | `--no-create-dirs` | do not create the output dir |
| `<file>` (non-flag) | `--input <file>` | one `--input` per positional; **order preserved** |

zic-rs is always invoked as `compile … --all-supported --unsupported skip --out <dir>`.

## Refused / ignored flags (recorded, never silently mis-translated)

| flag | shim behaviour | why |
|---|---|---|
| `-p <rules>` | **ignored** (arg consumed) | reference `zic` itself deprecates `-p` ("obsolete and likely ineffective") |
| `-y <cmd>` | **ignored** (arg consumed) | the legacy `yearistype` command-name selector; not a package-build need |
| `-r <range>` | **ignored** (arg consumed) | range truncation is not part of a normal package build |
| `-v` | **dropped** | verbosity; zic-rs warnings still print |
| `--version` | forwarded to `zic-rs --version` | |
| any other `-X` | **ignored** (single token) | forward-compat: an unknown flag never crashes the build |

## Behavioural guarantees (the contract — asserted by `shim-test.sh`)

1. **Argument ordering.** Flags and files may interleave; each `-X` consumes exactly its value token; every
   non-flag token becomes an `--input` in **the order given** (input order is the order-sensitive identity
   axis — see T12.3).
2. **Multiple input files.** All positionals are passed as repeated `--input`; none is dropped or reordered.
3. **Error propagation.** The shim `exec`s zic-rs, so **zic-rs's exit status is the shim's exit status**
   (bad input → exit 1; usage → exit 2). No status is swallowed or rewritten.
4. **stdout/stderr policy.** `exec` means zic-rs's stdout (the per-zone compile log) and stderr (diagnostics)
   pass through **unaltered**; the shim writes nothing of its own.
5. **No shell-glob / word-split surprises.** Every argument is quoted (`"$2"`, `"$@"`, `set --`); the shim
   never re-expands or globs a value. A path containing spaces is passed verbatim.
6. **Path safety is preserved, not bypassed.** The shim passes paths straight to zic-rs; **zic-rs's
   output-root safety (`ZIC008`) still applies** — an absolute/`..`/outside `-t` is **refused by zic-rs**, and
   the shim does not (and cannot) relax that. (See LOCALTIME-STAGING below.)
7. **Ambient-environment quarantine.** The shim reads **exactly one** environment variable — `ZICRS` (the
   zic-rs binary path, default `zic-rs`). It reads no `TZ`, `LC_*`, `TZDIR`, `PATH`-derived data, or any other
   ambient state; behaviour depends only on argv + `ZICRS`.

## Threat model

- **Capability:** the shim is a pure argv translator — it cannot make zic-rs do anything zic-rs's own CLI
  cannot. In particular it **cannot write outside `--out`** (zic-rs's `ZIC008` boundary is unchanged), cannot
  execute input data, and cannot escalate the localtime `-t` policy.
- **`ZICRS` trust:** a hostile `ZICRS` value would run an arbitrary binary — but that is identical to a
  hostile `PATH`/`$(ZIC)` in any Makefile and is the operator's responsibility, not a new exposure.
- **Injection:** all values are passed as separate argv elements (`exec "$ZICRS" "$@"`), never through a
  shell string, so a value like `; rm -rf /` is just a (rejected) input path, not a command.
- **Failure mode:** an unknown flag is ignored (build proceeds) rather than aborting — a deliberate
  forward-compat choice so a future reference `zic` flag does not break the slot; the build's correctness is
  still gated by the zone-tree comparison, not by the shim accepting every flag.

## LOCALTIME-STAGING (the sanctioned `-t` recipe)

zic-rs **refuses** an absolute/outside `-t` (e.g. `$DEST/etc/localtime`) — `ZIC008` — and **accepts a safe
relative name under `--out`**. The blessed replacement pattern for a staged `DESTDIR` build:

```sh
# reference:
zic -d "$TZDIR" -l Factory -t "$DEST/etc/localtime"
# zic-rs (via the shim): materialise localtime as a safe relative name UNDER the output root,
zic-shim.sh -d "$DEST/usr/share/zoneinfo" -l Factory -t localtime tzdata.zi
# then the PACKAGE SCRIPT places/links it within DESTDIR (a package-policy step, never the compiler's job):
ln -sf ../usr/share/zoneinfo/Factory "$DEST/etc/localtime"   # or cp, per distro policy
```

The message for maintainers: **zic-rs can do localtime — it just refuses the dangerous *shape* and leaves the
"write to `/etc`" step to the package script, where it belongs.**

## Non-claims

- `zic-shim-v1` is verified against the **canonical tzcode build pattern** (PACKAGE-ACCEPTANCE.1) + the
  conformance test; it is **not** claimed to translate every historical `zic` flag combination.
- The shim is a **packaging-integration** artifact, not part of the zic-rs crate's public API.
