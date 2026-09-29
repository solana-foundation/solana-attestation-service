# CLAUDE.md

Solana program (Pinocchio) for on-chain attestations: credentials, schemas,
attestations, and optional Token-2022 soulbound tokenization. Codama derive
macros describe the program, `program/build.rs` writes the IDL, and Codama
renderers generate the Rust and TypeScript clients.
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

**The IDL is written by a build script, not a CLI.** `program/build.rs` emits
`idl/solana_attestation_service.json` only when `GENERATE_IDL` is set, which
`pnpm run generate-idl` does; an ordinary `cargo build` leaves the file alone.
The IDL embeds the workspace version, so bumping `version` in the root
`Cargo.toml` is not complete until `just generate-clients` has run and the IDL
diff is committed. `just check-generated` catches it, and CI fails on it.

**Codama reads the source, so annotations are the contract.** Account lists,
argument types, PDA seeds, account defaults, events and error messages all come
from `#[codama(...)]` attributes on `program/src/instructions.rs`, `state/`,
`constants.rs`, `events.rs` and `error.rs`. `Vec<u8>` fields carry
`type = bytes` plus `size_prefix` because the bare mapping renders an array of
numbers rather than a byte string, and `payer` / `system_program` accounts
carry explicit `default_value`s because nothing infers them. Two directives
cannot share one attribute; write them as separate lines, and put them after
the `derive`, since they are derive helper attributes. Codama only maps a
field to a public key when its type is written as a bare `Address` (imported
from `pinocchio`) or `solana_address::Address`; a `pinocchio::Address` path
renders as an unknown defined type.

**Four structs in `constants.rs` exist only to declare PDA seeds.** Codama
attaches seeds to accounts, so `SchemaMint`, `AttestationMint`,
`EventAuthority` and `SasAuthority` are empty structs whose only job is
`#[codama(seed(...))]`. `scripts/generate-clients.ts` drops any account with no
fields before rendering, which keeps the PDA helpers and skips decoders for an
account that holds no data.

**The TypeScript renderer is pinned to the kit major.**
`@codama/renderers-js` 2.5 generates against `@solana/kit` 8, so it is pinned
with `~2.5`; a renderer minor that targets the next kit major breaks the
TypeScript client until kit is upgraded with it.

**Only the TypeScript renderer understands events.** `scripts/generate-clients.ts`
mirrors every event into a defined type for the Rust render, which is what keeps
`types::CloseAttestationEvent` available to Rust callers; the TypeScript client
gets real event codecs under `events/`. The event's one-byte type discriminator
is a struct field rather than a second Codama discriminator, so the mirrored
type matches the wire format on its own.

**The Rust render drops the account-to-PDA links.** A generated `find_pda` for
a PDA with an unprefixed string seed, which `credential` and `schema` both
have, is typed `TrailingStr` and drags in the `spl-collections` crate. The
links are stripped in `scripts/generate-clients.ts` before the Rust render to
keep that dependency out of the published client; the PDA nodes stay, so the
TypeScript client keeps its `find*Pda` and `fetch*FromSeeds` helpers.

**Generated client sources are gitignored.** `clients/*/src/generated/` is
produced by `just generate-clients`. Never hand-edit it, and never commit it.
Because `cargo package` honors `.gitignore`, `clients/rust/Cargo.toml` carries
an explicit `include` list and the publish workflow passes `--allow-dirty`;
breaking either one publishes a crate with no source.

**Integration tests need the built `.so` on disk.** `cargo test` alone does
not build it. Run `just integration-test`, which runs `cargo-build-sbf` first
and sets `SBF_OUT_DIR` to `target/sbpf-solana-solana/release`.

**Token-2022 metadata and group CPIs are hand-written.** `pinocchio-token-2022`
has no builders for `InitializeTokenMetadata`, `UpdateField`, `InitializeGroup`
or `InitializeMember`, so they live in `processor/shared/token_ext.rs`. Delete
each one once upstream ships it; `test_tokenization.rs` re-parses the minted
state with the SPL interface crates, so an encoding change fails there.

**litesvm is held at 0.12 by the toolchain pin.** litesvm 0.13 and later pull
Agave 4.x, which uses standard library APIs newer than Rust 1.92. Bumping it
means bumping `rust-toolchain.toml` and the CI setup action together.

**The release profile changed the binary.** `overflow-checks` and fat LTO are
on, so arithmetic that used to wrap now aborts the instruction, and the next
deployment will not reproduce any prior deployment's build hash.

**`declare_id!` in `program/src/lib.rs` is parsed twice by text.** The
`program-id` recipe seds it, and Codama only recognizes the unqualified macro
call, which is why it is imported rather than called through
`pinocchio::address::`. Keep it a single literal line.

## Conventions

- Pinocchio, never `anchor-lang`.
- Every processor is `#[inline(always)]`, destructures accounts by array
  pattern, validates through `processor/shared/` helpers, then runs its logic.
- Ownership, signer, writability, discriminator, and PDA derivation are all
  checked explicitly; there is no framework doing it.
- Crate versions are inherited from the workspace, so one edit in the root
  `Cargo.toml` moves the program and the Rust client together. The TypeScript
  client's version lives in `clients/typescript/package.json` and must be
  bumped separately.
- Schema layouts are numeric type identifiers; the mapping is in `README.md`.
