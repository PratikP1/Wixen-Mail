#!/usr/bin/env bash
#
# Would each guard still go red?
#
# A guard test is written, taken red once against the defect it defends, and
# then never measured again. The suite runs it thousands of times afterwards
# and every one of those runs asks the same weak question: does it pass. None
# of them asks the question it was written to answer: would it fail if the
# thing it guards broke.
#
# So a guard can quietly stop guarding anything. It happened here between two
# commits in one afternoon. One of them added an arm to a decision in the
# contacts sync, and two tests that had been reaching the arm their guard was
# about started reaching the new one instead. Both arms do nothing, so the
# counts, the sentence and the flag those tests assert came out identical
# either way. Two guards became one. Both kept their names. Nothing was red at
# any point, and it was found by hand three commits later.
#
# `guards/guards.toml` is that measurement written down: for each guard, the
# exact edit that should break it and the tests that should go red when it
# does. This applies each one, runs the whole library, and requires the tests
# that failed to be exactly the ones the record names.
#
# Exactly, in both directions, and the second direction is newer than the
# first. Running only the named tests answers "would these go red" and cannot
# see the answer that matters as much: did anything else. A record here named
# eight tests for a break that reddens seventeen, because a later commit gave
# the function two new callers and nobody measured it again. Nine tests were
# guarding something nobody had written down, the run said nothing, and a
# record shorter than the truth is the thing that file exists to stop.
#
# Not part of scripts/check.sh, and not in the commit hook. It is one build and
# one full run of the library per guard, an hour or two for all of them rather
# than seconds, so it belongs with mutation testing: run it after a change that
# touches code a guard is about, and read the answer.
#
# Nothing else may be building while it runs. A commit hook running the suite
# in the middle of one already reported three guards green that go red on their
# own.
#
# Usage:
#   scripts/guards.sh              every guard in the record
#   scripts/guards.sh deletion     only the guards whose names match
#
# It fails three ways and says which. A named test that stayed green is a guard
# that has stopped defending anything. A test that went red and is not named is
# a guard nobody wrote down. A break that no longer matches the file means
# somebody moved the code underneath it. All three are the moment to measure
# that guard by hand again rather than to edit the record until it applies.
set -euo pipefail

cd "$(dirname "$0")/.."

if ! command -v python >/dev/null 2>&1; then
    echo "python is not on the path, and the record is read with it."
    exit 1
fi

exec python scripts/guards.py "$@"
