extern crate alloc;

use alloc::vec::Vec;
use pinocchio::{program_error::ProgramError, pubkey::Pubkey};
use shank::ShankAccount;

// PDA ["schema", credential, name]
#[derive(Clone, Debug, PartialEq, ShankAccount)]
#[repr(C)]
pub struct Schema {
    /// The Credential that manages this Schema
    pub credential: Pubkey,
    /// Name of Schema
    pub name: Vec<u8>,
    /// Description of what schema does
    pub description: Vec<u8>,
    /// Encoding of the `CustomData` struct that needs to be asserted against?
    pub data_schema: Vec<u8>,
    /// Whether or not this schema is still valid
    pub is_revoked: bool,
}

impl Schema {
    pub fn try_from_bytes(_data: &[u8]) -> Result<Self, ProgramError> {
        // TODO implement
        Err(ProgramError::UnsupportedSysvar)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(self.credential.as_ref());

        data.extend_from_slice(&(self.name.len() as u32).to_le_bytes());
        data.extend_from_slice(self.name.as_ref());

        data.extend_from_slice(&(self.description.len() as u32).to_le_bytes());
        data.extend_from_slice(self.description.as_ref());

        data.extend_from_slice(&(self.data_schema.len() as u32).to_le_bytes());
        data.extend_from_slice(self.data_schema.as_ref());

        data.extend_from_slice(&[self.is_revoked as u8]);

        data
    }
}
