use const_crypto::ed25519;
use pinocchio::pubkey::Pubkey;

pub const ATTESTATION_SEED: &[u8] = b"attestation";
pub const CREDENTIAL_SEED: &[u8] = b"credential";
pub const SCHEMA_SEED: &[u8] = b"schema";
pub const EVENT_AUTHORITY_SEED: &[u8] = b"eventAuthority";
pub const SAS_SEED: &[u8] = b"sas";
pub const SCHEMA_MINT_SEED: &[u8] = b"schemaMint";
pub const ATTESTATION_MINT_SEED: &[u8] = b"attestationMint";

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
