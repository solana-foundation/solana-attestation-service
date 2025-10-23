use crate::programs::SOLANA_ATTESTATION_SERVICE_ID;
use const_crypto::ed25519;
use solana_program::pubkey::Pubkey;

// PDA seeds
pub const ATTESTATION_SEED: &[u8] = b"attestation";
pub const CREDENTIAL_SEED: &[u8] = b"credential";
pub const SCHEMA_SEED: &[u8] = b"schema";
pub const EVENT_AUTHORITY_SEED: &[u8] = b"__event_authority";
pub const SAS_SEED: &[u8] = b"sas";
pub const SCHEMA_MINT_SEED: &[u8] = b"schemaMint";
pub const ATTESTATION_MINT_SEED: &[u8] = b"attestationMint";

const EVENT_AUTHORITY_AND_BUMP: ([u8; 32], u8) =
    ed25519::derive_program_address(&[EVENT_AUTHORITY_SEED], &SOLANA_ATTESTATION_SERVICE_ID.to_bytes());

const SAS_AND_BUMP: ([u8; 32], u8) =
    ed25519::derive_program_address(&[SAS_SEED], &SOLANA_ATTESTATION_SERVICE_ID.to_bytes());

/// The EVENT_AUTHORITY PDA address.
pub const EVENT_AUTHORITY_PDA: Pubkey = Pubkey::new_from_array(EVENT_AUTHORITY_AND_BUMP.0);

/// The SAS authority PDA address.
pub const SAS_AUTHORITY_PDA: Pubkey = Pubkey::new_from_array(SAS_AND_BUMP.0);

/// Get the EVENT_AUTHORITY_ADDRESS.
#[inline(always)]
pub const fn derive_event_authority_pda() -> Pubkey {
    EVENT_AUTHORITY_PDA
}

/// Get the SAS_ADDRESS.
#[inline(always)]
pub const fn derive_sas_authority_pda() -> Pubkey {
    SAS_AUTHORITY_PDA
}

/// Derive a Credential (aka Issuer) PDA.
///
/// # Arguments
/// * `authority` - The Pubkey that controls the Credential account.
/// * `name` - A name for the credential. NOTE that only the first 32 bytes
///   will be used for the PDA due to seed size limits.
pub fn derive_credential_pda(authority: &Pubkey, name: &str) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[CREDENTIAL_SEED, authority.as_ref(), name.as_bytes()],
        &SOLANA_ATTESTATION_SERVICE_ID,
    )
}

/// Derive a Schema PDA.
///
/// # Arguments
/// * `credential` - The Credential that the Schema is associated with.
/// * `name` - A name for the schema. NOTE that only the first 32 bytes
///   will be used for the PDA due to seed size limits.
/// * `version` - The version number (up to u8::MAX) of the Schema.
pub fn derive_schema_pda(credential: &Pubkey, name: &str, version: u8) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            SCHEMA_SEED,
            credential.as_ref(),
            name.as_bytes(),
            &[version],
        ],
        &SOLANA_ATTESTATION_SERVICE_ID,
    )
}

/// Derive an Attestation PDA.
///
/// # Arguments
/// * `credential` - The Credential (aka Issuer) that controls the Attestation state.
/// * `schema` - The Schema that the Attestation adheres to.
/// * `nonce` - A Pubkey that may either represent the Wallet the Attestation
///   is associated with OR a randomly generated Pubkey to prevent PDA collision.
pub fn derive_attestation_pda(
    credential: &Pubkey,
    schema: &Pubkey,
    nonce: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            ATTESTATION_SEED,
            credential.as_ref(),
            schema.as_ref(),
            nonce.as_ref(),
        ],
        &SOLANA_ATTESTATION_SERVICE_ID,
    )
}

/* PDAs for tokenization */

/// Derive the Token2022 Mint address of the tokenized Schema.
///
/// # Arguments
/// * `schema` - The Schema that the token belongs to.
pub fn derive_schema_mint_pda(schema: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[SCHEMA_MINT_SEED, schema.as_ref()],
        &SOLANA_ATTESTATION_SERVICE_ID,
    )
}

/// Derive the Token2022 Mint address of the tokenized Attestation.
///
/// # Arguments
/// * `attestation` - The Attestation that the token belongs to.
pub fn derive_attestation_mint_pda(attestation: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[ATTESTATION_MINT_SEED, attestation.as_ref()],
        &SOLANA_ATTESTATION_SERVICE_ID,
    )
}
