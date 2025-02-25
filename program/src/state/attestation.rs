extern crate alloc;

use alloc::vec::Vec;
use pinocchio::pubkey::Pubkey;
use shank::ShankAccount;

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
