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

# shellcheck source=scripts/shell-suite.sh
. "$root/scripts/shell-suite.sh"

# Everything this suite writes, under one directory and one trap. Two traps on
# EXIT are one trap: the second replaces the first, and the files the first
# named are left behind.
scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT

# Where the subject is run from. The repository root for every case but one:
# the case that proves which-checks.sh really reads the index runs from a
# repository of its own, so it cannot be answered by whatever happens to be
# staged here and cannot touch it either.
run_from="$root"

expect() {
    local want="$1" desc="$2"
    shift 2
    local got status
    got="$(cd "$run_from" && "$subject" "$@" 2>/dev/null)"
    status=$?
    if [ "$status" -ne 0 ]; then
        suite_case_failed "$desc" \
            "refused (exit $status) instead of answering '$want'" "args: $*"
    elif [ "$got" != "$want" ]; then
        suite_case_failed "$desc" "answered '$got', wanted '$want'" "args: $*"
    else
        suite_case_passed "$desc"
    fi
}

expect_from() {
    local where="$1"
    shift
    local previously="$run_from"
    run_from="$where"
    expect "$@"
    run_from="$previously"
}

# A refusal is an answer too, and the cases below are the ones where answering
# anything at all would be the defect.
expect_refused() {
    local desc="$1"
    shift
    if "$subject" "$@" >/dev/null 2>&1; then
        suite_case_failed "$desc" "answered instead of refusing" "args: $*"
    else
        suite_case_passed "$desc"
    fi
}

# ── Where you are: main always earns everything ─────────────────────────────
# Every commit here lands on main, so the four checks are what stands between a
# broken commit and the branch CI builds. What changed does not soften that.
expect all "main with code changed" main src/presentation/wx_app.rs
expect all "main with no file list" main

# But a document cannot break the release build or a test that never reads one,
# wherever it is committed. Deferring the slow half is about the branch; what a
# change can possibly break is about the change. These are separate questions
# and main only answers the first.
expect docs_only "main with only documents changed" main docs/changelog.md
expect docs_only "master with only documents changed" master README.md

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
#
# The manifests really do earn everything, and until 2026-09-02 this file said
# so in the comment above while asserting `affected` underneath. `affected` maps
# a changed file to a module by its path, and a manifest maps to none, so a
# manifest commit ran formatting, clippy and the two tree guards and no tests at
# all. Clippy catches a manifest change that breaks the build. It does not catch
# one that breaks a test reading the manifest as data, and that is exactly what
# happened: adding a `[features]` section put this package into its own
# dependency list and reddened the census that reads `Cargo.toml`, which then
# survived three commits.
#
# This is the same shape as the markdown rule at the top of this file, which was
# extended to documents because `house_style` reads them, and never extended to
# manifests. A manifest changes rarely, so `all` costs little in aggregate and
# is the honest answer: the dependency graph and the feature set reach every
# target.
#
# With one exception, and it is the version line. This project puts the bump in
# the same commit as the change it describes, because a version arriving later
# describes a build nobody made, so nearly every commit in a phase touches both
# manifests and nearly every commit was paying the whole gate for it. A diff
# confined to the package's own version adds no dependency, no feature and no
# build input: nothing that reaches a target reaches it. The tests that do read
# the shipped version live in `tests/house_style.rs`, which every scoped run
# ends with, so they are not skipped by the softer answer.
#
# The diff is handed over here rather than read from the index, so these answers
# do not depend on what happens to be staged while the suite runs. One case
# below hands over nothing on purpose.
manifest_diff() {
    cat > "$scratch/$1"
    printf '%s' "$scratch/$1"
}

version_bump="$(manifest_diff version-bump.diff <<'DIFF'
diff --git a/Cargo.lock b/Cargo.lock
index acf60ed..99ec611 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -6720,7 +6720,7 @@ dependencies = [
 [[package]]
 name = "wixen-mail"
-version = "0.99.0"
+version = "0.100.0"
 dependencies = [
diff --git a/Cargo.toml b/Cargo.toml
index 51b601b..62db6ac 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "wixen-mail"
-version = "0.99.0"
+version = "0.100.0"
 edition = "2024"
DIFF
)"

a_dependency_moved_too="$(manifest_diff dependency.diff <<'DIFF'
diff --git a/Cargo.lock b/Cargo.lock
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -100,7 +100,7 @@
 [[package]]
 name = "serde"
-version = "1.0.200"
+version = "1.0.201"
 source = "registry+https://github.com/rust-lang/crates.io-index"
@@ -6720,7 +6720,7 @@
 [[package]]
 name = "wixen-mail"
-version = "0.99.0"
+version = "0.100.0"
 dependencies = [
DIFF
)"

a_feature_too="$(manifest_diff feature.diff <<'DIFF'
diff --git a/Cargo.toml b/Cargo.toml
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -1,6 +1,9 @@
 [package]
 name = "wixen-mail"
-version = "0.99.0"
+version = "0.100.0"
 edition = "2024"
+
+[features]
+telemetry = []
DIFF
)"

a_dependencys_own_version="$(manifest_diff dependency-table.diff <<'DIFF'
diff --git a/Cargo.toml b/Cargo.toml
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -40,7 +40,7 @@
 [dependencies.wxdragon]
-version = "0.9.0"
+version = "0.10.0"
 features = ["webview"]
DIFF
)"

nothing_readable="$(manifest_diff empty.diff < /dev/null)"

expect all "Cargo.toml" --manifest-diff-file="$a_feature_too" gsd/x Cargo.toml
expect all "Cargo.lock" --manifest-diff-file="$a_dependency_moved_too" gsd/x Cargo.lock
expect all "a manifest beside a source file" \
    --manifest-diff-file="$a_feature_too" gsd/x src/lib.rs Cargo.toml

# The version line, and only the version line.
expect affected "a version bump and nothing else in the manifest" \
    --manifest-diff-file="$version_bump" gsd/x Cargo.toml Cargo.lock

# Everything else in a manifest still earns everything, and these are the four
# ways the softer answer could leak. The lock file is the one worth naming: a
# dependency's version line reads exactly like the package's own, and what tells
# them apart is whose `name` sits above it in the hunk.
expect all "a lock file that also moved a dependency version" \
    --manifest-diff-file="$a_dependency_moved_too" gsd/x Cargo.lock
expect all "a feature added beside a version bump" \
    --manifest-diff-file="$a_feature_too" gsd/x Cargo.toml
expect all "a version line under a dependency table" \
    --manifest-diff-file="$a_dependencys_own_version" gsd/x Cargo.toml
expect all "a manifest diff that came back empty" \
    --manifest-diff-file="$nothing_readable" gsd/x Cargo.toml

# A check that cannot see what changed must not hand out the softer answer, and
# on main the branch decides first anyway: everything that is not a document
# earns everything, whatever the diff says.
expect all "main with a version bump" \
    --manifest-diff-file="$version_bump" main Cargo.toml Cargo.lock

# And the path a commit really takes, which hands over no diff at all. Without
# this case the git read could come back empty on every commit, every manifest
# would answer `all` exactly as it does today, and every case above would still
# pass: a fixture proves the reading and not the reader.
a_repo_of_its_own="$scratch/a-repo"
mkdir -p "$a_repo_of_its_own/no-hooks"
git -C "$a_repo_of_its_own" init --quiet
# This project's hook is configured on this repository rather than globally, so
# a fresh one does not inherit it. Pinned at an empty directory anyway: a global
# hooksPath set one day would otherwise run the whole gate from inside a suite
# the gate is running.
git -C "$a_repo_of_its_own" config core.hooksPath "$a_repo_of_its_own/no-hooks"
git -C "$a_repo_of_its_own" config user.email "suite@example.invalid"
git -C "$a_repo_of_its_own" config user.name "the suite"
printf '[package]\nname = "wixen-mail"\nversion = "0.99.0"\nedition = "2024"\n' \
    > "$a_repo_of_its_own/Cargo.toml"
git -C "$a_repo_of_its_own" add Cargo.toml
git -C "$a_repo_of_its_own" commit --quiet -m "the manifest before the bump"
printf '[package]\nname = "wixen-mail"\nversion = "0.100.0"\nedition = "2024"\n' \
    > "$a_repo_of_its_own/Cargo.toml"
git -C "$a_repo_of_its_own" add Cargo.toml

expect_from "$a_repo_of_its_own" affected "a version bump staged in a repository of its own" \
    gsd/x Cargo.toml
expect affected "the hook itself" gsd/x .githooks/pre-commit
expect affected "this decision" gsd/x scripts/which-checks.sh

# `guards/guards.toml` names breaks in source and the runner applies them, so a
# record change is not a document change however much it reads like one.
expect affected "a guard record" gsd/x guards/guards.toml

# ── No file list means we cannot tell, so defer only what the branch allows ──
expect all_but_slow "a branch with nothing said about the change" gsd/x

# ── A commit that says which tests must fail ────────────────────────────────
# Red/green needs a commit whose tests fail, and this gate refuses one unless it
# says so and is held to it. `red-commit.sh` reads the marker; this file decides
# only where such a commit is allowed to be made.

red_marker="$scratch/red-marker"
plain_message="$scratch/plain-message"
broken_marker="$scratch/broken-marker"

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

expect red "a branch and a commit naming the tests that must fail" \
    --message-file="$red_marker" gsd/plan-02-02 src/application/saved_searches.rs

# A document change can redden a document-reading test, so a red commit is not
# a code-only idea and is not refused for touching only markdown.
expect red "a branch and a red commit touching only documents" \
    --message-file="$red_marker" gsd/plan-02-02 docs/changelog.md

# The overwhelmingly common case, and it must not get slower or stranger for
# the sake of the rare one.
expect affected "a message with no marker changes nothing" \
    --message-file="$plain_message" gsd/plan-02-02 src/application/saved_searches.rs

expect docs_only "a message with no marker and only documents changed" \
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

suite_verdict
