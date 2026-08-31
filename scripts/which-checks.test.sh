#!/usr/bin/env bash
# What `which-checks.sh` answers, for every branch and every kind of change.
#
# The rule it encodes has two halves. Where you are decides whether the slow
# checks can be deferred at all: `main` is what CI builds and what ships, so it
# always earns everything. What you changed decides which tests can say anything
# about it, and that half holds on any branch.
#
# A guard that refuses the wrong thing is worse than none, so the allow cases
# below matter as much as the refusals. In particular: a markdown change can
# break a Rust test in this repository, because `tests/house_style.rs` reads
# documents. That caught two real em-dash breaks on 2026-08-31, and a rule that
# skipped tests for markdown would have let both through.
set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
subject="$root/scripts/which-checks.sh"

failures=0

expect() {
    local want="$1" desc="$2"
    shift 2
    local got status
    got="$("$subject" "$@" 2>/dev/null)"
    status=$?
    if [ "$status" -ne 0 ]; then
        echo "FAIL [$desc]: refused (exit $status) instead of answering '$want'"
        echo "       args: $*"
        failures=$((failures + 1))
    elif [ "$got" != "$want" ]; then
        echo "FAIL [$desc]: answered '$got', wanted '$want'"
        echo "       args: $*"
        failures=$((failures + 1))
    fi
}

# A refusal is an answer too, and the cases below are the ones where answering
# anything at all would be the defect.
expect_refused() {
    local desc="$1"
    shift
    if "$subject" "$@" >/dev/null 2>&1; then
        echo "FAIL [$desc]: answered instead of refusing"
        echo "       args: $*"
        failures=$((failures + 1))
    fi
}

# ── Where you are: main always earns everything ─────────────────────────────
# Every commit here lands on main, so the four checks are what stands between a
# broken commit and the branch CI builds. What changed does not soften that.
expect all "main, code changed" main src/presentation/wx_app.rs
expect all "main with no file list" main

# But a document cannot break the release build or a test that never reads one,
# wherever it is committed. Deferring the slow half is about the branch; what a
# change can possibly break is about the change. These are separate questions
# and main only answers the first.
expect docs_only "main, docs only" main docs/changelog.md
expect docs_only "master, docs only" master README.md

# A name that could be read two ways is not a licence.
expect affected "maintenance is a branch nobody builds" maintenance src/lib.rs
expect affected "mainline likewise" mainline src/lib.rs

# ── A check that cannot tell where it is answers with everything ────────────
expect all "detached HEAD" HEAD src/lib.rs
expect all "no branch name at all" "" src/lib.rs

# ── What changed: documents only ────────────────────────────────────────────
# Formatting, clippy, and the targets that read documents. Not the whole
# library, and not nothing.
expect docs_only "one planning file" gsd/plan-02-01 .planning/ROADMAP.md
expect docs_only "several docs" gsd/plan-02-01 docs/changelog.md docs/roadmap.md
expect docs_only "a summary and a context" gsd/x .planning/phases/01/01-SUMMARY.md .planning/phases/01/01-CONTEXT.md
expect docs_only "a readme" gsd/x README.md

# ── What changed: anything the compiler sees ────────────────────────────────
expect affected "one rust file" gsd/x src/application/threading.rs
expect affected "rust beside a doc" gsd/x src/application/threading.rs docs/changelog.md
expect affected "an integration test" gsd/x tests/wired.rs

# Build inputs are not documents, however they are spelled. A dependency bump
# or a lint change reaches everything, so it earns everything.
expect affected "Cargo.toml" gsd/x Cargo.toml
expect affected "Cargo.lock" gsd/x Cargo.lock
expect affected "the hook itself" gsd/x .githooks/pre-commit
expect affected "this decision" gsd/x scripts/which-checks.sh

# `guards/guards.toml` names breaks in source and the runner applies them, so a
# record change is not a document change however much it reads like one.
expect affected "a guard record" gsd/x guards/guards.toml

# ── No file list means we cannot tell, so defer only what the branch allows ──
expect all_but_slow "a branch, nothing said about the change" gsd/x

# ── A commit that says which tests must fail ────────────────────────────────
# Red/green needs a commit whose tests fail, and this gate refuses one unless it
# says so and is held to it. `red-commit.sh` reads the marker; this file decides
# only where such a commit is allowed to be made.

red_marker="$(mktemp)"
plain_message="$(mktemp)"
broken_marker="$(mktemp)"
trap 'rm -f "$red_marker" "$plain_message" "$broken_marker"' EXIT

cat > "$red_marker" <<'MSG'
test(02-02): failing tests for the narrower question set

Fails-until-green: application::saved_searches::tests::test_a
MSG

cat > "$plain_message" <<'MSG'
feat(02-02): the narrower question set

One question instead of three.
MSG

cat > "$broken_marker" <<'MSG'
test(02-02): failing tests

Fails-until-green:
MSG

expect red "a branch, a commit naming the tests that must fail" \
    --message-file="$red_marker" gsd/plan-02-02 src/application/saved_searches.rs

# A document change can redden a document-reading test, so a red commit is not
# a code-only idea and is not refused for touching only markdown.
expect red "a branch, a red commit touching only documents" \
    --message-file="$red_marker" gsd/plan-02-02 docs/changelog.md

# The overwhelmingly common case, and it must not get slower or stranger for
# the sake of the rare one.
expect affected "a message with no marker changes nothing" \
    --message-file="$plain_message" gsd/plan-02-02 src/application/saved_searches.rs

expect docs_only "a message with no marker, documents only" \
    --message-file="$plain_message" gsd/plan-02-02 README.md

# ── Where a red commit may not be made ──────────────────────────────────────

# `main` is what CI builds and every commit here lands on it. A commit that
# leaves a test failing on main is a broken branch for everybody, and no marker
# makes it not one. The red belongs on a branch, and the merge brings the pair.
expect_refused "a red commit on main" \
    --message-file="$red_marker" main src/application/saved_searches.rs

expect_refused "a red commit on master" \
    --message-file="$red_marker" master src/application/saved_searches.rs

# A check that cannot tell where it is must not hand out an exemption, for the
# same reason it answers `all` rather than `affected`.
expect_refused "a red commit on a detached HEAD" \
    --message-file="$red_marker" HEAD src/application/saved_searches.rs

expect_refused "a red commit with no branch name at all" \
    --message-file="$red_marker" "" src/application/saved_searches.rs

# A marker that names no test is a typo, and answering `affected` to it would
# run the tests and refuse the commit while naming the wrong cause.
expect_refused "a marker that names nothing" \
    --message-file="$broken_marker" gsd/plan-02-02 src/application/saved_searches.rs

expect_refused "a message file that is not there" \
    --message-file="$red_marker.absent" gsd/plan-02-02 src/application/saved_searches.rs

if [ "$failures" -eq 0 ]; then
    echo "which-checks: all cases pass"
else
    echo "which-checks: $failures case(s) failed"
    exit 1
fi
