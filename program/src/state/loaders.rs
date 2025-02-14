use pinocchio::{account_info::AccountInfo, msg, program_error::ProgramError};

pub trait VariableLenAccount {
    /// The minimum length of the type.
    ///
    /// This must be equal to the minimum size of each individual field in the type.
    const MIN_LEN: usize;

    // fn from_bytes(data: &[u8]) -> Self;

    // fn to_bytes()
}

/// Return a mutable `T` reference from the given bytes.
///
/// This function does not check if the data is initialized.
///
/// # Safety
///
/// The caller must ensure that `bytes` contains a valid representation of `T`.
#[inline(always)]
pub unsafe fn load_mut_unchecked<T: VariableLenAccount>(bytes: &mut [u8]) -> Result<&mut T, ProgramError> {
    if bytes.len() < T::MIN_LEN {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(&mut *(bytes.as_mut_ptr() as *mut T))
}

/// Loads the account as a signer, returning an error if it is not or if it is not writable while
/// expected to be.
///
/// # Arguments
/// * `info` - The account to load the signer from
/// * `expect_writable` - Whether the account should be writable
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_signer(info: &AccountInfo, expect_writable: bool) -> Result<(), ProgramError> {
    if !info.is_signer() {
        msg!("Account is not a signer");
        return Err(ProgramError::MissingRequiredSignature);
    }
    if expect_writable && !info.is_writable() {
        msg!("Signer is not writable");
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

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
    if info.owner().ne(&pinocchio_system::id()) {
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

/// Loads the account as a system program, returning an error if it is not.
///
/// # Arguments
/// * `info` - The account to load the system program from
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
pub fn load_system_program(info: &AccountInfo) -> Result<(), ProgramError> {
    if info.key().ne(&pinocchio_system::ID) {
        msg!("Account is not the system program");
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}
