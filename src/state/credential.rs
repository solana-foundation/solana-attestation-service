extern crate alloc;

use alloc::vec::Vec;
use pinocchio::pubkey::Pubkey;

// PDA ["credential", authority, name]
/// Tracks the authorized signers of for schemas and their attestations.
#[derive(Clone, Debug, PartialEq)]
pub struct Credential {
  /// Admin of this credential
  pub authority: Pubkey,
  /// UTF-8 encoded Name of this credential
  /// Includes 4 bytes for length of name
  pub name: Vec<u8>,
  /// List of signers that are allowed to "attest"
  pub authorized_signers: Vec<Pubkey>,
}
