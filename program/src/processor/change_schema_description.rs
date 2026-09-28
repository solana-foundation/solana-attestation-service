extern crate alloc;

use alloc::vec::Vec;
use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

use crate::{
    error::AttestationServiceError,
    processor::{resize_account, verify_owner_mutability, verify_signer, verify_system_program},
    require_len,
    state::{discriminator::AccountSerialize, Credential, Schema},
};

#[inline(always)]
pub fn process_change_schema_description(
    program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let args = process_instruction_data(instruction_data)?;
    let [payer_info, authority_info, credential_info, schema_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    verify_signer(authority_info)?;
    verify_owner_mutability(credential_info, program_id, false)?;
    verify_owner_mutability(schema_info, program_id, true)?;
    verify_system_program(system_program)?;

    let credential = Credential::try_from_bytes(&credential_info.try_borrow()?)?;
    credential.validate_authority(authority_info.address())?;

    let mut schema = Schema::try_from_bytes(&schema_info.try_borrow()?)?;
    if schema.credential.ne(credential_info.address()) {
        return Err(AttestationServiceError::InvalidSchema.into());
    }

    schema.description = args.description;
    let schema_bytes = schema.to_bytes();
    resize_account(schema_info, payer_info, schema_bytes.len())?;
    schema_info.try_borrow_mut()?.copy_from_slice(&schema_bytes);

    Ok(())
}

struct ChangeSchemaDescriptionArgs {
    description: Vec<u8>,
}

fn process_instruction_data(data: &[u8]) -> Result<ChangeSchemaDescriptionArgs, ProgramError> {
    let mut offset: usize = 0;

    require_len!(data, 4);
    let desc_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;

    require_len!(data, offset + desc_len);
    let description = data[offset..offset + desc_len].to_vec();

    Ok(ChangeSchemaDescriptionArgs { description })
}
