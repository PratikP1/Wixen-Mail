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
#   red           format, clippy, the tests that reach what changed, and the
#                 run held to exactly the failures the commit named
#
# `red` is the RED half of red/green, which this gate refused outright once the
# hook was turned on: a commit whose tests fail could not be made, and
# `--no-verify` is not available here. It is not an exemption. A commit earns it
# by naming the tests that must fail, and `red-commit.sh` then requires every
# named test to have run, every named test to have failed, and nothing else to
# have failed. That costs more to misuse than to use honestly.
#
# `scripts/check.sh` turns these into commands. The decision lives here so it can
# be asked a question without minutes of checking to find out the answer, and it
# is tested in `which-checks.test.sh`, both halves: what each answer should be,
# and the cases that must not collapse into a weaker one.
set -euo pipefail

# The commit message, when there is one. Only a commit knows whether it is the
# RED half of red/green, and only the message can say so, which is why the gate
# runs from `commit-msg` rather than `pre-commit`: the message does not exist
# yet when `pre-commit` runs.
message_file=""
case "${1-}" in
    --message-file=*)
        message_file="${1#--message-file=}"
        shift
        ;;
esac

branch="${1-}"
shift || true

# A commit that names the tests it expects to fail. `red-commit.sh` reads the
# marker and refuses a malformed one; this only decides where such a commit may
# be made, and the answer is: on a branch, never on main.
#
# Not an exemption. The `red` answer still runs the tests, and holds the run to
# exactly the tests the commit named. What it buys is the ability to commit a
# failing test at all, which this gate took away when the hook was turned on.
if [ -n "$message_file" ]; then
    named="$("$(dirname "$0")/red-commit.sh" names "$message_file")" || exit $?
    if [ -n "$named" ]; then
        case "$branch" in
            "" | HEAD | main | master)
                echo "which-checks: a commit naming tests that must fail cannot be made" >&2
                echo "  on '${branch:-an unknown branch}'. Every commit here lands on what CI" >&2
                echo "  builds, and a failing test on it is a broken branch for everybody." >&2
                echo "  Make the red commit and its green pair on a branch, and merge them" >&2
                echo "  together." >&2
                exit 1
                ;;
        esac
        echo red
        exit 0
    fi
fi

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
        # `main` cannot defer the slow half, because every commit here lands on
        # it and it is what CI builds. That is a statement about the branch. It
        # is not a statement about what a change can break, and those are
        # separate questions: a document cannot fail a release build or a test
        # that never reads one, wherever it is committed. So fall through to the
        # what-changed question with the slow half still owed.
        if [ "$#" -eq 0 ]; then
            echo all
            exit 0
        fi
        for path in "$@"; do
            case "$path" in
                *.md | *.txt) ;;
                *)
                    echo all
                    exit 0
                    ;;
            esac
        done
        echo docs_only
        exit 0
        ;;
esac

# Nothing said about what changed. The branch allows deferring the slow half,
# but with no file list there is nothing to scope the tests to.
if [ "$#" -eq 0 ]; then
    echo all_but_slow
    exit 0
fi

# A manifest reaches every target, so it earns every check.
#
# Not a special case bolted on: it is the markdown rule at the top of this file
# read the other way round. That rule exists because a document can break a Rust
# test, since `house_style` reads documents. A manifest can too, and something
# reads it: `service::outward` classifies every dependency by whether it can
# reach a server.
#
# Without this, a manifest change answered `affected`, which maps a changed file
# to a module by path and finds none for `Cargo.toml`, so the commit ran
# formatting, clippy and the two tree guards and no tests at all. Clippy catches
# a manifest change that breaks the build; it does not catch one that breaks a
# test reading the manifest as data. On 2026-09-02 adding a `[features]` section
# put this package into its own dependency list, reddened that census, and the
# break survived three commits before a hand-run of the library found it.
#
# Manifests change rarely, so answering `all` costs little in aggregate and asks
# nobody to remember which tests read them.
for path in "$@"; do
    case "$path" in
        Cargo.toml | Cargo.lock | */Cargo.toml | */Cargo.lock)
            echo all
            exit 0
            ;;
    esac
done

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
