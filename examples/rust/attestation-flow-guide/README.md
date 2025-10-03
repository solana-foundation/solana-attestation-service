# Solana Attestation Service Rust Examples

This repository contains Rust implementation examples for the Solana Attestation Service (SAS) to create, manage, verify, and close digital credentials on Solana. These examples demonstrate the complete attestation lifecycle using the `solana-attestation-service-client` crate. For more detailed explanations and step-by-step walkthroughs, see the comprehensive guides:

| Title | Directory | Guide | Description |
|-------|-----------|-------------|-------------|
| Standard Attestation Demo | `standard-demo/` | [How to Build Digital Credentials using Solana Attestation Service](https://attest.solana.com/docs/guides/rust/how-to-create-digital-credentials) | Basic credential and attestation workflow using Rust |
| Tokenized Attestation Demo | `tokenization-demo/` | [How to Create Tokenized Credentials using Solana Attestation Service](https://attest.solana.com/docs/guides/rust/tokenized-attestations) | Create credentials as SPL tokens using Token-2022 and Rust |
| Compression Attestation Demo | `compression-demo/` | tdb |  Basic compressed credential and attestation workflow using Rust |


## Requirements

- [Rust](https://rustup.rs/) (latest stable version)
- [Solana CLI](https://solana.com/docs/intro/installation) (v2.2.x or greater)

## Installation

Clone the repository and navigate to the Rust examples:

```bash
git clone https://github.com/solana-foundation/solana-attestation-service
cd examples/rust/attestation-flow-guide
```

## Usage

All demos will automatically:
- Create test wallets
- Request airdrop for payer wallet
- Execute the full attestation workflow

### Local Development

For local development and testing:

1. **Download the SAS program:**
   ```bash
   mkdir -p programs
   solana program dump -um 22zoJMtdu4tQc2PzL74ZUT7FrwgB1Udec8DdW4yw4BdG programs/sas.so
   ```

2. **Start local validator** (in a separate terminal):
   ```bash
   solana-test-validator -r --bpf-program 22zoJMtdu4tQc2PzL74ZUT7FrwgB1Udec8DdW4yw4BdG programs/sas.so
   ```

3. **Run the demos:**

   Standard Attestation Workflow:
   ```bash
   # Standard demo
   cd standard-demo
   cargo run
   ```

   Tokenized Attestation Workflow:

   ```bash
   # Tokenized demo
   cd tokenization-demo
   cargo run tokenized
   ```

   Compression Attestation Workflow (automatically spawns Light validator):

   ```bash
   # Compression demo
   cd compression-demo
   cargo run
   ```

   **Note:** The compression demo automatically spawns a local Light validator with indexer and prover server.

## Project Structure

Each demo is a complete Rust project with:

```
standard-demo/
├── src/
│   └── main.rs          # Complete demo implementation
└── Cargo.toml           # Dependencies and configuration

tokenization-demo/
├── src/
│   └── main.rs          # Complete tokenized demo implementation
└── Cargo.toml           # Dependencies and configuration

compression-demo/
├── src/
│   └── main.rs          # Complete compression demo implementation
└── Cargo.toml           # Dependencies and Light Protocol configuration
```
