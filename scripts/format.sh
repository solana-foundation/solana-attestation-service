#!/usr/bin/env bash

set -e


# Run Rust formatter
cargo +nightly fmt --all

# Run Rust linter with fixes
cargo clippy \
    --workspace \
    --no-deps \
    --all-features \
    --fix \
    --allow-dirty \
    --allow-staged \
    -- -D warnings
