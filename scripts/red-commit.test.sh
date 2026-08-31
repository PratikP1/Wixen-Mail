#!/usr/bin/env bash
# What `red-commit.sh` reads out of a commit message, and what verdict it
# reaches about a test run.
#
# Both halves matter and they fail in opposite directions. A marker this cannot
# see means a red commit is refused and somebody reaches for `--no-verify`. A
# marker it sees where none was meant means any commit can declare its failures
# expected, which is the escape hatch this whole mechanism exists to not be.
set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
subject="$root/scripts/red-commit.sh"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

failures=0

# What `names` prints for a message, as one string with newlines shown as `|`
# so a case reads on one line, **and** whether it succeeded.
#
# The exit status is checked as well as the output, and that is not tidiness. A
# case wanting no names is satisfied for free by a script that dies before
# printing anything, so without this the ordinary-commit cases below would pass
# against a parser that crashes on every message. Measured: they did. A marker
# matching every line makes this script refuse the blank line above the body,
# and all four absence cases stayed green until the status was checked too.
expect_names() {
    local want="$1" desc="$2" message="$3"
    printf '%s' "$message" > "$work/msg"

    local got status
    "$subject" names "$work/msg" > "$work/out" 2>/dev/null
    status=$?
    got="$(paste -sd '|' - < "$work/out")"

    if [ "$status" -ne 0 ]; then
        echo "FAIL [$desc]: refused the message (exit $status) instead of reading '$want'"
        failures=$((failures + 1))
    elif [ "$got" != "$want" ]; then
        echo "FAIL [$desc]: read '$got', wanted '$want'"
        failures=$((failures + 1))
    fi
}

expect_names_refused() {
    local desc="$1" message="$2"
    printf '%s' "$message" > "$work/msg"
    if "$subject" names "$work/msg" >/dev/null 2>&1; then
        echo "FAIL [$desc]: accepted a malformed marker instead of refusing it"
        failures=$((failures + 1))
    fi
}

# ── An ordinary commit says nothing about failing tests ─────────────────────
# The common case by far, and the one that must stay cheap: no marker, nothing
# read, and the commit earns whatever checks it would have earned anyway.

expect_names "" "a subject and nothing else" \
    "feat(02-02): the narrower question set"

expect_names "" "a body that does not mention it" \
    "feat(02-02): the narrower question set

One question instead of three, so there is no absent value to interpret."

# Prose about redness is not a marker. This repository's commit messages
# describe what went red in sentences, and `test(01-14)`'s body really does say
# \"Four are red\". A reader of messages must not mistake that for a machine
# instruction, or every honest description of a measurement becomes a licence.
expect_names "" "prose describing a measurement" \
    "test(01-14): the folder tree holds every account, not the open one

Six tests through folder_tree_updates. Four are red: no second branch, one
Inbox row instead of two, no id for the second account, and branches ignoring
the stored order. Red is the point of this commit."

# ── A marker is read, and read exactly ──────────────────────────────────────

expect_names "application::allowed::tests::test_a" "one named test" \
    "test(02-02): failing tests for the narrower question set

Fails-until-green: application::allowed::tests::test_a"

expect_names "application::allowed::tests::test_a|application::allowed::tests::test_b" \
    "two markers, in the order written" \
    "test(02-02): failing tests

Fails-until-green: application::allowed::tests::test_a
Fails-until-green: application::allowed::tests::test_b"

expect_names "application::allowed::tests::test_a|application::allowed::tests::test_b" \
    "two names on one marker" \
    "test(02-02): failing tests

Fails-until-green: application::allowed::tests::test_a, application::allowed::tests::test_b"

expect_names "application::allowed::tests::test_a" "spaces around the name" \
    "test(02-02): failing tests

Fails-until-green:    application::allowed::tests::test_a   "

# Spelling it in lower case is a mistake somebody will make, and the safe
# reading is to see it rather than to answer \"no marker\" and refuse the
# commit for a reason that names the wrong thing.
expect_names "application::allowed::tests::test_a" "spelled in lower case" \
    "test(02-02): failing tests

fails-until-green: application::allowed::tests::test_a"

# ── What must not be read as a marker ───────────────────────────────────────

# The subject is what shows up in every log, and a marker there would be a
# commit announcing its own exemption in the one line everybody reads. It is
# also how a marker would arrive by accident, from a subject that happens to
# start with the words.
expect_names "" "the subject line is not a trailer" \
    "Fails-until-green: application::allowed::tests::test_a

A body."

# An indented line is a quotation or a worked example, not an instruction. This
# was found by the tool refusing the commit that introduced it: that message
# showed the marker's own shape, indented, inside its explanation of what the
# marker is. A message that cannot describe the mechanism without invoking it
# is a mechanism nobody can document.
expect_names "" "an indented example of the marker" \
    "Let the red half of red/green be committed

A commit now names the tests that must fail:

    Fails-until-green: application::x::tests::test_a

Not an exemption."

# ── A marker that says nothing is refused, not ignored ──────────────────────
# Ignoring it answers \"ordinary commit\", the tests run, the named test fails,
# and the commit is refused with a message about a failing test rather than
# about the typo that caused it. A check that cannot tell must not answer safe.

expect_names_refused "a marker with no test after it" \
    "test(02-02): failing tests

Fails-until-green:"

expect_names_refused "a marker with only whitespace after it" \
    "test(02-02): failing tests

Fails-until-green:    "

expect_names_refused "a marker whose list ends in a stray comma" \
    "test(02-02): failing tests

Fails-until-green: application::allowed::tests::test_a,"

# ── The verdict on a run ────────────────────────────────────────────────────
# Three conditions, and dropping any one of them turns the marker into a way to
# commit anything. Every named test must have run; every named test must have
# failed; nothing else may have failed.

verdict_is() {
    local want="$1" desc="$2" named="$3" output="$4"
    printf '%s\n' "$named" > "$work/named"
    printf '%s\n' "$output" > "$work/output"

    local status
    "$subject" verdict "$work/named" "$work/output" > "$work/said" 2>&1
    status=$?

    local got=refused
    [ "$status" -eq 0 ] && got=accepted
    if [ "$got" != "$want" ]; then
        echo "FAIL [$desc]: $got the run, wanted it $want"
        echo "       said: $(head -2 "$work/said" | paste -sd ' ' -)"
        failures=$((failures + 1))
    fi
}

# The ordinary case, and the positive control for every refusal below: if this
# one did not pass, a refusal proves nothing, because a verdict that refuses
# everything would satisfy all of them.
verdict_is accepted "exactly the named tests failed" \
    "application::allowed::tests::test_a
application::allowed::tests::test_b" \
    "running 3 tests
test application::allowed::tests::test_a ... FAILED
test application::allowed::tests::test_b ... FAILED
test application::allowed::tests::test_c ... ok

test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out"

# A named test that passes means the commit's own account of itself is wrong.
# Either the test does not test what it claims, or the behaviour is already
# there and there was nothing to drive.
verdict_is refused "a named test passed" \
    "application::allowed::tests::test_a
application::allowed::tests::test_b" \
    "running 2 tests
test application::allowed::tests::test_a ... FAILED
test application::allowed::tests::test_b ... ok

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out"

# The one that makes this a gate rather than a hatch. Without it a commit could
# name one expected failure and carry twenty unexpected ones.
verdict_is refused "something failed that was not named" \
    "application::allowed::tests::test_a" \
    "running 2 tests
test application::allowed::tests::test_a ... FAILED
test data::config::tests::test_unrelated ... FAILED

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out"

# A name with a typo in it, or a test in a module the scoped run never reached.
# Both look identical to \"did not fail\" and neither is, so they are told apart
# from a passing test and reported as what they are.
verdict_is refused "a named test never ran" \
    "application::allowed::tests::test_a" \
    "running 1 test
test data::config::tests::test_unrelated ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"

# An ignored test did not fail and did not run. Counting it as either would be
# wrong, and counting it as a failure would let `#[ignore]` stand in for red.
verdict_is refused "a named test was ignored" \
    "application::allowed::tests::test_a" \
    "running 1 test
test application::allowed::tests::test_a ... ignored

test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out"

# A test that does not compile is not a red. It is a broken build, and it must
# not be reported as the failure the commit was expecting.
verdict_is refused "nothing compiled, so nothing ran" \
    "application::allowed::tests::test_a" \
    "   Compiling wixen-mail v0.46.0 (C:\\Users\\prati\\Documents\\projects\\Wixen-Mail)
error[E0425]: cannot find function \`may_i_read\` in this scope
   --> src/service/protocols/imap.rs:1107:14
error: could not compile \`wixen-mail\` (lib test) due to 1 previous error"

# The same test really is reported once per target when a run covers more than
# one, and two reports of one failure are still one failure.
verdict_is accepted "one failure reported by two targets" \
    "application::allowed::tests::test_a" \
    "running 1 test
test application::allowed::tests::test_a ... FAILED

running 1 test
test application::allowed::tests::test_a ... FAILED

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out"

if [ "$failures" -eq 0 ]; then
    echo "red-commit: all cases pass"
else
    echo "red-commit: $failures case(s) failed"
    exit 1
fi
