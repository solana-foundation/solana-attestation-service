use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_associated_token_account::instructions::CreateIdempotent;
use pinocchio_token::{
    extensions::{
        group_member_pointer::Initialize as InitializeGroupMemberPointer,
        metadata::{Field, InitializeTokenMetadata, UpdateField},
        metadata_pointer::Initialize as InitializeMetadataPointer,
        mint_close_authority::InitializeMintCloseAuthority,
        non_transferable::InitializeNonTransferableMint,
        permanent_delegate::InitializePermanentDelegate,
        token_group::InitializeMember,
    },
    instructions::{InitializeMint2, MintToChecked, TokenProgramVariant},
    TOKEN_2022_PROGRAM_ID,
};
use solana_program::pubkey::Pubkey as SolanaPubkey;

use crate::{
    constants::{sas_pda, ATTESTATION_MINT_SEED, SAS_SEED, SCHEMA_MINT_SEED},
    error::AttestationServiceError,
    processor::process_create_attestation,
    require_len,
};

use super::{create_pda_account, verify_ata_program, verify_token22_program};

#[inline(always)]
pub fn process_create_tokenized_attestation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [payer_info, _authorized_signer, _credential_info, schema_info, attestation_info, system_program, schema_mint_info, attestation_mint_info, sas_pda_info, recipient_token_account_info, recipient_info, token_program, ata_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Create Attestation first
    process_create_attestation(
        program_id,
        &accounts[0..6],
        instruction_data,
        Some(*recipient_token_account_info.key()),
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
    let (attestation_mint_pda, attestation_mint_bump) = SolanaPubkey::find_program_address(
        &[ATTESTATION_MINT_SEED, attestation_info.key()],
        &SolanaPubkey::from(*program_id),
    );
    if attestation_mint_info
        .key()
        .ne(&attestation_mint_pda.to_bytes())
    {
        return Err(AttestationServiceError::InvalidMint.into());
    }
    let (schema_mint_pda, _) = SolanaPubkey::find_program_address(
        &[SCHEMA_MINT_SEED, schema_info.key()],
        &SolanaPubkey::from(*program_id),
    );

    if schema_mint_info.key().ne(&schema_mint_pda.to_bytes()) {
        return Err(AttestationServiceError::InvalidMint.into());
    }

    // Validate that sas_pda matches
    if sas_pda_info.key().ne(&sas_pda::ID) {
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
            Seed::from(attestation_info.key()),
            Seed::from(&[attestation_mint_bump]),
        ],
        // Sufficient rent needs to be allocated or instruction fails with
        // "Lamport balance below rent-exempt threshold" or "InsufficientFundsForRent".
        Some(args.mint_account_space.into()),
    )?;

    // Initialize GroupMemberPointer extension
    InitializeGroupMemberPointer {
        mint: attestation_mint_info,
        authority: Some(sas_pda::ID),
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

    // Initialize Permanent Delegate extension
    InitializePermanentDelegate {
        mint: attestation_mint_info,
        delegate: *sas_pda_info.key(),
    }
    .invoke()?;

    // Initialize Mint Close extension
    InitializeMintCloseAuthority {
        mint: attestation_mint_info,
        close_authority: Some(*sas_pda_info.key()),
    }
    .invoke()?;

    // Initialize Mint on created account
    InitializeMint2 {
        mint: attestation_mint_info,
        decimals: 0,
        mint_authority: sas_pda_info.key(),
        freeze_authority: Some(sas_pda_info.key()),
    }
    .invoke(TokenProgramVariant::Token2022)?;

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
        field: Field::Key("attestation"),
        value: &bs58::encode(attestation_info.key()).into_string(),
    }
    .invoke_signed(&[Signer::from(&sas_pda_seeds)])?;

    UpdateField {
        metadata: attestation_mint_info,
        update_authority: sas_pda_info,
        field: Field::Key("schema"),
        value: &bs58::encode(schema_info.key()).into_string(),
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
    MintToChecked {
        mint: attestation_mint_info,
        account: recipient_token_account_info,
        mint_authority: sas_pda_info,
        amount: 1,
        decimals: 0,
    }
    .invoke_signed(
        &[Signer::from(&sas_pda_seeds)],
        TokenProgramVariant::Token2022,
    )?;

    Ok(())
}

struct CreateTokenizedAttestationArgs<'a> {
    name: &'a [u8],
    uri: &'a [u8],
    symbol: &'a [u8],
    mint_account_space: u16,
}

fn process_instruction_data<'a>(
    data: &'a [u8],
) -> Result<CreateTokenizedAttestationArgs<'a>, ProgramError> {
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

    Ok(CreateTokenizedAttestationArgs {
        name,
        uri,
        symbol,
        mint_account_space,
    })
}
