#!/usr/bin/env bash
# The bookkeeping every `scripts/*.test.sh` shares, and the one thing it exists
# for: a case a commit message can name.
#
#     . "$(dirname "${BASH_SOURCE[0]}")/shell-suite.sh"
#
#     suite_case_passed "a description"
#     suite_case_failed "a description" "what went wrong" "and any detail"
#     suite_verdict
#
# # Why a shell case has to be reportable
#
# Red/green needs a commit whose tests fail, and `scripts/red-commit.sh` makes
# one possible by having the commit name the tests that must fail and then
# holding the run to exactly that. It reads cargo's own `test NAME ... FAILED`
# lines, so it could only ever judge a Rust test. A shell suite printed
# `FAIL [desc]: ...` for a failing case and nothing at all for a passing one, so
# a commit naming a shell case was told the case never ran, which reads as a
# wrong name rather than as the red half of red/green. That was windows ledger
# 39, and its practical effect was that a new shell suite and the code making it
# pass had to arrive in one commit.
#
# So every case prints a line in the shape cargo prints, passing or failing:
#
#     test check::a source file no record names answers nothing ... ok
#     test check::the suites run before any mode branch ... FAILED
#
# The shape is reused rather than invented because `red-commit.sh` already reads
# it, and a second parser beside the first is two places describing one thing,
# which is how this repository keeps getting caught. A commit then names a shell
# case exactly as it names a Rust test:
#
#     Fails-until-green: check::the suites run before any mode branch
#
# A passing case has to print a line too, and that is the half worth saying out
# loud. Without it, a named case that passed and a named case that does not
# exist are the same silence, and `red-commit.sh` cannot hold a commit to "every
# named test ran" when nothing says a test ran.
#
# `scripts/check.sh` collects this output into the run log rather than letting a
# failing suite abort the gate, which was the other half of the same hole.
set -uo pipefail

# The suite's own name, taken from the file being run so it cannot drift from
# the name a commit message has to write.
case "$0" in
    *.test.sh) ;;
    *)
        echo "shell-suite: '$0' is not a *.test.sh, so it has no name a commit could write" >&2
        exit 70
        ;;
esac

suite_name="$(basename "$0" .test.sh)"

# The count `suite_verdict` reports on, and the case names already used. Every
# failure goes through `suite_case_failed`, so the exit status and the FAILED
# lines are the same fact rather than two that can disagree.
suite_failures=0
declare -A suite_case_count=()

# One case, reported in the shape cargo reports a test in.
#
# The three refusals are all one rule: the description has to be a name a commit
# message can carry, and the place to find out that it is not is where it is
# written. A case with no description cannot be named at all; one holding
# ` ... ` would be cut in the wrong place, because that is the separator between
# a name and its outcome; and one holding a comma would be split in two, because
# `red-commit.sh names` reads a `Fails-until-green:` value as a comma-separated
# list of Rust test paths, and a Rust path never holds a comma.
#
# The comma was found by using this. The first commit that tried to name a case
# named `which-checks::main, code changed`, and the gate reported two tests
# called `which-checks::main` and `code changed`, neither of which had ever run.
# Refused here rather than guessed at there: a reader deciding which commas were
# separators and which were prose would be a gate deciding by heuristic, and the
# leak in every version of that heuristic is a description whose parts happen to
# look like paths.
suite_case_line() {
    local desc="$1" outcome="$2" name

    if [ -z "$desc" ]; then
        echo "shell-suite: a case in $suite_name.test.sh has no description," >&2
        echo "  so no commit could name it." >&2
        exit 70
    fi
    case "$desc" in
        *" ... "*)
            echo "shell-suite: '$desc' holds ' ... ', which is where the reader" >&2
            echo "  of these lines cuts the name off. Word it another way." >&2
            exit 70
            ;;
        *,*)
            echo "shell-suite: '$desc' holds a comma, and the commit trailer that" >&2
            echo "  names a case separates names with commas. Word it another way." >&2
            exit 70
            ;;
    esac

    name="$suite_name::$desc"
    suite_case_count["$name"]=$(( ${suite_case_count["$name"]:-0} + 1 ))
    printf 'test %s ... %s\n' "$name" "$outcome"
}

suite_case_passed() {
    suite_case_line "$1" ok
}

# The `FAIL [desc]:` line stays, because it carries the detail a person reads,
# and the machine line goes beside it rather than instead of it.
suite_case_failed() {
    local desc="$1"
    shift
    echo "FAIL [$desc]: ${1-}"
    shift || true
    local extra
    for extra in "$@"; do
        echo "       $extra"
    done
    suite_case_line "$desc" FAILED
    suite_failures=$(( suite_failures + 1 ))
}

# Whether the suite reached its own end, said in a line a reader of case lines
# can see.
#
# A suite that dies partway through, on an unset variable or a syntax error,
# leaves its remaining cases unrun and says so nowhere a name could match. In
# the `red` mode that is the difference between a run that can be judged and one
# that cannot, so `check.sh` requires this line from every suite in every mode.
suite_ran_to_the_end="every case in this suite ran"

suite_verdict() {
    suite_case_line "$suite_ran_to_the_end" ok

    # Two cases sharing a name are a name a commit cannot use, because nothing
    # says which of them it meant, and the reader on the other side keeps only
    # one outcome per name. Reported as a case of its own rather than as a bare
    # message, so that it is a named failure like any other and cannot slip
    # through a run that is being judged by the case lines.
    local name
    local -a used_twice=()
    for name in "${!suite_case_count[@]}"; do
        [ "${suite_case_count[$name]}" -gt 1 ] || continue
        used_twice+=("'$name', ${suite_case_count[$name]} times")
    done
    if [ "${#used_twice[@]}" -gt 0 ]; then
        suite_case_failed "no case name is used twice" \
            "these names were reported more than once:" \
            "${used_twice[@]}"
    fi

    if [ "$suite_failures" -eq 0 ]; then
        echo "$suite_name: all cases pass"
    else
        echo "$suite_name: $suite_failures case(s) failed"
        exit 1
    fi
}
