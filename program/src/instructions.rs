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
    ///   0. `[w,s]` Payer that will fund the Credential account.
    ///   1. `[w]` Credential account.
    ///   2. `[s]` Credential authority.
    ///   3. `[]` System Program.
    #[account(0, writable, signer, name = "payer")]
    #[account(1, writable, name = "credential")]
    #[account(2, signer, name = "authority")]
    #[account(3, name = "system_program")]
    CreateCredential { name: String, signers: Vec<Pubkey> },
}
