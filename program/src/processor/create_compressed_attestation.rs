use crate::{
    constants::{
        ALLOWED_ADDRESS_TREE, ATTESTATION_SEED, LIGHT_CPI_SIGNER, MAX_COMPRESSED_ATTESTATION_SIZE,
    },
    error::AttestationServiceError,
    state::{discriminator::AccountSerialize, Attestation, Credential, Schema},
};
use light_sdk_pinocchio::{
    address::v2::derive_address,
    cpi::{
        v2::{CompressedAccountInfo, CpiAccounts, LightSystemProgramCpi, OutAccountInfo},
        InvokeLightSystemProgram, LightCpiInstruction,
    },
    instruction::{CompressedProof, NewAddressParamsAssignedPacked},
};
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{clock::Clock, Sysvar},
    ProgramResult,
};
use solana_pubkey::Pubkey as SolanaPubkey;

use super::{verify_owner_mutability, verify_signer};

#[inline(always)]
pub fn process_create_compressed_attestation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let args = process_instruction_data(instruction_data)?;
    // Expect 12 accounts.
    // 4 attestation
    // 6 light system accounts
    // 2 Merkle tree accounts
    if accounts.len() != 12 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let (constant_accounts, light_cpi_accounts) = accounts.split_at(4);
    let [payer_info, authority, credential_info, schema_info] = constant_accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    // Set up Light Protocol CPI accounts
    // Tree accounts:
    // Index 0: V2 Output queue
    // Index 1: V2 Address Merkle tree
    let light_cpi_accounts = CpiAccounts::new(payer_info, light_cpi_accounts, LIGHT_CPI_SIGNER);

    // Validate: authority should have signed
    verify_signer(authority, false)?;

    // Validate Credential and Schema are owned by our program
    verify_owner_mutability(credential_info, program_id, false)?;
    verify_owner_mutability(schema_info, program_id, false)?;

    let credential_data = credential_info.try_borrow_data()?;
    let credential = Credential::try_from_bytes(&credential_data)?;

    // Validate Authority is an authorized signer
    credential.validate_authorized_signer(authority.key())?;

    let schema_data = schema_info.try_borrow_data()?;
    let schema = Schema::try_from_bytes(&schema_data)?;
    // Validate Schema is not paused
    if schema.is_paused {
        return Err(AttestationServiceError::SchemaPaused.into());
    }

    // Validate Schema is owned by Credential
    if schema.credential.ne(credential_info.key()) {
        return Err(AttestationServiceError::InvalidCredential.into());
    }

    // Validate expiry is greater than current timestamp
    let clock = Clock::get()?;
    if args.expiry < clock.unix_timestamp && args.expiry != 0 {
        return Err(AttestationServiceError::InvalidAttestationData.into());
    }

    // Derive the regular PDA that will be used as seed for compressed address
    let (attestation_pda, _) = SolanaPubkey::find_program_address(
        &[
            ATTESTATION_SEED,
            credential_info.key(),
            schema_info.key(),
            &args.nonce,
        ],
        &SolanaPubkey::from(*program_id),
    );

    // Verify address tree matches allowed tree
    let address_tree = light_cpi_accounts
        .tree_pubkeys()
        .map_err(|e| ProgramError::Custom(u32::from(e)))?[1];
    // Check that all compressed attestations are in the same address tree
    // to ensure compressed pda uniqueness.
    if address_tree != ALLOWED_ADDRESS_TREE {
        return Err(AttestationServiceError::InvalidAddressTree.into());
    }

    // Derive compressed address using the PDA as seed
    let (address, address_seed) =
        derive_address(&[attestation_pda.as_ref()], &address_tree, program_id);

    let new_address_params = NewAddressParamsAssignedPacked {
        address_merkle_tree_account_index: 1,
        address_queue_account_index: 1,
        address_merkle_tree_root_index: args.address_root_index,
        seed: address_seed.0,
        assigned_account_index: 0,
        assigned_to_account: true,
    };

    // Create attestation struct

    let attestation = Attestation {
        nonce: args.nonce,
        credential: *credential_info.key(),
        schema: *schema_info.key(),
        data: args.data.to_vec(),
        signer: *authority.key(),
        expiry: args.expiry,
        token_account: Pubkey::default(),
    };
    // Validate the Attestation data matches the layout of the Schema
    attestation.validate_data(schema.layout)?;

    // Validate attestation data size to ensure it fits in transaction limits
    if attestation.data.len() > MAX_COMPRESSED_ATTESTATION_SIZE {
        return Err(AttestationServiceError::InvalidAttestationData.into());
    }
    let data_hash = attestation.hash();
    let data = attestation.to_bytes();

    let compressed_account = CompressedAccountInfo {
        address: Some(address),
        input: None,
        output: Some(OutAccountInfo {
            output_merkle_tree_index: 0,
            discriminator: Attestation::COMPRESSION_DISCRIMINATOR,
            lamports: 0,
            data,
            data_hash,
        }),
    };

    // Execute Light System Program CPI to create compressed account
    LightSystemProgramCpi::new_cpi(LIGHT_CPI_SIGNER, args.proof.into())
        .with_account_infos(&[compressed_account])
        .with_new_addresses(&[new_address_params])
        .invoke(light_cpi_accounts)?;

    Ok(())
}

struct CreateCompressedAttestationArgs<'a> {
    proof: CompressedProof,
    nonce: Pubkey,
    expiry: i64,
    address_root_index: u16,
    data: &'a [u8],
}

fn process_instruction_data<'a>(
    data: &'a [u8],
) -> Result<CreateCompressedAttestationArgs<'a>, ProgramError> {
    // Minimum size: 32 (proof_a) + 64 (proof_b) + 32 (proof_c) + 32 (nonce)
    //               + 8 (expiry) + 2 (address_root_index) + 4 (data_len) + 0 (min data)
    const MIN_INSTRUCTION_SIZE: usize = 174;

    if data.len() < MIN_INSTRUCTION_SIZE {
        return Err(ProgramError::InvalidInstructionData);
    }

    let (proof_a_bytes, remaining) = data.split_at(32);
    let (proof_b_bytes, remaining) = remaining.split_at(64);
    let (proof_c_bytes, remaining) = remaining.split_at(32);
    let proof = CompressedProof {
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

    // Parse nonce (32 bytes)
    let (nonce_bytes, remaining) = remaining.split_at(32);
    let nonce: Pubkey = nonce_bytes
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    // Parse expiry (8 bytes)
    let (expiry_bytes, remaining) = remaining.split_at(8);
    let expiry = i64::from_le_bytes(
        expiry_bytes
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    );

    // Parse address_root_index (2 bytes)
    let (address_tree_info_bytes, remaining) = remaining.split_at(2);
    let address_root_index = u16::from_le_bytes(
        address_tree_info_bytes
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    );

    // Parse data length (4 bytes)
    let (len_bytes, data) = remaining.split_at(4);
    let data_len = u32::from_le_bytes(
        len_bytes
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    ) as usize;

    // Validate remaining data matches expected length
    if data.len() != data_len {
        return Err(ProgramError::InvalidInstructionData);
    }

    Ok(CreateCompressedAttestationArgs {
        proof,
        nonce,
        expiry,
        address_root_index,
        data,
    })
}
