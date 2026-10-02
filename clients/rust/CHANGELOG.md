# Changelog — `solana-attestation` (Rust client)

Rust SDK for the Solana Attestation Service program. Published to crates.io; tagged `rust-client-vX.Y.Z`.

Versioning follows [Semantic Versioning](https://semver.org/).

Releases before 2.0.0 were published as `solana-attestation-service-client` and predate this changelog; see the git history and [crates.io](https://crates.io/crates/solana-attestation-service-client) for that period.

## [Unreleased]

### Changed

- The crate is renamed from `solana-attestation-service-client` to `solana-attestation`; import it as `solana_attestation`. `solana-attestation-service-client` stays at 1.0.9 and receives no further releases.

## [2.0.0] — 2026-09-25

_Supersedes 1.0.9. The crate is renumbered so the program, the Rust client and the TypeScript client share one version number._

### Changed

- **Breaking:** generated against the Solana 3.x component crates instead of `solana-program`. Public keys are `solana_address::Address` rather than `Pubkey`.
- The crate version is inherited from the workspace, so a single edit in the root `Cargo.toml` bumps the program and the client together.
- Generated sources under `src/generated/` are no longer committed. They are produced by `just generate-clients` and shipped to crates.io through an explicit `include` in `Cargo.toml`.

### Added

- `serde` feature: derives `Serialize` and `Deserialize` on the generated accounts, instruction arguments and types.
