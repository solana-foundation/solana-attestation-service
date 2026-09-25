# Changelog — `sas-lib`

TypeScript SDK for the Solana Attestation Service program. Published to npm; tagged `ts-client-vX.Y.Z`.

Versioning follows [Semantic Versioning](https://semver.org/).

Releases before 2.0.0 predate this changelog; see the git history and [npm](https://www.npmjs.com/package/sas-lib) for that period.

## [Unreleased]

## [2.0.0] — 2026-09-25

_Promotes the `2.0.0-beta.1` prerelease to a stable release._

### Changed

- **Breaking** — built on `@solana/kit` v7 and Codama `renderers-js` 2.x. ([#104])

### Fixed

- Schema `String` fields holding non-UTF-8 bytes decode to hex instead of Unicode replacement characters, so the original bytes are recoverable. ([#109])

### Removed

- Generated sources under `src/generated/` are no longer committed. They are produced by `just generate-clients` and bundled into the published `dist/`.

[#104]: https://github.com/solana-foundation/solana-attestation-service/pull/104
[#109]: https://github.com/solana-foundation/solana-attestation-service/pull/109
