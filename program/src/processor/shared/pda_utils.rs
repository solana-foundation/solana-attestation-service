use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    pubkey::Pubkey,
    sysvars::rent::Rent,
    ProgramResult,
};
use pinocchio_system::instructions::{Allocate, Assign, CreateAccount, Transfer};

/// Create a PDA account for the given seeds.
pub fn create_pda_account<const N: usize>(
    payer: &AccountInfo,
    rent: &Rent,
    space: usize,
    owner: &Pubkey,
    new_pda_account: &AccountInfo,
    new_pda_signer_seeds: [Seed; N],
    min_rent_space: Option<usize>,
) -> ProgramResult {
    let signers = [Signer::from(&new_pda_signer_seeds)];
    let rent_space = match min_rent_space {
        Some(min_space) => min_space.max(space),
        None => space,
    };
    let required_lamports = rent.minimum_balance(rent_space).max(1);

    if new_pda_account.lamports() > 0 {
        // someone can transfer lamports to accounts before they're initialized
        // in that case, creating the account won't work.
        // in order to get around it, you need to fund the account with enough lamports to be rent exempt,
        // then allocate the required space and set the owner to the current program
        let required_lamports = required_lamports.saturating_sub(new_pda_account.lamports());
        if required_lamports > 0 {
            Transfer {
                from: payer,
                to: new_pda_account,
                lamports: required_lamports,
            }
            .invoke()?;
        }
        Allocate {
            account: new_pda_account,
            space: space as u64,
        }
        .invoke_signed(&signers)?;
        Assign {
            account: new_pda_account,
            owner,
        }
        .invoke_signed(&signers)
    } else {
        CreateAccount {
            from: payer,
            to: new_pda_account,
            lamports: required_lamports,
            space: space as u64,
            owner,
        }
        .invoke_signed(&signers)
    }
}
