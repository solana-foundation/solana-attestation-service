use pinocchio::{
    account_info::AccountInfo, entrypoint, program_error::ProgramError, pubkey::Pubkey,
    ProgramResult,
};

use crate::processor::*;

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (discriminator, instruction_data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match discriminator {
        0 => process_create_credential(program_id, accounts, instruction_data),
        1 => process_create_schema(program_id, accounts, instruction_data),
        2 => process_change_schema_status(program_id, accounts, instruction_data),
        3 => process_change_authorized_signers(program_id, accounts, instruction_data),
        4 => process_change_schema_description(program_id, accounts, instruction_data),
        5 => process_change_schema_version(program_id, accounts, instruction_data),
        6 => process_create_attestation(program_id, accounts, instruction_data),
        7 => process_close_attestation(program_id, accounts),
        8 => process_emit_event(program_id, accounts, instruction_data),
        9 => process_tokenize_schema(program_id, accounts, instruction_data),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
