# Changelog — `@solana/attestation`

TypeScript SDK for the Solana Attestation Service program. Published to npm; tagged `ts-client-vX.Y.Z`.

Versioning follows [Semantic Versioning](https://semver.org/).

Releases before 2.0.0 were published as `sas-lib` and predate this changelog; see the git history and [npm](https://www.npmjs.com/package/sas-lib) for that period.

## [Unreleased]

### Changed

- The package is renamed from `sas-lib` to `@solana/attestation`. `sas-lib` stays at 1.0.10 and receives no further releases.

## [2.1.0] — 2026-09-29

### Changed

- The package now ships a dual ESM/CJS build behind an `exports` map and sets `sideEffects: false`, so bundlers can tree-shake it. `dist/test` is no longer published.

### Added

- `@solana/attestation/accounts` and `@solana/attestation/instructions` subpath entries for consumers that only need account or instruction codecs.

## [2.0.0] — 2026-09-25

_Promotes the `2.0.0-beta.1` prerelease to a stable release._

### Changed

- **Breaking** — built on `@solana/kit` v8 and Codama `renderers-js` 2.x. ([#104], [#129])

### Fixed

- Schema `String` fields holding non-UTF-8 bytes decode to hex instead of Unicode replacement characters, so the original bytes are recoverable. ([#109])

### Removed

- Generated sources under `src/generated/` are no longer committed. They are produced by `just generate-clients` and bundled into the published `dist/`.

[#104]: https://github.com/solana-foundation/solana-attestation-service/pull/104
[#129]: https://github.com/solana-foundation/solana-attestation-service/pull/129
[#109]: https://github.com/solana-foundation/solana-attestation-service/pull/109
