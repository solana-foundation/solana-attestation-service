extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use pinocchio::pubkey::Pubkey;
use shank::ShankAccount;

// PDA ["credential", authority, name]
/// Tracks the authorized signers of for schemas and their attestations.
#[derive(Clone, Debug, PartialEq, ShankAccount)]
pub struct Credential {
    /// Admin of this credential
    pub authority: Pubkey,
    /// UTF-8 encoded Name of this credential
    /// Includes 4 bytes for length of name
    pub name: String,
    /// List of signers that are allowed to "attest"
    pub authorized_signers: Vec<Pubkey>,
}
