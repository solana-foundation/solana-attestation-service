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
    CreateCredential { name: String, signers: Vec<Pubkey> } = 0,

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
    } = 1,

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
    ChangeSchemaStatus { is_paused: bool } = 2,

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
    ChangeAuthorizedSigners { signers: Vec<Pubkey> } = 3,

    /// Change description on a Schema
    #[account(0, writable, signer, name = "payer")]
    #[account(1, signer, name = "authority")]
    #[account(
        2,
        name = "credential",
        desc = "Credential the Schema is associated with"
    )]
    #[account(
        3,
        writable,
        name = "schema",
        desc = "Credential the Schema is associated with"
    )]
    #[account(4, name = "system_program")]
    ChangeSchemaDescription { description: String } = 4,

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
    } = 5,

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
    } = 6,

    /// Close an Attestation account.
    #[account(0, writable, signer, name = "payer")]
    #[account(
        1,
        signer,
        name = "authority",
        desc = "Authorized signer of the Schema's Credential"
    )]
    #[account(2, name = "credential")]
    #[account(3, writable, name = "attestation")]
    #[account(4, name = "event_authority")]
    #[account(5, name = "system_program")]
    #[account(6, name = "attestation_program")]
    CloseAttestation {} = 7,

    /// Enable tokenization for a Schema
    #[account(0, writable, signer, name = "payer")]
    #[account(1, signer, name = "authority")]
    #[account(
        2,
        name = "credential",
        desc = "Credential the Schema is associated with"
    )]
    #[account(3, name = "schema")]
    #[account(4, writable, name = "mint", desc = "Mint of Schema Token")]
    #[account(
        5,
        name = "sas_pda",
        desc = "Program derived address used as program signer authority"
    )]
    #[account(6, name = "system_program")]
    #[account(7, name = "token_program")]
    TokenizeSchema { max_size: u64 } = 9,

    /// Create attestation with token.
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
    #[account(6, writable, name = "schema_mint", desc = "Mint of Schema Token")]
    #[account(
        7,
        writable,
        name = "attestation_mint",
        desc = "Mint of Attestation Token"
    )]
    #[account(
        8,
        name = "sas_pda",
        desc = "Program derived address used as program signer authority"
    )]
    #[account(
        9,
        writable,
        name = "recipient_token_account",
        desc = "Associated token account of Recipient for Attestation Token"
    )]
    #[account(10, name = "recipient", desc = "Wallet to receive Attestation Token")]
    #[account(11, name = "token_program")]
    #[account(12, name = "associated_token_program")]
    CreateTokenizedAttestation {
        nonce: Pubkey,
        data: Vec<u8>,
        expiry: i64,
        name: String,
        uri: String,
        symbol: String,
        mint_account_space: u16,
    } = 10,

    /// Close an Attestation and Attestation token.
    #[account(0, writable, signer, name = "payer")]
    #[account(
        1,
        signer,
        name = "authority",
        desc = "Authorized signer of the Schema's Credential"
    )]
    #[account(2, name = "credential")]
    #[account(3, writable, name = "attestation")]
    #[account(4, name = "event_authority")]
    #[account(5, name = "system_program")]
    #[account(6, name = "attestation_program")]
    #[account(
        7,
        writable,
        name = "attestation_mint",
        desc = "Mint of Attestation Token"
    )]
    #[account(
        8,
        name = "sas_pda",
        desc = "Program derived address used as program signer authority"
    )]
    #[account(
        9,
        writable,
        name = "attestation_token_account",
        desc = "Associated token account of the related Attestation Token"
    )]
    #[account(10, name = "token_program")]
    CloseTokenizedAttestation {} = 11,

    /// Invoked via CPI from SAS Program to log event via instruction data.
    #[account(0, signer, name = "event_authority")]
    #[account(1, name = "attestation_program")]
    EmitEvent {} = 228,
}
