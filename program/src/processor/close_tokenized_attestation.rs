use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, Address, ProgramResult,
};

use crate::{
    constants::{sas_pda, ATTESTATION_MINT_SEED, SAS_SEED},
    error::AttestationServiceError,
    processor::{process_close_attestation, verify_sas_pda, verify_token22_program},
};
use pinocchio_token_2022::instructions::{BurnChecked, CloseAccount};

#[inline(always)]
pub fn process_close_tokenized_attestation(program_id: &Address, accounts: &mut [AccountView]) -> ProgramResult {
    let [payer_info, _authorized_signer, _credential_info, attestation_info, _event_authority_info, _system_program, _attestation_program, attestation_mint_info, sas_pda_info, attestation_token_account, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    verify_token22_program(token_program)?;

    let (attestation_mint_pda, _) =
        Address::find_program_address(&[ATTESTATION_MINT_SEED, attestation_info.address().as_ref()], program_id);
    if attestation_mint_info.address().ne(&attestation_mint_pda) {
        return Err(AttestationServiceError::InvalidMint.into());
    }

    verify_sas_pda(sas_pda_info)?;

    let bump_seed = [sas_pda::BUMP];
    let sas_pda_seeds = [Seed::from(SAS_SEED), Seed::from(&bump_seed)];

    BurnChecked::new(attestation_token_account, attestation_mint_info, sas_pda_info, 1, 0)
        .invoke_signed(&[Signer::from(&sas_pda_seeds)])?;

    CloseAccount::new(attestation_mint_info, payer_info, sas_pda_info)
        .invoke_signed(&[Signer::from(&sas_pda_seeds)])?;

    // Close Attestation: This needs to be called after closing of Mint due to Solana
    // limitations around lamports balance. This also verifies accounts[0..7] and
    // attestation_token_account.
    let attestation_token_account = *attestation_token_account.address();
    process_close_attestation(program_id, &mut accounts[0..7], Some(attestation_token_account))?;

    Ok(())
}
