extern crate alloc;

use alloc::vec::Vec;
use pinocchio::pubkey::Pubkey;
use shank::ShankType;

use crate::constants::EVENT_IX_TAG_LE;

#[repr(u8)]
pub enum EventDiscriminators {
    CloseEvent = 0,
    CompressEvent = 1,
}

#[derive(ShankType)]
pub struct CloseAttestationEvent {
    /// Unique u8 byte for event type.
    pub discriminator: u8,
    /// Reference to the Schema this Attestation adheres to
    pub schema: Pubkey,
    /// Data that was verified and matches the Schema
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

#[derive(ShankType, Clone, PartialEq, Eq, Debug)]
pub struct CompressAttestation {
    /// Reference to the Schema this Attestation adheres to
    pub schema: Pubkey,
    /// Data that was verified and matches the Schema
    pub attestation_data: Vec<u8>,
}

#[derive(ShankType, Clone, PartialEq, Eq, Debug)]
pub struct CompressAttestationEvent {
    /// Unique u8 byte for event type.
    pub discriminator: u8,
    /// Whether the source PDA accounts were closed after compression
    pub pdas_closed: bool,
    /// Attestations that were compressed
    pub attestations: Vec<CompressAttestation>,
}

impl CompressAttestationEvent {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        // Prepend IX Discriminator for emit_event.
        data.extend_from_slice(EVENT_IX_TAG_LE);
        data.push(self.discriminator);

        // Encode pdas_closed flag
        data.push(if self.pdas_closed { 1 } else { 0 });

        // Encode attestations array
        data.extend_from_slice(&(self.attestations.len() as u32).to_le_bytes());
        for attestation in &self.attestations {
            // Schema pubkey (32 bytes)
            data.extend_from_slice(attestation.schema.as_ref());
            // Attestation data length + data
            data.extend_from_slice(&(attestation.attestation_data.len() as u32).to_le_bytes());
            data.extend_from_slice(&attestation.attestation_data);
        }

        data
    }
}
