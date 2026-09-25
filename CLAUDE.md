# CLAUDE.md

Solana program (Pinocchio) for on-chain attestations: credentials, schemas,
attestations, and optional Token-2022 soulbound tokenization. Shank generates
the IDL, Codama generates the Rust and TypeScript clients.
Build/test recipes: `just --list`. Program overview and instruction list:
[`README.md`](./README.md).

## Gotchas

**Wire format is not Anchor's.** Instruction discriminators are one byte,
routed in `program/src/entrypoint.rs`. Discriminator 8 is unassigned; leave
the gap rather than filling it. Events are the exception: they are
emitted via self-CPI under discriminator 228, the first little-endian byte of
`EVENT_IX_TAG` (Anchor's `Sha256("anchor:event")[..8]`), so indexers pick them
up. Anything assuming Anchor's 8-byte layout will mis-decode instructions.

**Events go through self-CPI, never `sol_log`.** Log output is truncated at
the runtime's limit, which silently drops attestation lifecycle data. Emit
through the event authority PDA.

**Shank embeds the workspace version in the IDL.** Bumping `version` in the
root `Cargo.toml` changes `idl/solana_attestation_service.json`, so a version
bump is not complete until `just generate-clients` has run and the IDL diff is
committed. `just check-generated` catches it, and CI fails on it.

**Generated client sources are gitignored.** `clients/*/src/generated/` is
produced by `just generate-clients`. Never hand-edit it, and never commit it.
Because `cargo package` honors `.gitignore`, `clients/rust/Cargo.toml` carries
an explicit `include` list and the publish workflow passes `--allow-dirty`;
breaking either one publishes a crate with no source.

**Integration tests need the built `.so` on disk.** `cargo test` alone does
not build it. Run `just integration-test`, which runs `cargo-build-sbf` first
and sets `SBF_OUT_DIR` to `target/sbpf-solana-solana/release`.

**Pinocchio is a git fork, not a release.** All six `pinocchio-*` crates point
at a branch of `Nagaprasadvr/pinocchio` with no rev pinned, so the dependency
can move under a `cargo update` and verified builds are impossible until it is
pinned or upstreamed.

**`cargo audit` suppressions are test-tree only.** The ignore list in
`.cargo/audit.toml` exists because `solana-program-test` drags in the Agave
validator stack. Re-audit the whole list when that dependency is upgraded, and
never add an advisory that reaches the program or the published client.

**The release profile changed the binary.** `overflow-checks` and fat LTO are
on, so arithmetic that used to wrap now aborts the instruction, and the next
deployment will not reproduce any prior deployment's build hash.

**`declare_id!` in `program/src/lib.rs` is parsed by `sed`** in the `program-id`
recipe. Keep it a single literal line.

**`cereal_macro` only handles scalar fields.** The derive matches on the last
path segment of each field type, which is `Vec` for every vector, so all
thirteen `Vec<...>` arms are unreachable and such a field fails to compile
with "Unsupported type in struct". Only the integration tests and one `core`
unit test use the macro; nothing shipped depends on it.

**ESLint does not cover everything.** `examples/`, `scripts/`, and the
TypeScript tests are in the ignore list; changes there are unlinted.

## Conventions

- Pinocchio, never `anchor-lang`.
- Every processor is `#[inline(always)]`, destructures accounts by array
  pattern, validates through `processor/shared/` helpers, then runs its logic.
- Ownership, signer, writability, discriminator, and PDA derivation are all
  checked explicitly; there is no framework doing it.
- Crate versions are inherited from the workspace, so one edit in the root
  `Cargo.toml` moves the program and both clients together.
- Schema layouts are numeric type identifiers; the mapping is in `README.md`.
