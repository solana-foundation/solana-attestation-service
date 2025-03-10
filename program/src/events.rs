extern crate alloc;

use crate::state::discriminator::{AccountSerialize, Discriminator};
use alloc::vec::Vec;
use pinocchio::pubkey::Pubkey;
use shank::ShankAccount;

#[repr(u8)]
pub enum EventDiscriminators {
    CloseEvent = 0,
}

#[derive(ShankAccount)]
pub struct CloseAttestationEvent {
    /// Reference to the Schema this Attestation adheres to
    pub schema: Pubkey,
    /// Data that was verified and matches the Schema
    pub attestation_data: Vec<u8>,
}

impl Discriminator for CloseAttestationEvent {
    const DISCRIMINATOR: u8 = EventDiscriminators::CloseEvent as u8;
}

impl AccountSerialize for CloseAttestationEvent {
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.schema.as_ref());
        data.extend_from_slice(&(self.attestation_data.len() as u32).to_le_bytes());
        data.extend_from_slice(&self.attestation_data);

        data
    }
}
