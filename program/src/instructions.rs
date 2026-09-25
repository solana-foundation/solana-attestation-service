extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use codama::CodamaInstructions;
use pinocchio::pubkey::Pubkey;

/// Instructions for the Solana Attestation Service. This
/// is currently not used in the program business logic, but
/// we include it for IDL generation.
#[repr(C, u8)]
#[derive(Clone, Debug, PartialEq, CodamaInstructions)]
pub enum AttestationServiceInstruction {
    /// Creates the Credential PDA account for an Issuer.
    #[codama(account(name = "payer", signer, writable, default_value = payer))]
    #[codama(account(name = "credential", writable))]
    #[codama(account(name = "authority", signer))]
    #[codama(account(name = "system_program", default_value = program("system")))]
    CreateCredential { name: String, signers: Vec<Pubkey> } = 0,

    /// Create a Schema for a Credential that can eventually be attested to.
    #[codama(account(name = "payer", signer, writable, default_value = payer))]
    #[codama(account(name = "authority", signer))]
    #[codama(account(name = "credential", docs = "Credential the Schema is associated with"))]
    #[codama(account(name = "schema", writable))]
    #[codama(account(name = "system_program", default_value = program("system")))]
    CreateSchema {
        name: String,
        description: String,
        #[codama(type = bytes)]
        #[codama(size_prefix = number(u32))]
        layout: Vec<u8>,
        field_names: Vec<String>,
    } = 1,

    /// Sets Schema is_paused status
    #[codama(account(name = "authority", signer))]
    #[codama(account(name = "credential", docs = "Credential the Schema is associated with"))]
    #[codama(account(name = "schema", writable, docs = "Credential the Schema is associated with"))]
    ChangeSchemaStatus { is_paused: bool } = 2,

    /// Sets Credential authorized_signers
    #[codama(account(name = "payer", signer, writable, default_value = payer))]
    #[codama(account(name = "authority", signer))]
    #[codama(account(name = "credential", writable, docs = "Credential the Schema is associated with"))]
    #[codama(account(name = "system_program", default_value = program("system")))]
    ChangeAuthorizedSigners { signers: Vec<Pubkey> } = 3,

    /// Change description on a Schema
    #[codama(account(name = "payer", signer, writable, default_value = payer))]
    #[codama(account(name = "authority", signer))]
    #[codama(account(name = "credential", docs = "Credential the Schema is associated with"))]
    #[codama(account(name = "schema", writable, docs = "Credential the Schema is associated with"))]
    #[codama(account(name = "system_program", default_value = program("system")))]
    ChangeSchemaDescription { description: String } = 4,

    /// Change Schema version
    #[codama(account(name = "payer", signer, writable, default_value = payer))]
    #[codama(account(name = "authority", signer))]
    #[codama(account(name = "credential", docs = "Credential the Schema is associated with"))]
    #[codama(account(name = "existing_schema"))]
    #[codama(account(name = "new_schema", writable))]
    #[codama(account(name = "system_program", default_value = program("system")))]
    ChangeSchemaVersion {
        #[codama(type = bytes)]
        #[codama(size_prefix = number(u32))]
        layout: Vec<u8>,
        field_names: Vec<String>,
    } = 5,

    /// Create an Attestation for a Schema by an authorized signer.
    #[codama(account(name = "payer", signer, writable, default_value = payer))]
    #[codama(account(name = "authority", signer, docs = "Authorized signer of the Schema's Credential"))]
    #[codama(account(name = "credential", docs = "Credential the Schema is associated with"))]
    #[codama(account(name = "schema", docs = "Schema the Attestation is associated with"))]
    #[codama(account(name = "attestation", writable))]
    #[codama(account(name = "system_program", default_value = program("system")))]
    CreateAttestation {
        nonce: Pubkey,
        #[codama(type = bytes)]
        #[codama(size_prefix = number(u32))]
        data: Vec<u8>,
        expiry: i64,
    } = 6,

    /// Close an Attestation account.
    #[codama(account(name = "payer", signer, writable, default_value = payer))]
    #[codama(account(name = "authority", signer, docs = "Authorized signer of the Schema's Credential"))]
    #[codama(account(name = "credential"))]
    #[codama(account(name = "attestation", writable))]
    #[codama(account(name = "event_authority"))]
    #[codama(account(name = "system_program", default_value = program("system")))]
    #[codama(account(name = "attestation_program"))]
    CloseAttestation {} = 7,

    /// Enable tokenization for a Schema
    #[codama(account(name = "payer", signer, writable, default_value = payer))]
    #[codama(account(name = "authority", signer))]
    #[codama(account(name = "credential", docs = "Credential the Schema is associated with"))]
    #[codama(account(name = "schema"))]
    #[codama(account(name = "mint", writable, docs = "Mint of Schema Token"))]
    #[codama(account(name = "sas_pda", docs = "Program derived address used as program signer authority"))]
    #[codama(account(name = "system_program", default_value = program("system")))]
    #[codama(account(name = "token_program"))]
    TokenizeSchema { max_size: u64 } = 9,

    /// Create attestation with token.
    #[codama(account(name = "payer", signer, writable, default_value = payer))]
    #[codama(account(name = "authority", signer, docs = "Authorized signer of the Schema's Credential"))]
    #[codama(account(name = "credential", docs = "Credential the Schema is associated with"))]
    #[codama(account(name = "schema", docs = "Schema the Attestation is associated with"))]
    #[codama(account(name = "attestation", writable))]
    #[codama(account(name = "system_program", default_value = program("system")))]
    #[codama(account(name = "schema_mint", writable, docs = "Mint of Schema Token"))]
    #[codama(account(name = "attestation_mint", writable, docs = "Mint of Attestation Token"))]
    #[codama(account(name = "sas_pda", docs = "Program derived address used as program signer authority"))]
    #[codama(account(
        name = "recipient_token_account",
        writable,
        docs = "Associated token account of Recipient for Attestation Token"
    ))]
    #[codama(account(name = "recipient", docs = "Wallet to receive Attestation Token"))]
    #[codama(account(name = "token_program"))]
    #[codama(account(name = "associated_token_program"))]
    CreateTokenizedAttestation {
        nonce: Pubkey,
        #[codama(type = bytes)]
        #[codama(size_prefix = number(u32))]
        data: Vec<u8>,
        expiry: i64,
        name: String,
        uri: String,
        symbol: String,
        mint_account_space: u16,
    } = 10,

    /// Close an Attestation and Attestation token.
    #[codama(account(name = "payer", signer, writable, default_value = payer))]
    #[codama(account(name = "authority", signer, docs = "Authorized signer of the Schema's Credential"))]
    #[codama(account(name = "credential"))]
    #[codama(account(name = "attestation", writable))]
    #[codama(account(name = "event_authority"))]
    #[codama(account(name = "system_program", default_value = program("system")))]
    #[codama(account(name = "attestation_program"))]
    #[codama(account(name = "attestation_mint", writable, docs = "Mint of Attestation Token"))]
    #[codama(account(name = "sas_pda", docs = "Program derived address used as program signer authority"))]
    #[codama(account(
        name = "attestation_token_account",
        writable,
        docs = "Associated token account of the related Attestation Token"
    ))]
    #[codama(account(name = "token_program"))]
    CloseTokenizedAttestation {} = 11,

    /// Invoked via CPI from SAS Program to log event via instruction data.
    #[codama(account(name = "event_authority", signer))]
    EmitEvent {} = 228,
}
