use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Seed, Signer},
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};
use solana_program::pubkey::Pubkey as SolanaPubkey;

use crate::{
    constants::EVENT_AUTHORITY_SEED,
    error::AttestationServiceError,
    events::CloseAttestationEvent,
    state::{discriminator::AccountSerialize, Attestation},
};

use super::{
    verify_current_program, verify_owner_mutability, verify_signer, verify_system_program,
};

#[inline(always)]
pub fn process_close_attestation(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [payer_info, authorized_signer, attestation_info, event_authority_info, system_program, attestation_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate: authority should have signed
    verify_signer(authorized_signer, false)?;

    // Validate system program
    verify_system_program(system_program)?;

    // Verify attestation program
    verify_current_program(attestation_program)?;

    // Validate Attestation is owned by our program
    verify_owner_mutability(attestation_info, program_id, true)?;

    let attestation_data = attestation_info.try_borrow_data()?;
    let attestation = Attestation::try_from_bytes(&attestation_data)?;
    drop(attestation_data); // Drop immutable borrow.

    // Verify that attestation's signer matches signer.
    attestation.validate_signer(authorized_signer.key())?;

    // Close account and transfer rent to payer.
    let payer_lamports = payer_info.lamports();
    *payer_info.try_borrow_mut_lamports().unwrap() = payer_lamports
        .checked_add(attestation_info.lamports())
        .unwrap();
    *attestation_info.try_borrow_mut_lamports().unwrap() = 0;

    attestation_info.assign(&system_program.key());
    attestation_info.realloc(0, false)?;

    // Check that event authority PDA is valid.
    let (event_authority_pda, bump) = SolanaPubkey::find_program_address(
        &[EVENT_AUTHORITY_SEED],
        &SolanaPubkey::from(*program_id),
    );
    if event_authority_info
        .key()
        .ne(&event_authority_pda.to_bytes())
    {
        return Err(AttestationServiceError::InvalidEventAuthority.into());
    }

    // CPI to emit_event ix on same program to store event data in ix arg.
    let event = CloseAttestationEvent {
        schema: attestation.schema,
        attestation_data: attestation.data,
    };
    invoke_signed(
        &Instruction {
            program_id,
            accounts: &[
                AccountMeta::new(event_authority_info.key(), false, true),
                AccountMeta::new(program_id, false, false),
            ],
            // Prepend IX Discriminator for emit_event.
            data: &[&[8_u8], event.to_bytes().as_slice()].concat(),
        },
        &[event_authority_info, attestation_program],
        &[Signer::from(&[
            Seed::from(b"eventAuthority"),
            Seed::from(&[bump]),
        ])],
    )?;

    Ok(())
}
