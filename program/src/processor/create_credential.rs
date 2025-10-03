extern crate alloc;

use alloc::vec::Vec;
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
    processor::{create_pda_account, verify_signer, verify_system_account, verify_system_program},
    require_len,
    state::{discriminator::AccountSerialize, Credential},
};

#[inline(always)]
pub fn process_create_credential(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let args = process_instruction_data(instruction_data)?;
    let [payer_info, credential_info, authority_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate: should be owned by system account, empty, and writable
    verify_system_account(credential_info, true)?;
    // Validate: authority should have signed
    verify_signer(authority_info, false)?;
    // Validate: system program
    verify_system_program(system_program)?;

    // NOTE: this could be optimized further by removing the `solana-program` dependency
    // and using `pubkey::checked_create_program_address` from Pinocchio to verify the
    // pubkey and associated bump (needed to be added as arg) is valid.
    let (credential_pda, credential_bump) = SolanaPubkey::find_program_address(
        &[CREDENTIAL_SEED, authority_info.key(), args.name],
        &SolanaPubkey::from(*program_id),
    );

    if credential_info.key().ne(&credential_pda.to_bytes()) {
        // PDA was invalid
        return Err(AttestationServiceError::InvalidCredential.into());
    }

    // Account layout
    // discriminator - 1
    // authorized_signers - 4 + 32 * len
    // authority - 32
    // name - 4 + len
    let space = 1 + (4 + args.signers.len() * 32) + 32 + (4 + args.name.len());

    let rent = Rent::get()?;
    let bump_seed = [credential_bump];
    let signer_seeds = [
        Seed::from(CREDENTIAL_SEED),
        Seed::from(authority_info.key()),
        Seed::from(args.name),
        Seed::from(&bump_seed),
    ];
    create_pda_account(
        payer_info,
        &rent,
        space,
        program_id,
        credential_info,
        signer_seeds,
        None,
    )?;

    let credential = Credential {
        authority: *authority_info.key(),
        name: args.name.to_vec(),
        authorized_signers: args.signers,
    };
    let mut credential_data = credential_info.try_borrow_mut_data()?;
    credential_data.copy_from_slice(&credential.to_bytes());

    Ok(())
}

struct CreateCredentialArgs<'a> {
    name: &'a [u8],
    signers: Vec<Pubkey>,
}

fn process_instruction_data<'a>(data: &'a [u8]) -> Result<CreateCredentialArgs<'a>, ProgramError> {
    let mut offset: usize = 0;

    require_len!(data, 4);
    let name_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;

    require_len!(data, offset + name_len);
    let name = &data[offset..offset + name_len];
    offset += name_len;

    require_len!(data, offset + 4);
    let signers_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;

    require_len!(data, offset + signers_len * 32);
    let mut signers = Vec::with_capacity(signers_len);
    for _ in 0..signers_len {
        let signer: Pubkey = data[offset..offset + 32].try_into().unwrap();
        signers.push(signer);
        offset += 32;
    }

    Ok(CreateCredentialArgs { name, signers })
}
