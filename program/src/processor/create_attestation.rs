use pinocchio::{
    cpi::Seed,
    error::ProgramError,
    sysvars::{clock::Clock, rent::Rent, Sysvar},
    AccountView, Address, ProgramResult,
};

use crate::{
    constants::ATTESTATION_SEED,
    error::AttestationServiceError,
    require_len,
    state::{discriminator::AccountSerialize, Attestation, Credential, Schema},
};

use super::{create_pda_account, verify_owner_mutability, verify_signer, verify_system_program};

#[inline(always)]
pub fn process_create_attestation(
    program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
    token_account: Option<Address>,
) -> ProgramResult {
    let args = process_instruction_data(instruction_data)?;
    let [payer_info, authorized_signer, credential_info, schema_info, attestation_info, system_program] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    verify_signer(authorized_signer)?;
    verify_system_program(system_program)?;
    verify_owner_mutability(credential_info, program_id, false)?;
    verify_owner_mutability(schema_info, program_id, false)?;

    let credential = Credential::try_from_bytes(&credential_info.try_borrow()?)?;
    credential.validate_authorized_signer(authorized_signer.address())?;

    let schema = Schema::try_from_bytes(&schema_info.try_borrow()?)?;
    if schema.is_paused {
        return Err(AttestationServiceError::SchemaPaused.into());
    }
    if schema.credential.ne(credential_info.address()) {
        return Err(AttestationServiceError::InvalidCredential.into());
    }

    let clock = Clock::get()?;
    if args.expiry < clock.unix_timestamp && args.expiry != 0 {
        return Err(AttestationServiceError::InvalidAttestationData.into());
    }

    let (attestation_pda, attestation_bump) = Address::find_program_address(
        &[ATTESTATION_SEED, credential_info.address().as_ref(), schema_info.address().as_ref(), args.nonce.as_ref()],
        program_id,
    );

    if attestation_info.address().ne(&attestation_pda) {
        return Err(AttestationServiceError::InvalidAttestation.into());
    }

    let attestation = Attestation {
        nonce: args.nonce,
        credential: *credential_info.address(),
        schema: *schema_info.address(),
        data: args.data.to_vec(),
        signer: *authorized_signer.address(),
        expiry: args.expiry,
        token_account: token_account.unwrap_or_default(),
    };
    attestation.validate_data(&schema.layout)?;
    let attestation_bytes = attestation.to_bytes();

    let bump_seed = [attestation_bump];
    let signer_seeds = [
        Seed::from(ATTESTATION_SEED),
        Seed::from(credential_info.address().as_ref()),
        Seed::from(schema_info.address().as_ref()),
        Seed::from(args.nonce.as_ref()),
        Seed::from(&bump_seed),
    ];

    let rent = Rent::get()?;
    create_pda_account(payer_info, &rent, attestation_bytes.len(), program_id, attestation_info, signer_seeds, None)?;
    attestation_info.try_borrow_mut()?.copy_from_slice(&attestation_bytes);

    Ok(())
}

struct CreateAttestationArgs<'a> {
    nonce: Address,
    data: &'a [u8],
    expiry: i64,
}

fn process_instruction_data(data: &[u8]) -> Result<CreateAttestationArgs<'_>, ProgramError> {
    let mut offset: usize = 0;

    require_len!(data, 32);
    let nonce: Address = data[offset..offset + 32].try_into().unwrap();
    offset += 32;

    require_len!(data, offset + 4);
    let data_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;

    require_len!(data, offset + data_len);
    let data_bytes = &data[offset..offset + data_len];
    offset += data_len;

    require_len!(data, offset + 8);
    let expiry = i64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());

    Ok(CreateAttestationArgs { nonce, data: data_bytes, expiry })
}
