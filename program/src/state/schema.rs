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
    /// The schema layout where data will be encoded with.
    pub layout: Vec<u8>,
    /// Field names of schema.
    pub field_names: Vec<u8>,
    /// Whether or not this schema is still valid
    pub is_paused: bool,
}

impl Schema {
    pub fn try_from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        let mut offset: usize = 0;

        let credential: Pubkey = data[offset..offset + 32].try_into().unwrap();
        offset += 32;

        let name_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        let name = data[offset..(offset + 4 + name_len)].to_vec();
        offset += 4 + name_len;

        let desc_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        let description = data[offset..(offset + 4 + desc_len)].to_vec();
        offset += 4 + desc_len;

        let layout_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        let layout = data[offset..(offset + 4 + layout_len)].to_vec();
        offset += 4 + layout_len;

        let field_names_len =
            u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        let field_names: Vec<u8> = data[offset..(offset + 4 + field_names_len)].to_vec();
        offset += 4 + field_names_len;

        let is_paused = data[offset] == 1;

        Ok(Self {
            credential,
            name,
            description,
            layout,
            field_names,
            is_paused,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(self.credential.as_ref());
        data.extend_from_slice(self.name.as_ref());
        data.extend_from_slice(self.description.as_ref());
        data.extend_from_slice(self.layout.as_ref());
        data.extend_from_slice(self.field_names.as_ref());
        data.extend_from_slice(&[self.is_paused as u8]);

        data
    }
}
