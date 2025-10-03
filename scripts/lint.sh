#!/usr/bin/env bash

set -e

# Run Rust linter
cargo clippy \
    --workspace \
    --all-features \
    --all-targets \
    -- -D warnings
