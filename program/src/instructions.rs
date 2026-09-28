extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use codama::CodamaInstructions;
use pinocchio::Address;

/// Instructions for the Solana Attestation Service. This
/// is currently not used in the program business logic, but
/// we include it for IDL generation.
#[repr(C, u8)]
#[derive(Clone, Debug, PartialEq, CodamaInstructions)]
pub enum AttestationServiceInstruction {
    /// Creates the Credential PDA account for an Issuer.
    #[codama(account(name = "payer", signer, writable, default_value = payer))]
    #[codama(account(
        name = "credential",
        writable,
        default_value = pda("credential", [account("authority"), argument("name")])
    ))]
    #[codama(account(name = "authority", signer))]
    #[codama(account(name = "system_program", default_value = program("system")))]
    CreateCredential { name: String, signers: Vec<Address> } = 0,

    /// Create a Schema for a Credential that can eventually be attested to.
    #[codama(account(name = "payer", signer, writable, default_value = payer))]
    #[codama(account(name = "authority", signer))]
    #[codama(account(name = "credential", docs = "Credential the Schema is associated with"))]
    #[codama(account(
        name = "schema",
        writable,
        default_value = pda("schema", [account("credential"), argument("name"), seed("version", 1)])
    ))]
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
    ChangeAuthorizedSigners { signers: Vec<Address> } = 3,

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
    #[codama(account(
        name = "attestation",
        writable,
        default_value = pda("attestation", [account("credential"), account("schema"), argument("nonce")])
    ))]
    #[codama(account(name = "system_program", default_value = program("system")))]
    CreateAttestation {
        nonce: Address,
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
    #[codama(account(
        name = "event_authority",
        default_value = public_key("DzSpKpST2TSyrxokMXchFz3G2yn5WEGoxzpGEUDjCX4g")
    ))]
    #[codama(account(name = "system_program", default_value = program("system")))]
    #[codama(account(
        name = "attestation_program",
        default_value = public_key("22zoJMtdu4tQc2PzL74ZUT7FrwgB1Udec8DdW4yw4BdG")
    ))]
    CloseAttestation {} = 7,

    /// Enable tokenization for a Schema
    #[codama(account(name = "payer", signer, writable, default_value = payer))]
    #[codama(account(name = "authority", signer))]
    #[codama(account(name = "credential", docs = "Credential the Schema is associated with"))]
    #[codama(account(name = "schema"))]
    #[codama(account(
        name = "mint",
        writable,
        docs = "Mint of Schema Token",
        default_value = pda("schemaMint", [account("schema")])
    ))]
    #[codama(account(
        name = "sas_pda",
        docs = "Program derived address used as program signer authority",
        default_value = pda("sasAuthority")
    ))]
    #[codama(account(name = "system_program", default_value = program("system")))]
    #[codama(account(name = "token_program", default_value = program("token-2022")))]
    TokenizeSchema { max_size: u64 } = 9,

    /// Create attestation with token.
    #[codama(account(name = "payer", signer, writable, default_value = payer))]
    #[codama(account(name = "authority", signer, docs = "Authorized signer of the Schema's Credential"))]
    #[codama(account(name = "credential", docs = "Credential the Schema is associated with"))]
    #[codama(account(name = "schema", docs = "Schema the Attestation is associated with"))]
    #[codama(account(
        name = "attestation",
        writable,
        default_value = pda("attestation", [account("credential"), account("schema"), argument("nonce")])
    ))]
    #[codama(account(name = "system_program", default_value = program("system")))]
    #[codama(account(
        name = "schema_mint",
        writable,
        docs = "Mint of Schema Token",
        default_value = pda("schemaMint", [account("schema")])
    ))]
    #[codama(account(
        name = "attestation_mint",
        writable,
        docs = "Mint of Attestation Token",
        default_value = pda("attestationMint", [account("attestation")])
    ))]
    #[codama(account(
        name = "sas_pda",
        docs = "Program derived address used as program signer authority",
        default_value = pda("sasAuthority")
    ))]
    #[codama(account(
        name = "recipient_token_account",
        writable,
        docs = "Associated token account of Recipient for Attestation Token"
    ))]
    #[codama(account(name = "recipient", docs = "Wallet to receive Attestation Token"))]
    #[codama(account(name = "token_program", default_value = program("token-2022")))]
    #[codama(account(name = "associated_token_program", default_value = program("associated-token")))]
    CreateTokenizedAttestation {
        nonce: Address,
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
    #[codama(account(
        name = "event_authority",
        default_value = public_key("DzSpKpST2TSyrxokMXchFz3G2yn5WEGoxzpGEUDjCX4g")
    ))]
    #[codama(account(name = "system_program", default_value = program("system")))]
    #[codama(account(
        name = "attestation_program",
        default_value = public_key("22zoJMtdu4tQc2PzL74ZUT7FrwgB1Udec8DdW4yw4BdG")
    ))]
    #[codama(account(
        name = "attestation_mint",
        writable,
        docs = "Mint of Attestation Token",
        default_value = pda("attestationMint", [account("attestation")])
    ))]
    #[codama(account(
        name = "sas_pda",
        docs = "Program derived address used as program signer authority",
        default_value = pda("sasAuthority")
    ))]
    #[codama(account(
        name = "attestation_token_account",
        writable,
        docs = "Associated token account of the related Attestation Token"
    ))]
    #[codama(account(name = "token_program", default_value = program("token-2022")))]
    CloseTokenizedAttestation {} = 11,

    /// Invoked via CPI from SAS Program to log event via instruction data.
    #[codama(account(
        name = "event_authority",
        signer,
        default_value = public_key("DzSpKpST2TSyrxokMXchFz3G2yn5WEGoxzpGEUDjCX4g")
    ))]
    EmitEvent {} = 228,
}
