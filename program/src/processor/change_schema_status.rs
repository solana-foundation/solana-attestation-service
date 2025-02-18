use bs58;
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};
use pinocchio_log::log;
use solana_program::pubkey::Pubkey as SolanaPubkey;

use crate::{
    constants::{CREDENTIAL_SEED, SCHEMA_SEED},
    state::{load_signer, verify_owner_mutability, Credential, Schema},
};

#[inline(always)]
pub fn process_change_schema_status(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [authority_info, credential_info, schema_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate: authority should have signed
    load_signer(authority_info, false)?;

    // Verify program ownership, mutability and PDAs.
    verify_owner_mutability(credential_info, program_id, false)?;
    verify_owner_mutability(schema_info, program_id, true)?;

    // Read is_paused from instruction data.
    let is_paused = instruction_data
        .get(0)
        .ok_or(ProgramError::InvalidInstructionData)?
        .eq(&1);

    let credential = &Credential::try_from_bytes(&credential_info.try_borrow_data()?)?;
    let (credential_pda, _credential_bump) = SolanaPubkey::find_program_address(
        &[
            CREDENTIAL_SEED,
            authority_info.key(),
            credential.name.get(4..).unwrap(), // Convert Vec<u8> to UTF8 Array
        ],
        &SolanaPubkey::from(*program_id),
    );
    if credential_info.key().ne(&credential_pda.to_bytes()) {
        log!(
            "PDA Mismatch for {}",
            bs58::encode(credential_info.key()).into_string().as_str(),
        );
        return Err(ProgramError::InvalidAccountData);
    }

    // Verify signer matches credential authority.
    if credential.authority.ne(authority_info.key()) {
        return Err(ProgramError::IncorrectAuthority);
    }

    let mut schema_data = schema_info.try_borrow_mut_data()?;
    let mut schema = Schema::try_from_bytes(&schema_data)?;
    let (schema_pda, _schema_bump) = SolanaPubkey::find_program_address(
        &[
            SCHEMA_SEED,
            credential_info.key(),
            schema.name.get(4..).unwrap(), // Convert Vec<u8> to UTF8 Array
        ],
        &SolanaPubkey::from(*program_id),
    );
    if schema_info.key().ne(&schema_pda.to_bytes()) {
        log!(
            "PDA Mismatch for {}",
            bs58::encode(schema_info.key()).into_string().as_str()
        );
        return Err(ProgramError::InvalidAccountData);
    }

    schema.is_paused = is_paused;
    schema_data.copy_from_slice(&schema.to_bytes());

    Ok(())
}
