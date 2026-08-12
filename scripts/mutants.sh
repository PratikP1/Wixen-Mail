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
# A run in which the suite was never once run against a mutant is refused
# rather than summarised, and the report says which of the two ways it went:
# every mutant rejected by the compiler, or no mutants to begin with.
#
# What it found is read by scripts/mutants_report.py, and not by this script.
# The reading is the part that was wrong: this script used to decide whether a
# run had produced anything by asking whether a results file existed, and that
# file is created empty before the first mutant is built. So a run whose build
# failed, which tested nothing at all, printed "every mutant was caught" and
# exited zero. Read that file for the rest of it.
#
# Usage:
#   scripts/mutants.sh                    everything the config allows, slow
#   scripts/mutants.sh src/service        one directory
#   scripts/mutants.sh --since v0.19.0    only what changed since a commit
#
# Name a real commit or tag to compare against. Every commit here lands on
# `main`, so `--since main` compares `main` with itself and finds nothing, which
# it now says out loud instead of passing.
set -euo pipefail

cd "$(dirname "$0")/.."

if ! command -v cargo-mutants >/dev/null 2>&1; then
    echo "cargo-mutants is not installed."
    echo "    cargo install cargo-mutants --locked"
    exit 1
fi

if ! command -v python >/dev/null 2>&1; then
    echo "python is not on the path, and the run is read with it."
    exit 1
fi

# The wording rules the report has to follow, checked before the run starts
# rather than after it has spent an hour earning a result nobody can read.
# Milliseconds, and it fails the run.
python -m doctest scripts/mutants_report.py

# One job, because more than one is not reliable here.
#
# Each worker copies the tree to its own temp directory to build in, and on
# Windows they collide: every worker dies with "The file exists (os error 80)".
# At three it happens immediately after the baseline; at two it happens partway
# through, which is worse, because the run looks like it is working and then
# throws away everything it had not written out. cargo-mutants 27.1.0.
#
# Serial is roughly a minute per mutant. Raise it with MUTANTS_JOBS if a future
# version fixes this. Whether the run finished is no longer a matter of trusting
# the exit code: the report is read from what happened to each mutant.
JOBS="${MUTANTS_JOBS:-1}"
OUT="target/mutants"
mkdir -p "$OUT"

STATUS=0
if [ "${1:-}" = "--since" ]; then
    BASE="${2:-main}"
    if ! BASE_HASH=$(git rev-parse --short --verify "$BASE^{commit}" 2>/dev/null); then
        echo "There is no commit called $BASE, so there is nothing to compare with."
        exit 1
    fi
    HEAD_HASH=$(git rev-parse --short --verify HEAD)
    echo "== mutants in what changed since $BASE =="
    # Only the lines this branch touched, which is what makes it quick enough
    # to gate a change on rather than run overnight.
    #
    # `-- src`, and not a glob over it. A pathspec of 'src/**/*.rs' silently
    # drops every file sitting directly in src/, and this repository had that
    # same wrong pathspec written down in two places.
    git diff "$BASE"...HEAD -- src > "$OUT.diff"
    if [ ! -s "$OUT.diff" ]; then
        exec python scripts/mutants_report.py \
            --nothing-changed "$BASE" "$BASE_HASH" "$HEAD_HASH"
    fi
    cargo mutants --in-diff "$OUT.diff" -j "$JOBS" --output "$OUT" || STATUS=$?
elif [ -n "${1:-}" ]; then
    echo "== mutants in $1 =="
    cargo mutants --file "$1/**/*.rs" -j "$JOBS" --output "$OUT" || STATUS=$?
else
    echo "== mutants everywhere the config allows =="
    echo "This takes hours. scripts/mutants.sh <dir> is the usual way in."
    cargo mutants -j "$JOBS" --output "$OUT" || STATUS=$?
fi

# Every path above arrives here, including the ones that failed. A run that
# stopped early is not a run with nothing to say about it.
echo
echo "== what this run really did =="
exec python scripts/mutants_report.py --exit-status "$STATUS" "$OUT/mutants.out"
