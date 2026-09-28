use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, Address, ProgramResult,
};

use crate::{
    constants::{sas_pda, ATTESTATION_MINT_SEED, SAS_SEED},
    error::AttestationServiceError,
    processor::verify_token22_program,
};
use pinocchio_token_2022::instructions::{BurnChecked, CloseAccount};

use super::process_close_attestation;

#[inline(always)]
pub fn process_close_tokenized_attestation(program_id: &Address, accounts: &mut [AccountView]) -> ProgramResult {
    let [payer_info, _authorized_signer, _credential_info, attestation_info, _event_authority_info, _system_program, _attestation_program, attestation_mint_info, sas_pda_info, attestation_token_account, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify token program.
    verify_token22_program(token_program)?;

    // Validate that mint matches expected PDA
    let (attestation_mint_pda, _) =
        Address::find_program_address(&[ATTESTATION_MINT_SEED, attestation_info.address().as_ref()], program_id);
    if attestation_mint_info.address().ne(&attestation_mint_pda) {
        return Err(AttestationServiceError::InvalidMint.into());
    }

    // Validate that sas_pda matches
    if sas_pda_info.address().ne(&sas_pda::ID) {
        return Err(AttestationServiceError::InvalidProgramSigner.into());
    }

    let bump_seed = [sas_pda::BUMP];
    let sas_pda_seeds = [Seed::from(SAS_SEED), Seed::from(&bump_seed)];

    // Burn Attestation Token
    BurnChecked::new(attestation_token_account, attestation_mint_info, sas_pda_info, 1, 0)
        .invoke_signed(&[Signer::from(&sas_pda_seeds)])?;

    // Close Attestation Token Mint
    CloseAccount::new(attestation_mint_info, payer_info, sas_pda_info)
        .invoke_signed(&[Signer::from(&sas_pda_seeds)])?;

    // Close Attestation: This needs to be called after closing of Mint due to Solana
    // limitations around lamports balance. This also verifies accounts[0..7] and
    // attestation_token_account.
    let attestation_token_account = *attestation_token_account.address();
    process_close_attestation(program_id, &mut accounts[0..7], Some(attestation_token_account))?;

    Ok(())
}
