extern crate alloc;

use alloc::vec::Vec;
use codama::CodamaAccount;
use pinocchio::{error::ProgramError, Address};
use pinocchio_log::log;

use crate::error::AttestationServiceError;

use super::{
    discriminator::{AccountSerialize, AttestationAccountDiscriminators, Discriminator},
    SchemaDataTypes,
};

#[derive(Clone, Debug, PartialEq, CodamaAccount)]
#[codama(seed(type = string(utf8), value = "attestation"))]
#[codama(seed(name = "credential", type = public_key))]
#[codama(seed(name = "schema", type = public_key))]
#[codama(seed(name = "nonce", type = public_key))]
pub struct Attestation {
    /// A pubkey that may either be randomly generated OR associated with a User's wallet
    pub nonce: Address,
    /// Credential this attestation is related to
    pub credential: Address,
    /// Reference to the Schema this Attestation adheres to
    pub schema: Address,
    /// Data that was verified and matches the Schema
    #[codama(type = bytes)]
    #[codama(size_prefix = number(u32))]
    pub data: Vec<u8>,
    /// The pubkey of the signer. Must be one of the `authorized_signer`s at time of attestation
    pub signer: Address,
    /// Designates when the credential is expired. 0 means never expired
    pub expiry: i64,
    /// The pubkey of Attestation token account if created. Otherwise set to default pubkey.
    pub token_account: Address,
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
        data.extend_from_slice(self.token_account.as_ref());

        data
    }
}

fn invalid_data() -> ProgramError {
    AttestationServiceError::InvalidAttestationData.into()
}

fn read_len(data: &[u8], offset: usize) -> Result<usize, ProgramError> {
    let end = offset.checked_add(4).ok_or_else(invalid_data)?;
    let bytes = data.get(offset..end).ok_or_else(invalid_data)?;
    Ok(u32::from_le_bytes(bytes.try_into().unwrap()) as usize)
}

fn vec_size(data: &[u8], offset: usize, element_size: usize) -> Result<usize, ProgramError> {
    read_len(data, offset)?.checked_mul(element_size).and_then(|len| len.checked_add(4)).ok_or_else(invalid_data)
}

impl Attestation {
    /// Validate the data in the Attestation conforms to the Schema's
    /// layout.
    pub fn validate_data(&self, layout: &[u8]) -> Result<(), ProgramError> {
        let data = self.data.as_slice();
        let mut offset: usize = 0;
        for &data_type in layout {
            let size = match SchemaDataTypes::from(data_type) {
                SchemaDataTypes::U8 | SchemaDataTypes::I8 | SchemaDataTypes::Bool => 1,
                SchemaDataTypes::U16 | SchemaDataTypes::I16 => 2,
                SchemaDataTypes::U32 | SchemaDataTypes::I32 | SchemaDataTypes::Char => 4,
                SchemaDataTypes::U64 | SchemaDataTypes::I64 => 8,
                SchemaDataTypes::U128 | SchemaDataTypes::I128 => 16,
                SchemaDataTypes::String
                | SchemaDataTypes::VecU8
                | SchemaDataTypes::VecI8
                | SchemaDataTypes::VecBool => vec_size(data, offset, 1)?,
                SchemaDataTypes::VecU16 | SchemaDataTypes::VecI16 => vec_size(data, offset, 2)?,
                SchemaDataTypes::VecU32 | SchemaDataTypes::VecI32 | SchemaDataTypes::VecChar => {
                    vec_size(data, offset, 4)?
                }
                SchemaDataTypes::VecU64 | SchemaDataTypes::VecI64 => vec_size(data, offset, 8)?,
                SchemaDataTypes::VecU128 | SchemaDataTypes::VecI128 => vec_size(data, offset, 16)?,
                SchemaDataTypes::VecString => {
                    let count = read_len(data, offset)?;
                    let mut size: usize = 4;
                    for _ in 0..count {
                        let string_start = offset.checked_add(size).ok_or_else(invalid_data)?;
                        let string_len = read_len(data, string_start)?;
                        size = size.checked_add(4).and_then(|s| s.checked_add(string_len)).ok_or_else(invalid_data)?;
                    }
                    size
                }
            };
            offset = offset.checked_add(size).filter(|&end| end <= data.len()).ok_or_else(invalid_data)?;
        }
        if offset != data.len() {
            return Err(invalid_data());
        }
        Ok(())
    }

    pub fn try_from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        // Check discriminator
        if data[0] != Self::DISCRIMINATOR {
            log!("Invalid Attestation Data");
            return Err(ProgramError::InvalidAccountData);
        }

        // Start offset after Discriminator
        let mut offset: usize = 1;

        let nonce: Address = data[offset..offset + 32].try_into().unwrap();
        offset += 32;

        let credential: Address = data[offset..offset + 32].try_into().unwrap();
        offset += 32;

        let schema: Address = data[offset..offset + 32].try_into().unwrap();
        offset += 32;

        let data_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let attestation_data = data[offset..offset + data_len].to_vec();
        offset += data_len;

        let signer: Address = data[offset..offset + 32].try_into().unwrap();
        offset += 32;

        let expiry = i64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
        offset += 8;

        let token_account: Address = data[offset..offset + 32].try_into().unwrap();

        Ok(Self { nonce, credential, schema, data: attestation_data, signer, expiry, token_account })
    }
}

#[cfg(test)]
mod tests {
    use crate::processor::to_serialized_vec;

    use super::*;

    #[test]
    fn attestation_validate_data() {
        let mut attestation = Attestation {
            nonce: Address::default(),
            credential: Address::default(),
            schema: Address::default(),
            data: Vec::new(),
            signer: Address::default(),
            expiry: 0,
            token_account: Address::default(),
        };

        // u8
        let layout = alloc::vec![0];
        attestation.data = alloc::vec![10];
        assert!(attestation.validate_data(&layout).is_ok());

        // u8, Vec<String>, u128
        let layout = alloc::vec![0, 25, 4];
        let mut data: Vec<u8> = Vec::new();
        data.extend([10]);
        let strings = alloc::vec!["test1", "test2"];
        data.extend((strings.len() as u32).to_le_bytes());
        data.extend(strings.iter().flat_map(|s| to_serialized_vec(s.as_bytes())).collect::<Vec<_>>());
        data.extend(199u128.to_le_bytes());
        attestation.data = data;
        assert!(attestation.validate_data(&layout).is_ok());

        // u8
        let layout = alloc::vec![0];
        attestation.data = Vec::new();
        // Should fail when attestion has no data
        assert!(attestation.validate_data(&layout).is_err());

        // u16
        let layout = alloc::vec![1];
        attestation.data = Vec::new();
        // Should fail when attestion has no data
        assert!(attestation.validate_data(&layout).is_err());

        for layout in [alloc::vec![12], alloc::vec![25], alloc::vec![14]] {
            attestation.data = alloc::vec![1, 0];
            assert!(attestation.validate_data(&layout).is_err());
        }

        let layout = alloc::vec![25];
        attestation.data = [2u32.to_le_bytes(), 0u32.to_le_bytes()].concat();
        assert!(attestation.validate_data(&layout).is_err());

        let layout = alloc::vec![15];
        attestation.data = u32::MAX.to_le_bytes().to_vec();
        assert!(attestation.validate_data(&layout).is_err());
    }
}
