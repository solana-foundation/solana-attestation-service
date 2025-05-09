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
    processor::{create_pda_account, verify_signer, verify_system_account, verify_system_program},
    state::{discriminator::AccountSerialize, Credential, Schema},
};

use super::verify_owner_mutability;

#[inline(always)]
pub fn process_create_schema(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
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

    let args = CreateSchemaArgs::try_from_bytes(instruction_data)?;
    let name = args.name()?;
    let description = args.description()?;
    let layout = args.layout()?;
    let (field_names_count, field_names_bytes) = args.field_names()?;
    let version = &[1];

    // NOTE: this could be optimized further by removing the `solana-program` dependency
    // and using `pubkey::checked_create_program_address` from Pinocchio to verify the
    // pubkey and associated bump (needed to be added as arg) is valid.
    let (schema_pda, schema_bump) = SolanaPubkey::find_program_address(
        &[SCHEMA_SEED, credential_info.key(), name, version],
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
        + (4 + name.len())
        + (4 + description.len())
        + (4 + layout.len())
        + (4 + field_names_bytes.len())
        + 1
        + 1;
    let rent = Rent::get()?;
    let bump_seed = [schema_bump];
    let signer_seeds = [
        Seed::from(SCHEMA_SEED),
        Seed::from(credential_info.key()),
        Seed::from(name),
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
        name: name.to_vec(),
        description: description.to_vec(),
        layout: layout.to_vec(),
        field_names: field_names_bytes.to_vec(),
        is_paused: false,
        version: 1,
    };

    // Checks that layout and field names are valid.
    schema.validate(field_names_count)?;

    let mut schema_data = schema_info.try_borrow_mut_data()?;
    schema_data.copy_from_slice(&schema.to_bytes());

    Ok(())
}

/// Instruction data for the `CreateSchema` instruction.
pub struct CreateSchemaArgs<'a> {
    raw: *const u8,

    _data: PhantomData<&'a [u8]>,
}

impl CreateSchemaArgs<'_> {
    #[inline]
    pub fn try_from_bytes(bytes: &[u8]) -> Result<CreateSchemaArgs, ProgramError> {
        // TODO: KIV for further refactoring to enforce better byte len checks.
        // The minimum expected size of the instruction data.
        // - name (5 bytes. 4 len, 1 char)
        // - description (5 bytes. 4 len, 1 char)
        // - layout (5 bytes. 4 len, 1 field)
        // - field_names (5 bytes. 4 len, 1 field)
        if bytes.len() < 20 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(CreateSchemaArgs {
            raw: bytes.as_ptr(),
            _data: PhantomData,
        })
    }

    #[inline]
    pub fn name(&self) -> Result<&[u8], ProgramError> {
        // SAFETY: the `bytes` length was validated in `try_from_bytes`.
        unsafe {
            let len = *(self.raw as *const u32);
            let name_bytes = core::slice::from_raw_parts(self.raw.add(4), len as usize);
            Ok(name_bytes)
        }
    }

    #[inline]
    pub fn description(&self) -> Result<&[u8], ProgramError> {
        // SAFETY: the `bytes` length was validated in `try_from_bytes`.
        unsafe {
            // name length + 4 for encoded len
            let offset = *(self.raw as *const u32) + 4;
            // Len of description
            let len = *(self.raw.add(offset as usize) as *const u32);
            let offset = offset + 4;
            let description_bytes =
                core::slice::from_raw_parts(self.raw.add(offset as usize), len as usize);
            Ok(description_bytes)
        }
    }

    #[inline]
    pub fn layout(&self) -> Result<&[u8], ProgramError> {
        // SAFETY: the `bytes` length was validated in `try_from_bytes`.
        unsafe {
            // name length + 4 for encoded len
            let offset = *(self.raw as *const u32) + 4;
            let description_len = *(self.raw.add(offset as usize) as *const u32);
            // offset for start of the layout
            let offset = offset + 4 + description_len;
            // Len of layout
            let len = *(self.raw.add(offset as usize) as *const u32);
            let offset = offset + 4;
            let layout_bytes =
                core::slice::from_raw_parts(self.raw.add(offset as usize), len as usize);
            Ok(layout_bytes)
        }
    }

    #[inline]
    pub fn field_names(&self) -> Result<(u32, &[u8]), ProgramError> {
        // SAFETY: the `bytes` length was validated in `try_from_bytes`.
        unsafe {
            // name length + 4 for encoded len
            let offset = *(self.raw as *const u32) + 4;
            let description_len = *(self.raw.add(offset as usize) as *const u32);
            let offset = offset + 4 + description_len;
            let layout_len = *(self.raw.add(offset as usize) as *const u32);
            let offset = offset + 4 + layout_len;
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
