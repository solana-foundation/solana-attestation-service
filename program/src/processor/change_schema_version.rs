use pinocchio::{
    account_info::AccountInfo,
    instruction::Seed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use solana_program::pubkey::Pubkey as SolanaPubkey;

use crate::{
    constants::SCHEMA_SEED,
    error::AttestationServiceError,
    processor::{
        create_pda_account, verify_owner_mutability, verify_signer, verify_system_account,
        verify_system_program,
    },
    require_len,
    state::{discriminator::AccountSerialize, Credential, Schema},
};

#[inline(always)]
pub fn process_change_schema_version(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let args = process_instruction_data(instruction_data)?;
    let [payer_info, authority_info, credential_info, existing_schema_info, new_schema_info, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate: authority should have signed
    verify_signer(authority_info, false)?;
    // Validate: schema should be owned by system account, empty, and writable
    verify_system_account(new_schema_info, true)?;
    // Validate: system program
    verify_system_program(system_program)?;
    // Verify program ownership, mutability and PDAs.
    verify_owner_mutability(credential_info, program_id, false)?;
    verify_owner_mutability(existing_schema_info, program_id, false)?;

    let credential = &Credential::try_from_bytes(&credential_info.try_borrow_data()?)?;

    // Verify signer matches credential authority.
    if credential.authority.ne(authority_info.key()) {
        return Err(ProgramError::IncorrectAuthority);
    }

    let existing_schema_data = existing_schema_info.try_borrow_data()?;
    let existing_schema = Schema::try_from_bytes(&existing_schema_data)?;

    // Verify that existing schema is under the same credential.
    if existing_schema.credential.ne(credential_info.key()) {
        return Err(AttestationServiceError::InvalidSchema.into());
    }

    let name = &existing_schema.name;
    let description = existing_schema.description;
    let version = &[existing_schema.version.checked_add(1).unwrap()];

    // NOTE: this could be optimized further by removing the `solana-program` dependency
    // and using `pubkey::checked_create_program_address` from Pinocchio to verify the
    // pubkey and associated bump (needed to be added as arg) is valid.
    let (schema_pda, schema_bump) = SolanaPubkey::find_program_address(
        &[SCHEMA_SEED, credential_info.key(), name.as_ref(), version],
        &SolanaPubkey::from(*program_id),
    );

    if new_schema_info.key().ne(&schema_pda.to_bytes()) {
        // PDA was invalid
        return Err(AttestationServiceError::InvalidCredential.into());
    }

    // Account layout
    // discriminator - 1
    // credential - 32
    // name - 4 + length
    // description - 4 + length
    // layout - 4 + length
    // field_names - 4 + length
    // is_paused - 1
    // version - 1
    let space = 1
        + 32
        + (4 + name.len())
        + (4 + description.len())
        + (4 + args.layout.len())
        + (4 + args.field_names_bytes.len())
        + 1
        + 1;
    let rent = Rent::get()?;
    let bump_seed = [schema_bump];
    let signer_seeds = [
        Seed::from(SCHEMA_SEED),
        Seed::from(credential_info.key()),
        Seed::from(name.as_slice()),
        Seed::from(version),
        Seed::from(&bump_seed),
    ];
    create_pda_account(
        payer_info,
        &rent,
        space,
        program_id,
        new_schema_info,
        signer_seeds,
        None,
    )?;

    let schema = Schema {
        credential: *credential_info.key(),
        name: name.to_vec(),
        description,
        layout: args.layout.to_vec(),
        field_names: args.field_names_bytes.to_vec(),
        is_paused: false,
        version: version[0],
    };

    // Checks that layout and field names are valid.
    schema.validate(args.field_names_count)?;

    let mut schema_data = new_schema_info.try_borrow_mut_data()?;
    schema_data.copy_from_slice(&schema.to_bytes());

    Ok(())
}

struct ChangeSchemaVersionArgs<'a> {
    layout: &'a [u8],
    field_names_count: u32,
    field_names_bytes: &'a [u8],
}

fn process_instruction_data<'a>(
    data: &'a [u8],
) -> Result<ChangeSchemaVersionArgs<'a>, ProgramError> {
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

    Ok(ChangeSchemaVersionArgs {
        layout,
        field_names_count,
        field_names_bytes,
    })
}
