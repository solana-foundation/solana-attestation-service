use pinocchio::{error::ProgramError, AccountView, Address};
use pinocchio_associated_token_account::ID as ATA_PROGRAM_ID;
use pinocchio_log::log;
use pinocchio_token_2022::ID as TOKEN_2022_PROGRAM_ID;

use crate::{
    acc_info_as_str,
    constants::{event_authority_pda, sas_pda},
    error::AttestationServiceError,
    key_as_str, ID,
};

pub fn verify_signer(info: &AccountView) -> Result<(), ProgramError> {
    if !info.is_signer() {
        log!("Account {} is not a signer", acc_info_as_str!(info));
        return Err(ProgramError::MissingRequiredSignature);
    }

    Ok(())
}

/// Verify the account is an empty, writable, system-owned account ready to be created.
pub fn verify_system_account(info: &AccountView) -> Result<(), ProgramError> {
    if !info.owned_by(&pinocchio_system::ID) {
        log!("Account {} is not owned by the system program", acc_info_as_str!(info));
        return Err(ProgramError::InvalidAccountOwner);
    }

    if !info.is_data_empty() {
        log!("Account {} data is not empty", acc_info_as_str!(info));
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    if !info.is_writable() {
        log!("Account {} is not writable", acc_info_as_str!(info));
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

pub fn verify_system_program(info: &AccountView) -> Result<(), ProgramError> {
    if info.address().ne(&pinocchio_system::ID) {
        log!("Account {} is not the system program", acc_info_as_str!(info));
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}

pub fn verify_token22_program(info: &AccountView) -> Result<(), ProgramError> {
    if info.address().ne(&TOKEN_2022_PROGRAM_ID) {
        log!("Account {} is not the Token 2022 program", acc_info_as_str!(info));
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}

pub fn verify_ata_program(info: &AccountView) -> Result<(), ProgramError> {
    if info.address().ne(&ATA_PROGRAM_ID) {
        log!("Account {} is not the Associated Token program", acc_info_as_str!(info));
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}

pub fn verify_current_program(info: &AccountView) -> Result<(), ProgramError> {
    if info.address().ne(&ID) {
        log!("Account {} is not the current program", acc_info_as_str!(info));
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}

pub fn verify_owner_mutability(info: &AccountView, owner: &Address, expect_writable: bool) -> Result<(), ProgramError> {
    if !info.owned_by(owner) {
        log!("Owner of {} does not match {}", acc_info_as_str!(info), key_as_str!(owner),);
        return Err(ProgramError::InvalidAccountOwner);
    }
    if expect_writable && !info.is_writable() {
        log!("{} does not have the right write access", acc_info_as_str!(info),);
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

pub fn verify_event_authority(info: &AccountView) -> Result<(), ProgramError> {
    if info.address().ne(&event_authority_pda::ID) {
        return Err(AttestationServiceError::InvalidEventAuthority.into());
    }

    Ok(())
}

pub fn verify_sas_pda(info: &AccountView) -> Result<(), ProgramError> {
    if info.address().ne(&sas_pda::ID) {
        return Err(AttestationServiceError::InvalidProgramSigner.into());
    }

    Ok(())
}
