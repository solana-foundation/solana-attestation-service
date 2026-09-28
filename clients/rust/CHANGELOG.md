# Changelog — `solana-attestation-service-client` (Rust client)

Rust SDK for the Solana Attestation Service program. Published to crates.io; tagged `rust-client-vX.Y.Z`.

Versioning follows [Semantic Versioning](https://semver.org/).

Releases before 2.0.0 predate this changelog; see the git history and [crates.io](https://crates.io/crates/solana-attestation-service-client) for that period.

## [Unreleased]

## [2.0.0] — 2026-09-25

_Version bump only. Supersedes 1.0.9 with no API change; the crate is renumbered so the program, the Rust client and the TypeScript client share one version number._

### Changed

- The crate version is inherited from the workspace, so a single edit in the root `Cargo.toml` bumps the program and the client together.
- Generated sources under `src/generated/` are no longer committed. They are produced by `just generate-clients` and shipped to crates.io through an explicit `include` in `Cargo.toml`.
