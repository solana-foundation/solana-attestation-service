extern crate alloc;

use alloc::vec::Vec;
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::Transfer;

use crate::{
    processor::{verify_owner_mutability, verify_signer, verify_system_program},
    require_len,
    state::{discriminator::AccountSerialize, Credential},
};

#[inline(always)]
pub fn process_change_authorized_signers(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let args = process_instruction_data(instruction_data)?;
    let [payer_info, authority_info, credential_info, system_program] = accounts else {
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

    // Verify that signer matches credential authority.
    if credential.authority.ne(authority_info.key()) {
        return Err(ProgramError::IncorrectAuthority);
    }

    // Resize account if needed.
    let prev_space = credential_info.data_len();
    let mut new_space = prev_space;
    let prev_len = credential.authorized_signers.len();
    let new_len = args.signers.len();
    if new_len > prev_len {
        new_space += (new_len - prev_len) * 32;
    } else {
        new_space -= (prev_len - new_len) * 32;
    }
    if new_space != credential_info.data_len() {
        credential_info.resize(new_space)?;
        let diff = new_space.saturating_sub(prev_space);
        if diff > 0 {
            // top up lamports to account for additional rent.
            let rent = Rent::get()?;
            let min_rent = rent.minimum_balance(new_space);
            let current_rent = credential_info.lamports();
            let rent_diff = min_rent.saturating_sub(current_rent);
            if rent_diff > 0 {
                Transfer {
                    from: payer_info,
                    to: credential_info,
                    lamports: rent_diff,
                }
                .invoke()?;
            }
        }
    }

    // Update authorized_signers on struct.
    credential.authorized_signers = args.signers;

    // Write updated data.
    let mut credential_data = credential_info.try_borrow_mut_data()?;
    credential_data.copy_from_slice(&credential.to_bytes());

    Ok(())
}

struct ChangeAuthorizedSignersArgs {
    signers: Vec<Pubkey>,
}

fn process_instruction_data(data: &[u8]) -> Result<ChangeAuthorizedSignersArgs, ProgramError> {
    let mut offset: usize = 0;

    require_len!(data, 4);
    let signers_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;

    require_len!(data, 4 + signers_len * 32);
    let mut signers = Vec::with_capacity(signers_len);
    for _ in 0..signers_len {
        let signer: Pubkey = data[offset..offset + 32].try_into().unwrap();
        signers.push(signer);
        offset += 32;
    }

    Ok(ChangeAuthorizedSignersArgs { signers })
}
