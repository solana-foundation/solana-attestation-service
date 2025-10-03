use pinocchio::{
    account_info::AccountInfo,
    instruction::Seed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{clock::Clock, rent::Rent, Sysvar},
    ProgramResult,
};
use solana_program::pubkey::Pubkey as SolanaPubkey;

use crate::{
    constants::ATTESTATION_SEED,
    error::AttestationServiceError,
    require_len,
    state::{discriminator::AccountSerialize, Attestation, Credential, Schema},
};

use super::{create_pda_account, verify_owner_mutability, verify_signer, verify_system_program};

#[inline(always)]
pub fn process_create_attestation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
    token_account: Option<Pubkey>,
) -> ProgramResult {
    let args = process_instruction_data(instruction_data)?;
    let [payer_info, authority, credential_info, schema_info, attestation_info, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate: authority should have signed
    verify_signer(authority, false)?;

    // Validate system program
    verify_system_program(system_program)?;
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

    // NOTE: this could be optimized further by removing the `solana-program` dependency
    // and using `pubkey::checked_create_program_address` from Pinocchio to verify the
    // pubkey and associated bump (needed to be added as arg) is valid.
    let (attestation_pda, attestation_bump) = SolanaPubkey::find_program_address(
        &[
            ATTESTATION_SEED,
            credential_info.key(),
            schema_info.key(),
            &args.nonce,
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
    // token account - 32
    let space = 1 + 32 + 32 + 32 + (4 + args.data.len()) + 32 + 8 + 32;

    let bump_seed = [attestation_bump];
    let signer_seeds = [
        Seed::from(ATTESTATION_SEED),
        Seed::from(credential_info.key()),
        Seed::from(schema_info.key()),
        Seed::from(&args.nonce),
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
        nonce: args.nonce,
        credential: *credential_info.key(),
        schema: *schema_info.key(),
        data: args.data.to_vec(),
        signer: *authority.key(),
        expiry: args.expiry,
        token_account: token_account.unwrap_or_default(),
    };

    // Validate the Attestation data matches the layout of the Schema
    attestation.validate_data(schema.layout)?;

    let mut attestation_data = attestation_info.try_borrow_mut_data()?;
    attestation_data.copy_from_slice(&attestation.to_bytes());

    Ok(())
}

struct CreateAttestationArgs<'a> {
    nonce: Pubkey,
    data: &'a [u8],
    expiry: i64,
}

fn process_instruction_data<'a>(data: &'a [u8]) -> Result<CreateAttestationArgs<'a>, ProgramError> {
    let mut offset: usize = 0;

    require_len!(data, 32);
    let nonce: Pubkey = data[offset..offset + 32].try_into().unwrap();
    offset += 32;

    require_len!(data, offset + 4);
    let data_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;

    require_len!(data, offset + data_len);
    let data_bytes = &data[offset..offset + data_len];
    offset += data_len;

    require_len!(data, offset + 8);
    let expiry = i64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());

    Ok(CreateAttestationArgs {
        nonce,
        data: data_bytes,
        expiry,
    })
}
