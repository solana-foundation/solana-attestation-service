use pinocchio::{
    account_info::AccountInfo,
    instruction::Seed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use solana_pubkey::Pubkey as SolanaPubkey;

use crate::{
    constants::SCHEMA_SEED,
    error::AttestationServiceError,
    processor::{create_pda_account, verify_signer, verify_system_account, verify_system_program},
    require_len,
    state::{discriminator::AccountSerialize, Credential, Schema},
};

use super::verify_owner_mutability;

#[inline(always)]
pub fn process_create_schema(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let args = process_instruction_data(instruction_data)?;
    let [payer_info, authority_info, credential_info, schema_info, system_program] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate: authority should have signed
    verify_signer(authority_info, false)?;
    // Verify Credential is owned by current program.
    verify_owner_mutability(credential_info, program_id, false)?;
    // Validate: schema should be owned by system account, empty, and writable
    verify_system_account(schema_info, true)?;
    // Validate: system program
    verify_system_program(system_program)?;

    let credential = &Credential::try_from_bytes(&credential_info.try_borrow_data()?)?;
    // Verify signer matches credential authority.
    if credential.authority.ne(authority_info.key()) {
        return Err(ProgramError::IncorrectAuthority);
    }

    // NOTE: this could be optimized further by removing the `solana-program` dependency
    // and using `pubkey::checked_create_program_address` from Pinocchio to verify the
    // pubkey and associated bump (needed to be added as arg) is valid.
    let version = &[1];
    let (schema_pda, schema_bump) = SolanaPubkey::find_program_address(
        &[SCHEMA_SEED, credential_info.key(), args.name, version],
        &SolanaPubkey::from(*program_id),
    );

    if schema_info.key().ne(&schema_pda.to_bytes()) {
        // PDA was invalid
        return Err(AttestationServiceError::InvalidSchema.into());
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
        + (4 + args.name.len())
        + (4 + args.description.len())
        + (4 + args.layout.len())
        + (4 + args.field_names_bytes.len())
        + 1
        + 1;
    let rent = Rent::get()?;
    let bump_seed = [schema_bump];
    let signer_seeds = [
        Seed::from(SCHEMA_SEED),
        Seed::from(credential_info.key()),
        Seed::from(args.name),
        Seed::from(version),
        Seed::from(&bump_seed),
    ];
    create_pda_account(
        payer_info,
        &rent,
        space,
        program_id,
        schema_info,
        signer_seeds,
        None,
    )?;

    let schema = Schema {
        credential: *credential_info.key(),
        name: args.name.to_vec(),
        description: args.description.to_vec(),
        layout: args.layout.to_vec(),
        field_names: args.field_names_bytes.to_vec(),
        is_paused: false,
        version: version[0],
    };

    // Checks that layout and field names are valid.
    schema.validate(args.field_names_count)?;

    let mut schema_data = schema_info.try_borrow_mut_data()?;
    schema_data.copy_from_slice(&schema.to_bytes());

    Ok(())
}

struct CreateSchemaArgs<'a> {
    name: &'a [u8],
    description: &'a [u8],
    layout: &'a [u8],
    field_names_count: u32,
    field_names_bytes: &'a [u8],
}

fn process_instruction_data<'a>(data: &'a [u8]) -> Result<CreateSchemaArgs<'a>, ProgramError> {
    let mut offset: usize = 0;

    require_len!(data, 4);
    let name_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;

    require_len!(data, offset + name_len);
    let name = &data[offset..offset + name_len];
    offset += name_len;

    require_len!(data, offset + 4);
    let desc_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;

    require_len!(data, offset + desc_len);
    let description = &data[offset..offset + desc_len];
    offset += desc_len;

    require_len!(data, offset + 4);
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

    Ok(CreateSchemaArgs {
        name,
        description,
        layout,
        field_names_count,
        field_names_bytes,
    })
}
