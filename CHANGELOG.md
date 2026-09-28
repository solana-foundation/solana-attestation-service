# Changelog — Solana Attestation Service program

On-chain program `22zoJMtdu4tQc2PzL74ZUT7FrwgB1Udec8DdW4yw4BdG`, versioned by `program-vX.Y.Z` git tags.
SDK client changelogs are tracked separately: [`clients/typescript`](clients/typescript/CHANGELOG.md) and [`clients/rust`](clients/rust/CHANGELOG.md).

Versioning follows [Semantic Versioning](https://semver.org/).

Releases before 2.0.0 predate this changelog and were never tagged; see the git history for that period.

## [Unreleased]

## [2.0.0] — 2026-09-25

_Not yet deployed. The currently deployed binary predates this version. The program, the Rust client and the TypeScript client are unified on a single version number as of this release._

### Changed

- The release profile enables `overflow-checks` and fat LTO. Arithmetic that silently wrapped in release builds now aborts the instruction, and the next deployment will not reproduce the build hash of any prior deployment.

### Added

- CI gates on every pull request: build, unit and integration tests, formatting, clippy, IDL and generated-client drift, `cargo audit` and `pnpm audit`.
- `just` is the single task runner for build, test, lint, format and client generation.
- Both client publishes are manual (`workflow_dispatch`) and gated on a green test run, a branch guard and a dry-run default. They publish over OIDC instead of a long-lived registry token, and they push a git tag and create a GitHub Release.
