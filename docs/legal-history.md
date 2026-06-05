# Legal history & provenance posture (brief)

This is short by design and makes **no legal claims beyond its sources**. It exists so the
project's provenance posture is recorded, not to argue law.

## Context

The IANA Time Zone Database is distributed today as **public-domain** code and data,
coordinated through the IANA process ([RFC 6557](https://www.rfc-editor.org/rfc/rfc6557.html)
/ BCP 175). Its history is not frictionless: in 2011, *Astrolabe, Inc. v. Olson* (see the
[EFF case page](https://www.eff.org/cases/astrolabe-v-olson)) briefly disrupted the project
before the suit was dropped — part of why the dataset's open, public-domain availability is
treated as something to preserve carefully rather than assume.

## What this means for zic-rs

* **Provenance is preserved.** Fixtures record their source release and the reference
  toolchain (`fixtures/MANIFEST.toml`); the roadmap (T3.4) adds a per-compile release manifest
  so any output is traceable to its tzdb release.
* **No proprietary data.** zic-rs ingests only IANA tzdata (and tiny hand-written fixtures of
  the same form). It does **not** bundle, depend on, or compile any proprietary timezone
  dataset.
* **No editorialising.** Consistent with [tzdb-governance.md](tzdb-governance.md), zic-rs
  compiles the data as-is and does not curate civil-time history.

That is the whole point: respect the dataset's open status and keep the chain of custody from
IANA release → compiled TZif visible.
