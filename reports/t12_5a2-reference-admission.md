# T12.5a.2 — Source-Variant Reference Admission (gate lift for tzdb 2026b)

> **Type:** reference-admission receipt. This **lifts** the T12.5a.1 reference-pin gate
> (`reports/t12_5a1-reference-pin-requirement.md`) from **OPEN** to **lifted-for-2026b** by admitting
> the pristine IANA tzdb **2026b** reference set and recording its provenance + SHA-256 hashes. It is
> **admission only** — it does **not** implement or interpret any source-variant behaviour (that
> begins at T12.5b). No compiler behaviour, no manifest schema change (still v5).
>
> **Version-scoped (supply-chain discipline):** the lift applies **only** to the pinned 2026b
> reference below. A later release (2026c, 2027a, …) needs its own admission receipt; the gate is
> never "closed forever", only "lifted for *this* pinned reference".

## Archive provenance

| Field | Value |
|-------|-------|
| release | **2026b** (released 2026-04-22) |
| source_url | `https://data.iana.org/time-zones/releases/tzdb-2026b.tar.lz` (complete distribution) |
| transport | HTTPS (`data.iana.org`, Cloudflare), fetched 2026-05-31 |
| archive_sha256 | `ffad46a04c8d1624197056630af475a35f3556d0887f028ac1bd33b7d47dc653` |
| archive_bytes | 561562 |
| signature | `tzdb-2026b.tar.lz.asc` present (detached, OpenPGP) |
| signature_status | **verified** — `gpg --verify` reports **`GOODSIG` / `VALIDSIG`** by RSA key **`7E3792A9D8ACF7D633BC1588ED97E90E62AA7E34`** ("Paul Eggert <eggert@cs.ucla.edu>"). The signing key was imported from `keys.openpgp.org` and its primary-key fingerprint **matches the published IANA tz signing-key anchor** byte-for-byte. **Trust model:** fingerprint-anchored (the published tz fingerprint is the trust root), *not* OpenPGP web-of-trust — i.e. this proves the archive was signed by the key bearing the canonical tz fingerprint, which is the standard verification for tz releases. |
| extraction | `bsdtar -xf tzdb-2026b.tar.lz` (libarchive; `lzip` binary absent, `tar --lzip` unavailable) |
| local_path | `/tmp/zic-rs-tzdb-refpin/tzdb-2026b` — **ephemeral, NOT vendored into the repo.** Pinned **by hash**; re-verify by re-fetching from `source_url` and checking against the hashes here. (Vendoring a minimal reference subset for hermetic tests is a separate, later decision.) |

## Admitted reference files (SHA-256)

Files **present directly in the tarball**:

| File | sha256 | Consumed by |
|------|--------|-------------|
| `Makefile` | `0b4588ea467c969b23fc48335e91eb63f403574b4aac69380b84a00373c7e81d` | all of T12.5b–d (policy knobs) |
| `theory.html` | `8ad17f82587c12e8f22587e2c40105741c2389c93557b5023bcc040deb54587f` | T12.5b/d (backzone scope, DATAFORM prose) |
| `backward` | `d2f4c8953f204982ddf4dc0c2debf41b2464de376dad7d546d0fc70f889fa706` | T12.4d contrast / membership |
| `backzone` | `63fb39adae0b0d8b2179629725a9dfb694c7a386b99750b636a017d896d28dfa` | **T12.5b** (`PACKRATDATA`) |
| `zone.tab` | `4d8e389e5f4b0ec0466d5b14f42e5dfb0308c4376165fcf478339afd9ddcb00c` | **T12.5c** (`PACKRATLIST` subset target) |
| `zone1970.tab` | `406555546e685b34eb46c24d826b649dd35e9d202f4c13a3c621ff21eddc1583` | T12.5c (post-1970 selection table) |
| `leapseconds` | `d6c8c0330527d995de5e50af9c07b929fa9d69d92e723cfb623b4ae6fae6d3b5` | T11 `right/` cross-check |
| `tzdata.zi` (**pristine 2026b**) | `e500467b589740d2c0311c844896f2f3727ffc4baff7e82d65d0f8c1e7e6f0c1` | T12.5d (default `DATAFORM=main` artifact) |

Files **generated from the pinned tree** (`ziguard.awk` transform — *not* shipped in the tarball;
provenance = archive + `Makefile` + generation command + toolchain, deterministic):

| File | sha256 | Generation |
|------|--------|------------|
| `vanguard.zi` | `49e16da4a6252a2e432fc1f68bf6daac9a6f73507dde3e3bdbcbbf78e86727ce` | `make vanguard.zi` |
| `main.zi` | `e0225823ae0c3a99a016a4afd7e3c48cfd948132b65fbaa596a47c53ae45e4e1` | `make main.zi` |
| `rearguard.zi` | `91c4f362a6bb297efd3cd35bce6b62367a4c00a9721a773bae0cbb0d1bf9fe23` | `make rearguard.zi` |

- generation_command: `make vanguard.zi main.zi rearguard.zi`
- toolchain: GNU Make 4.4.1, GNU Awk 5.4.0 (record because the `.zi` witnesses are *derived*, so the
  generating toolchain is part of their provenance; a different awk could in principle differ).
- **`ziguard.awk` sha256: `e4600a2360b692242d6da76666411ece8ada76b61e6f8fb69cec79592b261785`** (the
  transform itself; ships *inside* the tarball so it is already bound transitively by `archive_sha256`,
  but pinned explicitly here for the **T12.5d `DATAFORM` `recipe_hash`** — which binds
  `archive_sha256` · `Makefile` sha256 · this `ziguard.awk` sha256 · `generation_command` · `toolchain`,
  all hashed as **raw bytes, never line-ending-normalized**). The three `.zi` rows above are the
  `DATAFORM` detection artifacts (T12.5d): an admitted source whose hash equals one of them is
  hash-backed evidence of that encoding form.

## Byte-confirmed Makefile facts (upgrading the T12.5a inventory from documented → pinned)

From the admitted `Makefile`:
- `DATAFORM = main` (default; vanguard/main/rearguard are the three forms).
- `BACKWARD = backward` (backward links included by default).
- `PACKRATDATA =` and `PACKRATLIST =` (both **empty by default** → no backzone).
- The subset-vs-all distinction is explicit: `PACKRATDATA=backzone PACKRATLIST=zone.tab` (selected
  entries) vs `PACKRATDATA=backzone PACKRATLIST=` (all backzone data).
- `vanguard.zi main.zi rearguard.zi: $(DSTDATA_ZI_DEPS)` via `ziguard.awk -v DATAFORM=$(@:.zi=)`;
  `tzdata.zi: $(DATAFORM).zi … zishrink.awk`.

This confirms (by bytes, not memory) the two-axis taxonomy and the default profile recorded in
`docs/build-profile-parity.md` → "Source Variant Policy Inventory (T12.5a)".

## Pristine vs installed (why detected-vs-claimed matters — now empirically pinned)

- pristine `tzdata.zi` (2026b, this archive): `e500467b589740d2c0311c844896f2f3727ffc4baff7e82d65d0f8c1e7e6f0c1` (`# version 2026b`).
- installed `/usr/share/zoneinfo/tzdata.zi`: `0078657fd0b768650be3943cba2b668390395b60a0989836652767af6334c371` (`# version 2026b-dirty`).
- **They differ.** The installed copy is *not* the pristine release — exactly the case the manifest's
  detected-vs-claimed reconciliation (T12.2) exists to catch. T12.5d must compare against the **pristine**
  `tzdata.zi`/`main.zi`/etc., never the dirty installed file.

## Gate status

**`source_variant_reference_pin_gate = lifted_for_2026b`.** T12.5b–d are **unblocked for the pinned
2026b reference set only**. Still **NOT implemented** — admission ≠ implementation; no backzone /
PACKRATLIST / DATAFORM / rearguard / vanguard behaviour or evidence-detection exists yet, and none is
claimed. The T12.5a.1 receipt is updated to point here.
