#!/usr/bin/env bash
# Run the four checks CI runs, in the same order, and fail on the first one.
#
# Touching lib.rs first is not optional. Cargo shares fingerprints between
# `check`, `build`, `test`, and `clippy`, so a clippy run after a build can be
# considered fresh and report success without linting anything. That has
# already put a clippy failure on main once.
set -euo pipefail

# Offer to run these on every commit, so the answer cannot be lost between
# getting it and committing. It has been twice: a stale fingerprint reporting
# clean, and this script's output piped somewhere so the pipeline's exit status
# was the pipe's rather than this script's.
if [ "$(git config core.hooksPath || true)" != ".githooks" ]; then
  echo "Not running on commit. To turn that on:"
  echo "    git config core.hooksPath .githooks"
  echo
fi

# Which checks this run does. Given as an argument, or worked out by
# `which-checks.sh` from where you are and what is staged, which is where that
# decision lives and where it is tested. Pass `all` to force the whole gate
# wherever you are, which is what merging a branch into main does first.
#
# Two questions, and they are separate. Whether the slow half can be deferred is
# about the branch: main cannot defer, because every commit here lands on it.
# What a change can possibly break is about the change, and that holds
# everywhere, including on main. So a documents-only commit runs the tests that
# read documents wherever it is made, and a code commit on a branch runs the
# tests reaching what it touched.
#
# Measured warm: the whole gate is about 330 seconds, of which the suite is 239
# and the release build 56. A documents-only run is about 51.
#
# The commit message, when a commit is what is running this. Only the message
# can say that this is the RED half of red/green, which is why the hook runs
# from `commit-msg`: at `pre-commit` time the message does not exist yet.
message_file=""
case "${1:-}" in
    --message-file=*)
        message_file="${1#--message-file=}"
        shift
        ;;
esac

mode="${1:-}"
# What is about to be committed, which is what the tests should be scoped to.
# Staged rather than working-tree, because that is what the hook is deciding
# about. Empty when run by hand outside a commit, and `which-checks.sh` answers
# `all_but_slow` for that rather than guessing at a narrower set.
changed=()
if [ -z "$mode" ]; then
    while IFS= read -r line; do
        [ -n "$line" ] && changed+=("$line")
    done < <(git diff --cached --name-only 2>/dev/null || true)
    mode="$("$(dirname "$0")/which-checks.sh" \
        ${message_file:+"--message-file=$message_file"} \
        "$(git rev-parse --abbrev-ref HEAD 2>/dev/null || true)" \
        "${changed[@]+"${changed[@]}"}")"
fi

touch src/lib.rs

echo "== rustfmt =="
cargo fmt --all -- --check

echo "== clippy =="
cargo clippy --all-targets --all-features -- -D warnings

# The scripts that decide what this gate does, checked by the gate itself.
#
# These suites existed for a day before anything ran them. `which-checks.sh`
# decides which checks every commit earns and `red-commit.sh` decides whether a
# failing test may be committed, so a defect in either is a defect in all of
# this, and both had tests that nothing invoked. That is the shape guardrail 4
# in `CLAUDE.md` is about: a check nobody reads is worse than none, because it
# reads as covered.
#
# In every mode and before every other decision, because they cost milliseconds
# and because the mode was chosen by the very script under test.
echo "== the scripts that decide what runs =="
for suite in "$(dirname "$0")"/*.test.sh; do
    [ -e "$suite" ] || continue
    bash "$suite"
done

if [ "$mode" = "all_but_slow" ]; then
    echo
    echo "Formatting and clippy passed. The test suite and the release build did"
    echo "not run: this is not main. Run 'scripts/check.sh all' before merging."
    exit 0
fi

# Documents can only break the tests that read documents, and they genuinely
# can: house_style's em-dash guard has caught two real breaks in markdown, so
# these run rather than being skipped as "not code".
if [ "$mode" = "docs_only" ]; then
    echo "== the targets that read documents =="
    # Five, not three. `help_page` reads `docs/ALPHA_TESTING.md` and the shipped
    # help pages from inside the library, so a documents-only run that skipped
    # `--lib` would miss the guard that catches a dead link in a help page. That
    # guard has already caught one this month. `checkbox_labels` and
    # `manager_delete_stays_open` read documents too.
    cargo test --lib help_page::
    cargo test --test house_style --test docs_links --test wired \
        --test checkbox_labels --test manager_delete_stays_open
    echo
    echo "Formatting, clippy and the document-reading tests passed. The rest of"
    echo "the suite and the release build did not run: nothing outside a document"
    echo "changed, so they had nothing to say. Run 'scripts/check.sh all' before"
    echo "merging."
    exit 0
fi

# The whole output of every scoped run, kept rather than summarised. It used to
# be piped through `tail -3` per module, which threw away the names of the tests
# that failed and left a gate that said something was wrong without saying what.
# `red` needs the full text anyway, to see which tests failed.
run_log="$(mktemp)"
trap 'rm -f "$run_log"' EXIT

# A unit test lives beside the code it covers, so a changed `src/a/b.rs` is
# covered by `--lib a::b::`. The source-reading guards run whatever changed,
# because they read across the whole tree and a change anywhere can redden one:
# that is how a guard record was found stale four times in one phase.
#
# Returns non-zero if any scoped run did. Never aborts on one, so a failure in
# the first module does not hide the rest, for the same reason the whole-suite
# run passes `--no-fail-fast`.
run_the_tests_that_reach_what_changed() {
    local status=0 path module target
    for path in "${changed[@]+"${changed[@]}"}"; do
        case "$path" in
            src/*.rs)
                module="${path#src/}"
                module="${module%.rs}"
                module="${module%/mod}"
                module="${module//\//::}"
                [ "$module" = "lib" ] && continue
                echo "-- $module"
                # A filter matching nothing exits zero, so a module with no
                # tests of its own is not a pass, it is a run that said nothing.
                cargo test --lib "${module}::" >> "$run_log" 2>&1 || status=1
                ;;
            tests/*.rs)
                target="$(basename "$path" .rs)"
                echo "-- $target"
                cargo test --test "$target" >> "$run_log" 2>&1 || status=1
                ;;
        esac
    done
    echo "-- the guards that read the whole tree"
    cargo test --test house_style --test wired >> "$run_log" 2>&1 || status=1
    return $status
}

# The RED half of red/green. The commit named the tests that must fail; the run
# is held to exactly that, in all three directions, by `red-commit.sh`.
if [ "$mode" = "red" ]; then
    named="$(mktemp)"
    trap 'rm -f "$run_log" "$named"' EXIT
    "$(dirname "$0")/red-commit.sh" names "$message_file" > "$named"

    echo "== the tests this commit says must fail =="
    sed 's/^/   /' "$named"
    echo
    echo "== the tests that reach what changed =="
    run_the_tests_that_reach_what_changed || true

    echo
    if ! "$(dirname "$0")/red-commit.sh" verdict "$named" "$run_log"; then
        echo
        echo "This commit is not the red it says it is." >&2
        cp "$run_log" ./red-run.log && echo "The whole run is in ./red-run.log" >&2
        exit 1
    fi
    echo "Formatting, clippy, and a red that is exactly the one this commit"
    echo "named. The green commit that follows runs the same tests and must"
    echo "carry no marker."
    exit 0
fi

# Scope the suite to the modules the change reaches.
if [ "$mode" = "affected" ]; then
    echo "== the tests that reach what changed =="
    if ! run_the_tests_that_reach_what_changed; then
        echo
        echo "Failed. What went red, with its output:" >&2
        sed -n '/^failures:/,$p' "$run_log" >&2
        exit 1
    fi
    echo
    echo "Formatting, clippy, the tests reaching what changed, and the"
    echo "tree-reading guards passed. The rest of the suite and the release"
    echo "build did not run. Run 'scripts/check.sh all' before merging."
    exit 0
fi

# The thread count is deliberately not set here, and that is a result rather
# than an omission.
#
# Run on its own, the library suite is much faster on four threads than on the
# harness default of one per core. Measured 2026-08-31 on 24 logical cores over
# 5,837 tests: 2 threads 131s, 4 threads 88s, 8 threads 106s, 16 threads 164s,
# default 196s. It is contended rather than compute-bound. `scripts/guards.py`
# takes that setting and keeps it, because it runs `--lib` on its own and the
# measurement is about exactly that.
#
# It does not carry to this gate, which runs `--all-targets`. Measured here, on
# the same machine on the same day with nothing else running:
#
#                        library test term    whole gate
#     default (24)              197.00s          335s
#     four threads              111.55s          353s
#
# The test term drops 86 seconds twice over and the total does not move. About
# 104 seconds appears somewhere else and was not accounted for; the two totals
# may simply be inside this machine's run-to-run spread. Either way there is no
# measured gain here, so nothing is set, because a number written down without a
# result behind it is the thing this file keeps warning about.
#
# Worth picking up again: the gate is compilation more than testing, and
# `target/debug` was 269GB when this was measured.

echo "== tests =="
# --no-fail-fast because without it cargo stops at the first target that fails,
# and the library is the first target. One failing test there means none of the
# fourteen files under tests/ run at all: not reported as skipped, never
# started. That is how a broken guard record once reached main while this gate
# looked like it had checked it. The run still fails; it just says everything
# that is wrong rather than the first thing.
cargo test --all-targets --no-fail-fast

echo "== release build =="
cargo build --release

echo "All four checks passed."
