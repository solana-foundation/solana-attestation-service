//! Shared utilities for compressed attestation operations

use crate::{
    constants::ALLOWED_ADDRESS_TREE,
    error::AttestationServiceError,
    state::{discriminator::AccountSerialize, Attestation},
};
use light_sdk_pinocchio::cpi::v2::{CompressedAccountInfo, CpiAccounts, OutAccountInfo};
use pinocchio::program_error::ProgramError;

/// Verify that the address tree matches the allowed tree for compressed attestations.
/// Returns the address tree pubkey if valid.
#[inline(always)]
pub fn verify_address_tree(light_cpi_accounts: &CpiAccounts) -> Result<[u8; 32], ProgramError> {
    let address_tree = light_cpi_accounts
        .tree_pubkeys()
        .map_err(|e| ProgramError::Custom(u32::from(e)))?[1];

    // Check that all compressed attestations are in the same address tree
    // to ensure compressed pda uniqueness.
    if address_tree != ALLOWED_ADDRESS_TREE {
        return Err(AttestationServiceError::InvalidAddressTree.into());
    }

    Ok(address_tree)
}

/// Create a CompressedAccountInfo from an Attestation for output (creation).
///
/// # Arguments
/// * `attestation` - The attestation to compress
/// * `address` - The derived compressed address
/// * `output_merkle_tree_index` - Index of the output merkle tree (typically 0)
#[inline(always)]
pub fn create_compressed_account_from_attestation(
    attestation: &Attestation,
    address: [u8; 32],
    output_merkle_tree_index: u8,
) -> CompressedAccountInfo {
    let data_hash = attestation.hash();
    let data = attestation.to_bytes();

    CompressedAccountInfo {
        address: Some(address),
        input: None,
        output: Some(OutAccountInfo {
            output_merkle_tree_index,
            discriminator: Attestation::COMPRESSION_DISCRIMINATOR,
            lamports: 0,
            data,
            data_hash,
        }),
    }
}
