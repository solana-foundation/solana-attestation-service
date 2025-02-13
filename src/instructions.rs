extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use pinocchio::pubkey::Pubkey;

// TODO this isn't actually used anywhere. It may only be necessary
// on client depending on how we decide to handle (de)serialization.

#[derive(Clone, Debug, PartialEq)]
pub struct CreateCredentialArgs {
    pub name: String,
    pub signers: Vec<Pubkey>,
}

/// Instructions for the Solana Attestation Service.
#[repr(C, u8)]
#[derive(Clone, Debug, PartialEq)]
pub enum AttestationServiceInstruction {
    /// Creates the Credential PDA account for an Issuer.
    ///   0. `[w,s]` Payer that will fund the Credential account.
    ///   1. `[w]` Credential account.
    ///   2. `[s]` Credential authority.
    ///   3. `[]` System Program.
    CreateCredential(CreateCredentialArgs),
}
