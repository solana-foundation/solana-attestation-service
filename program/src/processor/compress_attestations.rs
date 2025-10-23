use crate::{
    constants::{
        event_authority_pda, EVENT_AUTHORITY_SEED, LIGHT_CPI_SIGNER,
        MAX_COMPRESSED_ATTESTATION_SIZE,
    },
    error::AttestationServiceError,
    events::{CompressAttestation, CompressAttestationEvent, EventDiscriminators},
    state::{Attestation, Credential},
};
extern crate alloc;

use alloc::vec::Vec;
use light_sdk_pinocchio::{
    address::v2::derive_address,
    cpi::{
        v2::{CpiAccounts, LightSystemProgramCpi},
        InvokeLightSystemProgram, LightCpiInstruction,
    },
    instruction::{CompressedProof, NewAddressParamsAssignedPacked},
};
use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Seed, Signer},
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use super::{
    create_compressed_account_from_attestation, verify_address_tree, verify_owner_mutability,
    verify_signer,
};

#[inline(always)]
pub fn process_compress_attestations(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let args = process_instruction_data(instruction_data)?;

    // Validate num_attestations is non-zero
    if args.num_attestations == 0 {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Expected: 4 constant + 8 light CPI + N attestations = 12 + N
    // Minimum: 12 + 1 = 13 accounts
    if accounts.len() < 13 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    // Verify num_attestations matches actual attestation accounts provided
    let expected_total_accounts = 12 + args.num_attestations as usize;
    if accounts.len() != expected_total_accounts {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    // Account destructuring
    // First 5: constant accounts
    // Next 8: light CPI accounts (6 system + 2 trees)
    // Remaining: N attestation accounts
    let (constant_accounts, remaining) = accounts.split_at(4);
    let (light_and_tree_accounts, attestation_accounts) = remaining.split_at(8);

    let [payer_info, authority, credential_info, event_authority_info] = constant_accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Security validations
    verify_signer(payer_info, true)?;
    verify_signer(authority, false)?;
    verify_owner_mutability(credential_info, program_id, false)?;

    let credential_data = credential_info.try_borrow_data()?;
    let credential = Credential::try_from_bytes(&credential_data)?;
    credential.validate_authorized_signer(authority.key())?;

    // Check event authority PDA
    if event_authority_info.key().ne(&event_authority_pda::ID) {
        return Err(AttestationServiceError::InvalidEventAuthority.into());
    }

    // Set up Light Protocol CPI accounts
    // light_and_tree_accounts contains 6 light system accounts + 2 tree accounts
    // Tree accounts: Index 0 = output_queue, Index 1 = address_merkle_tree
    let light_cpi_accounts =
        CpiAccounts::new(payer_info, light_and_tree_accounts, LIGHT_CPI_SIGNER);

    // Verify address tree matches allowed tree
    let address_tree = verify_address_tree(&light_cpi_accounts)?;

    // Vectors to collect compressed accounts and new addresses
    let mut compressed_accounts = Vec::with_capacity(args.num_attestations as usize);
    let mut new_address_params = Vec::with_capacity(args.num_attestations as usize);

    // Loop through all attestation PDAs
    for attestation_info in attestation_accounts {
        // Validate attestation account ownership
        verify_owner_mutability(attestation_info, program_id, args.close_accounts)?;

        let attestation_data = attestation_info.try_borrow_data()?;
        let attestation = Attestation::try_from_bytes(&attestation_data)?;

        // Validate attestation belongs to this credential
        if attestation.credential.ne(credential_info.key()) {
            return Err(AttestationServiceError::InvalidCredential.into());
        }

        // Validate attestation is not tokenized (compressed attestations cannot be tokenized)
        if attestation.token_account.ne(&Pubkey::default()) {
            return Err(AttestationServiceError::InvalidTokenAccount.into());
        }

        // Validate attestation data size to ensure it fits in transaction limits
        if attestation.data.len() > MAX_COMPRESSED_ATTESTATION_SIZE {
            return Err(AttestationServiceError::AttestationDataTooLarge.into());
        }

        // Derive the compressed address from the attestation PDA
        let (address, address_seed) =
            derive_address(&[attestation_info.key()], &address_tree, program_id);

        let new_address_param = NewAddressParamsAssignedPacked {
            address_merkle_tree_account_index: 1,
            address_queue_account_index: 1,
            address_merkle_tree_root_index: args.address_root_index,
            seed: address_seed.0,
            assigned_account_index: compressed_accounts.len() as u8,
            assigned_to_account: true,
        };

        let compressed_account =
            create_compressed_account_from_attestation(&attestation, address, 0);

        compressed_accounts.push(compressed_account);
        new_address_params.push(new_address_param);
    }

    // Execute single Light System Program CPI for all compressed accounts
    LightSystemProgramCpi::new_cpi(LIGHT_CPI_SIGNER, args.proof.into())
        .with_account_infos(&compressed_accounts)
        .with_new_addresses(&new_address_params)
        .invoke(light_cpi_accounts)?;

    // Collect event data while closing accounts
    let mut event_attestations = Vec::with_capacity(args.num_attestations as usize);

    for attestation_info in attestation_accounts {
        // Read attestation for event data BEFORE closing
        let attestation_data = attestation_info.try_borrow_data()?;
        let attestation = Attestation::try_from_bytes(&attestation_data)?;

        event_attestations.push(CompressAttestation {
            schema: attestation.schema,
            attestation_data: attestation.data.clone(),
        });
        drop(attestation_data);

        if args.close_accounts {
            // Close account and transfer rent to payer
            let payer_lamports = payer_info.lamports();
            let attestation_lamports = attestation_info.lamports();
            *payer_info.try_borrow_mut_lamports()? = payer_lamports
                .checked_add(attestation_lamports)
                .ok_or(ProgramError::ArithmeticOverflow)?;
            *attestation_info.try_borrow_mut_lamports()? = 0;
            attestation_info.close()?;
        }
    }

    // Emit single CompressAttestationEvent for the batch
    let event = CompressAttestationEvent {
        discriminator: EventDiscriminators::CompressEvent as u8,
        pdas_closed: true,
        attestations: event_attestations,
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

struct CompressAttestationsArgs {
    proof: CompressedProof,
    close_accounts: bool,
    address_root_index: u16,
    num_attestations: u8,
}

fn process_instruction_data(data: &[u8]) -> Result<CompressAttestationsArgs, ProgramError> {
    // Expected: proof(128) + close_accounts(1) + address_root_index(2) + num_attestations(1) = 132 bytes
    if data.len() < 132 {
        return Err(ProgramError::InvalidInstructionData);
    }
    // Parse CompressedProof (128 bytes: 32 + 64 + 32)
    let (proof_bytes, remaining) = data.split_at(128);
    let proof =
        CompressedProof::try_from(proof_bytes).map_err(|e| ProgramError::Custom(u32::from(e)))?;

    // Parse close_accounts (1 byte)
    let (close_bytes, remaining) = remaining.split_at(1);
    let close_accounts = close_bytes[0] != 0;

    // Parse address_root_index (2 bytes)
    let (root_index_bytes, remaining) = remaining.split_at(2);
    let address_root_index = u16::from_le_bytes(
        root_index_bytes
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    );

    // Parse num_attestations (1 byte)
    let num_attestations = remaining[0];

    Ok(CompressAttestationsArgs {
        proof,
        close_accounts,
        address_root_index,
        num_attestations,
    })
}
