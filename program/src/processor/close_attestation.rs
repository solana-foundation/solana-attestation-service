use pinocchio::{
    cpi::{invoke_signed, Seed, Signer},
    error::ProgramError,
    instruction::{InstructionAccount, InstructionView},
    AccountView, Address, ProgramResult,
};

use crate::{
    constants::{event_authority_pda, EVENT_AUTHORITY_SEED},
    error::AttestationServiceError,
    events::{CloseAttestationEvent, EventDiscriminators},
    state::{Attestation, Credential},
};

use super::{
    verify_current_program, verify_event_authority, verify_owner_mutability, verify_signer, verify_system_program,
};

#[inline(always)]
pub fn process_close_attestation(
    program_id: &Address,
    accounts: &mut [AccountView],
    token_account: Option<Address>,
) -> ProgramResult {
    let [payer_info, authorized_signer, credential_info, attestation_info, event_authority_info, system_program, attestation_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    verify_signer(authorized_signer)?;
    verify_system_program(system_program)?;
    verify_current_program(attestation_program)?;
    verify_owner_mutability(credential_info, program_id, false)?;
    verify_owner_mutability(attestation_info, program_id, true)?;

    let credential = Credential::try_from_bytes(&credential_info.try_borrow()?)?;
    credential.validate_authorized_signer(authorized_signer.address())?;

    let attestation = Attestation::try_from_bytes(&attestation_info.try_borrow()?)?;

    if let Some(token_account) = token_account {
        if token_account.ne(&attestation.token_account) {
            return Err(AttestationServiceError::InvalidTokenAccount.into());
        }
    } else if attestation.token_account.ne(&Address::default()) {
        return Err(AttestationServiceError::InvalidTokenAccount.into());
    }

    if attestation.credential.ne(credential_info.address()) {
        return Err(AttestationServiceError::InvalidCredential.into());
    }

    let payer_lamports = payer_info.lamports();
    payer_info.set_lamports(payer_lamports.checked_add(attestation_info.lamports()).unwrap());
    attestation_info.close()?;

    verify_event_authority(event_authority_info)?;

    let event = CloseAttestationEvent {
        discriminator: EventDiscriminators::CloseEvent as u8,
        schema: attestation.schema,
        attestation_data: attestation.data,
    };
    invoke_signed(
        &InstructionView {
            program_id,
            accounts: &[InstructionAccount::readonly_signer(event_authority_info.address())],
            data: event.to_bytes().as_slice(),
        },
        &[&*event_authority_info],
        &[Signer::from(&[Seed::from(EVENT_AUTHORITY_SEED), Seed::from(&[event_authority_pda::BUMP])])],
    )?;

    Ok(())
}
