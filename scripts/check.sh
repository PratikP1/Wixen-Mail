#!/usr/bin/env bash
# Run the four checks CI runs, in the same order, and fail on the first one.
#
# Touching lib.rs first is not optional. Cargo shares fingerprints between
# `check`, `build`, `test`, and `clippy`, so a clippy run after a build can be
# considered fresh and report success without linting anything. That has
# already put a clippy failure on main once.
set -euo pipefail

# The targets every scoped run ends with, whatever changed, because they read
# across the whole tree and a change anywhere can redden one. Named once, here,
# because two things decide about them: the scoped run, which always adds them,
# and the registry mapping below, which must not answer one of them and make the
# run pay for the same target twice.
guards_that_read_the_whole_tree=(house_style wired)

# Which integration targets guard a changed source file.
#
#     check.sh --suites-for <registry> [changed-file ...]
#
# A unit test lives beside the code it covers, so `--lib a::b::` reaches it. A
# guard under `tests/` does not: it covers a `src/` module from outside, so the
# scoped run reaches it only when the test file itself changes. That is a guard
# running on the wrong commits, and a guard nobody reads is worse than none
# because it reads as covered.
#
# `guards/guards.toml` already declares the coupling, so it is read rather than
# restated. Every record names the `file` its break is applied to, and a record
# whose break reddens an integration target names that target as `suite`. A
# second list beside the first would drift from it, which is a shape this
# repository keeps getting caught by.
#
# **This is a line scan, not a TOML parse.** A shell script has no TOML reader,
# and the file is written one key per line. So the scan keys on a `[[guard]]`
# header and then on `file = "..."` and `suite = "..."` at column 0 inside it,
# remembering the pair when the next header arrives. What it would miss: a
# record spelled as an inline table, a `file` or `suite` sharing a line with
# another key, a value quoted some other way, or a `before`/`after` string
# holding a line that begins `file = ` at column 0. Measured 2026-09-02: 550
# records and 550 such lines, so no record has that shape today and nothing
# would say so if one arrived. A record this misses is a guard that does not run
# on the commit that could break it, which is the defect this function exists to
# fix, so the limit is written down here rather than left to be found.
#
# Its suite is `scripts/check.test.sh`, which this script runs in every mode.
the_suites_that_guard_what_changed() {
    local registry="$1"
    shift

    # A registry that is not there costs the extra targets and nothing else.
    # This mapping adds to what a commit earns, so it must never be the reason a
    # commit cannot be made.
    if [ ! -r "$registry" ]; then
        return 0
    fi

    local -a couplings=() answers=()
    local line file="" suite="" coupling path candidate already seen

    # Narrowed to the three kinds of line that carry a coupling before the loop
    # reads them, so the scan is over about 1,650 lines rather than 12,800 and
    # the shapes it reads are visible in one place.
    while IFS= read -r line; do
        case "$line" in
            '[[guard]]')
                if [ -n "$file" ] && [ -n "$suite" ]; then
                    couplings+=("$file|$suite")
                fi
                file=""
                suite=""
                ;;
            'file = "'*'"')
                file="${line#file = \"}"
                file="${file%\"}"
                ;;
            'suite = "'*'"')
                suite="${line#suite = \"}"
                suite="${suite%\"}"
                ;;
        esac
    done < <(grep -E '^(\[\[guard\]\]|file = |suite = )' "$registry" || true)
    if [ -n "$file" ] && [ -n "$suite" ]; then
        couplings+=("$file|$suite")
    fi

    for path in "$@"; do
        # Only a source module. A record whose break lands anywhere else is a
        # true record that says nothing here: `--lib` was never going to reach
        # it, and a changed `tests/*.rs` already runs its own target.
        case "$path" in
            src/*.rs) ;;
            *) continue ;;
        esac

        for coupling in "${couplings[@]+"${couplings[@]}"}"; do
            [ "${coupling%%|*}" = "$path" ] || continue
            candidate="${coupling#*|}"

            # Dropped if the scoped run already ends with it, and dropped if
            # another record has already answered it. Several records couple one
            # source file to one target, and the target is one run either way.
            seen=""
            for already in "${guards_that_read_the_whole_tree[@]}" \
                           "${answers[@]+"${answers[@]}"}"; do
                if [ "$already" = "$candidate" ]; then
                    seen=yes
                fi
            done
            if [ -n "$seen" ]; then
                continue
            fi

            answers+=("$candidate")
        done
    done

    if [ "${#answers[@]}" -eq 0 ]; then
        return 0
    fi
    printf '%s\n' "${answers[@]}"
}

# The mapping on its own, so `scripts/check.test.sh` can ask it about a made-up
# file list and a made-up registry rather than running the whole gate. Answered
# here, before anything with a side effect and before the mode is looked at, so
# a suite asking this question can never start a gate run that would run that
# suite again.
if [ "${1:-}" = "--suites-for" ]; then
    shift
    the_suites_that_guard_what_changed "$@"
    exit 0
fi

# Offer to run these on every commit, so the answer cannot be lost between
# getting it and committing. It has been twice: a stale fingerprint reporting
# clean, and this script's output piped somewhere so the pipeline's exit status
# was the pipe's rather than this script's.
# Asked as a question rather than matched against one spelling. The old form
# compared `core.hooksPath` to the literal `.githooks`, and this machine holds
# the absolute path, which is what makes the hook resolve from a worktree. So it
# printed "Not running on commit" on every run, including the runs the hook
# itself started, and the fix it advised would have replaced a working absolute
# path with a narrower relative one. Found on 2026-09-02 by somebody reading the
# output of a run the hook had started.
#
# What matters is whether a commit will run this, so that is what is asked: does
# the configured hooks directory hold an executable commit-msg hook. A wrong
# answer here is not cosmetic, it tells somebody to change a setting that works.
hooks_path="$(git config core.hooksPath || true)"
case "$hooks_path" in
    "") will_run_on_commit="" ;;
    /* | [A-Za-z]:[/\\]*) will_run_on_commit="$hooks_path/commit-msg" ;;
    *) will_run_on_commit="$(git rev-parse --show-toplevel)/$hooks_path/commit-msg" ;;
esac

if [ -z "$will_run_on_commit" ] || [ ! -f "$will_run_on_commit" ]; then
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
# **Those figures are warm and they are a floor, not an estimate.** Measured
# again on 2026-09-02, a documents-only commit took 2m56s: it followed two
# commits that had changed test files, so it paid for a clippy rebuild, and the
# document-reading list includes two targets that build a live window. Quoting
# 51 seconds to somebody planning work was quoting a measurement without its
# conditions. What holds in every case is the shape: a documents commit runs a
# small fraction of the gate. What it costs on the day depends on what the
# commits before it left to rebuild.
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

# An argument this script does not know is refused rather than falling through
# to the whole gate.
#
# Not a tidiness point. This script runs every `scripts/*.test.sh`, and one of
# those suites now runs this script. A typo in the argument that suite passes
# used to be read as a mode, fall past every branch below, and run the whole
# gate, which ran the suite, which ran the gate. Measured on 2026-09-02 by
# writing exactly that typo, and it had to be killed. A misspelled `all` also
# quietly bought somebody the full gate and told them nothing.
case "$mode" in
    "" | all | all_but_slow | affected | docs_only | red) ;;
    *)
        echo "check.sh: '$mode' is not a mode this script knows." >&2
        echo "  Modes: all, all_but_slow, affected, docs_only, red." >&2
        echo "  Or no argument at all, and which-checks.sh decides." >&2
        echo "  Or --suites-for <registry> [changed-file ...] for the mapping" >&2
        echo "  from a changed source file to the guards that cover it." >&2
        exit 64
        ;;
esac

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
    # --no-fail-fast because this names five targets, and without it a red
    # `house_style` meant the other four never started. Found on 2026-09-03 by
    # `test_one_failing_target_does_not_hide_the_rest`, the moment its exemption
    # was narrowed from "the line names a target" to "the line names exactly
    # one". This is the same defect that was fixed in the scoped run below, in
    # the same shape, on a line the wider exemption could not see.
    cargo test --no-fail-fast --test house_style --test docs_links --test wired \
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
    local status=0 path module target suite
    local -a tree_targets=()
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
    # The integration targets `guards/guards.toml` couples to a changed source
    # module. Without these, a guard that lives under `tests/` and covers a
    # `src/` module runs only on commits that change the test file, which is
    # every commit except the ones that could break it.
    while IFS= read -r suite; do
        [ -n "$suite" ] || continue
        echo "-- $suite (coupled to what changed by guards/guards.toml)"
        cargo test --test "$suite" >> "$run_log" 2>&1 || status=1
    done < <(the_suites_that_guard_what_changed \
        "$(dirname "$0")/../guards/guards.toml" \
        "${changed[@]+"${changed[@]}"}")

    echo "-- the guards that read the whole tree"
    for suite in "${guards_that_read_the_whole_tree[@]}"; do
        tree_targets+=(--test "$suite")
    done
    # --no-fail-fast because this is more than one target, and this line used to
    # be `cargo test --test house_style --test wired` without it: a failure in
    # house_style meant wired never started, not reported as skipped. That is
    # the same defect the whole-suite run below carries the flag for, in a line
    # small enough that nobody looked at it twice.
    #
    # Found on 2026-09-02 by `test_one_failing_target_does_not_hide_the_rest`,
    # which reads this file. It used to exempt a line carrying `--test ` as one
    # that runs a named target on purpose, so the old spelling was exempt while
    # having the defect; building the targets into an array took the flag out of
    # the text and the guard spoke. The exemption now counts the targets and
    # covers only a line naming exactly one, which found the same defect on the
    # documents-only run above the moment it was narrowed.
    cargo test --no-fail-fast "${tree_targets[@]}" >> "$run_log" 2>&1 || status=1
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
