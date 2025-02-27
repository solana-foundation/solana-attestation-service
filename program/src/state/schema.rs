extern crate alloc;

use alloc::vec::Vec;
use pinocchio::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};
use pinocchio_log::log;
use shank::ShankAccount;
use solana_program::pubkey::Pubkey as SolanaPubkey;

use crate::{acc_info_as_str, constants::SCHEMA_SEED, error::AttestationServiceError};

use super::discriminator::{AccountSerialize, AttestationAccountDiscriminators, Discriminator};

#[repr(u8)]
pub enum SchemaDataTypes {
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
    Bool = 10,
    Char = 11,
    String = 12,
    VecU8 = 13,
    VecU16 = 14,
    VecU32 = 15,
    VecU64 = 16,
    VecU128 = 17,
    VecI8 = 18,
    VecI16 = 19,
    VecI32 = 20,
    VecI64 = 21,
    VecI128 = 22,
    VecBool = 23,
    VecChar = 24,
    VecString = 25, // Max Value
}

impl SchemaDataTypes {
    pub fn max() -> u8 {
        SchemaDataTypes::VecString as u8
    }
}

// PDA ["schema", credential, name, version]
#[derive(Clone, Debug, PartialEq, ShankAccount)]
#[repr(C)]
pub struct Schema {
    /// The Credential that manages this Schema
    pub credential: Pubkey,
    /// Name of Schema, in UTF8-encoded byte string.
    pub name: Vec<u8>,
    /// Description of what schema does, in UTF8-encoded byte string.
    pub description: Vec<u8>,
    /// The schema layout where data will be encoded with, in array of SchemaDataTypes.
    pub layout: Vec<u8>,
    /// Field names of schema stored as serialized array of Strings.
    /// First 4 bytes are number of bytes in array.
    pub field_names: Vec<u8>,
    /// Whether or not this schema is still valid
    pub is_paused: bool,
    /// Version of this schema. Defaults to 1.
    pub version: u8,
}

impl Discriminator for Schema {
    const DISCRIMINATOR: u8 = AttestationAccountDiscriminators::SchemaDiscriminator as u8;
}

impl AccountSerialize for Schema {
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(self.credential.as_ref());
        data.extend(&(self.name.len() as u32).to_le_bytes());
        data.extend_from_slice(self.name.as_ref());
        data.extend(&(self.description.len() as u32).to_le_bytes());
        data.extend_from_slice(self.description.as_ref());
        data.extend(&(self.layout.len() as u32).to_le_bytes());
        data.extend_from_slice(self.layout.as_ref());
        data.extend(&(self.field_names.len() as u32).to_le_bytes());
        data.extend_from_slice(self.field_names.as_ref());
        data.extend_from_slice(&[self.is_paused as u8]);
        data.extend_from_slice(&[self.version as u8]);

        data
    }
}

impl Schema {
    pub fn verify_pda(
        &self,
        acc_info: &AccountInfo,
        program_id: &Pubkey,
    ) -> Result<(), ProgramError> {
        let (expected_schema_pda, _bump) = SolanaPubkey::find_program_address(
            &[
                SCHEMA_SEED,
                self.credential.as_ref(),
                self.name.as_ref(), // Convert Vec<u8> to UTF8 Array
                &[self.version],
            ],
            &SolanaPubkey::from(*program_id),
        );
        if acc_info.key().ne(&expected_schema_pda.to_bytes()) {
            log!("PDA Mismatch for {}", acc_info_as_str!(acc_info));
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }

    pub fn validate(&self, field_names_count: u32) -> Result<(), ProgramError> {
        let size_offset = 4;
        let layout_len = self.layout.len();

        for i in size_offset..self.layout.len() {
            if self.layout[i] > SchemaDataTypes::max() {
                return Err(AttestationServiceError::InvalidSchemaDataType.into());
            }
        }

        // Expect number of field names to match number of fields in layout.
        if field_names_count != u32::try_from(layout_len).unwrap() {
            log!("Field names does not match layout length");
            return Err(AttestationServiceError::InvalidSchema.into());
        }
        Ok(())
    }

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
        offset += 4;
        let name = data[offset..offset + name_len].to_vec();
        offset += name_len;

        let desc_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let description = data[offset..offset + desc_len].to_vec();
        offset += desc_len;

        let layout_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let layout = data[offset..offset + layout_len].to_vec();
        offset += layout_len;

        let field_names_byte_len =
            u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let field_names: Vec<u8> = data[offset..offset + field_names_byte_len].to_vec();
        offset += field_names_byte_len;

        let is_paused = data[offset] == 1;
        offset += 1;

        let version = data[offset];

        Ok(Self {
            credential,
            name,
            description,
            layout,
            field_names,
            is_paused,
            version,
        })
    }
}
