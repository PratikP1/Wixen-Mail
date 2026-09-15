#!/usr/bin/env bash
#
# Which tests would fail if the code were wrong.
#
# A passing suite says the code does what the tests say. It does not say the
# tests would notice if it stopped. Those are different claims, and only one of
# them is what a test is for.
#
# This project needs to know the difference. Red/green started at 18a02454 on
# 2026-07-26, so most of the tests here were written after the code they cover,
# which makes them a description rather than a specification. How much of the
# history predates that commit is computed and printed on every commit by
# test_the_share_of_history_before_red_green_is_computed_and_printed in
# tests/every_number_carries_its_command_and_its_date.rs: 8.9% on 2026-09-14.
# This comment gave the commit's position and the count on the day it was
# written, wrapped across two lines so that a line grep never saw it, and the
# two numbers stayed true while the share fell from 53%. In one session three
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
#   scripts/mutants.sh --shard 0/496 [--out DIR] [--in-place] [--file GLOB] [-- cargo test args [-- binary args]]
#                                         one shard of everything, to DIR/shard-0-of-496,
#                                         with what it ran under written beside it
#                                         before it starts and its timing after
#   scripts/mutants.sh --shards 496 [--out DIR] [--in-place] [--file GLOB] [-- ...]
#                                         every shard in turn, skipping the ones already
#                                         complete, waiting for a quiet machine before
#                                         each; killed and started again with the same
#                                         command, it picks up at the first incomplete one
#   scripts/mutants.sh --shards 18 --file 'src/service/protocols/**' --in-place -- --all-targets
#                                         one area in shards: the glob goes to the tool's
#                                         --file, the shards divide what it leaves in, and
#                                         the merger refuses shards whose globs differ
#
# Name a real commit or tag to compare against. Every commit here lands on
# `main`, so `--since main` compares `main` with itself and finds nothing, which
# it now says out loud instead of passing.
#
# A whole-tree run is a set of shards on one commit. `--shard k/n` divides the
# list the tool builds from the tree it runs in, so two shards from two commits
# are two lists, and a mutant can fall between them or land in both. Run the
# shards in a worktree that does not move, and read them together with
# `scripts/mutants_report.py --shards DIR n`, which refuses a shard from a
# different commit, with different arguments, or that stopped short. Everything
# after `--` goes to `cargo test`, and after a second `--` to the test binary:
# `-- --lib -- --test-threads=8` runs the library alone at eight threads, which
# is the cheaper of the two suite shapes and leaves the targets under tests/
# out of the judgement. `--in-place` mutates the tree the script runs in rather
# than a copy, so the build is incremental; only in a worktree nobody is
# editing, and it refuses to start over a tree with a tracked file modified,
# because that is what a kill mid-mutant leaves.
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

# The shard options, read before the older modes so those still see their own
# first argument. PASS is everything after `--`, handed to cargo mutants after
# its own `--`; COPY is what the conditions record says about where the build
# happened.
SHARD=""
SHARDS=""
COPY="scratch"
IN_PLACE=()
FILES=""
FILE_ARGS=()
PASS=()
while [ $# -gt 0 ]; do
    case "$1" in
        --shard) SHARD="$2"; shift 2 ;;
        --shards) SHARDS="$2"; shift 2 ;;
        --out) OUT="$2"; shift 2 ;;
        --in-place) COPY="in-place"; IN_PLACE=(--in-place); shift ;;
        # One area rather than the whole tree. The shards divide whatever
        # this leaves in, so the glob is written into every shard's record
        # and the merger refuses two shards whose globs differ.
        --file) FILES="$2"; FILE_ARGS=(--file "$2"); shift 2 ;;
        --) shift; PASS=("$@"); break ;;
        *) break ;;
    esac
done
mkdir -p "$OUT"

# The tool refuses `--jobs` beside `--in-place`, even at one: an in-place run
# has one tree to mutate, so there is nothing for a second job to work in. So
# a shard run in place is passed no job count, and a MUTANTS_JOBS above one is
# refused here rather than by the tool after the conditions were written.
JOBS_ARGS=(-j "$JOBS")
if [ "$COPY" = "in-place" ]; then
    if [ "$JOBS" != 1 ]; then
        echo "--in-place mutates the one tree the script runs in, so MUTANTS_JOBS=$JOBS cannot apply to it."
        exit 1
    fi
    JOBS_ARGS=()
fi

# One shard: what it ran under written before the run, so a shard killed
# partway still says what it was; the run; its timing line, printed and kept
# beside the record so a launcher's overwritten stdout loses nothing; and the
# report. Sets REPORT_STATUS rather than returning it, because inside --shards
# a shard with a missed mutant is the usual case and not a reason to stop.
run_shard() {
    local shard="$1" dir="$2" baseline="$3"
    shift 3
    # A run mutating the tree in place must start from a clean one. The tool
    # puts each mutated file back when it is done with the mutant, and a kill
    # mid-mutant skips that; with the baseline skipped, nothing else would
    # notice, and every later shard would judge a tree with that mutant in it.
    # The refusal names the file and the checkout that puts it back.
    if [ "$COPY" = "in-place" ]; then
        python scripts/mutants_report.py --tree-is-clean
    fi
    mkdir -p "$dir"
    printf 'commit = %s\nshard = %s\narguments = %s\ncopy = %s\nbaseline = %s\nfiles = %s\n' \
        "$(git rev-parse HEAD)" "$shard" "${PASS[*]}" "$COPY" "$baseline" "$FILES" \
        > "$dir/conditions.txt"

    local started finished status=0
    started=$(date +%s)
    cargo mutants --shard "$shard" ${JOBS_ARGS[@]+"${JOBS_ARGS[@]}"} --output "$dir" \
        ${IN_PLACE[@]+"${IN_PLACE[@]}"} ${FILE_ARGS[@]+"${FILE_ARGS[@]}"} "$@" \
        -- ${PASS[@]+"${PASS[@]}"} || status=$?
    finished=$(date +%s)

    python scripts/mutants_report.py --timing "$dir/mutants.out" "$started" "$finished" \
        | tee "$dir/timing.txt"
    echo
    echo "== what shard $shard really did =="
    REPORT_STATUS=0
    python scripts/mutants_report.py --exit-status "$status" "$dir/mutants.out" || REPORT_STATUS=$?
}

# Every shard in turn. A complete shard is skipped, which is what makes a
# restart pick up where the kill happened. The machine must be quiet before
# each, because a build running beside a shard produces timeouts that are the
# machine and not the code.
#
# The baseline is the tree's own tests run on the unmutated tree, and the first
# complete shard proved they pass at this commit. The worktree does not move
# between shards, so every shard after that one skips the baseline, which is a
# suite run fewer per shard. Skipping it also drops the timeout the tool derives
# from the baseline, and the tool falls back to 300 seconds, which under the
# every-target shape is less than the config's rule gives and would file a busy
# minute as a hang; so the launcher passes the timeout the config's rule gives
# over the first complete shard's baseline.
run_every_shard() {
    local n="$1" k dir first_complete=""
    for ((k = 0; k < n; k++)); do
        dir="$OUT/shard-$k-of-$n"
        if python scripts/mutants_report.py --complete "$dir/mutants.out" >/dev/null; then
            echo "shard $k of $n is complete, skipping"
            if [ -z "$first_complete" ]; then first_complete="$dir"; fi
            continue
        fi
        echo
        echo "== shard $k of $n =="
        python scripts/mutants_report.py --wait-until-quiet
        local baseline=run extra=()
        if [ -n "$first_complete" ]; then
            baseline=skip
            extra=(--baseline skip --timeout "$(python scripts/mutants_report.py --timeout-after "$first_complete/mutants.out")")
        fi
        run_shard "$k/$n" "$dir" "$baseline" ${extra[@]+"${extra[@]}"}
        if ! python scripts/mutants_report.py --complete "$dir/mutants.out" >/dev/null; then
            echo
            echo "Shard $k of $n did not complete, so this stops here rather than meeting the same thing $((n - k - 1)) more times."
            echo "Read what it says above, then start again with the same command; it picks up at shard $k."
            return 1
        fi
        if [ -z "$first_complete" ]; then first_complete="$dir"; fi
    done
    echo
    echo "Every shard is complete: $n of $n. Read them together with:"
    echo "    python scripts/mutants_report.py --shards $OUT $n"
}

if [ -n "$SHARD" ]; then
    echo "== mutants in shard $SHARD =="
    run_shard "$SHARD" "$OUT/shard-${SHARD%/*}-of-${SHARD#*/}" run
    exit "$REPORT_STATUS"
elif [ -n "$SHARDS" ]; then
    echo "== mutants everywhere the config allows, in $SHARDS shards =="
    run_every_shard "$SHARDS"
    exit 0
fi

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
