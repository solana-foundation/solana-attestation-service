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
        create_pda_account, to_serialized_vec, verify_signer, verify_system_account,
        verify_system_program,
    },
    state::{Credential, Schema},
};

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
    let data_schema = args.data()?;

    // NOTE: this could be optimized further by removing the `solana-program` dependency
    // and using `pubkey::checked_create_program_address` from Pinocchio to verify the
    // pubkey and associated bump (needed to be added as arg) is valid.
    let (schema_pda, schema_bump) = SolanaPubkey::find_program_address(
        &[SCHEMA_SEED, credential_info.key(), name],
        &SolanaPubkey::from(*program_id),
    );

    if schema_info.key() != &schema_pda.to_bytes() {
        // PDA was invalid
        return Err(AttestationServiceError::InvalidCredential.into());
    }

    // Account layout
    // discriminator - 1
    // credential - 32
    // name - 4 + length
    // description - 4 + length
    // data_schema - 4 + length
    // is_revoked - 1
    let space = 1 + 32 + (4 + name.len()) + (4 + description.len()) + (4 + data_schema.len()) + 1;
    let rent = Rent::get()?;
    let bump_seed = [schema_bump];
    let signer_seeds = [
        Seed::from(SCHEMA_SEED),
        Seed::from(credential_info.key()),
        Seed::from(name),
        Seed::from(&bump_seed),
    ];
    create_pda_account(
        payer_info,
        &rent,
        space,
        program_id,
        schema_info,
        signer_seeds,
    )?;

    let schema = Schema {
        credential: *credential_info.key(),
        name: to_serialized_vec(name),
        description: to_serialized_vec(description),
        data_schema: to_serialized_vec(data_schema),
        is_paused: false,
    };
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
        // The minimum expected size of the instruction data.
        // - name (5 bytes. 4 len, 1 char)
        // - description (5 bytes. 4 len, 1 char)
        // - data (5 bytes. 4 len, 1 field)
        if bytes.len() < 15 {
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
    pub fn data(&self) -> Result<&[u8], ProgramError> {
        // SAFETY: the `bytes` length was validated in `try_from_bytes`.
        unsafe {
            // name length + 4 for encoded len
            let offset = *(self.raw as *const u32) + 4;
            let description_len = *(self.raw.add(offset as usize) as *const u32);
            // offset for start of the data
            let offset = offset + 4 + description_len;
            // Len of data
            let len = *(self.raw.add(offset as usize) as *const u32);
            let offset = offset + 4;
            let data_bytes =
                core::slice::from_raw_parts(self.raw.add(offset as usize), len as usize);
            Ok(data_bytes)
        }
    }
}
