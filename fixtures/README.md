# Fixtures

Test inputs and their expected outputs, with provenance in [`MANIFEST.toml`](MANIFEST.toml).

```
fixtures/
├── minimal/     hand-written tzdata source, one construct at a time
│   ├── utc.zi   Zone Etc/UTC 0 - UTC  +  Link Etc/UTC UTC   (fixed offset + link)
│   ├── fixed.zi Zone Test/Fixed -5:00 - EST                 (fixed offset)
│   ├── dst.zi   finite US-style DST, Sun>=N, wall AT, %s     (T2)
│   ├── euro.zi  finite EU-style DST, lastSun, UT AT, %s      (T2)
│   ├── sle.zi   finite DST, Sun<=25, standard AT, %s         (T2)
│   └── eastern.zi recurring DST (max), POSIX footer          (T3)
├── expected/    byte-for-byte reference `zic` output (the oracle), pinned to tzcode 2026b
│   ├── Etc_UTC.tzif       (fixed-offset zones only — see below)
│   └── Test_Fixed.tzif
└── MANIFEST.toml  provenance for every fixture (source, zic command, parity, known diffs)
```

Only **fixed-offset** zones have pinned `expected/` blobs (byte parity). The DST and
recurring fixtures are verified by the **`zdump` behaviour oracle** over a declared year
horizon against the local reference `zic` (`zic-rs compare --mode zdump --horizon LO,HI`),
because `zic`'s type/abbreviation ordering and explicit-transition horizon differ from ours
while the local-time behaviour is identical.

## How the `expected/` blobs were produced

```sh
zic -d /tmp/ref fixtures/minimal/utc.zi fixtures/minimal/fixed.zi
cp /tmp/ref/Etc/UTC      fixtures/expected/Etc_UTC.tzif
cp /tmp/ref/Test/Fixed   fixtures/expected/Test_Fixed.tzif
```

Reference compiler: **`zic` (tzcode) 2026b**. These are the *only* fixtures for which we
assert **byte** parity, precisely because the bytes are pinned here. For everything else the
binding contract is **semantic** parity (see `docs/oracle-testing.md`).

## Naming

`expected/` files replace `/` with `_` in the zone name because they are flat files in one
directory; the live output tree (under `--out`) uses the real nested layout.
