#!/usr/bin/env bash
# Run the four checks CI runs, in the same order, and fail on the first one.
#
# Touching lib.rs first is not optional. Cargo shares fingerprints between
# `check`, `build`, `test`, and `clippy`, so a clippy run after a build can be
# considered fresh and report success without linting anything. That has
# already put a clippy failure on main once.
set -euo pipefail

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
