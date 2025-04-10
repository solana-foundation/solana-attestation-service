use core::marker::PhantomData;

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Seed, Signer},
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_log::log;
use pinocchio_token::{
    extensions::{
        group_member_pointer::Initialize as InitializeGroupMemberPointer,
        metadata_pointer::Initialize as InitializeMetadataPointer,
        non_transferable::InitializeNonTransferableMint, token_group::InitializeMember,
    },
    instructions::{InitializeMint2, TokenProgramVariant},
    TOKEN_2022_PROGRAM_ID,
};
use solana_program::pubkey::Pubkey as SolanaPubkey;

use crate::{
    acc_info_as_str,
    constants::{ATTESTATION_MINT_SEED, SAS_SEED, SCHEMA_MINT_SEED},
    error::AttestationServiceError,
    processor::{process_create_attestation, shared::verify_signer},
    state::schema,
};

use super::{create_pda_account, verify_system_program};

#[inline(always)]
pub fn process_create_tokenized_attestation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Create Attestation first
    process_create_attestation(program_id, &accounts[0..6], instruction_data)?;

    let [payer_info, _authorized_signer, _credential_info, schema_info, attestation_info, system_program, schema_mint_info, attestation_mint_info, sas_pda_info, _token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate that mint to initialize matches expected PDA
    let (mint_pda, mint_bump) = SolanaPubkey::find_program_address(
        &[ATTESTATION_MINT_SEED, attestation_info.key()],
        &SolanaPubkey::from(*program_id),
    );
    if attestation_mint_info.key().ne(&mint_pda.to_bytes()) {
        return Err(AttestationServiceError::InvalidMint.into());
    }

    // Validate that sas_pda matches
    let (sas_pda, sas_bump) =
        SolanaPubkey::find_program_address(&[SAS_SEED], &SolanaPubkey::from(*program_id));
    if sas_pda_info.key().ne(&sas_pda.to_bytes()) {
        return Err(AttestationServiceError::InvalidProgramSigner.into());
    }

    let args = CreateTokenizedAttestionArgs::try_from_bytes(instruction_data)?;
    let name = args.name()?;
    let uri = args.uri()?;

    // Initialize new account owned by token_program.
    create_pda_account(
        payer_info,
        &Rent::get()?,
        306, // Size before TokenGroupMember Extension
        &TOKEN_2022_PROGRAM_ID,
        attestation_mint_info,
        [
            Seed::from(ATTESTATION_MINT_SEED),
            Seed::from(attestation_info.key()),
            Seed::from(&[mint_bump]),
        ],
        Some(500), // TODO: Update with correct size.
    )?;

    // Initialize GroupMemberPointer extension
    InitializeGroupMemberPointer {
        mint: attestation_mint_info,
        authority: Some(sas_pda.to_bytes()),
        member_address: Some(*attestation_mint_info.key()),
    }
    .invoke()?;

    // Initialize NonTransferable extension
    InitializeNonTransferableMint {
        mint: attestation_mint_info,
    }
    .invoke()?;

    // Initialize MetadataPointer extension
    InitializeMetadataPointer {
        mint: attestation_mint_info,
        authority: Some(*sas_pda_info.key()),
        metadata_address: Some(*attestation_mint_info.key()),
    }
    .invoke()?;

    // TODO: Call init metadata extension.

    // Initialize Mint on created account
    InitializeMint2 {
        mint: attestation_mint_info,
        decimals: 9,
        mint_authority: sas_pda_info.key(),
        freeze_authority: Some(sas_pda_info.key()),
    }
    .invoke(TokenProgramVariant::Token2022)?;

    // Initialize TokenGroupMember extension
    let bump_seed = [sas_bump];
    let sas_pda_seeds = [Seed::from(SAS_SEED), Seed::from(&bump_seed)];
    InitializeMember {
        group: schema_mint_info,
        group_update_authority: sas_pda_info,
        member: attestation_mint_info,
        member_mint: attestation_mint_info,
        member_mint_authority: sas_pda_info,
    }
    .invoke_signed(&[Signer::from(&sas_pda_seeds)])?;

    Ok(())
}

/// Instruction data for the `CreateAttestationWithToken` instruction.
pub struct CreateTokenizedAttestionArgs<'a> {
    raw: *const u8,

    _data: PhantomData<&'a [u8]>,
}

impl CreateTokenizedAttestionArgs<'_> {
    #[inline]
    pub fn try_from_bytes(bytes: &[u8]) -> Result<CreateTokenizedAttestionArgs, ProgramError> {
        // The minimum expected size of the instruction data.
        // - nonce (32 bytes)
        // - data (5 bytes. 4 len, 1 byte)
        // - expiry (8 bytes)
        // - name (5 bytes. 4 len, 1 byte)
        // - uri (5 bytes. 4 len, 1 byte)
        if bytes.len() < 55 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(CreateTokenizedAttestionArgs {
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
