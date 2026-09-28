extern crate alloc;

use alloc::vec::Vec;
use codama::CodamaEvent;
use pinocchio::Address;

use crate::constants::EVENT_IX_TAG_LE;

#[repr(u8)]
pub enum EventDiscriminators {
    CloseEvent = 0,
}

#[derive(CodamaEvent)]
#[codama(discriminator(bytes = [228, 69, 165, 46, 81, 203, 154, 29], offset = 0))]
pub struct CloseAttestationEvent {
    /// Unique u8 byte for event type.
    pub discriminator: u8,
    /// Reference to the Schema this Attestation adheres to
    pub schema: Address,
    /// Data that was verified and matches the Schema
    #[codama(type = bytes)]
    #[codama(size_prefix = number(u32))]
    pub attestation_data: Vec<u8>,
}

impl CloseAttestationEvent {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        // Prepend IX Discriminator for emit_event.
        data.extend_from_slice(EVENT_IX_TAG_LE);
        data.push(self.discriminator);
        data.extend_from_slice(self.schema.as_ref());
        data.extend_from_slice(&(self.attestation_data.len() as u32).to_le_bytes());
        data.extend_from_slice(&self.attestation_data);

        data
    }
}
