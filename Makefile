.PHONY: help format lint build build-sbf test test-sbf test-unit generate-idl generate-clients install

# Default target
help:
	@echo "Solana Attestation Service - Available Commands:"
	@echo ""
	@echo "Formatting & Linting:"
	@echo "  make format          - Format Rust code and auto-fix clippy issues"
	@echo "  make lint            - Run clippy linter without fixes"
	@echo ""
	@echo "Building:"
	@echo "  make build           - Build Rust workspace and TypeScript clients"
	@echo "  make build-sbf       - Build Solana program (BPF)"
	@echo ""
	@echo "Testing:"
	@echo "  make test            - Run all tests (unit + integration)"
	@echo "  make test-sbf        - Run Solana program integration tests only"
	@echo "  make test-unit       - Run unit tests only"
	@echo ""
	@echo "Code Generation:"
	@echo "  make generate-idl    - Generate IDL from program using Shank"
	@echo "  make generate-clients - Generate Rust and TypeScript clients from IDL"
	@echo ""
	@echo "Other:"
	@echo "  make install         - Install dependencies (pnpm)"

# Format Rust code and auto-fix clippy issues
format:
	cargo +nightly fmt --all
	cargo clippy \
		--workspace \
		--no-deps \
		--all-features \
		--fix \
		--allow-dirty \
		--allow-staged \
		-- -D warnings

# Run clippy linter without fixes
lint:
	cargo clippy \
		--workspace \
		--all-features \
		--all-targets \
		-- -D warnings

# Build Solana program (BPF)
build-sbf:
	cargo-build-sbf

# Build Rust workspace and TypeScript clients
build: build-sbf
	pnpm -r build

# Run all tests (unit + integration)
test: build-sbf
	SBF_OUT_DIR=$$(pwd)/target/deploy cargo test

# Run Solana program integration tests only
test-sbf: build-sbf
	cargo test-sbf -p tests-solana-attestation-service

# Run unit tests only
test-unit:
	cargo test --lib

# Generate IDL from program using Shank
generate-idl:
	shank idl -r program -o idl

# Generate Rust and TypeScript clients from IDL
generate-clients:
	node ./scripts/generate-clients.js

# Install dependencies
install:
	pnpm install
