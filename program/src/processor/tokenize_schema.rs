use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, Address, ProgramResult,
};
use pinocchio_token_2022::{
    instructions::{group_pointer::Initialize as InitializeGroupPointer, InitializeMint2},
    ID as TOKEN_2022_PROGRAM_ID,
};

use crate::{
    constants::{sas_pda, SAS_SEED, SCHEMA_MINT_SEED},
    error::AttestationServiceError,
    processor::{create_pda_account, token_ext::InitializeGroup, verify_signer, verify_system_program},
    require_len,
    state::{Credential, Schema},
};

use super::{verify_owner_mutability, verify_token22_program};

#[inline(always)]
pub fn process_tokenize_schema(
    program_id: &Address,
    accounts: &mut [AccountView],
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
    let credential = &Credential::try_from_bytes(&credential_info.try_borrow()?)?;
    if credential.authority.ne(authority_info.address()) {
        return Err(ProgramError::IncorrectAuthority);
    }

    // Validate Schema is owned by Credential
    let schema = Schema::try_from_bytes(&schema_info.try_borrow()?)?;
    if schema.credential.ne(credential_info.address()) {
        return Err(AttestationServiceError::InvalidCredential.into());
    }

    // Validate that mint to initialize matches expected PDA
    let (mint_pda, mint_bump) =
        Address::find_program_address(&[SCHEMA_MINT_SEED, schema_info.address().as_ref()], program_id);
    if mint_info.address().ne(&mint_pda) {
        return Err(AttestationServiceError::InvalidMint.into());
    }

    // Validate that sas_pda matches
    if sas_pda_info.address().ne(&sas_pda::ID) {
        return Err(AttestationServiceError::InvalidProgramSigner.into());
    }

    // Initialize new account owned by token_program.
    create_pda_account(
        payer_info,
        &Rent::get()?,
        234, // Size before Group Extension
        &TOKEN_2022_PROGRAM_ID,
        mint_info,
        [Seed::from(SCHEMA_MINT_SEED), Seed::from(schema_info.address().as_ref()), Seed::from(&[mint_bump])],
        Some(318), // Size after Group Extension
    )?;

    // Initialize GroupPointer extension.
    InitializeGroupPointer {
        mint: mint_info,
        authority: Some(sas_pda_info.address()),
        group_address: Some(sas_pda_info.address()),
        token_program: &TOKEN_2022_PROGRAM_ID,
    }
    .invoke()?;

    // Initialize Mint on created account.
    InitializeMint2::new(mint_info, 0, sas_pda_info.address(), Some(sas_pda_info.address())).invoke()?;

    // Initialize Group extension.
    let bump_seed = [sas_pda::BUMP];
    let sas_pda_seeds: [Seed<'_>; 2] = [Seed::from(SAS_SEED), Seed::from(&bump_seed)];
    InitializeGroup {
        group: mint_info,
        mint: mint_info,
        mint_authority: sas_pda_info,
        update_authority: sas_pda_info.address(),
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
