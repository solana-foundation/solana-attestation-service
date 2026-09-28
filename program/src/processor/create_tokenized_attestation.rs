use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, Address, ProgramResult,
};
use pinocchio_associated_token_account::instructions::CreateIdempotent;
use pinocchio_token_2022::{
    instructions::{
        group_member_pointer::Initialize as InitializeGroupMemberPointer,
        metadata_pointer::Initialize as InitializeMetadataPointer, mint_close_authority::InitializeMintCloseAuthority,
        permanent_delegate::InitializePermanentDelegate, InitializeMint2, InitializeNonTransferableMint, MintToChecked,
    },
    ID as TOKEN_2022_PROGRAM_ID,
};

use crate::{
    constants::{sas_pda, ATTESTATION_MINT_SEED, SAS_SEED, SCHEMA_MINT_SEED},
    error::AttestationServiceError,
    processor::process_create_attestation,
    require_len,
};

use super::{
    create_pda_account,
    token_ext::{InitializeMember, InitializeTokenMetadata, UpdateField},
    verify_ata_program, verify_token22_program,
};

#[inline(always)]
pub fn process_create_tokenized_attestation(
    program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let [payer_info, authorized_signer, credential_info, schema_info, attestation_info, system_program, schema_mint_info, attestation_mint_info, sas_pda_info, recipient_token_account_info, recipient_info, token_program, ata_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Create Attestation first
    process_create_attestation(
        program_id,
        &mut [*payer_info, *authorized_signer, *credential_info, *schema_info, *attestation_info, *system_program],
        instruction_data,
        Some(*recipient_token_account_info.address()),
    )?;

    let args = process_instruction_data(instruction_data)?;

    // Validate Recipient TokenAccount is writable
    if !recipient_token_account_info.is_writable() {
        return Err(ProgramError::InvalidAccountData);
    }

    // Verify token programs.
    verify_token22_program(token_program)?;
    verify_ata_program(ata_program)?;

    // Validate that mint matches expected PDA
    let (attestation_mint_pda, attestation_mint_bump) =
        Address::find_program_address(&[ATTESTATION_MINT_SEED, attestation_info.address().as_ref()], program_id);
    if attestation_mint_info.address().ne(&attestation_mint_pda) {
        return Err(AttestationServiceError::InvalidMint.into());
    }
    let (schema_mint_pda, _) =
        Address::find_program_address(&[SCHEMA_MINT_SEED, schema_info.address().as_ref()], program_id);

    if schema_mint_info.address().ne(&schema_mint_pda) {
        return Err(AttestationServiceError::InvalidMint.into());
    }

    // Validate that sas_pda matches
    if sas_pda_info.address().ne(&sas_pda::ID) {
        return Err(AttestationServiceError::InvalidProgramSigner.into());
    }

    // Initialize new account owned by token_program.
    create_pda_account(
        payer_info,
        &Rent::get()?,
        378, // Size before Token extensions after InitializeMint2
        &TOKEN_2022_PROGRAM_ID,
        attestation_mint_info,
        [
            Seed::from(ATTESTATION_MINT_SEED),
            Seed::from(attestation_info.address().as_ref()),
            Seed::from(&[attestation_mint_bump]),
        ],
        // Sufficient rent needs to be allocated or instruction fails with
        // "Lamport balance below rent-exempt threshold" or "InsufficientFundsForRent".
        Some(args.mint_account_space.into()),
    )?;

    // Initialize GroupMemberPointer extension
    InitializeGroupMemberPointer {
        mint: attestation_mint_info,
        authority: Some(&sas_pda::ID),
        member_address: Some(attestation_mint_info.address()),
        token_program: &TOKEN_2022_PROGRAM_ID,
    }
    .invoke()?;

    // Initialize NonTransferable extension
    InitializeNonTransferableMint { mint: attestation_mint_info, token_program: &TOKEN_2022_PROGRAM_ID }.invoke()?;

    // Initialize MetadataPointer extension
    InitializeMetadataPointer {
        mint: attestation_mint_info,
        authority: Some(sas_pda_info.address()),
        metadata_address: Some(attestation_mint_info.address()),
        token_program: &TOKEN_2022_PROGRAM_ID,
    }
    .invoke()?;

    // Initialize Permanent Delegate extension
    InitializePermanentDelegate {
        mint: attestation_mint_info,
        delegate: sas_pda_info.address(),
        token_program: &TOKEN_2022_PROGRAM_ID,
    }
    .invoke()?;

    // Initialize Mint Close extension
    InitializeMintCloseAuthority {
        mint: attestation_mint_info,
        close_authority: Some(sas_pda_info.address()),
        token_program: &TOKEN_2022_PROGRAM_ID,
    }
    .invoke()?;

    // Initialize Mint on created account
    InitializeMint2::new(attestation_mint_info, 0, sas_pda_info.address(), Some(sas_pda_info.address())).invoke()?;

    // Initialize TokenMetadata extension
    let bump_seed = [sas_pda::BUMP];
    let sas_pda_seeds = [Seed::from(SAS_SEED), Seed::from(&bump_seed)];

    InitializeTokenMetadata {
        metadata: attestation_mint_info,
        update_authority: sas_pda_info,
        mint: attestation_mint_info,
        mint_authority: sas_pda_info,
        name: core::str::from_utf8(args.name).unwrap(),
        symbol: core::str::from_utf8(args.symbol).unwrap(),
        uri: core::str::from_utf8(args.uri).unwrap(),
    }
    .invoke_signed(&[Signer::from(&sas_pda_seeds)])?;

    // Set attestation and schema metadata using UpdateField extension
    UpdateField {
        metadata: attestation_mint_info,
        update_authority: sas_pda_info,
        key: "attestation",
        value: &bs58::encode(attestation_info.address()).into_string(),
    }
    .invoke_signed(&[Signer::from(&sas_pda_seeds)])?;

    UpdateField {
        metadata: attestation_mint_info,
        update_authority: sas_pda_info,
        key: "schema",
        value: &bs58::encode(schema_info.address()).into_string(),
    }
    .invoke_signed(&[Signer::from(&sas_pda_seeds)])?;

    // Initialize TokenGroupMember extension
    InitializeMember {
        group: schema_mint_info,
        group_update_authority: sas_pda_info,
        member: attestation_mint_info,
        member_mint: attestation_mint_info,
        member_mint_authority: sas_pda_info,
    }
    .invoke_signed(&[Signer::from(&sas_pda_seeds)])?;

    // Only create the ATA when the TokenAccount is owned by the System program with empty data.
    // Create new associated token account to hold Attestation token.
    CreateIdempotent {
        funding_account: payer_info,
        account: recipient_token_account_info,
        wallet: recipient_info,
        mint: attestation_mint_info,
        system_program,
        token_program,
    }
    .invoke()?;

    // Mint to recipient token account.
    MintToChecked::new(attestation_mint_info, recipient_token_account_info, sas_pda_info, 1, 0)
        .invoke_signed(&[Signer::from(&sas_pda_seeds)])?;

    Ok(())
}

struct CreateTokenizedAttestationArgs<'a> {
    name: &'a [u8],
    uri: &'a [u8],
    symbol: &'a [u8],
    mint_account_space: u16,
}

fn process_instruction_data(data: &[u8]) -> Result<CreateTokenizedAttestationArgs<'_>, ProgramError> {
    let mut offset: usize = 32; // Skip Nonce

    require_len!(data, offset + 4);
    let data_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4 + data_len; // Skip Data field
    offset += 8; // Skip Expiry

    require_len!(data, offset + 4);
    let name_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;

    require_len!(data, offset + name_len);
    let name = &data[offset..offset + name_len];
    offset += name_len;

    require_len!(data, offset + 4);
    let uri_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;

    require_len!(data, offset + uri_len);
    let uri = &data[offset..offset + uri_len];
    offset += uri_len;

    require_len!(data, offset + 4);
    let symbol_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;

    require_len!(data, offset + symbol_len);
    let symbol = &data[offset..offset + symbol_len];
    offset += symbol_len;

    require_len!(data, offset + 2);
    let mint_account_space = u16::from_le_bytes(data[offset..offset + 2].try_into().unwrap());

    Ok(CreateTokenizedAttestationArgs { name, uri, symbol, mint_account_space })
}
