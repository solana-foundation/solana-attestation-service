use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Seed, Signer},
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{
    constants::{event_authority_pda, EVENT_AUTHORITY_SEED},
    error::AttestationServiceError,
    events::{CloseAttestationEvent, EventDiscriminators},
    state::{Attestation, Credential},
};

use super::{
    verify_current_program, verify_owner_mutability, verify_signer, verify_system_program,
};

#[inline(always)]
pub fn process_close_attestation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    token_account: Option<Pubkey>,
) -> ProgramResult {
    let [payer_info, authority, credential_info, attestation_info, event_authority_info, system_program, attestation_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate: authority should have signed
    verify_signer(authority, false)?;

    // Validate system program
    verify_system_program(system_program)?;

    // Verify attestation program
    verify_current_program(attestation_program)?;

    // Validate Credential and Attestation is owned by our program
    verify_owner_mutability(credential_info, program_id, false)?;
    verify_owner_mutability(attestation_info, program_id, true)?;

    // Check that one of credential's authorized signers have signed.
    let credential_data = credential_info.try_borrow_data()?;
    let credential = Credential::try_from_bytes(&credential_data)?;
    credential.validate_authorized_signer(authority.key())?;

    let attestation_data = attestation_info.try_borrow_data()?;
    let attestation = Attestation::try_from_bytes(&attestation_data)?;
    drop(attestation_data); // Drop immutable borrow.

    // Verify token_account matches address in Attestation
    if let Some(token_account) = token_account {
        if token_account.ne(&attestation.token_account) {
            return Err(AttestationServiceError::InvalidTokenAccount.into());
        }
    } else if attestation.token_account.ne(&Pubkey::default()) {
        return Err(AttestationServiceError::InvalidTokenAccount.into());
    }

    // Check that credential matches attestation's.
    if !attestation.credential.eq(credential_info.key()) {
        return Err(AttestationServiceError::InvalidCredential.into());
    }

    // Close account and transfer rent to payer.
    let payer_lamports = payer_info.lamports();
    *payer_info.try_borrow_mut_lamports().unwrap() = payer_lamports
        .checked_add(attestation_info.lamports())
        .unwrap();
    *attestation_info.try_borrow_mut_lamports().unwrap() = 0;
    attestation_info.close()?;

    // Check that event authority PDA is valid.
    if event_authority_info.key().ne(&event_authority_pda::ID) {
        return Err(AttestationServiceError::InvalidEventAuthority.into());
    }

    // CPI to emit_event ix on same program to store event data in ix arg.
    let event = CloseAttestationEvent {
        discriminator: EventDiscriminators::CloseEvent as u8,
        schema: attestation.schema,
        attestation_data: attestation.data,
    };
    invoke_signed(
        &Instruction {
            program_id,
            accounts: &[AccountMeta::new(event_authority_info.key(), false, true)],
            data: event.to_bytes().as_slice(),
        },
        &[event_authority_info],
        &[Signer::from(&[
            Seed::from(EVENT_AUTHORITY_SEED),
            Seed::from(&[event_authority_pda::BUMP]),
        ])],
    )?;

    Ok(())
}
