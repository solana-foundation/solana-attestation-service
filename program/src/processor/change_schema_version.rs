use core::marker::PhantomData;

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
        create_pda_account, to_serialized_vec, verify_owner_mutability, verify_signer,
        verify_system_account, verify_system_program,
    },
    state::{discriminator::AccountSerialize, Credential, Schema},
};

#[inline(always)]
pub fn process_change_schema_version(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
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
    credential.verify_pda(credential_info, program_id)?;

    // Verify signer matches credential authority.
    if credential.authority.ne(authority_info.key()) {
        return Err(ProgramError::IncorrectAuthority);
    }

    let existing_schema_data = existing_schema_info.try_borrow_data()?;
    let existing_schema = Schema::try_from_bytes(&existing_schema_data)?;

    let args = ChangeSchemaVersionArgs::try_from_bytes(instruction_data)?;
    let layout = args.layout()?;
    let (field_names_count, field_names_bytes) = args.field_names()?;

    let name = &existing_schema.name;
    let description = existing_schema.description;
    let version = &[existing_schema.version.checked_add(1).unwrap()];

    // NOTE: this could be optimized further by removing the `solana-program` dependency
    // and using `pubkey::checked_create_program_address` from Pinocchio to verify the
    // pubkey and associated bump (needed to be added as arg) is valid.
    let (schema_pda, schema_bump) = SolanaPubkey::find_program_address(
        &[
            SCHEMA_SEED,
            credential_info.key(),
            name.get(4..).unwrap(), // Convert Vec<u8> to UTF8 Array
            version,
        ],
        &SolanaPubkey::from(*program_id),
    );

    if new_schema_info.key() != &schema_pda.to_bytes() {
        // PDA was invalid
        return Err(AttestationServiceError::InvalidCredential.into());
    }

    // Account layout
    // discriminator - 1
    // credential - 32
    // name - length (len header included)
    // description - length (len header included)
    // layout - 4 + length
    // field_names - 4 + length
    // is_paused - 1
    // version - 1
    let space = 1
        + 32
        + (name.len())
        + (description.len())
        + (4 + layout.len())
        + (4 + field_names_bytes.len())
        + 1
        + 1;
    let rent = Rent::get()?;
    let bump_seed = [schema_bump];
    let signer_seeds = [
        Seed::from(SCHEMA_SEED),
        Seed::from(credential_info.key()),
        Seed::from(name.as_slice().get(4..).unwrap()),
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
    )?;

    let schema = Schema {
        credential: *credential_info.key(),
        name: name.to_vec(),
        description,
        layout: to_serialized_vec(layout),
        field_names: to_serialized_vec(field_names_bytes),
        is_paused: false,
        version: version[0],
    };

    // Checks that layout and field names are valid.
    schema.validate(field_names_count)?;

    let mut schema_data = new_schema_info.try_borrow_mut_data()?;
    schema_data.copy_from_slice(&schema.to_bytes());

    Ok(())
}

/// Instruction data for the `CreateSchema` instruction.
pub struct ChangeSchemaVersionArgs<'a> {
    raw: *const u8,

    _data: PhantomData<&'a [u8]>,
}

impl ChangeSchemaVersionArgs<'_> {
    #[inline]
    pub fn try_from_bytes(bytes: &[u8]) -> Result<ChangeSchemaVersionArgs, ProgramError> {
        // The minimum expected size of the instruction data.
        // - layout (5 bytes. 4 len, 1 field)
        // - field_names (5 bytes. 4 len, 1 field)
        if bytes.len() < 10 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(ChangeSchemaVersionArgs {
            raw: bytes.as_ptr(),
            _data: PhantomData,
        })
    }

    #[inline]
    pub fn layout(&self) -> Result<&[u8], ProgramError> {
        // SAFETY: the `bytes` length was validated in `try_from_bytes`.
        unsafe {
            // Len of layout
            let len = *(self.raw as *const u32);
            let layout_bytes = core::slice::from_raw_parts(self.raw.add(4), len as usize);
            Ok(layout_bytes)
        }
    }

    #[inline]
    pub fn field_names(&self) -> Result<(u32, &[u8]), ProgramError> {
        // SAFETY: the `bytes` length was validated in `try_from_bytes`.
        unsafe {
            let layout_len = *(self.raw as *const u32);
            let offset = 4 + layout_len;
            // Number of field names.
            let field_names_count = *(self.raw.add(offset as usize) as *const u32);
            let offset = offset + 4;

            let mut byte_len = 0;
            for _ in 0..field_names_count {
                let len = *(self.raw.add((offset + byte_len) as usize) as *const u32);
                byte_len += 4 + len
            }

            let field_names_bytes =
                core::slice::from_raw_parts(self.raw.add(offset as usize), byte_len as usize);
            Ok((field_names_count, field_names_bytes))
        }
    }
}
