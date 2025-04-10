use core::marker::PhantomData;

use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_token::{
    extensions::{
        group_pointer::Initialize as InitializeGroupPointer, token_group::InitializeGroup,
    },
    instructions::{InitializeMint2, TokenProgramVariant},
    TOKEN_2022_PROGRAM_ID,
};
use solana_program::pubkey::Pubkey as SolanaPubkey;

use crate::{
    constants::{SAS_SEED, SCHEMA_MINT_SEED},
    error::AttestationServiceError,
    processor::{create_pda_account, verify_signer, verify_system_program},
    state::{Credential, Schema},
};

use super::verify_owner_mutability;

#[inline(always)]
pub fn process_tokenize_schema(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [payer_info, authority_info, credential_info, schema_info, mint_info, sas_pda_info, system_program, _token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate: authority should have signed
    verify_signer(authority_info, false)?;
    // Validate Credential and Schema are owned by our program
    verify_owner_mutability(credential_info, program_id, false)?;
    verify_owner_mutability(schema_info, program_id, false)?;
    // Validate: system program
    verify_system_program(system_program)?;

    // Verify signer matches credential authority.
    let credential = &Credential::try_from_bytes(&credential_info.try_borrow_data()?)?;
    if credential.authority.ne(authority_info.key()) {
        return Err(ProgramError::IncorrectAuthority);
    }

    // Validate Schema is owned by Credential
    let schema = Schema::try_from_bytes(&schema_info.try_borrow_data()?)?;
    if schema.credential.ne(credential_info.key()) {
        return Err(AttestationServiceError::InvalidCredential.into());
    }

    // Validate that mint to initialize matches expected PDA
    let (mint_pda, mint_bump) = SolanaPubkey::find_program_address(
        &[SCHEMA_MINT_SEED, schema_info.key()],
        &SolanaPubkey::from(*program_id),
    );
    if mint_info.key().ne(&mint_pda.to_bytes()) {
        return Err(AttestationServiceError::InvalidMint.into());
    }

    // Validate that sas_pda matches
    let (sas_pda, sas_bump) =
        SolanaPubkey::find_program_address(&[SAS_SEED], &SolanaPubkey::from(*program_id));
    if sas_pda_info.key().ne(&sas_pda.to_bytes()) {
        return Err(AttestationServiceError::InvalidProgramSigner.into());
    }

    // Read instruction args
    let args: TokenizeSchemaArgs<'_> = TokenizeSchemaArgs::try_from_bytes(instruction_data)?;
    let max_size = args.max_size()?;

    // Initialize new account owned by token_program.
    create_pda_account(
        payer_info,
        &Rent::get()?,
        234, // Size before Group Extension
        &TOKEN_2022_PROGRAM_ID,
        mint_info,
        [
            Seed::from(SCHEMA_MINT_SEED),
            Seed::from(schema_info.key()),
            Seed::from(&[mint_bump]),
        ],
        Some(318), // Size after Group Extension
    )?;

    // Initialize GroupPointer extension.
    InitializeGroupPointer {
        mint: mint_info,
        authority: Some(*sas_pda_info.key()),
        group_address: Some(*sas_pda_info.key()),
    }
    .invoke()?;

    // Initialize Mint on created account.
    InitializeMint2 {
        mint: mint_info,
        decimals: 9,
        mint_authority: sas_pda_info.key(),
        freeze_authority: Some(sas_pda_info.key()),
    }
    .invoke(TokenProgramVariant::Token2022)?;

    // Initialize Group extension.
    let bump_seed = [sas_bump];
    let sas_pda_seeds = [Seed::from(SAS_SEED), Seed::from(&bump_seed)];
    InitializeGroup {
        group: mint_info,
        mint: mint_info,
        mint_authority: sas_pda_info,
        update_authority: Some(*sas_pda_info.key()),
        max_size,
    }
    .invoke_signed(&[Signer::from(&sas_pda_seeds)])?;

    Ok(())
}

/// Instruction data for the `TokenizeSchema` instruction.
pub struct TokenizeSchemaArgs<'a> {
    raw: *const u8,

    _data: PhantomData<&'a [u8]>,
}

impl TokenizeSchemaArgs<'_> {
    #[inline]
    pub fn try_from_bytes(bytes: &[u8]) -> Result<TokenizeSchemaArgs, ProgramError> {
        // max_size (8 bytes)
        if bytes.len() < 8 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(TokenizeSchemaArgs {
            raw: bytes.as_ptr(),
            _data: PhantomData,
        })
    }

    #[inline]
    pub fn max_size(&self) -> Result<u64, ProgramError> {
        unsafe {
            let max_size_bytes = *(self.raw as *const u64);
            Ok(max_size_bytes)
        }
    }
}
