use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

use crate::{constants::event_authority_pda, error::AttestationServiceError, processor::verify_signer};

#[inline(always)]
pub fn process_emit_event(_program_id: &Address, accounts: &mut [AccountView]) -> ProgramResult {
    let [event_authority] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if event_authority.address().ne(&event_authority_pda::ID) {
        return Err(AttestationServiceError::InvalidEventAuthority.into());
    }

    // No-op, besides checking for event authority signing.
    verify_signer(event_authority, false)?;

    Ok(())
}
