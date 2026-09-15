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
# does. This applies each one, runs the whole of the suite that guard names,
# and requires the tests that failed to be exactly the ones the record names.
# The suite is the library unless the record says otherwise, which a handful of
# them do because a rule about what the tree says is checked from `tests/` and
# `cargo test --lib` never builds that.
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
# one full run per guard, hours for all of them rather than seconds, so it
# belongs with mutation testing: run it after a change that touches code a
# guard is about, and read the answer. How many hours is a rate times a count,
# and both terms are rows on docs/development/measurements.md with the product
# beside them; the runner prints the rate after every record as a `timed:`
# line. This comment put the whole run at one or two hours from 2026-08-08,
# when it was written, until 2026-09-14.
#
# Nothing else may be building while it runs. A commit hook running the suite
# in the middle of one already reported three guards green that go red on their
# own. That was a sentence from 2026-08-08 until 2026-09-14, and it is now a
# check: `--wait-until-quiet` below.
#
# Usage:
#   scripts/guards.sh              every guard in the record
#   scripts/guards.sh deletion     only the guards whose names match
#   scripts/guards.sh --remeasure "a name" "another"
#                                  exactly those records, and write down the
#                                  tree each one agreed with
#   scripts/guards.sh --log sweep.log --wait-until-quiet
#                                  the whole sweep, detached: every line
#                                  appended to the log as well as printed,
#                                  and each record held until no cargo or
#                                  rustc this run did not start is alive
#   scripts/guards.sh --log sweep.log --resume --wait-until-quiet
#                                  pick a stopped sweep up from its own log:
#                                  every record the log holds a verdict for
#                                  is skipped, a name with no verdict or one
#                                  marked contended is measured again, and
#                                  it refuses to start over a guarded file
#                                  the tree shows as modified
#   scripts/guards.sh --log sweep.log --resume --stop-after 20
#                                  a chunk of a known size, then a line
#                                  saying how many remain and how to resume;
#                                  --stop-after 0 reports what remains and
#                                  measures nothing
#   scripts/guards.sh --shard 3/41 --log sweep-3-of-41.log
#                                  shard 3 of 41, counting from 0: one
#                                  contiguous block of the file's records, the
#                                  same on every machine for the same file and
#                                  N, which is what one runner job takes when
#                                  .github/workflows/guards.yml fans the sweep
#                                  out over N jobs; the logs are concatenated
#                                  in shard order and read back with --resume
#
# `--log` appends and never truncates, so a resumed run continues the file it
# read; `Start-Process` has no append and a shell redirection would overwrite
# the log the run is resuming from. The runner's `finally` puts a broken file
# back on an interrupt and not on a process-tree kill, which is why `--resume`
# reads `git status` over every guarded file first and prints the `git
# checkout` that cleans one it finds modified.
#
# `--remeasure` is what the commit gate sends people here for. Each record
# writes down how many tests were in the files its red list names, and
# `test_every_guard_record_says_how_many_tests_the_files_it_names_held` fails
# the moment one of those files gains or loses a test. That test prints the
# records and the command, so the remedy is a build and a run each rather than
# the whole file's worth of them.
#
# The counts are written only for records whose measurement agreed with what
# they say. A record that came out short is corrected by hand first, and it is
# the run after that which writes its count down.
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

# The wording rule this script's own output has to follow, checked before the
# output is written. The product keeps a guard called "a count and the thing it
# counts agree in number" for exactly this, and the tool that measures that
# guard printed "1 tests went red". Milliseconds, and it fails the run.
python -m doctest scripts/guards.py

exec python scripts/guards.py "$@"
