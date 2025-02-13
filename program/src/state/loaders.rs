use pinocchio::{account_info::AccountInfo, msg, program_error::ProgramError};

/// Loads the account as a system account, returning an error if it is not or if it is not writable
/// while expected to be.
///
/// # Arguments
/// * `info` - The account to load the system account from
/// * `is_writable` - Whether the account should be writable
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_system_account(info: &AccountInfo, is_writable: bool) -> Result<(), ProgramError> {
    if info.owner() != &pinocchio_system::id() {
        msg!("Account is not owned by the system program");
        return Err(ProgramError::InvalidAccountOwner);
    }

    if !info.data_is_empty() {
        msg!("Account data is not empty");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    if is_writable && !info.is_writable() {
        msg!("Account is not writable");
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}
