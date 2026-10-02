set shell := ["bash", "-uc"]

program_dir := "program"
ts_client_dir := "clients/typescript"
idl_file := "idl/solana_attestation_service.json"
sbf_out_dir := justfile_directory() / "target/sbpf-solana-solana/release"
fmt_packages := "-p solana-attestation-service -p tests-solana-attestation-service"

# List available recipes
default:
    @just --list

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

# Build everything (program + clients)
build: build-program build-client

# Compile Solana program to .so
build-program:
    cd {{program_dir}} && cargo-build-sbf
    @echo "✓ Program built"

# Generate IDL from Rust source via the Codama build script
generate-idl:
    pnpm run generate-idl
    @echo "✓ IDL generated"

# Generate TypeScript and Rust clients from IDL
generate-clients: generate-idl
    pnpm run generate-clients
    @echo "✓ Clients generated"

# Regenerate the IDL and clients, then fail if the committed IDL changed
check-generated: generate-clients
    #!/usr/bin/env bash
    set -euo pipefail

    if ! git diff --quiet -- idl || [[ -n "$(git ls-files --others --exclude-standard -- idl)" ]]; then
        echo "Error: IDL is out of date"
        echo "Run: just generate-clients"
        git status --short -- idl
        git diff -- idl
        exit 1
    fi

    echo "✓ IDL is up-to-date"

# Build TypeScript client
build-client: generate-clients
    cd {{ts_client_dir}} && pnpm run build
    @echo "✓ TypeScript client built"

# Run all tests
test *args: unit-test (integration-test args) test-client test-surfpool

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

# Run TypeScript tests against the built program on Surfpool
test-surfpool: build-program generate-clients
    cd {{ts_client_dir}} && pnpm run test:surfpool

# Clean build artifacts and dependencies
clean:
    #!/usr/bin/env bash
    set -euo pipefail

    echo "Cleaning Rust build artifacts..."
    cargo clean

    echo "Cleaning TypeScript build artifacts..."
    rm -rf {{ts_client_dir}}/dist node_modules {{ts_client_dir}}/node_modules

    echo "✓ Clean complete"

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
    @cargo clippy --workspace --exclude solana-attestation --all-targets --no-deps --fix -- -D warnings
    @echo "Linting TypeScript..."
    @pnpm run lint:fix
    @echo "✓ Code linted"

# Check linting without fixing
lint-check: generate-clients
    @echo "Checking Rust lint..."
    @cargo clippy --workspace --exclude solana-attestation --all-targets --no-deps -- -D warnings
    @cargo check -p solana-attestation --all-features
    @echo "Checking TypeScript lint..."
    @pnpm run lint
    @echo "✓ Lint check passed"

# Run all code quality checks
check: fmt-check lint-check
