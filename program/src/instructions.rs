extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use pinocchio::pubkey::Pubkey;
use shank::ShankInstruction;

/// Instructions for the Solana Attestation Service. This
/// is currently not used in the program business logic, but
/// we include it for IDL generation.
#[repr(C, u8)]
#[derive(Clone, Debug, PartialEq, ShankInstruction)]
pub enum AttestationServiceInstruction {
    /// Creates the Credential PDA account for an Issuer.
    #[account(0, writable, signer, name = "payer")]
    #[account(1, writable, name = "credential")]
    #[account(2, signer, name = "authority")]
    #[account(3, name = "system_program")]
    CreateCredential { name: String, signers: Vec<Pubkey> },

    /// Create a Schema for a Credential that can eventually be attested to.
    #[account(0, writable, signer, name = "payer")]
    #[account(
        1,
        name = "credential",
        desc = "Credential the Schema is associated with"
    )]
    #[account(2, writable, name = "schema")]
    #[account(3, name = "system_program")]
    CreateSchema {
        name: String,
        description: String,
        data: Vec<u8>,
    },

    /// Sets Schema is_paused status
    #[account(0, signer, name = "authority")]
    #[account(
        1,
        name = "credential",
        desc = "Credential the Schema is associated with"
    )]
    #[account(
        2,
        writable,
        name = "schema",
        desc = "Credential the Schema is associated with"
    )]
    ChangeSchemaStatus { is_paused: bool },
}
