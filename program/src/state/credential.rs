extern crate alloc;

use alloc::vec::Vec;
use pinocchio::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};
use pinocchio_log::log;
use shank::ShankAccount;
use solana_program::pubkey::Pubkey as SolanaPubkey;

use crate::{acc_info_as_str, constants::CREDENTIAL_SEED, error::AttestationServiceError};

use super::discriminator::{AccountSerialize, AttestationAccountDiscriminators, Discriminator};

// PDA ["credential", authority, name]
/// Tracks the authorized signers of for schemas and their attestations.
#[derive(Clone, Debug, PartialEq, ShankAccount)]
#[repr(C)]
pub struct Credential {
    /// Admin of this credential
    pub authority: Pubkey,
    /// UTF-8 encoded Name of this credential
    /// Includes 4 bytes for length of name
    pub name: Vec<u8>,
    /// List of signers that are allowed to "attest"
    pub authorized_signers: Vec<Pubkey>,
}

impl Discriminator for Credential {
    const DISCRIMINATOR: u8 = AttestationAccountDiscriminators::CredentialDiscriminator as u8;
}

impl AccountSerialize for Credential {
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::new();
        // Authority encoding
        data.extend_from_slice(self.authority.as_ref());

        // Name encoding
        data.extend_from_slice(self.name.as_ref());

        // Authorized signers encoding
        data.extend_from_slice(&(self.authorized_signers.len() as u32).to_le_bytes());
        for signer in &self.authorized_signers {
            data.extend_from_slice(signer.as_ref());
        }

        data
    }
}

impl Credential {
    pub fn verify_pda(
        &self,
        acc_info: &AccountInfo,
        program_id: &Pubkey,
    ) -> Result<(), ProgramError> {
        let (credential_pda, _credential_bump) = SolanaPubkey::find_program_address(
            &[
                CREDENTIAL_SEED,
                self.authority.as_ref(),
                self.name.get(4..).unwrap(), // Convert Vec<u8> to UTF8 Array
            ],
            &SolanaPubkey::from(*program_id),
        );
        if acc_info.key().ne(&credential_pda.to_bytes()) {
            log!("PDA Mismatch for {}", acc_info_as_str!(acc_info));
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }

    pub fn validate_authority(&self, authority: &Pubkey) -> Result<(), ProgramError> {
        if self.authority.ne(authority) {
            log!("Authority Mismatch");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }

    /// Validate the signer is one of the authorized signers.
    pub fn validate_authorized_signer(&self, signer: &Pubkey) -> Result<(), ProgramError> {
        if !self.authorized_signers.contains(signer) {
            return Err(AttestationServiceError::SignerNotAuthorized.into());
        }
        Ok(())
    }

    pub fn try_from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        // Check discriminator
        if data[0] != Self::DISCRIMINATOR {
            msg!("Invalid Credential Data");
            return Err(ProgramError::InvalidAccountData);
        }

        // Start offset after Discriminator
        let mut offset: usize = 1;

        let authority: Pubkey = data[offset..offset + 32].try_into().unwrap();
        offset += 32;

        let name_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        let name = data[offset..(offset + 4 + name_len)].to_vec();
        offset += 4 + name_len;

        let signers_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        let mut authorized_signers: Vec<Pubkey> = Vec::new();

        offset += 4;
        for _ in 0..signers_len {
            let signer: Pubkey = data[offset..offset + 32].try_into().unwrap();
            authorized_signers.push(signer);
            offset += 32;
        }

        Ok(Self {
            authority,
            name,
            authorized_signers,
        })
    }
}
