extern crate alloc;

use alloc::vec::Vec;
use pinocchio::{msg, program_error::ProgramError, pubkey::Pubkey};
use shank::ShankAccount;

use crate::error::AttestationServiceError;

use super::{
    discriminator::{AccountSerialize, AttestationAccountDiscriminators, Discriminator},
    SchemaDataTypes,
};

// PDA ["attestation", credential, signer, schema, nonce]
#[derive(Clone, Debug, PartialEq, ShankAccount)]
#[repr(C)]
pub struct Attestation {
    /// A pubkey that may either be randomly generated OR associated with a User's wallet
    pub nonce: Pubkey,
    /// Credential this attestation is related to
    pub credential: Pubkey,
    /// Reference to the Schema this Attestation adheres to
    pub schema: Pubkey,
    /// Data that was verified and matches the Schema
    pub data: Vec<u8>,
    /// The pubkey of the signer. Must be one of the `authorized_signer`s at time of attestation
    pub signer: Pubkey,
    /// Designates when the credential is expired. 0 means never expired
    pub expiry: i64,
}

impl Discriminator for Attestation {
    const DISCRIMINATOR: u8 = AttestationAccountDiscriminators::AttestationDiscriminator as u8;
}

impl AccountSerialize for Attestation {
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(self.nonce.as_ref());
        data.extend_from_slice(self.credential.as_ref());
        data.extend_from_slice(self.schema.as_ref());
        data.extend_from_slice(&(self.data.len() as u32).to_le_bytes());
        data.extend_from_slice(self.data.as_ref());
        data.extend_from_slice(self.signer.as_ref());
        data.extend_from_slice(&self.expiry.to_le_bytes());

        data
    }
}

#[inline]
fn get_size_of_vec(offset: usize, element_size: usize, data: &Vec<u8>) -> usize {
    let len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    4 + len * element_size
}

impl Attestation {
    /// Validate the data in the Attestation conforms to the Schema's
    /// layout.
    pub fn validate_data(&self, layout: Vec<u8>) -> Result<(), ProgramError> {
        // Iterate over the data and ensure there are no overflows.
        // If we do not overflow and match with the end of the data,
        // then we can assume the data is valid for the schema.
        let mut data_offset = 0;
        for data_type in layout {
            let schema_data_type: SchemaDataTypes = data_type.into();
            match schema_data_type {
                // u8 -> u128
                SchemaDataTypes::U8 => data_offset += 1,
                SchemaDataTypes::U16 => data_offset += 2,
                SchemaDataTypes::U32 => data_offset += 4,
                SchemaDataTypes::U64 => data_offset += 8,
                SchemaDataTypes::U128 => data_offset += 16,
                // i8 -> i128
                SchemaDataTypes::I8 => data_offset += 1,
                SchemaDataTypes::I16 => data_offset += 2,
                SchemaDataTypes::I32 => data_offset += 4,
                SchemaDataTypes::I64 => data_offset += 8,
                SchemaDataTypes::I128 => data_offset += 16,
                // bool
                SchemaDataTypes::Bool => data_offset += 1,
                // char
                SchemaDataTypes::Char => data_offset += 4,
                // String
                SchemaDataTypes::String => {
                    data_offset += get_size_of_vec(data_offset, 1, &self.data)
                }
                // Vec<u8> -> Vec<u128>
                SchemaDataTypes::VecU8 => {
                    data_offset += get_size_of_vec(data_offset, 1, &self.data)
                }
                SchemaDataTypes::VecU16 => {
                    data_offset += get_size_of_vec(data_offset, 2, &self.data)
                }
                SchemaDataTypes::VecU32 => {
                    data_offset += get_size_of_vec(data_offset, 4, &self.data)
                }
                SchemaDataTypes::VecU64 => {
                    data_offset += get_size_of_vec(data_offset, 8, &self.data)
                }
                SchemaDataTypes::VecU128 => {
                    data_offset += get_size_of_vec(data_offset, 16, &self.data)
                }
                // Vec<i8> -> Vec<i128>
                SchemaDataTypes::VecI8 => {
                    data_offset += get_size_of_vec(data_offset, 1, &self.data)
                }
                SchemaDataTypes::VecI16 => {
                    data_offset += get_size_of_vec(data_offset, 2, &self.data)
                }
                SchemaDataTypes::VecI32 => {
                    data_offset += get_size_of_vec(data_offset, 4, &self.data)
                }
                SchemaDataTypes::VecI64 => {
                    data_offset += get_size_of_vec(data_offset, 8, &self.data)
                }
                SchemaDataTypes::VecI128 => {
                    data_offset += get_size_of_vec(data_offset, 16, &self.data)
                }
                // Vec<bool>
                SchemaDataTypes::VecBool => {
                    data_offset += get_size_of_vec(data_offset, 1, &self.data)
                }
                // Vec<char>
                SchemaDataTypes::VecChar => {
                    data_offset += get_size_of_vec(data_offset, 4, &self.data)
                }
                // Vec<String>
                SchemaDataTypes::VecString => {
                    let len = u32::from_le_bytes(
                        self.data[data_offset..data_offset + 4].try_into().unwrap(),
                    ) as usize;
                    data_offset += 4;
                    // must iterate over the strings using their len
                    for _ in 0..len {
                        let string_len = u32::from_le_bytes(
                            self.data[data_offset..data_offset + 4].try_into().unwrap(),
                        ) as usize;
                        data_offset += 4 + string_len;
                    }
                }
                _ => return Err(AttestationServiceError::InvalidSchemaDataType.into()),
            }

            // Check data size at end of each iteration and error if offset exceeds the data length.
            if data_offset > self.data.len() {
                return Err(AttestationServiceError::InvalidAttestationData.into());
            }
        }
        if data_offset != self.data.len() {
            return Err(AttestationServiceError::InvalidAttestationData.into());
        }
        Ok(())
    }

    pub fn try_from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        // Check discriminator
        if data[0] != Self::DISCRIMINATOR {
            msg!("Invalid Attestation Data");
            return Err(ProgramError::InvalidAccountData);
        }

        // Start offset after Discriminator
        let mut offset: usize = 1;

        let nonce: Pubkey = data[offset..offset + 32].try_into().unwrap();
        offset += 32;

        let credential: Pubkey = data[offset..offset + 32].try_into().unwrap();
        offset += 32;

        let schema: Pubkey = data[offset..offset + 32].try_into().unwrap();
        offset += 32;

        let data_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let attestation_data = data[offset..offset + data_len].to_vec();
        offset += data_len;

        let signer: Pubkey = data[offset..offset + 32].try_into().unwrap();
        offset += 32;

        let expiry = i64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());

        Ok(Self {
            nonce,
            credential,
            schema,
            data: attestation_data,
            signer,
            expiry,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::processor::to_serialized_vec;

    use super::*;

    #[test]
    fn attestation_validate_data() {
        let mut attestation = Attestation {
            nonce: Pubkey::default(),
            credential: Pubkey::default(),
            schema: Pubkey::default(),
            data: Vec::new(),
            signer: Pubkey::default(),
            expiry: 0,
        };

        // u8
        let layout = alloc::vec![0];
        attestation.data = alloc::vec![10];
        assert!(attestation.validate_data(layout).is_ok());

        // u8, Vec<String>, u128
        let layout = alloc::vec![0, 25, 4];
        let mut data: Vec<u8> = Vec::new();
        data.extend([10]);
        let strings = alloc::vec!["test1", "test2"];
        data.extend((strings.len() as u32).to_le_bytes());
        data.extend(
            strings
                .iter()
                .map(|s| to_serialized_vec(s.as_bytes()))
                .flatten()
                .collect::<Vec<_>>(),
        );
        data.extend(199u128.to_le_bytes());
        attestation.data = data;
        assert!(attestation.validate_data(layout).is_ok());

        // u8
        let layout = alloc::vec![0];
        attestation.data = Vec::new();
        // Should fail when attestion has no data
        assert!(attestation.validate_data(layout).is_err());

        // u16
        let layout = alloc::vec![1];
        attestation.data = Vec::new();
        // Should fail when attestion has no data
        assert!(attestation.validate_data(layout).is_err());
    }
}
