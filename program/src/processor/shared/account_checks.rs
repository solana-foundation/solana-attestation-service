use bs58;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use pinocchio_log::log;

use crate::{acc_info_as_str, key_as_str};

/// Verify account as a signer, returning an error if it is not or if it is not writable while
/// expected to be.
///
/// # Arguments
/// * `info` - The account to verify.
/// * `expect_writable` - Whether the account should be writable
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn verify_signer(info: &AccountInfo, expect_writable: bool) -> Result<(), ProgramError> {
    if !info.is_signer() {
        log!("Account {} is not a signer", acc_info_as_str!(info));
        return Err(ProgramError::MissingRequiredSignature);
    }
    if expect_writable && !info.is_writable() {
        log!("Signer {} is not writable", acc_info_as_str!(info));
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Verify account as a system account, returning an error if it is not or if it is not writable
/// while expected to be.
///
/// # Arguments
/// * `info` - The account to verify.
/// * `is_writable` - Whether the account should be writable.
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn verify_system_account(info: &AccountInfo, is_writable: bool) -> Result<(), ProgramError> {
    if info.owner().ne(&pinocchio_system::id()) {
        log!(
            "Account {} is not owned by the system program",
            acc_info_as_str!(info)
        );
        return Err(ProgramError::InvalidAccountOwner);
    }

    if !info.data_is_empty() {
        log!("Account {} data is not empty", acc_info_as_str!(info));
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    if is_writable && !info.is_writable() {
        log!("Account {} is not writable", acc_info_as_str!(info));
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Verify account as system program, returning an error if it is not.
///
/// # Arguments
/// * `info` - The account to verify.
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn verify_system_program(info: &AccountInfo) -> Result<(), ProgramError> {
    if info.key().ne(&pinocchio_system::ID) {
        log!(
            "Account {} is not the system program",
            acc_info_as_str!(info)
        );
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}

/// Verify account's owner and account mutability.
///
/// # Arguments
/// * `info` - The account to verify.
/// * `owner` - The expected owner of the account.
/// * `is_writable` - Whether the account is expected to be writable.
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn verify_owner_mutability(
    info: &AccountInfo,
    owner: &Pubkey,
    is_writable: bool,
) -> Result<(), ProgramError> {
    if info.owner().ne(owner) {
        log!(
            "Owner of {} does not match {}",
            acc_info_as_str!(info),
            key_as_str!(owner),
        );
        return Err(ProgramError::InvalidAccountOwner);
    }
    if is_writable != info.is_writable() {
        log!(
            "{} does not have the right write access",
            acc_info_as_str!(info),
        );
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}
