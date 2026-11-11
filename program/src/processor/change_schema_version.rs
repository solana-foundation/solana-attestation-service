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
        create_pda_account, parse_layout_args, verify_owner_mutability, verify_signer, verify_system_account,
        verify_system_program,
    },
    state::{discriminator::AccountSerialize, Credential, Schema},
};

#[inline(always)]
pub fn process_change_schema_version(
    program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let args = parse_layout_args(instruction_data, 0)?;
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
