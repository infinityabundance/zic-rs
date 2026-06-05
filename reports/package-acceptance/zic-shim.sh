#!/bin/sh
# zic-shim-v1 — zic argv-compat shim -> zic-rs (PACKAGE-ACCEPTANCE.1 / SHIM-CONTRACT.1).
# CONTRACT + THREAT MODEL: reports/package-acceptance/SHIM-CONTRACT.md; conformance test: shim-test.sh.
# A FIRST-CLASS compatibility surface (versioned contract + test), not test glue.
# Translates the reference-`zic` command line a distro tzdata package build uses (`zic -d dir [-L leaps]
# [-l zone] [-t name] [-b slim|fat] [-D] [-m mode] files...`) into zic-rs's subcommand CLI. The CLI-SHAPE
# divergence (subcommand + `--out` instead of bare `-d`) is the documented bucket-3 difference; this shim
# is the bridge that lets zic-rs occupy the `$(ZIC)` slot without changing the package's Makefile.
ZICRS="${ZICRS:-zic-rs}"
out=""; leaps=""; localtime=""; ltname=""; bloat=""; nocreate=""; mode=""
# Phase 1: pull flags out of "$@", leaving ONLY the input files in "$@" (order preserved). Inputs are
# re-appended to the tail via `set -- "$@" "$a"` so paths-with-spaces survive verbatim — no string
# interpolation/word-split (SHIM-CONTRACT guarantee 5). `argc` bounds the loop to the original argv.
argc=$#
while [ "$argc" -gt 0 ]; do
  a="$1"; shift; argc=$((argc - 1))
  case "$a" in
    -d) out="$1"; shift; argc=$((argc - 1));;
    -L) leaps="$1"; shift; argc=$((argc - 1));;
    -l) localtime="$1"; shift; argc=$((argc - 1));;
    -t) ltname="$1"; shift; argc=$((argc - 1));;
    -b) bloat="$1"; shift; argc=$((argc - 1));;
    -m) mode="$1"; shift; argc=$((argc - 1));;
    -p|-y|-r) shift; argc=$((argc - 1));;   # ignored, consume the value token
    -D) nocreate=1;;
    -v) ;;
    --version) exec "$ZICRS" --version;;
    -*) ;;                                   # unknown flag: dropped (forward-compat), never crashes
    *) set -- "$@" "$a";;                    # input file: re-append at tail, quoting preserved
  esac
done
# Phase 2: prefix each input file with --input (still no interpolation).
argc=$#
while [ "$argc" -gt 0 ]; do
  a="$1"; shift; argc=$((argc - 1))
  set -- "$@" --input "$a"
done
set -- compile --all-supported --unsupported skip --out "$out" "$@"
[ -n "$leaps" ] && set -- "$@" --leapseconds "$leaps"
[ -n "$bloat" ] && set -- "$@" --bloat "$bloat"
[ -n "$nocreate" ] && set -- "$@" --no-create-dirs
[ -n "$mode" ] && set -- "$@" --mode "$mode"
[ -n "$localtime" ] && set -- "$@" --localtime "$localtime"
[ -n "$ltname" ] && set -- "$@" --localtime-name "$ltname"
exec "$ZICRS" "$@"
