extern crate alloc;

use alloc::vec::Vec;
use pinocchio::{program_error::ProgramError, pubkey::Pubkey};
use shank::ShankAccount;

use crate::error::AttestationServiceError;

use super::discriminator::{AccountSerialize, AttestationAccountDiscriminators, Discriminator};

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
    // TODO is the signature needed?
    /// Signature from the authorized signer attesting to the data
    // pub signature: [u8; 64],
    /// Designates when the credential is expired. 0 means never expired
    pub expiry: i64,
    /// Whether the attestation has been revoked or not
    pub is_revoked: bool,
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
        data.extend_from_slice(self.data.as_ref());
        data.extend_from_slice(self.signer.as_ref());
        data.extend_from_slice(&self.expiry.to_le_bytes());
        data.extend_from_slice(&[self.is_revoked as u8]);

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
            match data_type {
                // u8 -> u128
                0 => data_offset += 1,
                1 => data_offset += 2,
                2 => data_offset += 4,
                3 => data_offset += 8,
                4 => data_offset += 16,
                // i8 -> i128
                5 => data_offset += 1,
                6 => data_offset += 2,
                7 => data_offset += 4,
                8 => data_offset += 8,
                9 => data_offset += 16,
                // bool
                10 => data_offset += 1,
                // char
                11 => data_offset += 4,
                // String
                12 => data_offset += get_size_of_vec(data_offset, 1, &self.data),
                // Vec<u8> -> Vec<u128>
                13 => data_offset += get_size_of_vec(data_offset, 1, &self.data),
                14 => data_offset += get_size_of_vec(data_offset, 2, &self.data),
                15 => data_offset += get_size_of_vec(data_offset, 4, &self.data),
                16 => data_offset += get_size_of_vec(data_offset, 8, &self.data),
                17 => data_offset += get_size_of_vec(data_offset, 16, &self.data),
                // Vec<i8> -> Vec<i128>
                18 => data_offset += get_size_of_vec(data_offset, 1, &self.data),
                19 => data_offset += get_size_of_vec(data_offset, 2, &self.data),
                20 => data_offset += get_size_of_vec(data_offset, 4, &self.data),
                21 => data_offset += get_size_of_vec(data_offset, 8, &self.data),
                22 => data_offset += get_size_of_vec(data_offset, 16, &self.data),
                // Vec<bool>
                23 => data_offset += get_size_of_vec(data_offset, 1, &self.data),
                // Vec<char>
                24 => data_offset += get_size_of_vec(data_offset, 4, &self.data),
                // Vec<String>
                25 => {
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
            is_revoked: false,
        };

        // u8
        let layout = alloc::vec![0];
        attestation.data = alloc::vec![10];
        assert!(attestation.validate_data(layout).is_ok());

        // u8, Vec<String>, u128
        let layout = alloc::vec![0, 25, 4];
        let mut data: Vec<u8> = Vec::new();
        data.extend([10]);
        let strings = alloc::vec![to_serialized_vec(b"test1"), to_serialized_vec(b"test2")];
        let string_bytes = strings.iter().flatten().collect::<Vec<_>>();
        // [5, 0, 0, 0, 116, 101, 115, 116, 49, 5, 0, 0, 0, 116, 101, 115, 116, 50]
        // data.extend(to_serialized_vec(string_bytes));
        data.extend(199u128.to_le_bytes());
        // core::panic!("string bytes {:?} \n\n data {:?} \n\n", string_bytes, data);
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
