#!/usr/bin/env bash
# Run the four checks CI runs, in the same order, and fail on the first one.
#
# Touching lib.rs first is not optional. Cargo shares fingerprints between
# `check`, `build`, `test`, and `clippy`, so a clippy run after a build can be
# considered fresh and report success without linting anything. That has
# already put a clippy failure on main once.
set -euo pipefail

# Offer to run these on every commit, so the answer cannot be lost between
# getting it and committing. It has been twice: a stale fingerprint reporting
# clean, and this script's output piped somewhere so the pipeline's exit status
# was the pipe's rather than this script's.
if [ "$(git config core.hooksPath || true)" != ".githooks" ]; then
  echo "Not running on commit. To turn that on:"
  echo "    git config core.hooksPath .githooks"
  echo
fi

# Which checks this run does. Given as an argument, or worked out from the
# branch by `which-checks.sh`, which is where that decision lives and where it
# is tested. Pass `all` to force the whole gate wherever you are, which is what
# merging a branch into main does before the merge.
#
# The slow two are 295 of the 311 seconds this takes warm, measured 2026-08-30.
# On a branch nobody builds they wait for the merge rather than running once per
# commit. On main they always run.
mode="${1:-}"
# What is about to be committed, which is what the tests should be scoped to.
# Staged rather than working-tree, because that is what the hook is deciding
# about. Empty when run by hand outside a commit, and `which-checks.sh` answers
# `all_but_slow` for that rather than guessing at a narrower set.
changed=()
if [ -z "$mode" ]; then
    while IFS= read -r line; do
        [ -n "$line" ] && changed+=("$line")
    done < <(git diff --cached --name-only 2>/dev/null || true)
    mode="$("$(dirname "$0")/which-checks.sh" \
        "$(git rev-parse --abbrev-ref HEAD 2>/dev/null || true)" \
        "${changed[@]+"${changed[@]}"}")"
fi

touch src/lib.rs

echo "== rustfmt =="
cargo fmt --all -- --check

echo "== clippy =="
cargo clippy --all-targets --all-features -- -D warnings

if [ "$mode" = "all_but_slow" ]; then
    echo
    echo "Formatting and clippy passed. The test suite and the release build did"
    echo "not run: this is not main. Run 'scripts/check.sh all' before merging."
    exit 0
fi

# Documents can only break the tests that read documents, and they genuinely
# can: house_style's em-dash guard has caught two real breaks in markdown, so
# these run rather than being skipped as "not code".
if [ "$mode" = "docs_only" ]; then
    echo "== the targets that read documents =="
    cargo test --test house_style --test docs_links --test wired
    echo
    echo "Formatting, clippy and the document-reading tests passed. The rest of"
    echo "the suite and the release build did not run: nothing outside a document"
    echo "changed, so they had nothing to say. Run 'scripts/check.sh all' before"
    echo "merging."
    exit 0
fi

# Scope the suite to the modules the change reaches. A unit test lives beside
# the code it covers, so a changed `src/a/b.rs` is covered by `--lib a::b::`.
# The source-reading guards run whatever changed, because they read across the
# whole tree and a change anywhere can redden one: that is how a guard record
# was found stale four times in one phase.
if [ "$mode" = "affected" ]; then
    echo "== the tests that reach what changed =="
    for path in "${changed[@]+"${changed[@]}"}"; do
        case "$path" in
            src/*.rs)
                module="${path#src/}"
                module="${module%.rs}"
                module="${module%/mod}"
                module="${module//\//::}"
                [ "$module" = "lib" ] && continue
                echo "-- $module"
                # A filter matching nothing exits zero, so a module with no
                # tests of its own is not a pass, it is a run that said nothing.
                cargo test --lib "${module}::" 2>&1 | tail -3
                ;;
            tests/*.rs)
                target="$(basename "$path" .rs)"
                echo "-- $target"
                cargo test --test "$target" 2>&1 | tail -3
                ;;
        esac
    done
    echo "== the guards that read the whole tree =="
    cargo test --test house_style --test wired
    echo
    echo "Formatting, clippy, the tests reaching what changed, and the"
    echo "tree-reading guards passed. The rest of the suite and the release"
    echo "build did not run. Run 'scripts/check.sh all' before merging."
    exit 0
fi

echo "== tests =="
# --no-fail-fast because without it cargo stops at the first target that fails,
# and the library is the first target. One failing test there means none of the
# fourteen files under tests/ run at all: not reported as skipped, never
# started. That is how a broken guard record once reached main while this gate
# looked like it had checked it. The run still fails; it just says everything
# that is wrong rather than the first thing.
cargo test --all-targets --no-fail-fast

echo "== release build =="
cargo build --release

echo "All four checks passed."
