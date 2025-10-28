//! Program constants for Solana Attestation Service.
//!
//! These constants are not auto-generated and should be maintained manually.

use solana_program::pubkey::Pubkey;

/// The default allowed address tree for compressed attestations.
/// This is the address merkle tree that is used for address validation.
pub const ALLOWED_ADDRESS_TREE: Pubkey =
    solana_program::pubkey!("amt2kaJA14v3urZbZvnc5v2np8jqvc4Z8zDep5wbtzx");
