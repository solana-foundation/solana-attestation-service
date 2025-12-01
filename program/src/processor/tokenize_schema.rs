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
use solana_pubkey::Pubkey as SolanaPubkey;

use crate::{
    constants::{sas_pda, SAS_SEED, SCHEMA_MINT_SEED},
    error::AttestationServiceError,
    processor::{create_pda_account, verify_signer, verify_system_program},
    require_len,
    state::{Credential, Schema},
};

use super::{verify_owner_mutability, verify_token22_program};

#[inline(always)]
pub fn process_tokenize_schema(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let args = process_instruction_data(instruction_data)?;
    let [payer_info, authority_info, credential_info, schema_info, mint_info, sas_pda_info, system_program, token_program] =
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
    verify_token22_program(token_program)?;

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
    if sas_pda_info.key().ne(&sas_pda::ID) {
        return Err(AttestationServiceError::InvalidProgramSigner.into());
    }

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
        decimals: 0,
        mint_authority: sas_pda_info.key(),
        freeze_authority: Some(sas_pda_info.key()),
    }
    .invoke(TokenProgramVariant::Token2022)?;

    // Initialize Group extension.
    let bump_seed = [sas_pda::BUMP];
    let sas_pda_seeds: [Seed<'_>; 2] = [Seed::from(SAS_SEED), Seed::from(&bump_seed)];
    InitializeGroup {
        group: mint_info,
        mint: mint_info,
        mint_authority: sas_pda_info,
        update_authority: Some(*sas_pda_info.key()),
        max_size: args.max_size,
    }
    .invoke_signed(&[Signer::from(&sas_pda_seeds)])?;

    Ok(())
}

struct TokenizeSchemaArgs {
    max_size: u64,
}

fn process_instruction_data(data: &[u8]) -> Result<TokenizeSchemaArgs, ProgramError> {
    require_len!(data, 8);
    let max_size = u64::from_le_bytes(data[0..8].try_into().unwrap());

    Ok(TokenizeSchemaArgs { max_size })
}
