# Solana Attestation Service

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/solana-foundation/solana-attestation-service)

Solana program and clients for issuing, verifying, and revoking on-chain attestations.

## Overview

An issuer registers a **Credential** holding its name and the set of authorized signers. Under that credential it publishes a **Schema**: a field layout, field names, and a description that together define the shape of the data being attested. Authorized signers then write **Attestation** accounts conforming to that schema, each bound to a nonce and an optional expiry, and can close them to revoke.

A schema can additionally be **tokenized**. Tokenizing mints an SPL Token-2022 group mint for the schema; attestations issued against it mint a soulbound token to the subject, so wallets and existing token tooling can display and verify an attestation without knowing anything about this program. Tokenized attestations use the `NonTransferable`, `MetadataPointer`, `GroupMemberPointer`, `PermanentDelegate`, and `MintCloseAuthority` extensions.

Lifecycle changes are emitted as events through a self-CPI rather than `sol_log`, which keeps them out of the truncated log buffer so indexers can read them reliably.

This repository contains:

- A Rust Solana program built with [Pinocchio](https://github.com/anza-xyz/pinocchio)
- IDL and client generation via [Codama](https://github.com/codama-idl/codama)
- A TypeScript client (`sas-lib`) in `clients/typescript`
- A Rust client (`solana-attestation-service-client`) in `clients/rust`
- Worked examples in `examples/`

## Program ID

```
22zoJMtdu4tQc2PzL74ZUT7FrwgB1Udec8DdW4yw4BdG
```

Print it from source at any time with `just program-id`.

## Instructions

| Instruction                  | Purpose                                                            |
| ---------------------------- | ------------------------------------------------------------------ |
| `CreateCredential`           | Register an issuer and its authorized signers                      |
| `ChangeAuthorizedSigners`    | Replace the credential's authorized signer set                     |
| `CreateSchema`               | Publish a schema under a credential                                |
| `ChangeSchemaStatus`         | Pause or resume issuance against a schema                          |
| `ChangeSchemaDescription`    | Update a schema's description                                      |
| `ChangeSchemaVersion`        | Publish a new version of a schema                                  |
| `CreateAttestation`          | Issue an attestation against a schema                              |
| `CloseAttestation`           | Close an attestation and reclaim its rent                          |
| `TokenizeSchema`             | Create the Token-2022 group mint backing a schema                  |
| `CreateTokenizedAttestation` | Issue an attestation and mint its soulbound token in one operation |
| `CloseTokenizedAttestation`  | Close a tokenized attestation, burning the token and closing mint  |
| `EmitEvent`                  | Self-CPI target used for event emission, not called directly       |

## Schema layout types

A schema's `layout` is an array of numeric type identifiers; `fieldNames` names each one positionally. A layout of `[12, 0, 12]` with field names `["name", "age", "country"]` defines a String, a U8, and a String.

| Value | Type | Value | Type    | Value | Type      |
| ----- | ---- | ----- | ------- | ----- | --------- |
| 0     | U8   | 9     | I128    | 18    | VecI8     |
| 1     | U16  | 10    | Bool    | 19    | VecI16    |
| 2     | U32  | 11    | Char    | 20    | VecI32    |
| 3     | U64  | 12    | String  | 21    | VecI64    |
| 4     | U128 | 13    | VecU8   | 22    | VecI128   |
| 5     | I8   | 14    | VecU16  | 23    | VecBool   |
| 6     | I16  | 15    | VecU32  | 24    | VecChar   |
| 7     | I32  | 16    | VecU64  | 25    | VecString |
| 8     | I64  | 17    | VecU128 |       |           |

## Project structure

```text
solana-attestation-service/
├── program/                 # Rust Solana program (Pinocchio)
│   ├── src/
│   │   ├── processor/       # Instruction handlers
│   │   │   └── shared/      # Account checks, PDA and data utilities
│   │   ├── state/           # Credential, Schema, Attestation accounts
│   │   ├── instructions.rs  # Codama instruction definitions (IDL source)
│   │   ├── entrypoint.rs    # Discriminator routing
│   │   ├── events.rs        # Event definitions
│   │   └── constants.rs     # Seeds and program constants
├── idl/                     # Codama-generated IDL (committed)
├── clients/
│   ├── typescript/          # sas-lib SDK + tests
│   └── rust/                # solana-attestation-service-client
├── integration_tests/       # Rust integration tests
├── examples/                # Rust and TypeScript attestation flow guides
├── scripts/                 # IDL processing and client generation
├── .github/                 # CI workflows and shared setup action
├── .githooks/               # pre-push: format and lint checks
└── justfile                 # Task runner
```

## Quick start

```bash
git clone git@github.com:solana-foundation/solana-attestation-service.git
cd solana-attestation-service
just setup
just build
just test
```

### Prerequisites

`just setup` checks for `pnpm`, `cargo`, and `cargo-build-sbf`.

| Tool       | Install                                                                                                |
| ---------- | ------------------------------------------------------------------------------------------------------ |
| Rust       | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh`                                      |
| Solana CLI | `sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"`                                        |
| pnpm       | `curl -fsSL https://get.pnpm.io/install.sh \| sh -`                                                    |
| Just       | `curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh \| bash -s -- --to ~/.local/bin` |

Rust is pinned in `rust-toolchain.toml`, Node.js in `.nvmrc`, and pnpm in the `packageManager` field of `package.json`.

## Build and test

`just --list` shows every recipe.

### Build

| Recipe                  | Description                                                 |
| ----------------------- | ----------------------------------------------------------- |
| `just build`            | Program `.so` plus the TypeScript client                    |
| `just build-program`    | Compile the SBF program                                     |
| `just generate-idl`     | Regenerate `idl/solana_attestation_service.json` via Codama |
| `just generate-clients` | Regenerate both clients from the IDL via Codama             |
| `just build-client`     | Build `clients/typescript` into `dist/`                     |

Generated client sources under `clients/*/src/generated/` are not committed. They are produced from the IDL by `just generate-clients` and bundled into the published packages. The IDL itself is committed, and `just check-generated` fails if either has drifted from the program source.

### Test

| Recipe                  | Description                                    |
| ----------------------- | ---------------------------------------------- |
| `just test`             | Everything below                               |
| `just unit-test`        | Rust unit tests                                |
| `just integration-test` | Rust integration tests against the built `.so` |
| `just test-client`      | TypeScript client tests                        |

### Code quality

| Recipe                     | Description                                      |
| -------------------------- | ------------------------------------------------ |
| `just check`               | `fmt-check` plus `lint-check`, run by pre-push   |
| `just fmt` / `fmt-check`   | Format Rust and TypeScript, or check only        |
| `just lint` / `lint-check` | Clippy and ESLint, with or without autofix       |
| `just check-generated`     | Verify the IDL and generated clients are current |
| `just clean`               | Remove build artifacts and dependencies          |

## Clients

TypeScript:

```bash
pnpm add sas-lib
```

```typescript
import { deriveCredentialPda, deriveSchemaPda, serializeAttestationData } from 'sas-lib';
```

The package re-exports the Codama-generated instruction builders, account decoders, and PDA finders, plus hand-written helpers: PDA derivation shorthands (`deriveCredentialPda`, `deriveSchemaPda`, `deriveAttestationPda`, `deriveSchemaMintPda`, `deriveAttestationMintPda`, `deriveEventAuthorityAddress`, `deriveSasAuthorityAddress`) and schema-driven codecs (`getAttestationDataCodec`, `serializeAttestationData`, `deserializeAttestationData`). It is built on `@solana/kit` v7, declared as a peer dependency.

Rust:

```bash
cargo add solana-attestation-service-client
```

```rust
use solana_attestation_service_client::instructions::*;
```

End-to-end walkthroughs live in `examples/typescript/attestation-flow-guides` and `examples/rust/attestation-flow-guide`.

## CI

| Workflow       | Description                                                 |
| -------------- | ----------------------------------------------------------- |
| **Build**      | Compile the program and both clients                        |
| **Test**       | Rust unit, Rust integration, and TypeScript client tests    |
| **Format**     | Rust and TypeScript formatting                              |
| **Lint**       | Clippy and ESLint                                           |
| **IDL Check**  | Fail on drift between the program, the IDL, and the clients |
| **Security**   | `cargo audit` and `pnpm audit`                              |
| **PR hygiene** | Commit signatures, AI disclosure, and AI tool attribution   |

Publishing is manual. `Publish Rust Client` and `Publish TypeScript Client` are `workflow_dispatch` only, gated on a green test run and a branch guard, default to a dry run, and authenticate to the registries over OIDC.

## Security

Report vulnerabilities privately through the process in [SECURITY.md](SECURITY.md), not in a public issue.

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request. Changelogs are kept per artifact: the [program](CHANGELOG.md), the [Rust client](clients/rust/CHANGELOG.md), and the [TypeScript client](clients/typescript/CHANGELOG.md).

## License

[MIT](LICENSE).
