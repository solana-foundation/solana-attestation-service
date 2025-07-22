use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};
use pinocchio_token::extensions::token_group::UpdateGroupMaxSize;
use solana_program::pubkey::Pubkey as SolanaPubkey;

use crate::{
    constants::{sas_pda, SAS_SEED, SCHEMA_MINT_SEED},
    error::AttestationServiceError,
    processor::{verify_owner_mutability, verify_signer, verify_token22_program},
    require_len,
    state::{Credential, Schema},
};

#[inline(always)]
pub fn process_change_tokenized_schema_size(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let args = process_instruction_data(instruction_data)?;
    let [authority_info, schema_mint_info, schema_info, credential_info, sas_pda_info, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate: authority should have signed
    verify_signer(authority_info, false)?;
    // Validate Credential and Schema are owned by our program
    verify_owner_mutability(credential_info, program_id, false)?;
    verify_owner_mutability(schema_info, program_id, false)?;
    // Validate token program is Token2022 and schema mint is owned by it
    verify_token22_program(token_program)?;
    verify_owner_mutability(schema_mint_info, token_program.key(), false)?;

    // Validate Schema Mint
    let (schema_mint_pda, _) = SolanaPubkey::find_program_address(
        &[SCHEMA_MINT_SEED, schema_info.key()],
        &SolanaPubkey::from(*program_id),
    );

    if schema_mint_info.key().ne(&schema_mint_pda.to_bytes()) {
        return Err(AttestationServiceError::InvalidMint.into());
    }

    // Validate Schema's Credential Authority matches the signer authority
    let schema = Schema::try_from_bytes(&schema_info.try_borrow_data()?)?;
    let credential = Credential::try_from_bytes(&credential_info.try_borrow_data()?)?;
    if schema.credential.ne(credential_info.key()) {
        return Err(AttestationServiceError::InvalidCredential.into());
    }
    if credential.authority.ne(authority_info.key()) {
        return Err(AttestationServiceError::InvalidAuthority.into());
    }

    // Validate that sas_pda matches
    if sas_pda_info.key().ne(&sas_pda::ID) {
        return Err(AttestationServiceError::InvalidProgramSigner.into());
    }

    // Update the max size of the tokenized schema collection
    let bump_seed = [sas_pda::BUMP];
    let sas_pda_seeds: [Seed<'_>; 2] = [Seed::from(SAS_SEED), Seed::from(&bump_seed)];
    UpdateGroupMaxSize {
        group: schema_mint_info,
        update_authority: sas_pda_info,
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