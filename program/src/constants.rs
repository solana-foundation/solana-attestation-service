use const_crypto::ed25519;
use light_macros::pubkey_array;
use light_sdk_pinocchio::{derive_light_cpi_signer, CpiSigner};
use pinocchio::pubkey::Pubkey;

pub const ATTESTATION_SEED: &[u8] = b"attestation";
pub const CREDENTIAL_SEED: &[u8] = b"credential";
pub const SCHEMA_SEED: &[u8] = b"schema";
pub const EVENT_AUTHORITY_SEED: &[u8] = b"__event_authority";
pub const SAS_SEED: &[u8] = b"sas";
pub const SCHEMA_MINT_SEED: &[u8] = b"schemaMint";
pub const ATTESTATION_MINT_SEED: &[u8] = b"attestationMint";

pub const LIGHT_CPI_SIGNER: CpiSigner =
    derive_light_cpi_signer!("22zoJMtdu4tQc2PzL74ZUT7FrwgB1Udec8DdW4yw4BdG");

pub const ALLOWED_ADDRESS_TREE: Pubkey =
    pubkey_array!("amt2kaJA14v3urZbZvnc5v2np8jqvc4Z8zDep5wbtzx");

/// Maximum compressed attestation data size to ensure create + close transactions
/// fit within Solana's 1232 byte mainnet MTU
pub const MAX_COMPRESSED_ATTESTATION_SIZE: usize = 350;

// Anchor Compatitable Discriminator: Sha256(anchor:event)[..8]
pub const EVENT_IX_TAG: u64 = 0x1d9acb512ea545e4;
pub const EVENT_IX_TAG_LE: &[u8] = EVENT_IX_TAG.to_le_bytes().as_slice();

pub mod event_authority_pda {
    use super::*;

    const EVENT_AUTHORITY_AND_BUMP: ([u8; 32], u8) =
        ed25519::derive_program_address(&[EVENT_AUTHORITY_SEED], &crate::ID);

    pub const ID: Pubkey = EVENT_AUTHORITY_AND_BUMP.0;
    pub const BUMP: u8 = EVENT_AUTHORITY_AND_BUMP.1;
}

pub mod sas_pda {
    use super::*;

    const SAS_AND_BUMP: ([u8; 32], u8) = ed25519::derive_program_address(&[SAS_SEED], &crate::ID);

    pub const ID: Pubkey = SAS_AND_BUMP.0;
    pub const BUMP: u8 = SAS_AND_BUMP.1;
}
