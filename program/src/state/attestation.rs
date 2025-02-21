extern crate alloc;

use alloc::vec::Vec;
use pinocchio::pubkey::Pubkey;
use shank::ShankAccount;

// PDA ["attestation", credential, signer, schema]
#[derive(Clone, Debug, PartialEq, ShankAccount)]
#[repr(C)]
pub struct Attestation {
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

// TODO add discriminator for Attestation
