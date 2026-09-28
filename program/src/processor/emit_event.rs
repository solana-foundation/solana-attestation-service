use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

use crate::processor::{verify_event_authority, verify_signer};

#[inline(always)]
pub fn process_emit_event(_program_id: &Address, accounts: &mut [AccountView]) -> ProgramResult {
    let [event_authority] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    verify_event_authority(event_authority)?;
    verify_signer(event_authority)?;

    Ok(())
}
