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

# Where the subject is run from. The repository root for every case but one
# until 2026-09-23: the case that proves which-checks.sh really reads the index
# runs from a repository of its own, so it cannot be answered by whatever
# happens to be staged here and cannot touch it either.
#
# Since that day (12-03.2) every other case answers from a tree of its own too.
# The subject now reads `src/` under the directory it is run from, for the files
# a source compiles in, so a case asked from the repository root would read this
# repository's sources and could go red on a commit that owes this suite
# nothing. The tree holds copies of the include lines the cases need, each
# compared with the real one when it was written, and the files they name.
a_tree="$scratch/a-tree"
mkdir -p "$a_tree/src/service/spellcheck" "$a_tree/src/common" "$a_tree/src/presentation" \
    "$a_tree/data" "$a_tree/locales/en-US"
# Copied from `src/service/spellcheck/mod.rs:1050` and `src/common/catalogue.rs:153`
# on 2026-09-23, each compared with its source by a read-only command before a
# case used it. The third is not a copy: it is the shape rustfmt gives an
# include whose argument does not fit on the line with the macro.
printf '%s\n' 'const CORE_ENGLISH_WORDS: &str = include_str!("../../../data/dictionary_en.txt");' \
    > "$a_tree/src/service/spellcheck/mod.rs"
printf '%s\n' '    dates: include_str!("../../locales/en-US/dates.ftl"),' \
    > "$a_tree/src/common/catalogue.rs"
printf '%s\n' 'const THE_WORDS_A_LONG_NAME_NEEDS: &str = include_str!(' \
    '    "../../data/the_words_somebody_gave_a_long_name_to.txt"' \
    ');' > "$a_tree/src/presentation/wrapped.rs"
for named in data/dictionary_en.txt locales/en-US/dates.ftl \
    data/the_words_somebody_gave_a_long_name_to.txt data/nothing_compiles_this.txt; do
    : > "$a_tree/$named"
done
run_from="$a_tree"

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

# ── An option this script does not know ─────────────────────────────────────
# Added 2026-09-23 by 12-03.2. The option loop took the two it knew and read
# anything else as the branch name, so `--merge main src/lib.rs` answered as a
# branch called `--merge` with `main` as a changed path. A misspelt option is
# refused, naming it, rather than answered as a branch nobody has.
unknown_option_said="$(cd "$run_from" && "$subject" --not-an-option gsd/x src/lib.rs 2>&1)"
unknown_option_status=$?
if [ "$unknown_option_status" -eq 64 ] &&
    printf '%s\n' "$unknown_option_said" | grep -q -- "--not-an-option"; then
    suite_case_passed "an option this script does not know is refused"
else
    suite_case_failed "an option this script does not know is refused" \
        "exited $unknown_option_status, wanted 64 with the option named" \
        "said: $unknown_option_said"
fi

# ── What changed: a file the program compiles in ────────────────────────────
# Added 2026-09-23 by 12-03.2. `data/dictionary_en.txt` is compiled into the
# spellchecker with `include_str!`, so it is a build input however it is spelled,
# and a commit changing only it answered `docs_only` and ran no spellcheck test
# before CI. Pratik approved the rule for the dictionary that day and answered
# the same day that it covers every file the program compiles in, the date
# catalogue included (ledger 373).
expect affected "a dictionary the program compiles in is not a document" \
    gsd/x data/dictionary_en.txt
expect all "a compiled-in dictionary on main earns everything" \
    main data/dictionary_en.txt
expect src/service/spellcheck/mod.rs "the sources compiling the dictionary are named" \
    --sources-compiling data/dictionary_en.txt
expect src/common/catalogue.rs "the sources compiling the date catalogue are named" \
    --sources-compiling locales/en-US/dates.ftl
expect affected "a file compiled in by a macro rustfmt wrapped is not a document" \
    gsd/x data/the_words_somebody_gave_a_long_name_to.txt

# The other half: a text file no source compiles in is still a document.
expect docs_only "a text file nothing compiles in is still a document" \
    gsd/x data/nothing_compiles_this.txt

# And these cases answer from the tree above, not from this repository.
if [ "$run_from" != "$root" ] && [ -f "$run_from/src/service/spellcheck/mod.rs" ]; then
    suite_case_passed "the cases here answer from a tree of their own"
else
    suite_case_failed "the cases here answer from a tree of their own" \
        "run from '$run_from', which is the repository or holds no fixture source"
fi

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
#
# A repository of its own: one commit holding the manifest at 0.99.0, and the
# bump to 0.100.0 staged on top of it. This project's hook is configured on this
# repository rather than globally, so a fresh one does not inherit it. Pinned at
# an empty directory anyway: a global hooksPath set one day would otherwise run
# the whole gate from inside a suite the gate is running.
a_repository_of_its_own() {
    local repo="$1"
    mkdir -p "$repo/no-hooks"
    git -C "$repo" init --quiet
    git -C "$repo" config core.hooksPath "$repo/no-hooks"
    git -C "$repo" config user.email "suite@example.invalid"
    git -C "$repo" config user.name "the suite"
    printf '[package]\nname = "wixen-mail"\nversion = "0.99.0"\nedition = "2024"\n' \
        > "$repo/Cargo.toml"
    git -C "$repo" add Cargo.toml
    git -C "$repo" commit --quiet -m "the manifest before the bump"
    printf '[package]\nname = "wixen-mail"\nversion = "0.100.0"\nedition = "2024"\n' \
        > "$repo/Cargo.toml"
    git -C "$repo" add Cargo.toml
}

a_repo_of_its_own="$scratch/a-repo"
a_repository_of_its_own "$a_repo_of_its_own"

expect_from "$a_repo_of_its_own" affected "a version bump staged in a repository of its own" \
    gsd/x Cargo.toml

# ── A repository of its own is its own whatever a hook handed the suite ──────
# The commit hook runs `check.sh`, which runs every suite here as a child, and
# git hands a hook the environment of the commit in progress: `GIT_DIR`, which
# is absolute in a linked worktree, and `GIT_INDEX_FILE`, which is absolute
# when a partial commit builds a temporary index. `git -C` changes the
# directory and not the repository once either is absolute, so on 2026-09-18
# the fixture above, run by the hook from the linked worktree `wixen-mail-sweep`,
# marked this repository bare, replaced its hooks path and put its commit on
# `main`; and the same evening, under a partial commit's temporary index, the
# subject read the real commit's index from inside the fixture and answered
# `all`. The harness every suite sources clears those variables before any
# suite's first `git`, and these two cases hold it to that.
#
# Neither case touches this repository. Each is handed a throwaway repository,
# `elsewhere`, and the case asks whether the fixture stayed out of it. If the
# clearing ever stops, the fixture lands in `elsewhere` and the case is red;
# nothing points at the repository running the suite.
#
# What a hook does to a suite, done to one: a fresh `bash` that sources the
# harness first, as every suite does, then builds the fixture and runs the
# subject from it, with the one variable under test handed in and the other
# four absent. So the case is about that variable alone, whatever this suite
# was itself handed by whoever ran it.
export -f a_repository_of_its_own
a_suite_the_hook_handed() {
    local variable="$1" value="$2" repo="$3"
    env -u GIT_DIR -u GIT_WORK_TREE -u GIT_INDEX_FILE -u GIT_PREFIX -u GIT_COMMON_DIR \
        "$variable=$value" \
        bash -c '. "$1" && a_repository_of_its_own "$2" && cd "$2" && "$3" gsd/x Cargo.toml' \
        a-suite-the-hook-ran.test.sh "$root/scripts/shell-suite.sh" "$repo" "$subject" 2>/dev/null
}

elsewhere="$scratch/elsewhere"
a_repository_of_its_own "$elsewhere"
# And one more file in its index, so an index of `elsewhere`'s read from inside
# the fixture holds a blob the fixture's repository does not have and the
# subject cannot mistake it for the bump alone.
printf 'what elsewhere holds and the fixture does not\n' > "$elsewhere/elsewhere.txt"
git -C "$elsewhere" add elsewhere.txt
elsewhere_head="$(git -C "$elsewhere" rev-parse HEAD)"
elsewhere_config="$scratch/elsewhere-config-as-handed"
elsewhere_index="$scratch/elsewhere-index-as-handed"
cp "$elsewhere/.git/config" "$elsewhere_config"
cp "$elsewhere/.git/index" "$elsewhere_index"

desc="a repository of its own under an absolute GIT_DIR pointing elsewhere leaves that elsewhere untouched"
answer="$(a_suite_the_hook_handed GIT_DIR "$elsewhere/.git" "$scratch/under-a-git-dir")"
head_now="$(git -C "$elsewhere" rev-parse HEAD)"
if [ "$head_now" != "$elsewhere_head" ] || ! cmp -s "$elsewhere/.git/config" "$elsewhere_config"; then
    suite_case_failed "$desc" "the fixture landed in elsewhere" \
        "HEAD was $elsewhere_head and is $head_now" \
        "core.bare is '$(git -C "$elsewhere" config --get core.bare || true)'" \
        "core.hooksPath is '$(git -C "$elsewhere" config --get core.hooksPath || true)', pinned at '$elsewhere/no-hooks'" \
        "the subject answered '$answer'"
elif [ "$answer" != affected ]; then
    suite_case_failed "$desc" "answered '$answer', wanted 'affected'"
else
    suite_case_passed "$desc"
fi

desc="a repository of its own under an exported GIT_INDEX_FILE reads its own index"
an_index_the_hook_handed="$scratch/an-index-the-hook-handed"
cp "$elsewhere_index" "$an_index_the_hook_handed"
answer="$(a_suite_the_hook_handed GIT_INDEX_FILE "$an_index_the_hook_handed" "$scratch/under-an-index-file")"
if ! cmp -s "$an_index_the_hook_handed" "$elsewhere_index"; then
    suite_case_failed "$desc" "the fixture wrote to the index it was handed" \
        "the subject answered '$answer'"
elif [ "$answer" != affected ]; then
    suite_case_failed "$desc" "answered '$answer', wanted 'affected'"
else
    suite_case_passed "$desc"
fi
expect affected "the hook itself" gsd/x .githooks/pre-commit
expect affected "this decision" gsd/x scripts/which-checks.sh

# `guards/guards.toml` names breaks in source and the runner applies them, so a
# record change is not a document change however much it reads like one.
expect affected "a guard record" gsd/x guards/guards.toml

# ── What changed: the installer script ──────────────────────────────────────
# The manifest rule one layer down. Three tests read
# `installer/Wixen-Mail-Setup.iss` as data, and until this rule existed an
# installer change ran none of them: the scoped run maps a changed `src/*.rs` to
# a module and a changed `tests/*.rs` to a target, and an `.iss` path matches
# neither, so no scoped target was chosen and all three sit in `src/`.
expect all "an installer script on a branch" gsd/x installer/Wixen-Mail-Setup.iss
expect all "an installer script beside a source file" \
    gsd/x src/lib.rs installer/Wixen-Mail-Setup.iss

# The combination this project makes most often, and the one that answers wrongly
# if the rule is written in the wrong place. The version-bump exception sits
# between the manifest collection and everything after it, so a rule written
# inside the manifest branch, or anywhere that exception can skip past, lets an
# installer change that also bumps the version answer `affected`. The convention
# here puts the bump in the same commit as the change it describes, so that is
# nearly every installer commit there will ever be.
expect all "an installer script beside a version bump" \
    --manifest-diff-file="$version_bump" gsd/x installer/Wixen-Mail-Setup.iss Cargo.toml Cargo.lock

# Already the answer, because on main the branch decides first. Here so that a
# later reader can see the new rule is not what makes it right.
expect all "an installer script on main" main installer/Wixen-Mail-Setup.iss

# The rule keys on the extension and not on the folder, and this is the case
# that says so. `installer/*` would be the obvious way to write "the installer
# earns the gate" and would take a document in that folder with it, which no
# other case in this file would notice.
expect docs_only "a document inside the installer folder" gsd/x installer/README.md

# ── What changed: a workflow ────────────────────────────────────────────────
# The installer rule one layer along. Two tests in
# `src/presentation/scan_target.rs` read `.github/workflows/accessibility.yml`
# as data, one for the flag it passes and one for the windows it asks for, and
# until this rule existed a workflow change ran neither: a `.yml` is not
# `src/*.rs` and not `tests/*.rs`, so the scoped run chose no target for it,
# and both tests sit in `src/`. Measured 2026-09-14 on a branch: a workflow
# with one window taken out of its list answered `affected`, the gate passed in
# 64 seconds, and the test that names the missing window was red when run by
# hand on the same tree.
expect all "a workflow file on a branch" gsd/x .github/workflows/accessibility.yml
expect all "a workflow file beside a document" \
    gsd/x .github/workflows/accessibility.yml docs/changelog.md

# Below the version-bump exception, for the reason the installer case above
# gives: a rule written where that exception can skip past it lets a workflow
# change that also bumps the version answer `affected`.
expect all "a workflow file beside a version bump" \
    --manifest-diff-file="$version_bump" gsd/x .github/workflows/accessibility.yml Cargo.toml Cargo.lock

# The rule keys on the folder and not on the extension, which is the opposite
# of the installer rule, and these two cases are what hold it to that. An
# `.iss` anywhere is a setup script; a `.yml` anywhere is not a workflow, and a
# document beside the workflows is still a document.
expect docs_only "a document inside the .github folder" gsd/x .github/PULL_REQUEST_TEMPLATE.md
expect affected "a yml file outside the workflows folder" gsd/x .github/dependabot.yml

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
