extern crate alloc;

use alloc::vec::Vec;
use pinocchio::{program_error::ProgramError, pubkey::Pubkey};
use shank::ShankAccount;

// PDA ["credential", authority, name]
/// Tracks the authorized signers of for schemas and their attestations.
#[derive(Clone, Debug, PartialEq, ShankAccount)]
#[repr(C)]
pub struct Credential {
    /// Admin of this credential
    pub authority: Pubkey,
    /// UTF-8 encoded Name of this credential
    /// Includes 4 bytes for length of name
    pub name: Vec<u8>,
    /// List of signers that are allowed to "attest"
    pub authorized_signers: Vec<Pubkey>,
}

impl Credential {
    pub fn try_from_bytes(_data: &[u8]) -> Result<Self, ProgramError> {
        // TODO implement
        Err(ProgramError::UnsupportedSysvar)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        // Authority encoding
        data.extend_from_slice(self.authority.as_ref());

        // Name encoding
        data.extend_from_slice(self.name.as_ref());

        // Authorized signers encoding
        data.extend_from_slice(&(self.authorized_signers.len() as u32).to_le_bytes());
        for signer in &self.authorized_signers {
            data.extend_from_slice(signer.as_ref());
        }

        data
    }
}
