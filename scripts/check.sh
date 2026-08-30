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
if [ -z "$mode" ]; then
    mode="$("$(dirname "$0")/which-checks.sh" "$(git rev-parse --abbrev-ref HEAD 2>/dev/null || true)")"
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
