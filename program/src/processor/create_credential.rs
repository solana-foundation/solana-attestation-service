extern crate alloc;

use alloc::vec::Vec;
use pinocchio::{
    cpi::Seed,
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, Address, ProgramResult,
};

use crate::{
    constants::CREDENTIAL_SEED,
    error::AttestationServiceError,
    processor::{create_pda_account, verify_signer, verify_system_account, verify_system_program},
    require_len,
    state::{discriminator::AccountSerialize, Credential},
};

#[inline(always)]
pub fn process_create_credential(
    program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let args = process_instruction_data(instruction_data)?;
    let [payer_info, credential_info, authority_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    verify_system_account(credential_info)?;
    verify_signer(authority_info)?;
    verify_system_program(system_program)?;

    let (credential_pda, credential_bump) =
        Address::find_program_address(&[CREDENTIAL_SEED, authority_info.address().as_ref(), args.name], program_id);

    if credential_info.address().ne(&credential_pda) {
        return Err(AttestationServiceError::InvalidCredential.into());
    }

    let credential =
        Credential { authority: *authority_info.address(), name: args.name.to_vec(), authorized_signers: args.signers };
    let credential_bytes = credential.to_bytes();

    let rent = Rent::get()?;
    let bump_seed = [credential_bump];
    let signer_seeds = [
        Seed::from(CREDENTIAL_SEED),
        Seed::from(authority_info.address().as_ref()),
        Seed::from(args.name),
        Seed::from(&bump_seed),
    ];
    create_pda_account(payer_info, &rent, credential_bytes.len(), program_id, credential_info, signer_seeds, None)?;
    credential_info.try_borrow_mut()?.copy_from_slice(&credential_bytes);

    Ok(())
}

struct CreateCredentialArgs<'a> {
    name: &'a [u8],
    signers: Vec<Address>,
}

fn process_instruction_data(data: &[u8]) -> Result<CreateCredentialArgs<'_>, ProgramError> {
    let mut offset: usize = 0;

    require_len!(data, 4);
    let name_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;

    require_len!(data, offset + name_len);
    let name = &data[offset..offset + name_len];
    offset += name_len;

    require_len!(data, offset + 4);
    let signers_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;

    require_len!(data, offset + signers_len * 32);
    let mut signers = Vec::with_capacity(signers_len);
    for _ in 0..signers_len {
        let signer: Address = data[offset..offset + 32].try_into().unwrap();
        signers.push(signer);
        offset += 32;
    }

    Ok(CreateCredentialArgs { name, signers })
}
