# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Solana Attestation Service (SAS) is a Solana program that enables creating, managing, and verifying digital attestations on the Solana blockchain. The system provides a framework for issuers to create schemas and issue attestations that can optionally be tokenized as SPL Token-2022 NFTs.

The repository consists of several main components:

- `program/`: Main Solana program written in Rust using Pinocchio framework
- `core/`: Shared types and utilities for schema serialization 
- `clients/`: Auto-generated client libraries for Rust and TypeScript
- `integration_tests/`: Comprehensive test suite for all program functionality
- `cereal_macro/`: Procedural macro for schema serialization

## Common Development Commands

### Building and Testing

```bash
# Build the Solana program
cargo-build-sbf

# Run integration tests
cargo-build-sbf && SBF_OUT_DIR=$(pwd)/target/sbf-solana-solana/release cargo test

# Run TypeScript client tests
cd clients/typescript && npm test
```

### IDL Generation

```bash
# Generate IDL using Shank
shank idl -r program -o idl

# Or use the npm script
pnpm run generate-idl
```

### Client Generation

```bash
# Generate Rust and TypeScript clients from IDL
pnpm run generate-clients
```

### TypeScript Client Development

```bash
# Build TypeScript client
cd clients/typescript && npm run build

# Run tests
cd clients/typescript && npm test
```

## Architecture Overview

### Core Components

The system consists of three main components:

1. **Credential**: Represents an issuer's identity and authority to create schemas and attestations
   - Contains issuer name and list of authorized signers
   - Controls who can create schemas and attestations

2. **Schema**: Defines the structure and metadata for attestations
   - Includes field definitions, layout, and validation rules
   - Can be paused/resumed and versioned
   - Supports tokenization for creating NFT attestations utilizing Token 2022

3. **Attestation**: Actual attestation data conforming to a schema
   - Contains structured data, expiry time, and nonce
   - Can be regular attestations or tokenized (as SPL tokens)
   - Can be closed by authorized signers

### Key Features

- **Schema Versioning**: Schemas can be updated while maintaining backwards compatibility
- **Tokenization**: Schemas can be tokenized to create NFT-like attestations with metadata
- **Access Control**: Multi-signature support for credential management
- **Event Logging**: Comprehensive event system for attestation lifecycle tracking
- **Expiry Management**: Attestations can have expiration dates

### Program Structure

- **program/**: Main Solana program written in Rust using Pinocchio framework
  - **src/processor/**: Business logic for each instruction with shared utilities
    - **shared/**: Common utilities (`account_checks.rs`, `pda_utils.rs`, `data_utils.rs`)
  - **src/state/**: On-chain account structures (`attestation.rs`, `credential.rs`, `schema.rs`)
  - **src/**: Core files with specific purposes:
    - `entrypoint.rs`: Program entry point and instruction discriminator routing
    - `instructions.rs`: Shank instruction definitions for IDL generation
    - `error.rs`: Custom error types and error code definitions
    - `events.rs`: Event definitions for program logging and monitoring
    - `constants.rs`: Program constants, seeds, and configuration values
    - `lib.rs`: Main library entry point and module declarations
    - `macros.rs`: Helper macros for common operations

- **core/**: Shared types and utilities for schema serialization
  - Primitive data types and schema field validation logic

- **cereal_macro/**: Procedural macros for schema serialization
  - `SchemaStructSerialize` derive macro for automatic byte representation

- **clients/**: Auto-generated client libraries for Rust and TypeScript
  - **rust/src/generated/**: Rust client (accounts, instructions, types, errors)
  - **typescript/src/generated/**: TypeScript client (accounts, instructions, programs, types)
  - **typescript/src/**: Client utilities (`pdas.ts`, `utils.ts`) and tests

- **integration_tests/**: Comprehensive test suite for all program functionality
  - **tests/helpers/**: Test utility functions and program context setup
  - **tests/**: Individual test files for each instruction type and tokenization

- **scripts/**: Build and generation scripts
  - Client generation (`generate-clients.js`) and IDL processing (`process-idl.js`)

- **examples/**: Usage examples and practical demonstrations
  - **typescript/**: TypeScript examples for credential setup, schemas, and attestations

- **idl/**: Interface Definition Language files
  - **solana_attestation_service.json**: Program IDL defining accounts, instructions, and types

### Important Constants

- Program ID: `22zoJMtdu4tQc2PzL74ZUT7FrwgB1Udec8DdW4yw4BdG`
- Token Program: `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb` (Token-2022)
- Event Authority PDA: `DzSpKpST2TSyrxokMXchFz3G2yn5WEGoxzpGEUDjCX4g`

## Technology Stack

### Program Framework

The program uses [**Pinocchio**](https://github.com/anza-xyz/pinocchio) instead of Anchor, chosen for:
- Smaller program size and reduced compute costs
- More control over instruction parsing and execution
- Direct account validation and manipulation

### IDL and Client Generation

- [**Shank**](https://github.com/metaplex-foundation/shank): Used for IDL generation with `#[derive(ShankInstruction)]` and `#[account()]` attributes
- [**Codama**](https://github.com/codama-idl/codama): Transforms Shank-generated IDL into TypeScript and Rust clients
- Automated generation pipeline ensures clients stay in sync with program changes

### Token Extensions

Uses SPL Token-2022 with extensions for tokenized attestations:
- **NonTransferable**: Creates soulbound tokens that cannot be transferred
- **MetadataPointer**: Enables on-chain metadata storage
- **GroupMemberPointer**: Links attestations to schema token groups
- **PermanentDelegate**: Gives program permanent control over tokens
- **MintCloseAuthority**: Allows program to close mints

### Event System

- Uses instruction discriminator 228 for event emission via CPI
- Prevents log truncation issues common with standard Solana logging
- Emits structured events for attestation lifecycle tracking

## Code Style & Best Practices

### Processor Code Structure

All instruction processors must follow this consistent structure:

1. **Function Signature**: Use `#[inline(always)]` for all processor functions
2. **Account Destructuring**: Use array pattern matching to destructure accounts
3. **Verification & Security Checks**: Use shared verification functions from `processor/shared/`
4. **Business Logic**: Implement the core instruction logic
5. **Event Emission**: Emit events if required for the instruction

**Example processor structure:**
```rust
#[inline(always)]
pub fn process_my_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // 1. Account destructuring
    let [account1, account2, account3] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // 2. Verification and security checks
    verify_account_owner(account1)?;
    verify_pda_derivation(account2, seeds)?;
    
    // 3. Business logic
    // ... implementation ...

    // 4. Event emission (if required)
    emit_event(...)?;
    
    Ok(())
}
```

### Error Handling

- Custom error types defined in `program/src/error.rs`
- Use `AttestationServiceError` enum for program-specific errors
- Convert errors at module boundaries for consistent error propagation

### Schema Serialization

- Custom procedural macro (`cereal_macro`) generates schema serialization
- Maps Rust types to numeric identifiers for compact representation
- Supports primitive types, vectors, and strings with length prefixes

### Performance & Memory Management

- Use `#[inline(always)]` for ALL instruction processor functions
- Minimize heap allocations in program execution
- Keep account sizes as small as possible to reduce storage costs and improve performance
- When account resizing is required, implement it performantly with proper size calculations
- Use stack allocation where possible
- Efficient string and vector handling with length prefixes
- Careful buffer management for variable-length data

### Shared Utilities

- Use PDA utilities from `program/src/processor/shared/pda_utils.rs` for account derivation
- Reuse verification functions from `program/src/processor/shared/` for consistency
- Comprehensive PDA validation for all accounts
- Verify program ownership and account derivation
- Check account mutability and signer requirements

## Development Guidelines

### Instruction Implementation

When implementing new instructions:

1. Add instruction variant to `AttestationServiceInstruction` enum in `instructions.rs`
2. Update discriminator matching in `entrypoint.rs`
3. Implement processor function in `processor/` directory with `#[inline(always)]`
4. Follow processor code structure:
   - Account destructuring using array pattern matching
   - Verification and security checks using shared functions from `processor/shared/`
   - Business logic implementation
   - Account resizing if required (implement performantly with proper calculations)
   - Event emission if required
5. Use PDA utilities from `processor/shared/pda_utils.rs` for account derivation
6. Add comprehensive tests in `integration_tests/`
7. Regenerate clients with `pnpm run generate-clients`

### Schema Design

Schema layouts use numeric type identifiers:
### Schema Layout Data Types

The `layout` field uses numeric values to specify data types for each field in the schema:

| Value | Data Type | Description |
|-------|-----------|-------------|
| 0 | U8 | Unsigned 8-bit integer |
| 1 | U16 | Unsigned 16-bit integer |
| 2 | U32 | Unsigned 32-bit integer |
| 3 | U64 | Unsigned 64-bit integer |
| 4 | U128 | Unsigned 128-bit integer |
| 5 | I8 | Signed 8-bit integer |
| 6 | I16 | Signed 16-bit integer |
| 7 | I32 | Signed 32-bit integer |
| 8 | I64 | Signed 64-bit integer |
| 9 | I128 | Signed 128-bit integer |
| 10 | Bool | Boolean value |
| 11 | Char | Single character |
| 12 | String | Variable-length string |
| 13 | VecU8 | Vector of unsigned 8-bit integers |
| 14 | VecU16 | Vector of unsigned 16-bit integers |
| 15 | VecU32 | Vector of unsigned 32-bit integers |
| 16 | VecU64 | Vector of unsigned 64-bit integers |
| 17 | VecU128 | Vector of unsigned 128-bit integers |
| 18 | VecI8 | Vector of signed 8-bit integers |
| 19 | VecI16 | Vector of signed 16-bit integers |
| 20 | VecI32 | Vector of signed 32-bit integers |
| 21 | VecI64 | Vector of signed 64-bit integers |
| 22 | VecI128 | Vector of signed 128-bit integers |
| 23 | VecBool | Vector of boolean values |
| 24 | VecChar | Vector of characters |
| 25 | VecString | Vector of strings |

For example, a layout of `[12, 0, 12]` would define three fields: a String, followed by a U8 integer, followed by another String. The `fieldNames` array provides human-readable names that correspond positionally to each layout value, so with field names `["name", "age", "country"]`, the first String field would be named "name", the U8 field would be "age", and the second String field would be "country". Here's an example of how those would be utilized when creating a Schema:


### Tokenization Flow

For tokenized attestations:

1. Create regular attestation first
2. Initialize Token-2022 mint with required extensions
3. Set up metadata and group membership
4. Create associated token account for recipient
5. Mint exactly 1 token to recipient

### Testing Practices

- Test both success and failure cases
- Use helper functions for common setup operations
- Verify account states before and after operations
- Test edge cases like expired attestations and invalid signers

### Client Integration

- TypeScript clients provide typed interfaces for all instructions
- Rust clients use `solana_program` types for compatibility
- Both clients handle account resolution and instruction building

## Configuration & Environment

### Program Constants

Key seeds and identifiers are defined in `program/src/constants.rs`:
- `CREDENTIAL_SEED`: For credential PDA derivation
- `SCHEMA_SEED`: For schema PDA derivation
- `ATTESTATION_SEED`: For attestation PDA derivation
- `SCHEMA_MINT_SEED`: For tokenized schema mints
- `ATTESTATION_MINT_SEED`: For tokenized attestation mints

### Development Environment

- Uses Solana CLI tools for program deployment
- Requires `cargo-build-sbf` for BPF compilation
- Integration tests run against program test framework
- TypeScript development requires Node.js and npm/pnpm

## Security Considerations

**SECURITY IS THE UTMOST PRIORITY** - Every instruction must implement comprehensive security checks to prevent exploits and unauthorized access.

### Critical Security Checks

All instruction processors MUST implement these security validations:

1. **Program Ownership Assertions**: 
   - Assert that all program accounts are owned by the correct program
   - Verify system accounts are owned by System Program
   - Check token accounts are owned by Token Program

2. **Signer Verification**:
   - Assert required accounts are signers using `account.is_signer()`
   - Verify authorized signers against credential's signer list
   - Validate authority accounts have signing privileges

3. **Account Mutability Checks**:
   - Assert writable accounts using `account.is_writable()`
   - Verify rent-exempt status for account modifications

4. **Discriminator Validation**:
   - Assert account discriminators match expected types (Credential, Schema, Attestation)
   - Prevent type confusion attacks by validating account data structure
   - Check account initialization state

5. **PDA Derivation Verification**:
   - Assert PDA accounts match expected derivation using seeds
   - Verify bump seeds to prevent canonical bump attacks
   - Validate account addresses against program-derived addresses

### Access Control

- All operations require authorized signer verification against credential's authorized signers list
- Credential owners control schema creation and management 
- Only authorized signers can create, modify, and close attestations
- Schema pause/resume functionality restricted to credential authority

### Data Validation

- Comprehensive schema validation ensures attestation data integrity and type safety
- Account ownership verification prevents unauthorized access 
- PDA derivation validation ensures account authenticity
- Instruction data parsing with proper error handling and bounds checking

### Token Security

- Non-transferable tokens prevent unauthorized transfers
- Permanent delegate ensures program maintains control over tokenized attestations
- Mint close authority allows proper cleanup

## Testing Infrastructure

### Integration Tests

- Located in `integration_tests/tests/`
- Cover all instruction flows and edge cases
- Use helper modules for common operations
- Test both individual instructions and complete workflows

### Client Tests

- TypeScript tests in `clients/typescript/test/`
- Verify generated client functionality
- Test instruction building and account resolution
- Ensure type safety and proper error handling

### Test Utilities

- Helper functions in `integration_tests/tests/helpers/`
- Common setup for credentials, schemas, and attestations
- Utility functions for account creation and validation
- Test data generators for consistent testing

## Behavioral Instructions

- Always run `cargo-build-sbf` before testing program changes
- Regenerate clients after modifying instruction interfaces
- Run both Rust and TypeScript tests for comprehensive validation
- Use integration tests to verify end-to-end functionality
- Follow existing patterns for new instruction implementations