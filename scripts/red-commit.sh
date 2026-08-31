#!/usr/bin/env bash
# The RED half of red/green, made possible on a branch and measured rather than
# waved through.
#
#   red-commit.sh names <message-file>
#
# # Why this exists
#
# Red/green needs a commit whose tests fail. The pre-commit gate refuses one:
# on a branch it runs the tests reaching what changed, so a failing test cannot
# be committed, and `--no-verify` is not available here for good reasons. Phase
# 1 committed failing tests freely only because the hook was not turned on yet.
# Turning it on closed a gate nobody meant to close.
#
# The obvious fix is a marker that skips the tests. That is an escape hatch,
# and this repository's own history says what happens to those: a rule loses to
# the easier path, every time, and nobody notices because everything stays
# green.
#
# So the marker does not skip the tests. It names the tests that must fail, and
# the run is then held to it in both directions, the way `scripts/guards.sh`
# holds a guard record: every named test must have run, every named test must
# have failed, and nothing else may have failed. A commit that names nothing
# gets no exemption, and a commit that names a test which passes is refused.
# That makes the marker cost more to misuse than to use honestly, which is the
# only kind of marker that survives.
#
# A red commit is therefore stronger evidence than an unchecked one: it records
# which tests were red and proves they were, at the commit that claims it.
#
# # The marker
#
#     test(02-02): failing tests for the narrower question set
#
#     Fails-until-green: application::saved_searches::tests::test_a
#     Fails-until-green: application::saved_searches::tests::test_b
#
# A trailer rather than a word in the subject, because the subject is the line
# everybody reads in a log and a commit should not announce its own exemption
# there. Named `Fails-until-green` rather than anything shorter because the
# commit messages here describe measurements in prose and really do contain
# sentences like "Four are red". A marker a prose sentence can produce by
# accident is a marker that fires when nobody meant it.
set -euo pipefail

readonly MARKER='fails-until-green:'

usage() {
    echo "usage: red-commit.sh names <message-file>" >&2
    exit 64
}

# The tests a commit message says must fail, one per line. Nothing at all when
# the message carries no marker, which is almost every commit.
#
# Refuses rather than shrugs when a marker is there but says nothing. Answering
# "no marker" to a typo'd one would run the tests, refuse the commit for the
# named test failing, and report the wrong cause: the author would read a
# message about a broken test when what is broken is their marker.
names() {
    local file="$1"
    [ -r "$file" ] || {
        echo "red-commit: cannot read $file" >&2
        exit 66
    }

    local line_number=0
    local line lowered value item found=0
    while IFS= read -r line || [ -n "$line" ]; do
        line_number=$((line_number + 1))
        # The subject, and git's own commentary. Neither is where a trailer
        # lives, and the subject is where one would arrive by accident.
        [ "$line_number" -eq 1 ] && continue
        case "$line" in '#'*) continue ;; esac

        # At column 0, which is where a git trailer lives. An indented line is a
        # quotation or a worked example, and reading one as an instruction is
        # how this refused the very commit that introduced it: that message
        # showed the marker's shape, indented, while explaining what it was for.
        lowered="$(printf '%s' "$line" | tr '[:upper:]' '[:lower:]')"
        case "$lowered" in "$MARKER"*) ;; *) continue ;; esac

        found=1
        value="${line#*:}"
        # Split on commas, then trim each. An empty element is a stray comma or
        # an empty marker, and both are refused: a list that says nothing is
        # not a list of nothing.
        local rest="$value"
        while :; do
            item="${rest%%,*}"
            item="${item#"${item%%[![:space:]]*}"}"
            item="${item%"${item##*[![:space:]]}"}"
            if [ -z "$item" ]; then
                echo "red-commit: '$line' names no test to fail" >&2
                exit 65
            fi
            printf '%s\n' "$item"
            # A trailing comma leaves one more element to look at, and it is
            # empty, so the refusal above catches it. Breaking on an empty
            # remainder instead would step over exactly that case.
            case "$rest" in
                *,*) rest="${rest#*,}" ;;
                *) break ;;
            esac
        done
    done < "$file"

    [ "$found" -eq 1 ] || return 0
}

# Whether a test run is the red the commit said it would be.
#
# Three conditions, all of them load-bearing, and each one is what stops a
# different way of getting an unearned commit through:
#
#   every named test ran:     a typo, or a test in a module the scoped run
#                             never reached, otherwise reads as "did not fail"
#   every named test failed:  otherwise a commit can name a passing test and
#                             buy itself an exemption with nothing at stake
#   nothing else failed:      otherwise one named failure carries twenty
#                             unnamed ones through with it
#
# Every problem is reported, not just the first, because they are usually the
# same mistake seen from two sides and fixing them one commit at a time is how
# a five-minute gate becomes a twenty-minute one.
verdict() {
    local named_file="$1" output_file="$2"
    for f in "$named_file" "$output_file"; do
        [ -r "$f" ] || {
            echo "red-commit: cannot read $f" >&2
            exit 66
        }
    done

    declare -A outcome=()
    local line name
    while IFS= read -r line || [ -n "$line" ]; do
        case "$line" in
            # Anchored at both ends. `test result: FAILED. ...` also begins
            # with the word, and counting that as a test called `result:`
            # would put a failure in the set that no name can match.
            "test "*" ... ok")
                name="${line#test }"
                name="${name% ... ok}"
                [ "${outcome[$name]:-}" = FAILED ] || outcome["$name"]=ok
                ;;
            "test "*" ... FAILED")
                name="${line#test }"
                name="${name% ... FAILED}"
                outcome["$name"]=FAILED
                ;;
            "test "*" ... ignored")
                name="${line#test }"
                name="${name% ... ignored}"
                [ -n "${outcome[$name]:-}" ] || outcome["$name"]=ignored
                ;;
        esac
    done < "$output_file"

    local ran=0 problems=0
    for name in "${!outcome[@]}"; do
        case "${outcome[$name]}" in ok | FAILED) ran=$((ran + 1)) ;; esac
    done

    # Nothing ran at all. That is never the red a commit meant, and the two
    # causes need telling apart: a build that failed says nothing about any
    # test, and a filter matching nothing exits zero and looks like success.
    if [ "$ran" -eq 0 ]; then
        if grep -qE '^error(\[|:)' "$output_file"; then
            echo "red-commit: nothing compiled, so no test failed and none passed." >&2
            echo "  A test that does not build is not a red. Build it first." >&2
        else
            echo "red-commit: no test ran, so this run says nothing." >&2
            echo "  A filter matching nothing exits zero and reads as success." >&2
        fi
        exit 1
    fi

    while IFS= read -r name || [ -n "$name" ]; do
        [ -n "$name" ] || continue
        case "${outcome[$name]:-absent}" in
            FAILED) ;;
            ok)
                echo "red-commit: $name was named as failing and it passed." >&2
                problems=$((problems + 1))
                ;;
            ignored)
                echo "red-commit: $name was named as failing and is ignored." >&2
                problems=$((problems + 1))
                ;;
            *)
                echo "red-commit: $name was named as failing and never ran." >&2
                echo "  Either the name is wrong, or this commit's scoped tests" >&2
                echo "  do not reach the module it is in." >&2
                problems=$((problems + 1))
                ;;
        esac
    done < "$named_file"

    for name in "${!outcome[@]}"; do
        [ "${outcome[$name]}" = FAILED ] || continue
        grep -qxF "$name" "$named_file" && continue
        echo "red-commit: $name failed and the commit did not name it." >&2
        problems=$((problems + 1))
    done

    [ "$problems" -eq 0 ] || exit 1
}

case "${1:-}" in
    names)
        [ "$#" -eq 2 ] || usage
        names "$2"
        ;;
    verdict)
        [ "$#" -eq 3 ] || usage
        verdict "$2" "$3"
        ;;
    *)
        usage
        ;;
esac
