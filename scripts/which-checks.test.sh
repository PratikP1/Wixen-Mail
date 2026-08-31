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
    local got
    got="$("$subject" "$@" 2>/dev/null)"
    if [ "$got" != "$want" ]; then
        echo "FAIL [$desc]: answered '$got', wanted '$want'"
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

if [ "$failures" -eq 0 ]; then
    echo "which-checks: all cases pass"
else
    echo "which-checks: $failures case(s) failed"
    exit 1
fi
