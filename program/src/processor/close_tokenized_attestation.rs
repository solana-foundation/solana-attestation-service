use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};
use solana_pubkey::Pubkey as SolanaPubkey;

use crate::{
    constants::{sas_pda, ATTESTATION_MINT_SEED, SAS_SEED},
    error::AttestationServiceError,
    processor::verify_token22_program,
};
use pinocchio_token::instructions::{BurnChecked, CloseAccount, TokenProgramVariant};

use super::process_close_attestation;

#[inline(always)]
pub fn process_close_tokenized_attestation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [payer_info, _authorized_signer, _credential_info, attestation_info, _event_authority_info, _system_program, _attestation_program, attestation_mint_info, sas_pda_info, attestation_token_account, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify token program.
    verify_token22_program(token_program)?;

    // Validate that mint matches expected PDA
    let (attestation_mint_pda, _) = SolanaPubkey::find_program_address(
        &[ATTESTATION_MINT_SEED, attestation_info.key()],
        &SolanaPubkey::from(*program_id),
    );
    if attestation_mint_info
        .key()
        .ne(&attestation_mint_pda.to_bytes())
    {
        return Err(AttestationServiceError::InvalidMint.into());
    }

    // Validate that sas_pda matches
    if sas_pda_info.key().ne(&sas_pda::ID) {
        return Err(AttestationServiceError::InvalidProgramSigner.into());
    }

    let bump_seed = [sas_pda::BUMP];
    let sas_pda_seeds = [Seed::from(SAS_SEED), Seed::from(&bump_seed)];

    // Burn Attestation Token
    BurnChecked {
        account: attestation_token_account,
        mint: attestation_mint_info,
        authority: sas_pda_info,
        amount: 1,
        decimals: 0,
    }
    .invoke_signed(
        &[Signer::from(&sas_pda_seeds)],
        TokenProgramVariant::Token2022,
    )?;

    // Close Attestation Token Mint
    CloseAccount {
        account: attestation_mint_info,
        destination: payer_info,
        authority: sas_pda_info,
    }
    .invoke_signed(
        &[Signer::from(&sas_pda_seeds)],
        TokenProgramVariant::Token2022,
    )?;

    // Close Attestation: This needs to be called after closing of Mint due to Solana
    // limitations around lamports balance. This also verifies accounts[0..7] and
    // attestation_token_account.
    process_close_attestation(
        program_id,
        &accounts[0..7],
        Some(*attestation_token_account.key()),
    )?;

    Ok(())
}
