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
    pub fn try_from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        let mut offset: usize = 0;

        let authority: Pubkey = data[offset..offset + 32].try_into().unwrap();
        offset += 32;

        let name_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        let name = data[offset..(offset + 4 + name_len)].to_vec();
        offset += 4 + name_len;

        let signers_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        let mut authorized_signers: Vec<Pubkey> = Vec::new();

        offset += 4;
        for _ in 0..signers_len {
            let signer: Pubkey = data[offset..offset + 32].try_into().unwrap();
            authorized_signers.push(signer);
            offset += 32;
        }

        Ok(Self {
            authority,
            name,
            authorized_signers,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        // Authority encoding
        data.extend_from_slice(self.authority.as_ref());

        // Name encoding
        data.extend_from_slice(&(self.name.len() as u32).to_le_bytes());
        data.extend_from_slice(self.name.as_ref());

        // Authorized signers encoding
        data.extend_from_slice(&(self.authorized_signers.len() as u32).to_le_bytes());
        for signer in &self.authorized_signers {
            data.extend_from_slice(signer.as_ref());
        }

        data
    }
}
