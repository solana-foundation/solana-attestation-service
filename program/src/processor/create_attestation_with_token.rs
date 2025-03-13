extern crate alloc;

use alloc::vec::Vec;
use core::marker::PhantomData;

use mpl_core::programs::MPL_CORE_ID;
use mpl_core::types::DataState;
use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction},
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};
use pinocchio_log::log;

use crate::{acc_info_as_str, processor::process_create_attestation};

#[inline(always)]
pub fn process_create_attestation_with_token(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Create Attestation first
    process_create_attestation(program_id, &accounts[0..6], instruction_data)?;

    let [payer_info, _authorized_signer, _credential_info, _schema_info, _attestation_info, system_program, asset_info, core_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if core_program.key().ne(&MPL_CORE_ID.to_bytes()) {
        log!(
            "Account {} is not the mpl core program",
            acc_info_as_str!(core_program)
        );
        return Err(ProgramError::IncorrectProgramId);
    }

    let args = CreateAttestationWithTokenArgs::try_from_bytes(instruction_data)?;
    let name = args.name()?;
    let uri = args.uri()?;

    let mut data: Vec<u8> = Vec::new();

    data.push(20); // CreateV2 Ix Discriminator
    data.push(DataState::AccountState as u8);
    data.extend(&(name.len() as u32).to_le_bytes());
    data.extend_from_slice(name);
    data.extend(&(uri.len() as u32).to_le_bytes());
    data.extend_from_slice(uri);
    data.extend(&0_u8.to_le_bytes()); // No plugins
    data.extend(&0_u8.to_le_bytes()); // No external plugins

    let ix = &Instruction {
        program_id: core_program.key(),
        accounts: &[
            AccountMeta::new(asset_info.key(), true, true),
            AccountMeta::new(core_program.key(), false, false), // Placeholder for collection
            AccountMeta::new(core_program.key(), false, false), // Placeholder for authority
            AccountMeta::new(payer_info.key(), true, true),
            AccountMeta::new(core_program.key(), false, false), // Placeholder for owner
            AccountMeta::new(core_program.key(), false, false), // Placeholder for update_authority
            AccountMeta::new(system_program.key(), false, false),
            AccountMeta::new(core_program.key(), false, false), // Placeholder for log_wrapper
        ],
        data: data.as_slice(),
    };
    invoke(
        ix,
        &[
            asset_info,
            core_program,
            core_program,
            payer_info,
            core_program,
            core_program,
            system_program,
            core_program,
        ],
    )?;

    Ok(())
}

/// Instruction data for the `CreateAttestationWithToken` instruction.
pub struct CreateAttestationWithTokenArgs<'a> {
    raw: *const u8,

    _data: PhantomData<&'a [u8]>,
}

impl CreateAttestationWithTokenArgs<'_> {
    #[inline]
    pub fn try_from_bytes(bytes: &[u8]) -> Result<CreateAttestationWithTokenArgs, ProgramError> {
        // The minimum expected size of the instruction data.
        // - nonce (32 bytes)
        // - data (5 bytes. 4 len, 1 byte)
        // - expiry (8 bytes)
        // - name (5 bytes. 4 len, 1 byte)
        // - uri (5 bytes. 4 len, 1 byte)
        if bytes.len() < 55 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(CreateAttestationWithTokenArgs {
            raw: bytes.as_ptr(),
            _data: PhantomData,
        })
    }

    #[inline]
    pub fn name(&self) -> Result<&[u8], ProgramError> {
        // SAFETY: the `bytes` length was validated in `try_from_bytes`.
        unsafe {
            let mut offset: u32 = 32; // Nonce
            let data_len = *(self.raw.add(offset as usize) as *const u32);
            offset += data_len + 4; // Data
            offset += 8; // Expiry

            let name_len = *(self.raw.add(offset as usize) as *const u32);
            offset += 4;
            let name_bytes =
                core::slice::from_raw_parts(self.raw.add(offset as usize), name_len as usize);
            Ok(name_bytes)
        }
    }

    #[inline]
    pub fn uri(&self) -> Result<&[u8], ProgramError> {
        // SAFETY: the `bytes` length was validated in `try_from_bytes`.
        unsafe {
            let mut offset: u32 = 32; // Nonce
            let data_len = *(self.raw.add(offset as usize) as *const u32);
            offset += data_len + 4; // Data
            offset += 8; // Expiry
            let name_len = *(self.raw.add(offset as usize) as *const u32);
            offset += name_len + 4; // Name

            let uri_len = *(self.raw.add(offset as usize) as *const u32);
            offset += 4;
            let uri_bytes =
                core::slice::from_raw_parts(self.raw.add(offset as usize), uri_len as usize);
            Ok(uri_bytes)
        }
    }
}
