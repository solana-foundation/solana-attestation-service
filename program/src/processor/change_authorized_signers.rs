extern crate alloc;

use alloc::vec::Vec;
use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

use crate::{
    processor::{resize_account, verify_owner_mutability, verify_signer, verify_system_program},
    require_len,
    state::{discriminator::AccountSerialize, Credential},
};

#[inline(always)]
pub fn process_change_authorized_signers(
    program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let args = process_instruction_data(instruction_data)?;
    let [payer_info, authority_info, credential_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    verify_signer(authority_info)?;
    verify_system_program(system_program)?;
    verify_owner_mutability(credential_info, program_id, true)?;

    let mut credential = Credential::try_from_bytes(&credential_info.try_borrow()?)?;
    credential.validate_authority(authority_info.address())?;

    credential.authorized_signers = args.signers;
    let credential_bytes = credential.to_bytes();
    resize_account(credential_info, payer_info, credential_bytes.len())?;
    credential_info.try_borrow_mut()?.copy_from_slice(&credential_bytes);

    Ok(())
}

struct ChangeAuthorizedSignersArgs {
    signers: Vec<Address>,
}

fn process_instruction_data(data: &[u8]) -> Result<ChangeAuthorizedSignersArgs, ProgramError> {
    let mut offset: usize = 0;

    require_len!(data, 4);
    let signers_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;

    require_len!(data, 4 + signers_len * 32);
    let mut signers = Vec::with_capacity(signers_len);
    for _ in 0..signers_len {
        let signer: Address = data[offset..offset + 32].try_into().unwrap();
        signers.push(signer);
        offset += 32;
    }

    Ok(ChangeAuthorizedSignersArgs { signers })
}
