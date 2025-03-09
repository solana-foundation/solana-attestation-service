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
    #[account(1, signer, name = "authority")]
    #[account(
        2,
        name = "credential",
        desc = "Credential the Schema is associated with"
    )]
    #[account(3, writable, name = "schema")]
    #[account(4, name = "system_program")]
    CreateSchema {
        name: String,
        description: String,
        layout: Vec<u8>,
        field_names: Vec<String>,
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

    /// Sets Credential authorized_signers
    #[account(0, writable, signer, name = "payer")]
    #[account(1, signer, name = "authority")]
    #[account(
        2,
        writable,
        name = "credential",
        desc = "Credential the Schema is associated with"
    )]
    #[account(3, name = "system_program")]
    ChangeAuthorizedSigners { signers: Vec<Pubkey> },

    /// Change description on a Schema
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
    #[account(3, name = "system_program")]
    ChangeSchemaDescription { description: String },

    /// Change Schema version
    #[account(0, writable, signer, name = "payer")]
    #[account(1, signer, name = "authority")]
    #[account(
        2,
        name = "credential",
        desc = "Credential the Schema is associated with"
    )]
    #[account(3, name = "existing_schema")]
    #[account(4, writable, name = "new_schema")]
    #[account(5, name = "system_program")]
    ChangeSchemaVersion {
        layout: Vec<u8>,
        field_names: Vec<String>,
    },

    /// Create an Attestation for a Schema by an authorized signer.
    #[account(0, writable, signer, name = "payer")]
    #[account(
        1,
        signer,
        name = "authority",
        desc = "Authorized signer of the Schema's Credential"
    )]
    #[account(
        2,
        name = "credential",
        desc = "Credential the Schema is associated with"
    )]
    #[account(3, name = "schema", desc = "Schema the Attestation is associated with")]
    #[account(4, writable, name = "attestation")]
    #[account(5, name = "system_program")]
    CreateAttestation {
        nonce: Pubkey,
        data: Vec<u8>,
        expiry: i64,
    },

    /// Close an Attestation account.
    #[account(0, writable, signer, name = "payer")]
    #[account(
        1,
        signer,
        name = "authority",
        desc = "Signer that issued the Attestation"
    )]
    #[account(2, writable, name = "attestation")]
    #[account(3, name = "system_program")]
    CloseAttestation {},
}
