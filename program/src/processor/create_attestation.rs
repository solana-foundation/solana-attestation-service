use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use crate::{
    error::AttestationServiceError,
    state::{Credential, Schema},
};

use super::{verify_signer, verify_system_program};

#[inline(always)]
pub fn process_create_attestation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [payer_info, authority_info, credential_info, schema_info, attestation_info, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate: authority should have signed
    verify_signer(authority_info, false)?;

    // Validate system program
    verify_system_program(system_program)?;

    let credential_data = credential_info.try_borrow_data()?;
    let credential = Credential::try_from_bytes(&credential_data)?;
    // Validate Credential PDA
    credential.verify_pda(credential_info, program_id)?;

    // Validate Authority is an authorized signer
    // credential.authorized_signers

    // TODO validate Schema PDA
    let schema_data = schema_info.try_borrow_data()?;
    let schema = Schema::try_from_bytes(&schema_data)?;

    // Validate Schema is owned by Credential
    if schema.credential.ne(credential_info.key()) {
        return Err(AttestationServiceError::InvalidCredential.into());
    }

    // TODO validate data with schema

    Ok(())
}
