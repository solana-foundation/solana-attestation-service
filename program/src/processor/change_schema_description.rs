use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use crate::{
    error::AttestationServiceError,
    processor::{verify_owner_mutability, verify_signer, verify_system_program},
    state::{discriminator::AccountSerialize, Credential, Schema},
};

#[inline(always)]
pub fn process_change_schema_description(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [authority_info, credential_info, schema_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate: authority should have signed
    verify_signer(authority_info, false)?;

    // Verify program ownership, mutability and PDAs.
    verify_owner_mutability(credential_info, program_id, false)?;
    verify_owner_mutability(schema_info, program_id, true)?;
    verify_system_program(system_program)?;

    let credential = &Credential::try_from_bytes(&credential_info.try_borrow_data()?)?;
    credential.verify_pda(credential_info, program_id)?;

    // Verify signer matches credential authority.
    if credential.authority.ne(authority_info.key()) {
        return Err(ProgramError::IncorrectAuthority);
    }

    let schema_data = schema_info.try_borrow_data()?;
    let mut schema = Schema::try_from_bytes(&schema_data)?;
    drop(schema_data); // Drop immutable borrow.

    schema.verify_pda(schema_info, program_id)?;

    // Verify that schema is under the same credential.
    if schema.credential.ne(credential_info.key()) {
        return Err(AttestationServiceError::InvalidSchema.into());
    }

    let prev_description_len = schema.description.len();

    // Get and update description on struct.
    let data_len = u32::from_le_bytes(instruction_data[0..4].try_into().unwrap()) as usize;
    schema.description = instruction_data[4..4 + data_len].to_vec();

    // Resize account if needed.
    let new_description_len = schema.description.len();
    if new_description_len != prev_description_len {
        let new_space = schema_info.data_len() + new_description_len - prev_description_len;
        schema_info.realloc(new_space, false)?;
    }

    // Write updated data.
    let mut schema_data = schema_info.try_borrow_mut_data()?;
    schema_data.copy_from_slice(&schema.to_bytes());

    Ok(())
}
