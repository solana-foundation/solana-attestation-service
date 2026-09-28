use pinocchio::{
    cpi::Seed,
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, Address, ProgramResult,
};

use crate::{
    constants::SCHEMA_SEED,
    error::AttestationServiceError,
    processor::{
        create_pda_account, verify_owner_mutability, verify_signer, verify_system_account, verify_system_program,
    },
    require_len,
    state::{discriminator::AccountSerialize, Credential, Schema},
};

#[inline(always)]
pub fn process_change_schema_version(
    program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let args = process_instruction_data(instruction_data)?;
    let [payer_info, authority_info, credential_info, existing_schema_info, new_schema_info, system_program] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    verify_signer(authority_info)?;
    verify_system_account(new_schema_info)?;
    verify_system_program(system_program)?;
    verify_owner_mutability(credential_info, program_id, false)?;
    verify_owner_mutability(existing_schema_info, program_id, false)?;

    let credential = Credential::try_from_bytes(&credential_info.try_borrow()?)?;
    credential.validate_authority(authority_info.address())?;

    let existing_schema = Schema::try_from_bytes(&existing_schema_info.try_borrow()?)?;
    if existing_schema.credential.ne(credential_info.address()) {
        return Err(AttestationServiceError::InvalidSchema.into());
    }

    let version = &[existing_schema.version.checked_add(1).ok_or(ProgramError::ArithmeticOverflow)?];

    let (schema_pda, schema_bump) = Address::find_program_address(
        &[SCHEMA_SEED, credential_info.address().as_ref(), existing_schema.name.as_ref(), version],
        program_id,
    );

    if new_schema_info.address().ne(&schema_pda) {
        return Err(AttestationServiceError::InvalidSchema.into());
    }

    let schema = Schema {
        credential: *credential_info.address(),
        name: existing_schema.name,
        description: existing_schema.description,
        layout: args.layout.to_vec(),
        field_names: args.field_names_bytes.to_vec(),
        is_paused: false,
        version: version[0],
    };
    schema.validate(args.field_names_count)?;
    let schema_bytes = schema.to_bytes();

    let rent = Rent::get()?;
    let bump_seed = [schema_bump];
    let signer_seeds = [
        Seed::from(SCHEMA_SEED),
        Seed::from(credential_info.address().as_ref()),
        Seed::from(schema.name.as_slice()),
        Seed::from(version),
        Seed::from(&bump_seed),
    ];
    create_pda_account(payer_info, &rent, schema_bytes.len(), program_id, new_schema_info, signer_seeds, None)?;
    new_schema_info.try_borrow_mut()?.copy_from_slice(&schema_bytes);

    Ok(())
}

struct ChangeSchemaVersionArgs<'a> {
    layout: &'a [u8],
    field_names_count: u32,
    field_names_bytes: &'a [u8],
}

fn process_instruction_data(data: &[u8]) -> Result<ChangeSchemaVersionArgs<'_>, ProgramError> {
    let mut offset: usize = 0;

    require_len!(data, 4);
    let layout_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;

    require_len!(data, offset + layout_len);
    let layout = &data[offset..offset + layout_len];
    offset += layout_len;

    require_len!(data, offset + 4);
    let field_names_count = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
    offset += 4;

    let mut byte_len = 0;
    for _ in 0..field_names_count {
        let start = offset + byte_len;
        let end = start + 4;
        require_len!(data, end);

        let name_len = u32::from_le_bytes(data[start..end].try_into().unwrap()) as usize;
        byte_len += 4 + name_len;
    }

    require_len!(data, offset + byte_len);
    let field_names_bytes = &data[offset..offset + byte_len];

    Ok(ChangeSchemaVersionArgs { layout, field_names_count, field_names_bytes })
}
