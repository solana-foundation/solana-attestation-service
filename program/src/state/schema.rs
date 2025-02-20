extern crate alloc;

use alloc::vec::Vec;
use pinocchio::{msg, program_error::ProgramError, pubkey::Pubkey};
use shank::ShankAccount;

use super::discriminator::{AccountSerialize, AttestationAccountDiscriminators, Discriminator};

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
    pub is_paused: bool,
}

impl Discriminator for Schema {
    const DISCRIMINATOR: u8 = AttestationAccountDiscriminators::SchemaDiscriminator as u8;
}

impl AccountSerialize for Schema {
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(self.credential.as_ref());
        data.extend_from_slice(self.name.as_ref());
        data.extend_from_slice(self.description.as_ref());
        data.extend_from_slice(self.data_schema.as_ref());
        data.extend_from_slice(&[self.is_paused as u8]);

        data
    }
}

impl Schema {
    pub fn try_from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        // Check discriminator
        if data[0] != Self::DISCRIMINATOR {
            msg!("Invalid Schema Data");
            return Err(ProgramError::InvalidAccountData);
        }

        // Start offset after Discriminator
        let mut offset: usize = 1;

        let credential: Pubkey = data[offset..offset + 32].try_into().unwrap();
        offset += 32;

        let name_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        let name = data[offset..(offset + 4 + name_len)].to_vec();
        offset += 4 + name_len;

        let desc_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        let description = data[offset..(offset + 4 + desc_len)].to_vec();
        offset += 4 + desc_len;

        let schema_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        let data_schema = data[offset..(offset + 4 + schema_len)].to_vec();
        offset += 4 + schema_len;

        let is_paused = data[offset] == 1;

        Ok(Self {
            credential,
            name,
            description,
            data_schema,
            is_paused,
        })
    }
}
