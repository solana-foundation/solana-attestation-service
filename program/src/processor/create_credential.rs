extern crate alloc;

use alloc::vec::Vec;
use core::marker::PhantomData;
use pinocchio::{
    account_info::AccountInfo,
    instruction::Seed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use solana_program::pubkey::Pubkey as SolanaPubkey;

use crate::{
    constants::CREDENTIAL_SEED,
    error::AttestationServiceError,
    processor::{
        create_pda_account, to_serialized_vec, verify_signer, verify_system_account,
        verify_system_program,
    },
    state::{discriminator::AccountSerialize, Credential},
};

#[inline(always)]
pub fn process_create_credential(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [payer_info, credential_info, authority_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate: should be owned by system account, empty, and writable
    verify_system_account(credential_info, true)?;
    // Validate: authority should have signed
    verify_signer(authority_info, false)?;
    // Validate: system program
    verify_system_program(system_program)?;

    let args = CreateCredentialArgs::try_from_bytes(instruction_data)?;
    let name = args.name()?;
    let signers = args.signers()?;

    // NOTE: this could be optimized further by removing the `solana-program` dependency
    // and using `pubkey::checked_create_program_address` from Pinocchio to verify the
    // pubkey and associated bump (needed to be added as arg) is valid.
    let (credential_pda, credential_bump) = SolanaPubkey::find_program_address(
        &[CREDENTIAL_SEED, authority_info.key(), name],
        &SolanaPubkey::from(*program_id),
    );

    if credential_info.key() != &credential_pda.to_bytes() {
        // PDA was invalid
        return Err(AttestationServiceError::InvalidCredential.into());
    }

    // Account layout
    // discriminator - 1
    // authorized_signers - 4 + 32 * len
    // authority - 32
    // name - 4 + len
    let space = 1 + (4 + signers.len() * 32) + 32 + (4 + name.len());

    let rent = Rent::get()?;
    let bump_seed = [credential_bump];
    let signer_seeds = [
        Seed::from(CREDENTIAL_SEED),
        Seed::from(authority_info.key()),
        Seed::from(name),
        Seed::from(&bump_seed),
    ];
    create_pda_account(
        payer_info,
        &rent,
        space,
        program_id,
        credential_info,
        signer_seeds,
    )?;

    let credential = Credential {
        authority: *authority_info.key(),
        name: to_serialized_vec(name),
        authorized_signers: signers,
    };
    let mut credential_data = credential_info.try_borrow_mut_data()?;
    credential_data.copy_from_slice(&credential.to_bytes());

    Ok(())
}

/// Instruction data for the `CreateCredential` instruction.
pub struct CreateCredentialArgs<'a> {
    raw: *const u8,

    _data: PhantomData<&'a [u8]>,
}

impl CreateCredentialArgs<'_> {
    #[inline]
    pub fn try_from_bytes(bytes: &[u8]) -> Result<CreateCredentialArgs, ProgramError> {
        // The minimum expected size of the instruction data.
        // - name (5 bytes. 4 len, 1 char)
        // - signers (36 bytes. 4 len, 32 pubkey)
        if bytes.len() < 41 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(CreateCredentialArgs {
            raw: bytes.as_ptr(),
            _data: PhantomData,
        })
    }

    #[inline]
    pub fn name(&self) -> Result<&[u8], ProgramError> {
        // SAFETY: the `bytes` length was validated in `try_from_bytes`.
        unsafe {
            let len = *(self.raw as *const u32);
            let name_bytes = core::slice::from_raw_parts(self.raw.add(4), len as usize);
            Ok(name_bytes)
        }
    }

    #[inline]
    pub fn signers(&self) -> Result<Vec<Pubkey>, ProgramError> {
        // SAFETY: the `bytes` length was validated in `try_from_bytes`.
        unsafe {
            // use length of name to determine offset of Vec<Pubkey>
            let name_offset_including_length = *(self.raw as *const u32) + 4; // add for the length field
                                                                              // Length of Vec<Pubkey>
            let signers_length =
                *(self.raw.add(name_offset_including_length as usize) as *const u32);

            let mut offset = name_offset_including_length + 4; // Move past signers length field
            let mut signers = Vec::with_capacity(signers_length as usize);

            for _ in 0..signers_length {
                let signer_ptr = self.raw.add(offset as usize) as *const Pubkey;
                signers.push(*signer_ptr);
                offset += 32;
            }

            Ok(signers)
        }
    }
}
