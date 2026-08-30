#!/usr/bin/env bash
# Which checks a branch earns, given its name.
#
# `main` earns all four, because every commit here lands on it and the four
# checks are the whole of what stands between a broken commit and the branch CI
# builds. A branch nobody builds earns everything except the two slow ones, the
# full test suite and the release build, which are 295 of the gate's 311
# seconds, measured warm on 2026-08-30.
#
# This is not a way to commit less-checked work. It is a way to pay for the slow
# half once, where it counts, instead of once per commit on a branch that is
# going to be checked in full before it reaches main anyway. `.githooks/pre-commit`
# already said this in words:
#
#     To commit without it, for a work in progress on a branch nobody builds:
#         git commit --no-verify
#     Not for main.
#
# The difference is that a rule written in a comment is one somebody has to
# remember, and `--no-verify` turns off every check rather than the slow two. So
# the rule lives here, the machine applies it, and nothing has to reach for a
# flag that would also skip formatting and clippy.
#
# Whoever merges such a branch runs `scripts/check.sh` in full first. That is
# the one run the slow half is paid for.
#
# Answers, deliberately spelled out rather than boolean, so a third answer can
# be added later without every caller having to guess what `false` meant:
#
#   all           every check, the gate as it has always been
#   all_but_slow  formatting and clippy; the suite and the release build wait
set -euo pipefail

branch="${1-}"

case "$branch" in
    # Not a branch name at all. A detached HEAD, a rebase in progress, or a git
    # call that returned nothing. A check that cannot tell where it is must not
    # answer "safe": it answers with the whole suite and costs somebody four
    # minutes, which is the right way round for a guard to be wrong.
    "" | HEAD)
        echo all
        ;;
    # Matched exactly. `maintenance` and `mainline` are branches nobody builds
    # and must not inherit main's answer by sharing its first four letters.
    main | master)
        echo all
        ;;
    *)
        echo all_but_slow
        ;;
esac
