use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use crate::state::Attestation;

use super::{verify_owner_mutability, verify_signer, verify_system_program};

#[inline(always)]
pub fn process_close_attestation(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [payer_info, authorized_signer, attestation_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate: authority should have signed
    verify_signer(authorized_signer, false)?;

    // Validate system program
    verify_system_program(system_program)?;

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

    Ok(())
}
