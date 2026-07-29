#!/usr/bin/env bash
#
# Which tests would fail if the code were wrong.
#
# A passing suite says the code does what the tests say. It does not say the
# tests would notice if it stopped. Those are different claims, and only one of
# them is what a test is for.
#
# This project needs to know the difference. Red/green started at commit 182 of
# 344, so most of the tests here were written after the code they cover, which
# makes them a description rather than a specification. In one session three
# tests written on purpose to catch a specific bug passed against that bug.
#
# Mutation testing settles it: change the code in a small way, run the suite,
# see whether anything fails. A mutant nothing catches is either untested
# behaviour or dead code, and both are worth knowing about.
#
# Usage:
#   scripts/mutants.sh                   everything the config allows, slow
#   scripts/mutants.sh src/service       one directory
#   scripts/mutants.sh --since main      only what changed, fast enough for CI
#
set -euo pipefail

cd "$(dirname "$0")/.."

if ! command -v cargo-mutants >/dev/null 2>&1; then
    echo "cargo-mutants is not installed."
    echo "    cargo install cargo-mutants --locked"
    exit 1
fi

# One job, because more than one is not reliable here.
#
# Each worker copies the tree to its own temp directory to build in, and on
# Windows they collide: every worker dies with "The file exists (os error 80)".
# At three it happens immediately after the baseline; at two it happens partway
# through, which is worse, because the run looks like it is working and then
# throws away everything it had not written out. cargo-mutants 27.1.0.
#
# Serial is roughly a minute per mutant. Raise it with MUTANTS_JOBS if a future
# version fixes this, and check the run actually finished rather than trusting
# the exit code: the failure above still exits zero through a pipe.
JOBS="${MUTANTS_JOBS:-1}"
OUT="target/mutants"

if [ "${1:-}" = "--since" ]; then
    BASE="${2:-main}"
    echo "== mutants in what changed since $BASE =="
    # Only the lines this branch touched, which is what makes it quick enough
    # to gate a change on rather than run overnight.
    git diff "$BASE"...HEAD -- 'src/**/*.rs' > "$OUT.diff" 2>/dev/null || {
        mkdir -p "$(dirname "$OUT.diff")"
        git diff "$BASE"...HEAD -- 'src/**/*.rs' > "$OUT.diff"
    }
    if [ ! -s "$OUT.diff" ]; then
        echo "Nothing changed under src/. Nothing to check."
        exit 0
    fi
    cargo mutants --in-diff "$OUT.diff" -j "$JOBS" --output "$OUT"
    exit $?
fi

TARGET="${1:-}"
if [ -n "$TARGET" ]; then
    echo "== mutants in $TARGET =="
    cargo mutants --file "$TARGET/**/*.rs" -j "$JOBS" --output "$OUT" || true
else
    echo "== mutants everywhere the config allows =="
    echo "This takes hours. scripts/mutants.sh <dir> is the usual way in."
    cargo mutants -j "$JOBS" --output "$OUT" || true
fi

echo
echo "== what nothing noticed =="
MISSED="$OUT/mutants.out/missed.txt"
CAUGHT="$OUT/mutants.out/caught.txt"
if [ ! -f "$CAUGHT" ]; then
    echo "The run produced no results at all. Read $OUT/mutants.out/ for why."
    exit 1
fi
if [ -s "$MISSED" ]; then
    cat "$MISSED"
    echo
    echo "Each line is behaviour no test is watching. Either pin it or delete it."
else
    echo "Nothing. Every mutant was caught."
fi
echo
echo "caught $(wc -l < "$CAUGHT"), missed $(wc -l < "$MISSED" 2>/dev/null || echo 0)"
