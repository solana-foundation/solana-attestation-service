extern crate alloc;

use alloc::vec::Vec;
use pinocchio::{program_error::ProgramError, pubkey::Pubkey};
use pinocchio_log::log;
use shank::ShankAccount;

use crate::error::AttestationServiceError;

#[repr(u8)]
pub enum DataTypes {
    U8 = 0,
    U16 = 1,
    U32 = 2,
    U64 = 3,
    U128 = 4,
    I8 = 5,
    I16 = 6,
    I32 = 7,
    I64 = 8,
    I128 = 9,
    F32 = 10,
    F64 = 11,
    Bool = 12,
    Char = 13,
    String = 14,
    VecU8 = 15,
    VecU16 = 16,
    VecU32 = 17,
    VecU64 = 18,
    VecU128 = 19,
    VecI8 = 20,
    VecI16 = 21,
    VecI32 = 22,
    VecI64 = 23,
    VecI128 = 24,
    VecF32 = 25,
    VecF64 = 26,
    VecBool = 27,
    VecChar = 28,
    VecString = 29, // Max Value
}

impl DataTypes {
    pub fn max() -> u8 {
        DataTypes::VecString as u8
    }
}

// PDA ["schema", credential, name]
#[derive(Clone, Debug, PartialEq, ShankAccount)]
#[repr(C)]
pub struct Schema {
    /// The Credential that manages this Schema
    pub credential: Pubkey,
    /// Name of Schema, in UTF8-encoded byte string.
    pub name: Vec<u8>,
    /// Description of what schema does, in UTF8-encoded byte string.
    pub description: Vec<u8>,
    /// The schema layout where data will be encoded with, in array of DataTypes.
    pub layout: Vec<u8>,
    /// Field names of schema stored as serialized array of Strings.
    /// First 4 bytes are number of bytes in array.
    pub field_names: Vec<u8>,
    /// Whether or not this schema is still valid
    pub is_paused: bool,
}

impl Schema {
    pub fn validate(&self, field_names_count: u32) -> Result<(), ProgramError> {
        let layout_len = self.layout.len().checked_sub(4).unwrap();

        for i in 4..(4 + layout_len) {
            if self.layout[i] > DataTypes::max() {
                return Err(AttestationServiceError::InvalidSchema.into());
            }
        }

        // Expect number of field names to match number of fields in layout.
        if field_names_count != u32::try_from(layout_len).unwrap() {
            return Err(AttestationServiceError::InvalidSchema.into());
        }
        Ok(())
    }

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

        let field_names_byte_len =
            u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        let field_names: Vec<u8> = data[offset..(offset + 4 + field_names_byte_len)].to_vec();
        offset += 4 + field_names_byte_len;

        let is_paused = data[offset] == 1;
        offset += 1;

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
