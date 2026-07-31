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

touch src/lib.rs

echo "== rustfmt =="
cargo fmt --all -- --check

echo "== clippy =="
cargo clippy --all-targets --all-features -- -D warnings

echo "== tests =="
cargo test --all-targets

echo "== release build =="
cargo build --release

echo "All four checks passed."
