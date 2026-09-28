# Solana Attestation Service - build automation
# https://github.com/casey/just

# Use bash for all recipes
set shell := ["bash", "-uc"]

# Variables
program_dir := "program"
ts_client_dir := "clients/typescript"
idl_file := "idl/solana_attestation_service.json"
sbf_out_dir := justfile_directory() / "target/sbpf-solana-solana/release"
generated_paths := "idl clients/typescript/src/generated clients/rust/src/generated"
fmt_packages := "-p solana-attestation-service -p tests-solana-attestation-service"

# List available recipes
default:
    @just --list

# ============================================
# Setup and initialization
# ============================================

# Install dependencies and configure git hooks
setup: setup-hooks
    #!/usr/bin/env bash
    set -euo pipefail

    commands=(pnpm cargo cargo-build-sbf)
    for cmd in "${commands[@]}"; do
        if ! command -v "$cmd" &>/dev/null; then
            echo "Error: $cmd is required but not installed"
            exit 1
        fi
    done

    pnpm install
    echo "✓ Setup complete"

# Configure git hooks path
setup-hooks:
    git config core.hooksPath .githooks
    @echo "✓ Git hooks configured"

# Print program ID from declare_id! in program source
program-id:
    @sed -n 's/.*declare_id!("\([^"]*\)").*/\1/p' "{{program_dir}}/src/lib.rs"

# ============================================
# Build recipes
# ============================================

# Build everything (program + clients)
build: build-program build-client

# Compile Solana program to .so
build-program:
    cd {{program_dir}} && cargo-build-sbf
    @echo "✓ Program built"

# Generate IDL from Rust source (requires the shank CLI)
generate-idl: check-shank
    pnpm run generate-idl
    @echo "✓ IDL generated"

# Generate TypeScript and Rust clients from IDL
generate-clients: generate-idl
    pnpm run generate-clients
    @echo "✓ Clients generated"

# Check that committed IDL and generated clients are current
check-generated: generate-clients
    #!/usr/bin/env bash
    set -euo pipefail

    if ! git diff --quiet -- {{generated_paths}} || [[ -n "$(git ls-files --others --exclude-standard -- {{generated_paths}})" ]]; then
        echo "Error: IDL or generated clients are out of date"
        echo "Run: just generate-clients"
        git status --short -- {{generated_paths}}
        git diff -- {{generated_paths}}
        exit 1
    fi

    echo "✓ IDL and generated clients are up-to-date"

# Build TypeScript client
build-client: generate-clients
    cd {{ts_client_dir}} && pnpm run build
    @echo "✓ TypeScript client built"

[private]
check-shank:
    @command -v shank >/dev/null 2>&1 || { echo "Error: shank not installed. Run: cargo install shank-cli"; exit 1; }

# ============================================
# Test recipes
# ============================================

# Run all tests
test *args: unit-test (integration-test args) test-client

# Run Rust unit tests
unit-test:
    cargo test -p solana-attestation-service --lib

# Run Rust integration tests against the built program
integration-test *args: build-program generate-clients
    #!/usr/bin/env bash
    set -euo pipefail
    SBF_OUT_DIR={{sbf_out_dir}} cargo test -p tests-solana-attestation-service "$@"

# Run TypeScript client tests
test-client: generate-clients
    cd {{ts_client_dir}} && pnpm run test

# ============================================
# Clean recipes
# ============================================

# Clean build artifacts and dependencies
clean:
    #!/usr/bin/env bash
    set -euo pipefail

    echo "Cleaning Rust build artifacts..."
    cargo clean

    echo "Cleaning TypeScript build artifacts..."
    rm -rf {{ts_client_dir}}/dist node_modules {{ts_client_dir}}/node_modules

    echo "✓ Clean complete"

# ============================================
# Format and lint recipes
# ============================================

# Check formatting without fixing
fmt-check:
    @echo "Checking Rust formatting..."
    @cargo fmt {{fmt_packages}} --check
    @echo "Checking TypeScript formatting..."
    @pnpm run format:check
    @echo "✓ Format check passed"

# Auto-format all code
fmt:
    @echo "Formatting Rust..."
    @cargo fmt {{fmt_packages}}
    @echo "Formatting TypeScript..."
    @pnpm run format
    @echo "✓ Code formatted"

# Lint with auto-fix
lint: generate-clients
    @echo "Linting Rust..."
    @cargo clippy --workspace --exclude solana-attestation-service-client --all-targets --no-deps --fix -- -D warnings
    @echo "Linting TypeScript..."
    @pnpm run lint:fix
    @echo "✓ Code linted"

# Check linting without fixing
lint-check: generate-clients
    @echo "Checking Rust lint..."
    @cargo clippy --workspace --exclude solana-attestation-service-client --all-targets --no-deps -- -D warnings
    @echo "Checking TypeScript lint..."
    @pnpm run lint
    @echo "✓ Lint check passed"

# Run all code quality checks
check: fmt-check lint-check
