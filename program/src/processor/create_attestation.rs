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
    constants::ATTESTATION_SEED,
    error::AttestationServiceError,
    state::{discriminator::AccountSerialize, Attestation, Credential, Schema},
};

use super::{create_pda_account, verify_owner_mutability, verify_signer, verify_system_program};

#[inline(always)]
pub fn process_create_attestation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [payer_info, authorized_signer, credential_info, schema_info, attestation_info, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate: authority should have signed
    verify_signer(authorized_signer, false)?;

    // Validate system program
    verify_system_program(system_program)?;
    // Validate Credential and Schema are owned by our program
    verify_owner_mutability(credential_info, program_id, false)?;
    verify_owner_mutability(schema_info, program_id, false)?;

    let credential_data = credential_info.try_borrow_data()?;
    let credential = Credential::try_from_bytes(&credential_data)?;
    // Validate Credential PDA
    credential.verify_pda(credential_info, program_id)?;

    // Validate Authority is an authorized signer
    credential.validate_authorized_signer(authorized_signer.key())?;

    let schema_data = schema_info.try_borrow_data()?;
    let schema = Schema::try_from_bytes(&schema_data)?;

    // Validate Schema PDA
    schema.verify_pda(schema_info, program_id)?;

    // Validate Schema is owned by Credential
    if schema.credential.ne(credential_info.key()) {
        return Err(AttestationServiceError::InvalidCredential.into());
    }

    let args = CreateAttestationArgs::try_from_bytes(instruction_data)?;
    let nonce = args.nonce()?;
    let data = args.data()?;
    let expiry = args.expiry()?;

    // NOTE: this could be optimized further by removing the `solana-program` dependency
    // and using `pubkey::checked_create_program_address` from Pinocchio to verify the
    // pubkey and associated bump (needed to be added as arg) is valid.
    let (attestation_pda, attestation_bump) = SolanaPubkey::find_program_address(
        &[
            ATTESTATION_SEED,
            credential_info.key(),
            authorized_signer.key(),
            schema_info.key(),
            nonce,
        ],
        &SolanaPubkey::from(*program_id),
    );

    // Validate attestation PDA is correct
    if attestation_info.key().ne(&attestation_pda.to_bytes()) {
        return Err(AttestationServiceError::InvalidAttestation.into());
    }

    // Create Attestation account

    // Account layout
    // discriminator - 1
    // nonce - 32
    // Credential - 32
    // Schema - 32
    // data - 4 + len
    // signer - 32
    // expiry - 8
    let space = 1 + 32 + 32 + 32 + (4 + data.len()) + 32 + 8;

    let bump_seed = [attestation_bump];
    let signer_seeds = [
        Seed::from(ATTESTATION_SEED),
        Seed::from(credential_info.key()),
        Seed::from(authorized_signer.key()),
        Seed::from(schema_info.key()),
        Seed::from(nonce),
        Seed::from(&bump_seed),
    ];

    let rent = Rent::get()?;
    create_pda_account(
        payer_info,
        &rent,
        space,
        program_id,
        attestation_info,
        signer_seeds,
        None,
    )?;

    let attestation = Attestation {
        nonce: *nonce,
        credential: *credential_info.key(),
        schema: *schema_info.key(),
        data: data.to_vec(),
        signer: *authorized_signer.key(),
        expiry,
    };

    // Validate the Attestation data matches the layout of the Schema
    attestation.validate_data(schema.layout)?;

    let mut attestation_data = attestation_info.try_borrow_mut_data()?;
    attestation_data.copy_from_slice(&attestation.to_bytes());

    Ok(())
}

/// Instruction data for the `CreateAttestation` instruction.
pub struct CreateAttestationArgs<'a> {
    raw: *const u8,

    _data: PhantomData<&'a [u8]>,
}

impl CreateAttestationArgs<'_> {
    #[inline]
    pub fn try_from_bytes(bytes: &[u8]) -> Result<CreateAttestationArgs, ProgramError> {
        // The minimum expected size of the instruction data.
        // - nonce (32 bytes)
        // - data (5 bytes. 4 len, 1 byte)
        // - expiry (8 bytes)
        if bytes.len() < 45 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(CreateAttestationArgs {
            raw: bytes.as_ptr(),
            _data: PhantomData,
        })
    }

    #[inline]
    pub fn nonce(&self) -> Result<&Pubkey, ProgramError> {
        // SAFETY: the `bytes` length was validated in `try_from_bytes`.
        unsafe {
            let nonce = &*(self.raw as *const Pubkey);
            Ok(nonce)
        }
    }

    pub fn _data_len(&self) -> usize {
        unsafe { *(self.raw.add(32) as *const u32) as usize }
    }

    #[inline]
    pub fn data(&self) -> Result<&[u8], ProgramError> {
        // SAFETY: the `bytes` length was validated in `try_from_bytes`.
        unsafe {
            let len = self._data_len();
            let data_bytes = core::slice::from_raw_parts(self.raw.add(36), len as usize);
            Ok(data_bytes)
        }
    }

    #[inline]
    pub fn expiry(&self) -> Result<i64, ProgramError> {
        // SAFETY: the `bytes` length was validated in `try_from_bytes`.
        unsafe {
            let data_len = self._data_len();
            let offset = 32 + 4 + data_len;
            let expiry = *(self.raw.add(offset) as *const i64);
            Ok(expiry)
        }
    }
}
