extern crate alloc;

use crate::{
    constants::{event_authority_pda, EVENT_AUTHORITY_SEED, LIGHT_CPI_SIGNER},
    error::AttestationServiceError,
    events::{CloseAttestationEvent, EventDiscriminators},
    state::{Attestation, Credential},
};
use alloc::vec::Vec;
use light_compressed_account::{
    compressed_account::PackedMerkleContext,
    instruction_data::{
        compressed_proof::ValidityProof,
        with_account_info::{CompressedAccountInfo, InAccountInfo},
    },
};
use light_sdk_pinocchio::cpi::{
    v2::{CpiAccounts, LightSystemProgramCpi},
    InvokeLightSystemProgram, LightCpiInstruction,
};
use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Seed, Signer},
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};
use shank::ShankType;

use super::{verify_owner_mutability, verify_signer};

#[inline(always)]
pub fn process_close_compressed_attestation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let args = process_instruction_data(instruction_data)?;

    // Account destructuring - 12 accounts
    if accounts.len() != 12 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let (constant_accounts, light_cpi_accounts) = accounts.split_at(4);
    let [payer_info, authority, credential_info, event_authority_info] = constant_accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Check event authority PDA
    if event_authority_info.key().ne(&event_authority_pda::ID) {
        return Err(AttestationServiceError::InvalidEventAuthority.into());
    }

    // Security validations
    verify_signer(authority, false)?;
    verify_owner_mutability(credential_info, program_id, false)?;

    let credential_data = credential_info.try_borrow_data()?;
    let credential = Credential::try_from_bytes(&credential_data)?;
    credential.validate_authorized_signer(authority.key())?;

    // Parse and validate attestation
    let attestation = args.attestation.into_attestation(*credential_info.key());

    // Hash attestation data.
    let data_hash = attestation.hash();

    let in_account_info = InAccountInfo {
        discriminator: Attestation::COMPRESSION_DISCRIMINATOR,
        data_hash,
        merkle_context: PackedMerkleContext {
            prove_by_index: args.proof.0.is_none(),
            leaf_index: args.leaf_index,
            queue_pubkey_index: 1,
            merkle_tree_pubkey_index: 0,
        },
        root_index: args.root_index,
        lamports: 0,
    };

    let compressed_account = CompressedAccountInfo {
        address: Some(args.compressed_address),
        input: Some(in_account_info),
        output: None, // Closing - no output
    };

    let light_cpi_accounts = CpiAccounts::new(payer_info, light_cpi_accounts, LIGHT_CPI_SIGNER);

    LightSystemProgramCpi::new_cpi(LIGHT_CPI_SIGNER, args.proof)
        .with_account_infos(&[compressed_account])
        .invoke(light_cpi_accounts)?;

    let event = CloseAttestationEvent {
        discriminator: EventDiscriminators::CloseEvent as u8,
        schema: attestation.schema,
        attestation_data: attestation.data,
    };

    invoke_signed(
        &Instruction {
            program_id,
            accounts: &[AccountMeta::new(event_authority_info.key(), false, true)],
            data: event.to_bytes().as_slice(),
        },
        &[event_authority_info],
        &[Signer::from(&[
            Seed::from(EVENT_AUTHORITY_SEED),
            Seed::from(&[event_authority_pda::BUMP]),
        ])],
    )?;

    Ok(())
}

struct CloseCompressedAttestationArgs {
    proof: ValidityProof,
    root_index: u16,
    leaf_index: u32,
    compressed_address: [u8; 32],
    attestation: CloseAttestation,
}

fn process_instruction_data(data: &[u8]) -> Result<CloseCompressedAttestationArgs, ProgramError> {
    // Minimum size: 1 (is_some) + 2 (root_index) + 4 (leaf_index) + 32 (address)
    //               + 32 (nonce) + 32 (schema) + 32 (signer) + 8 (expiry) + 4 (data_len) + 0 (min data)
    const MIN_INSTRUCTION_SIZE: usize = 7 + 32 + 108;

    if data.len() < MIN_INSTRUCTION_SIZE {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Parse ValidityProof (Option<CompressedProof>)
    // First byte indicates if Some or None
    let (is_some_bytes, remaining) = data.split_at(1);
    let is_some = is_some_bytes[0] != 0;

    // If proof is Some, validate we have enough bytes for the 128-byte proof
    if is_some && data.len() < MIN_INSTRUCTION_SIZE + 128 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let (proof, remaining) = if is_some {
        // Parse CompressedProof (32 + 64 + 32 = 128 bytes)
        let (proof_a_bytes, remaining) = remaining.split_at(32);
        let (proof_b_bytes, remaining) = remaining.split_at(64);
        let (proof_c_bytes, remaining) = remaining.split_at(32);

        let compressed_proof =
            light_compressed_account::instruction_data::compressed_proof::CompressedProof {
                a: proof_a_bytes
                    .try_into()
                    .map_err(|_| ProgramError::InvalidInstructionData)?,
                b: proof_b_bytes
                    .try_into()
                    .map_err(|_| ProgramError::InvalidInstructionData)?,
                c: proof_c_bytes
                    .try_into()
                    .map_err(|_| ProgramError::InvalidInstructionData)?,
            };

        (ValidityProof(Some(compressed_proof)), remaining)
    } else {
        (ValidityProof(None), remaining)
    };

    // Parse root_index (2 bytes)
    let (root_index_bytes, remaining) = remaining.split_at(2);
    let root_index = u16::from_le_bytes(
        root_index_bytes
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    );

    // Parse leaf_index (4 bytes)
    let (leaf_index_bytes, remaining) = remaining.split_at(4);
    let leaf_index = u32::from_le_bytes(
        leaf_index_bytes
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    );

    // Parse compressed_address (32 bytes)
    let (address_bytes, remaining) = remaining.split_at(32);
    let compressed_address: [u8; 32] = address_bytes
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    let attestation = CloseAttestation::try_from_slice(remaining)?;

    Ok(CloseCompressedAttestationArgs {
        proof,
        attestation,
        leaf_index,
        root_index,
        compressed_address,
    })
}

#[derive(Clone, Debug, PartialEq, ShankType)]
pub struct CloseAttestation {
    /// A pubkey that may either be randomly generated OR associated with a User's wallet
    pub nonce: Pubkey,
    /// Reference to the Schema this Attestation adheres to
    pub schema: Pubkey,
    /// The pubkey of the signer. Must be one of the `authority`s at time of attestation
    pub signer: Pubkey,
    /// Designates when the credential is expired. 0 means never expired
    pub expiry: i64,
    /// Data that was verified and matches the Schema
    pub data: Vec<u8>,
}

impl CloseAttestation {
    pub fn try_from_slice(data: &[u8]) -> Result<Self, ProgramError> {
        let mut offset = 0;

        // Parse nonce (32 bytes)
        let nonce = Pubkey::try_from(
            data.get(offset..offset + 32)
                .ok_or(ProgramError::InvalidInstructionData)?,
        )
        .map_err(|_| ProgramError::InvalidInstructionData)?;
        offset += 32;

        // Parse schema (32 bytes)
        let schema = Pubkey::try_from(
            data.get(offset..offset + 32)
                .ok_or(ProgramError::InvalidInstructionData)?,
        )
        .map_err(|_| ProgramError::InvalidInstructionData)?;
        offset += 32;

        // Parse signer (32 bytes)
        let signer = Pubkey::try_from(
            data.get(offset..offset + 32)
                .ok_or(ProgramError::InvalidInstructionData)?,
        )
        .map_err(|_| ProgramError::InvalidInstructionData)?;
        offset += 32;

        // Parse expiry (8 bytes)
        let expiry = i64::from_le_bytes(
            data.get(offset..offset + 8)
                .ok_or(ProgramError::InvalidInstructionData)?
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );
        offset += 8;

        // Parse data length (4 bytes)
        let data_len = u32::from_le_bytes(
            data.get(offset..offset + 4)
                .ok_or(ProgramError::InvalidInstructionData)?
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        ) as usize;
        offset += 4;

        // Parse data (variable length)
        let data = data
            .get(offset..offset + data_len)
            .ok_or(ProgramError::InvalidInstructionData)?
            .to_vec();

        Ok(Self {
            nonce,
            schema,
            signer,
            expiry,
            data,
        })
    }
}

impl CloseAttestation {
    pub fn into_attestation(self, credential: Pubkey) -> Attestation {
        Attestation {
            nonce: self.nonce,
            credential,
            schema: self.schema,
            data: self.data,
            signer: self.signer,
            expiry: self.expiry,
            token_account: Pubkey::default(),
        }
    }
}
