extern crate alloc;

use alloc::vec::Vec;
use core::marker::PhantomData;
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use crate::{
    processor::{verify_owner_mutability, verify_signer, verify_system_program},
    state::{discriminator::AccountSerialize, Credential},
};

#[inline(always)]
pub fn process_change_authorized_signers(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [_payer_info, authority_info, credential_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate: authority should have signed
    verify_signer(authority_info, false)?;
    // Validate: system program
    verify_system_program(system_program)?;
    // Verify program ownership, mutability and PDAs.
    verify_owner_mutability(credential_info, program_id, true)?;

    let data = credential_info.try_borrow_data()?;
    let mut credential = Credential::try_from_bytes(&data)?;
    drop(data); // Drop immutable borrow.

    // Verify PDA and that signer matches credential authority.
    credential.verify_pda(credential_info, program_id)?;
    if credential.authority.ne(authority_info.key()) {
        return Err(ProgramError::IncorrectAuthority);
    }

    // Read new authorized_signers from instruction data.
    let args: ChangeAuthorizedSignersArgs<'_> =
        ChangeAuthorizedSignersArgs::try_from_bytes(instruction_data)?;
    let signers = args.signers()?;

    // Resize account if needed.
    let mut new_space = credential_info.data_len();
    let prev_len = credential.authorized_signers.len();
    let new_len = signers.len();
    if new_len > prev_len {
        new_space += (new_len - prev_len) * 32;
    } else {
        new_space -= (prev_len - new_len) * 32;
    }
    if new_space != credential_info.data_len() {
        credential_info.realloc(new_space, false)?;
    }

    // Update authorized_signers on struct.
    credential.authorized_signers = signers;

    // Write updated data.
    let mut credential_data = credential_info.try_borrow_mut_data()?;
    credential_data.copy_from_slice(&credential.to_bytes());

    Ok(())
}

/// Instruction data for the `ChangeAuthorizedSigners` instruction.
pub struct ChangeAuthorizedSignersArgs<'a> {
    raw: *const u8,

    _data: PhantomData<&'a [u8]>,
}

impl ChangeAuthorizedSignersArgs<'_> {
    #[inline]
    pub fn try_from_bytes(bytes: &[u8]) -> Result<ChangeAuthorizedSignersArgs, ProgramError> {
        // The minimum expected size of the instruction data.
        // - signers (36 bytes. 4 len, 32 pubkey)
        if bytes.len() < 36 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(ChangeAuthorizedSignersArgs {
            raw: bytes.as_ptr(),
            _data: PhantomData,
        })
    }

    #[inline]
    pub fn signers(&self) -> Result<Vec<Pubkey>, ProgramError> {
        // SAFETY: the `bytes` length was validated in `try_from_bytes`.
        unsafe {
            let len = *(self.raw as *const u32);
            let mut offset = 4; // Move past signers length field
            let mut signers = Vec::with_capacity(len as usize);

            for _ in 0..len {
                let signer_ptr = self.raw.add(offset as usize) as *const Pubkey;
                signers.push(*signer_ptr);
                offset += 32;
            }

            Ok(signers)
        }
    }
}
