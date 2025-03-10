use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use crate::{
    constants::EVENT_AUTHORITY_SEED, error::AttestationServiceError, processor::verify_signer,
};
use solana_program::pubkey::Pubkey as SolanaPubkey;

#[inline(always)]
pub fn process_emit_event(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let [event_authority, _attestation_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let (event_authority_pda, _) = SolanaPubkey::find_program_address(
        &[EVENT_AUTHORITY_SEED],
        &SolanaPubkey::from(*program_id),
    );

    if event_authority.key().ne(&event_authority_pda.to_bytes()) {
        return Err(AttestationServiceError::InvalidEventAuthority.into());
    }

    // No-op, besides checking for event authority signing.
    verify_signer(event_authority, false)?;

    Ok(())
}
