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
        6 => process_create_attestation(program_id, accounts, instruction_data, None),
        7 => process_close_attestation(program_id, accounts, None),
        9 => process_tokenize_schema(program_id, accounts, instruction_data),
        10 => process_create_tokenized_attestation(program_id, accounts, instruction_data),
        11 => process_close_tokenized_attestation(program_id, accounts),
        12 => process_create_compressed_attestation(program_id, accounts, instruction_data),
        13 => process_close_compressed_attestation(program_id, accounts, instruction_data),
        14 => process_compress_attestations(program_id, accounts, instruction_data),
        228 => process_emit_event(program_id, accounts), // matches EVENT_IX_TAG[0]
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
