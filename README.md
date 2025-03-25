# Solana Attestation Service

Built by [Exo Technologies](https://exotechnologies.xyz) with support from Solana Foundation

## Running Tests

Run integration tests with the following script

```
cargo-build-sbf && SBF_OUT_DIR=$(pwd)/target/sbf-solana-solana/release cargo test
```

## Generating IDL

This repository uses Shank for IDL generation.

Install the Shank CLI

```
cargo install shank-cli
```

Generate IDL

```
shank idl -r program -o idl
// OR
yarn generate-idl
```

## Generating Clients

_Ensure the IDL has been created or updated using the above IDL generation steps._

Install dependencies

```
pnpm install
```

Run client generation script

```
pnpm run generate-clients
```
