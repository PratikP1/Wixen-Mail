#!/usr/bin/env bash
# Which checks a commit earns, given where it is and what it changes.
#
#   which-checks.sh <branch> [changed-file ...]
#
# Two questions, asked in that order.
#
# **Where you are** decides whether the slow checks can be deferred at all.
# `main` is what CI builds and what ships, and every commit here lands on it, so
# it always earns everything. A branch nobody builds can defer, because the full
# gate runs once before the merge.
#
# **What you changed** decides which tests can say anything. Running 5,819 tests
# and a release build to commit four markdown files proves nothing about the
# markdown and costs about 330 seconds; the same commit's useful checks take
# about 25.
#
# The obvious version of that second rule is wrong here, and it is worth saying
# why, because it looks right. **A markdown change can break a Rust test in this
# repository**: `tests/house_style.rs` reads documents, and its em-dash guard
# caught two real breaks on 2026-08-31, one in `CLAUDE.md` and one in a planning
# file. A rule that skipped tests for documents would have let both through. So
# a document change still runs the targets that read documents.
#
# Answers, spelled out rather than boolean so a fifth can be added without every
# caller having to guess what the fourth meant:
#
#   all           every check: format, clippy, the whole suite, the release build
#   affected      format, clippy, and the tests that reach what changed
#   docs_only     format, clippy, and the targets that read documents
#   all_but_slow  format and clippy; nothing said about what changed, so nothing
#                 can be scoped to it
#
# `scripts/check.sh` turns these into commands. The decision lives here so it can
# be asked a question without minutes of checking to find out the answer, and it
# is tested in `which-checks.test.sh`, both halves: what each answer should be,
# and the cases that must not collapse into a weaker one.
set -euo pipefail

branch="${1-}"
shift || true

case "$branch" in
    # Not a branch name at all: a detached HEAD, a rebase in progress, or a git
    # call that returned nothing. A check that cannot tell where it is must not
    # answer "safe". It answers with the whole suite and costs somebody five
    # minutes, which is the right way round for a guard to be wrong.
    "" | HEAD)
        echo all
        exit 0
        ;;
    # Matched exactly. `maintenance` and `mainline` are branches nobody builds
    # and must not inherit main's answer by sharing its first four letters.
    main | master)
        echo all
        exit 0
        ;;
esac

# Nothing said about what changed. The branch allows deferring the slow half,
# but with no file list there is nothing to scope the tests to.
if [ "$#" -eq 0 ]; then
    echo all_but_slow
    exit 0
fi

# A document is a file whose content only a document-reading test can judge.
# Everything else is a build input, however much it reads like prose:
# `guards/guards.toml` names breaks the runner applies to source, `Cargo.toml`
# and `Cargo.lock` reach every crate, and a change to this script or to the hook
# changes what checking even means.
for path in "$@"; do
    case "$path" in
        *.md | *.txt) ;;
        *)
            echo affected
            exit 0
            ;;
    esac
done

echo docs_only
