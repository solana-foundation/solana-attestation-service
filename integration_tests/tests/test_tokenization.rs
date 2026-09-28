use borsh::{BorshDeserialize, BorshSerialize};
use helpers::{program_test_context, TestContext};
use solana_attestation_service_client::{
    accounts::Attestation,
    instructions::{
        CloseTokenizedAttestationBuilder, CreateCredentialBuilder, CreateSchemaBuilder,
        CreateTokenizedAttestationBuilder, TokenizeSchemaBuilder,
    },
    programs::SOLANA_ATTESTATION_SERVICE_ID,
};
use solana_clock::Clock;
use solana_keypair::Keypair;
use solana_program_option::COption;
use solana_program_pack::Pack;
use solana_pubkey::{pubkey, Pubkey};
use solana_sdk_ids::system_program;
use solana_signer::Signer;
use solana_transaction::Transaction;
use spl_token_2022_interface::{
    extension::{
        group_member_pointer::GroupMemberPointer, group_pointer::GroupPointer, metadata_pointer::MetadataPointer,
        mint_close_authority::MintCloseAuthority, non_transferable::NonTransferable,
        permanent_delegate::PermanentDelegate, BaseStateWithExtensions, ExtensionType, StateWithExtensions,
    },
    state::{Account, Mint},
    ID as TOKEN_2022_PROGRAM_ID,
};
use spl_token_group_interface::state::{TokenGroup, TokenGroupMember};
use spl_token_metadata_interface::state::TokenMetadata;

mod helpers;

const ATA_PROGRAM_ID: Pubkey = pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

#[derive(BorshSerialize)]
struct TestData {
    name: String,
    location: u8,
}

struct TestFixtures {
    ctx: TestContext,
    credential: Pubkey,
    schema: Pubkey,
    authority: Keypair,
    schema_mint_pda: Pubkey,
    sas_pda: Pubkey,
    attestation_pda: Pubkey,
    attestation_mint_pda: Pubkey,
    recipient: Pubkey,
    recipient_token_account: Pubkey,
    nonce: Pubkey,
    serialized_attestation_data: Vec<u8>,
}

fn setup() -> TestFixtures {
    let mut ctx = program_test_context();

    let authority = Keypair::new();
    let credential_name = "test";
    let (credential_pda, _bump) = Pubkey::find_program_address(
        &[b"credential", &authority.pubkey().to_bytes(), credential_name.as_bytes()],
        &solana_attestation_service_client::programs::SOLANA_ATTESTATION_SERVICE_ID,
    );

    let create_credential_ix = CreateCredentialBuilder::new()
        .payer(ctx.payer.pubkey())
        .credential(credential_pda)
        .authority(authority.pubkey())
        .system_program(system_program::ID)
        .name(credential_name.to_string())
        .signers(vec![authority.pubkey()])
        .instruction();

    // Create Schema
    let schema_name = "test_data";
    let description = "schema for test data";
    let schema_data = vec![12, 0];
    let field_names = vec!["name".into(), "location".into()];
    let (schema_pda, _bump) = Pubkey::find_program_address(
        &[b"schema", &credential_pda.to_bytes(), schema_name.as_bytes(), &[1]],
        &solana_attestation_service_client::programs::SOLANA_ATTESTATION_SERVICE_ID,
    );
    let create_schema_ix = CreateSchemaBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential_pda)
        .schema(schema_pda)
        .system_program(system_program::ID)
        .description(description.to_string())
        .name(schema_name.to_string())
        .layout(schema_data.clone())
        .field_names(field_names)
        .instruction();

    let transaction = Transaction::new_signed_with_payer(
        &[create_credential_ix, create_schema_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(transaction).unwrap();

    let (sas_pda, _bump) = Pubkey::find_program_address(&[b"sas"], &SOLANA_ATTESTATION_SERVICE_ID);
    let (schema_mint_pda, _bump) =
        Pubkey::find_program_address(&[b"schemaMint", &schema_pda.to_bytes()], &SOLANA_ATTESTATION_SERVICE_ID);

    let nonce = Pubkey::new_unique();
    let attestation_pda = Pubkey::find_program_address(
        &[b"attestation", &credential_pda.to_bytes(), &schema_pda.to_bytes(), &nonce.to_bytes()],
        &SOLANA_ATTESTATION_SERVICE_ID,
    )
    .0;
    let (attestation_mint_pda, _bump) = Pubkey::find_program_address(
        &[b"attestationMint", &attestation_pda.to_bytes()],
        &SOLANA_ATTESTATION_SERVICE_ID,
    );

    let recipient = Pubkey::new_unique();
    let (recipient_token_account, _bump) = Pubkey::find_program_address(
        &[recipient.as_ref(), TOKEN_2022_PROGRAM_ID.as_ref(), attestation_mint_pda.as_ref()],
        &ATA_PROGRAM_ID,
    );

    let attestation_data = TestData { name: "attest".to_string(), location: 11 };
    let mut serialized_attestation_data = Vec::new();
    attestation_data.serialize(&mut serialized_attestation_data).unwrap();

    TestFixtures {
        ctx,
        credential: credential_pda,
        schema: schema_pda,
        authority,
        sas_pda,
        schema_mint_pda,
        attestation_pda,
        attestation_mint_pda,
        recipient,
        recipient_token_account,
        serialized_attestation_data,
        nonce,
    }
}

#[test]
fn tokenize_schema_success() {
    let TestFixtures {
        mut ctx,
        credential,
        schema,
        authority,
        sas_pda,
        schema_mint_pda,
        attestation_pda: _,
        attestation_mint_pda: _,
        recipient: _,
        recipient_token_account: _,
        nonce: _,
        serialized_attestation_data: _,
    } = setup();

    let max_size = 100;
    let tokenize_schema_ix = TokenizeSchemaBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .schema(schema)
        .mint(schema_mint_pda)
        .sas_pda(sas_pda)
        .max_size(max_size)
        .token_program(TOKEN_2022_PROGRAM_ID)
        .instruction();

    let transaction = Transaction::new_signed_with_payer(
        &[tokenize_schema_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(transaction).unwrap();

    let mint_account = ctx.svm.get_account(&schema_mint_pda).unwrap();

    let expected_acc_size =
        ExtensionType::try_calculate_account_len::<Mint>(&[ExtensionType::GroupPointer, ExtensionType::TokenGroup])
            .unwrap();
    assert_eq!(mint_account.data.len(), expected_acc_size);
    assert_eq!(mint_account.owner, TOKEN_2022_PROGRAM_ID);

    let mint_state = StateWithExtensions::<Mint>::unpack(&mint_account.data).unwrap();
    assert!(mint_state.base.is_initialized);
    assert_eq!(mint_state.base.decimals, 0);
    assert_eq!(mint_state.base.supply, 0);
    assert_eq!(mint_state.base.mint_authority, COption::Some(sas_pda));
    assert_eq!(mint_state.base.freeze_authority, COption::Some(sas_pda));

    // Verify the GroupPointer extension.
    let group_pointer = mint_state.get_extension::<GroupPointer>().unwrap();
    assert_eq!(group_pointer.authority.get(), Some(sas_pda));
    assert_eq!(group_pointer.group_address.get(), Some(sas_pda));

    // Verify the TokenGroup extension.
    let token_group = mint_state.get_extension::<TokenGroup>().unwrap();
    assert_eq!(token_group.update_authority.get(), Some(sas_pda));
    assert_eq!(token_group.mint, schema_mint_pda);
    assert_eq!(u64::from(token_group.size), 0);
    assert_eq!(u64::from(token_group.max_size), max_size);
}

#[test]
fn create_tokenized_attestation_success() {
    let TestFixtures {
        mut ctx,
        credential,
        schema,
        authority,
        sas_pda,
        schema_mint_pda,
        attestation_pda,
        attestation_mint_pda,
        recipient,
        recipient_token_account,
        nonce,
        serialized_attestation_data,
    } = setup();

    let tokenize_schema_ix = TokenizeSchemaBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .schema(schema)
        .mint(schema_mint_pda)
        .sas_pda(sas_pda)
        .max_size(100)
        .token_program(TOKEN_2022_PROGRAM_ID)
        .instruction();

    let transaction = Transaction::new_signed_with_payer(
        &[tokenize_schema_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(transaction).unwrap();

    let clock: Clock = ctx.svm.get_sysvar();
    let expiry: i64 = clock.unix_timestamp + 60;
    let name = "Test Asset".to_string();
    let uri = "https://x.com".to_string();
    let symbol = "VAT".to_string();
    let mint_account_space = 686;
    let create_attestation_ix = CreateTokenizedAttestationBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .schema(schema)
        .attestation(attestation_pda)
        .system_program(system_program::ID)
        .schema_mint(schema_mint_pda)
        .attestation_mint(attestation_mint_pda)
        .sas_pda(sas_pda)
        .recipient_token_account(recipient_token_account)
        .recipient(recipient)
        .token_program(TOKEN_2022_PROGRAM_ID)
        .associated_token_program(ATA_PROGRAM_ID)
        .data(serialized_attestation_data.clone())
        .expiry(expiry)
        .nonce(nonce)
        .name(name.clone())
        .uri(uri.clone())
        .symbol(symbol.clone())
        .mint_account_space(mint_account_space)
        .instruction();
    let transaction = Transaction::new_signed_with_payer(
        &[create_attestation_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.svm.latest_blockhash(),
    );

    ctx.svm.send_transaction(transaction).unwrap();

    // Assert attestation
    let attestation_account = ctx.svm.get_account(&attestation_pda).unwrap();
    let attestation = Attestation::try_from_slice(&attestation_account.data).unwrap();
    assert_eq!(attestation.data, serialized_attestation_data);
    assert_eq!(attestation.credential, credential);
    assert_eq!(attestation.expiry, expiry);
    assert_eq!(attestation.schema, schema);
    assert_eq!(attestation.signer, authority.pubkey());
    assert_eq!(attestation.nonce, nonce);
    assert_eq!(attestation.token_account, recipient_token_account);

    let attestation_mint_account = ctx.svm.get_account(&attestation_mint_pda).unwrap();

    let expected_lamports = ctx.svm.minimum_balance_for_rent_exemption(mint_account_space.into());
    assert_eq!(attestation_mint_account.lamports, expected_lamports);
    assert!(attestation_mint_account.data.len() <= mint_account_space.into());

    assert_eq!(attestation_mint_account.owner, TOKEN_2022_PROGRAM_ID);

    let mint_state = StateWithExtensions::<Mint>::unpack(&attestation_mint_account.data).unwrap();
    assert!(mint_state.base.is_initialized);
    assert_eq!(mint_state.base.decimals, 0);
    assert_eq!(mint_state.base.supply, 1);
    assert_eq!(mint_state.base.mint_authority, COption::Some(sas_pda));
    assert_eq!(mint_state.base.freeze_authority, COption::Some(sas_pda));

    // Verify the GroupMemberPointer extension.
    let group_member_pointer = mint_state.get_extension::<GroupMemberPointer>().unwrap();
    assert_eq!(group_member_pointer.authority.get(), Some(sas_pda));
    assert_eq!(group_member_pointer.member_address.get(), Some(attestation_mint_pda));

    // Verify the NonTransferableMint extension exists.
    let _non_transferable = mint_state.get_extension::<NonTransferable>().unwrap();

    // Verify the GroupMember extension.
    let token_group_member = mint_state.get_extension::<TokenGroupMember>().unwrap();
    assert_eq!(token_group_member.mint, attestation_mint_pda);
    assert_eq!(token_group_member.group, schema_mint_pda);
    assert_eq!(u64::from(token_group_member.member_number), 1);

    // Verify the Permanent Delegate extension.
    let permanent_delegate = mint_state.get_extension::<PermanentDelegate>().unwrap();
    assert_eq!(permanent_delegate.delegate.get(), Some(sas_pda));

    // Verify the Mint Close extension.
    let close_authority = mint_state.get_extension::<MintCloseAuthority>().unwrap();
    assert_eq!(close_authority.close_authority.get(), Some(sas_pda));

    // Verify the MetadataPointer extension.
    let metadata_pointer = mint_state.get_extension::<MetadataPointer>().unwrap();
    // Check that the metadata pointer was set to the attestation mint and points to the SAS PDA.
    assert_eq!(metadata_pointer.authority.get(), Some(sas_pda));
    assert_eq!(metadata_pointer.metadata_address.get(), Some(attestation_mint_pda));

    // Verify the TokenMetadata extension.
    let token_metadata = &mint_state.get_variable_len_extension::<TokenMetadata>().unwrap();
    assert_eq!(token_metadata.update_authority.get(), Some(sas_pda));
    assert_eq!(token_metadata.mint, attestation_mint_pda);
    assert_eq!(token_metadata.name, name);
    assert_eq!(token_metadata.uri, uri);
    assert_eq!(token_metadata.symbol, symbol);
    assert_eq!(token_metadata.additional_metadata.len(), 2);
    assert_eq!(token_metadata.additional_metadata[0].0, "attestation");
    assert_eq!(token_metadata.additional_metadata[0].1, attestation_pda.to_string());
    assert_eq!(token_metadata.additional_metadata[1].0, "schema");
    assert_eq!(token_metadata.additional_metadata[1].1, schema.to_string());

    let recipient_token_account_data = ctx.svm.get_account(&recipient_token_account).unwrap();

    // Verify that recipient has 1 attestation token.
    let token_account = Account::unpack(&recipient_token_account_data.data[..Account::LEN]).unwrap();
    assert_eq!(token_account.mint, attestation_mint_pda);
    assert_eq!(token_account.amount, 1);
}

#[test]
fn close_tokenized_attestation_success() {
    let TestFixtures {
        mut ctx,
        credential,
        schema,
        authority,
        sas_pda,
        schema_mint_pda,
        attestation_pda,
        attestation_mint_pda,
        recipient,
        recipient_token_account,
        nonce,
        serialized_attestation_data,
    } = setup();

    let tokenize_schema_ix = TokenizeSchemaBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .schema(schema)
        .mint(schema_mint_pda)
        .sas_pda(sas_pda)
        .max_size(100)
        .token_program(TOKEN_2022_PROGRAM_ID)
        .instruction();

    let clock: Clock = ctx.svm.get_sysvar();
    let expiry: i64 = clock.unix_timestamp + 60;
    let name = "Test Asset".to_string();
    let uri = "https://x.com".to_string();
    let symbol = "VAT".to_string();
    let mint_account_space = 686;
    let create_attestation_ix = CreateTokenizedAttestationBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .schema(schema)
        .attestation(attestation_pda)
        .system_program(system_program::ID)
        .schema_mint(schema_mint_pda)
        .attestation_mint(attestation_mint_pda)
        .sas_pda(sas_pda)
        .recipient_token_account(recipient_token_account)
        .recipient(recipient)
        .token_program(TOKEN_2022_PROGRAM_ID)
        .associated_token_program(ATA_PROGRAM_ID)
        .data(serialized_attestation_data.clone())
        .expiry(expiry)
        .nonce(nonce)
        .name(name.clone())
        .uri(uri.clone())
        .symbol(symbol.clone())
        .mint_account_space(mint_account_space)
        .instruction();

    let transaction = Transaction::new_signed_with_payer(
        &[tokenize_schema_ix, create_attestation_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(transaction).unwrap();

    let (event_auth_pda, _bump) = Pubkey::find_program_address(&[b"__event_authority"], &SOLANA_ATTESTATION_SERVICE_ID);

    let close_attestation_ix = CloseTokenizedAttestationBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .attestation(attestation_pda)
        .event_authority(event_auth_pda)
        .system_program(system_program::ID)
        .attestation_program(solana_attestation_service_client::programs::SOLANA_ATTESTATION_SERVICE_ID)
        .attestation_mint(attestation_mint_pda)
        .sas_pda(sas_pda)
        .attestation_token_account(recipient_token_account)
        .token_program(TOKEN_2022_PROGRAM_ID)
        .instruction();
    let transaction = Transaction::new_signed_with_payer(
        &[close_attestation_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(transaction).unwrap();

    let recipient_token_account_data = ctx.svm.get_account(&recipient_token_account).unwrap();

    // Check that attestation account is closed.
    let attestation_account = ctx.svm.get_account(&attestation_pda);
    assert!(attestation_account.is_none());

    // Check that mint account is closed.
    let mint_account = ctx.svm.get_account(&attestation_mint_pda);
    assert!(mint_account.is_none());

    // Verify that recipient has 0 attestation token.
    let token_account = Account::unpack(&recipient_token_account_data.data[..Account::LEN]).unwrap();
    assert_eq!(token_account.mint, attestation_mint_pda);
    assert_eq!(token_account.amount, 0);
}

#[test]
fn update_tokenized_attestation_success() {
    let TestFixtures {
        mut ctx,
        credential,
        schema,
        authority,
        sas_pda,
        schema_mint_pda,
        attestation_pda,
        attestation_mint_pda,
        recipient,
        recipient_token_account,
        nonce,
        serialized_attestation_data,
    } = setup();

    let tokenize_schema_ix = TokenizeSchemaBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .schema(schema)
        .mint(schema_mint_pda)
        .sas_pda(sas_pda)
        .max_size(100)
        .token_program(TOKEN_2022_PROGRAM_ID)
        .instruction();

    let clock: Clock = ctx.svm.get_sysvar();
    let expiry: i64 = clock.unix_timestamp + 60;
    let name = "Test Asset".to_string();
    let uri = "https://x.com".to_string();
    let symbol = "VAT".to_string();
    let mint_account_space = 686;
    let create_attestation_ix = CreateTokenizedAttestationBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .schema(schema)
        .attestation(attestation_pda)
        .system_program(system_program::ID)
        .schema_mint(schema_mint_pda)
        .attestation_mint(attestation_mint_pda)
        .sas_pda(sas_pda)
        .recipient_token_account(recipient_token_account)
        .recipient(recipient)
        .token_program(TOKEN_2022_PROGRAM_ID)
        .associated_token_program(ATA_PROGRAM_ID)
        .data(serialized_attestation_data.clone())
        .expiry(expiry)
        .nonce(nonce)
        .name(name.clone())
        .uri(uri.clone())
        .symbol(symbol.clone())
        .mint_account_space(mint_account_space)
        .instruction();

    let transaction = Transaction::new_signed_with_payer(
        &[tokenize_schema_ix, create_attestation_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(transaction).unwrap();

    let (event_auth_pda, _bump) = Pubkey::find_program_address(&[b"__event_authority"], &SOLANA_ATTESTATION_SERVICE_ID);

    let close_attestation_ix = CloseTokenizedAttestationBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .attestation(attestation_pda)
        .event_authority(event_auth_pda)
        .system_program(system_program::ID)
        .attestation_program(solana_attestation_service_client::programs::SOLANA_ATTESTATION_SERVICE_ID)
        .attestation_mint(attestation_mint_pda)
        .sas_pda(sas_pda)
        .attestation_token_account(recipient_token_account)
        .token_program(TOKEN_2022_PROGRAM_ID)
        .instruction();
    let transaction = Transaction::new_signed_with_payer(
        &[close_attestation_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(transaction).unwrap();

    let create_attestation_ix = CreateTokenizedAttestationBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .schema(schema)
        .attestation(attestation_pda)
        .system_program(system_program::ID)
        .schema_mint(schema_mint_pda)
        .attestation_mint(attestation_mint_pda)
        .sas_pda(sas_pda)
        .recipient_token_account(recipient_token_account)
        .recipient(recipient)
        .token_program(TOKEN_2022_PROGRAM_ID)
        .associated_token_program(ATA_PROGRAM_ID)
        .data(serialized_attestation_data.clone())
        .expiry(expiry)
        .nonce(nonce)
        .name(name.clone())
        .uri(uri.clone())
        .symbol(symbol.clone())
        .mint_account_space(mint_account_space)
        .instruction();

    let transaction = Transaction::new_signed_with_payer(
        &[create_attestation_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(transaction).unwrap();

    let attestation_mint_account = ctx.svm.get_account(&attestation_mint_pda).unwrap();
    let mint_state = StateWithExtensions::<Mint>::unpack(&attestation_mint_account.data).unwrap();
    assert!(mint_state.base.is_initialized);
    assert_eq!(mint_state.base.decimals, 0);
    assert_eq!(mint_state.base.supply, 1);
    assert_eq!(mint_state.base.mint_authority, COption::Some(sas_pda));
    assert_eq!(mint_state.base.freeze_authority, COption::Some(sas_pda));

    // Verify the GroupMemberPointer extension.
    let group_member_pointer = mint_state.get_extension::<GroupMemberPointer>().unwrap();
    assert_eq!(group_member_pointer.authority.get(), Some(sas_pda));
    assert_eq!(group_member_pointer.member_address.get(), Some(attestation_mint_pda));

    // Verify the NonTransferableMint extension exists.
    let _non_transferable = mint_state.get_extension::<NonTransferable>().unwrap();

    // Verify the GroupMember extension.
    let token_group_member = mint_state.get_extension::<TokenGroupMember>().unwrap();
    assert_eq!(token_group_member.mint, attestation_mint_pda);
    assert_eq!(token_group_member.group, schema_mint_pda);
    // Note: This is 2 because TokenGroups cannot decrement the group size.
    assert_eq!(u64::from(token_group_member.member_number), 2);

    // Verify the Permanent Delegate extension.
    let permanent_delegate = mint_state.get_extension::<PermanentDelegate>().unwrap();
    assert_eq!(permanent_delegate.delegate.get(), Some(sas_pda));

    // Verify the Mint Close extension.
    let close_authority = mint_state.get_extension::<MintCloseAuthority>().unwrap();
    assert_eq!(close_authority.close_authority.get(), Some(sas_pda));

    // Verify the MetadataPointer extension.
    let metadata_pointer = mint_state.get_extension::<MetadataPointer>().unwrap();
    // Check that the metadata pointer was set to the attestation mint and points to the SAS PDA.
    assert_eq!(metadata_pointer.authority.get(), Some(sas_pda));
    assert_eq!(metadata_pointer.metadata_address.get(), Some(attestation_mint_pda));

    // Verify the TokenMetadata extension.
    let token_metadata = &mint_state.get_variable_len_extension::<TokenMetadata>().unwrap();
    assert_eq!(token_metadata.update_authority.get(), Some(sas_pda));
    assert_eq!(token_metadata.mint, attestation_mint_pda);
    assert_eq!(token_metadata.name, name);
    assert_eq!(token_metadata.uri, uri);
    assert_eq!(token_metadata.symbol, symbol);
    assert_eq!(token_metadata.additional_metadata.len(), 2);
    assert_eq!(token_metadata.additional_metadata[0].0, "attestation");
    assert_eq!(token_metadata.additional_metadata[0].1, attestation_pda.to_string());
    assert_eq!(token_metadata.additional_metadata[1].0, "schema");
    assert_eq!(token_metadata.additional_metadata[1].1, schema.to_string());

    let recipient_token_account_data = ctx.svm.get_account(&recipient_token_account).unwrap();

    // Verify that recipient has 1 attestation token.
    let token_account = Account::unpack(&recipient_token_account_data.data[..Account::LEN]).unwrap();
    assert_eq!(token_account.mint, attestation_mint_pda);
    assert_eq!(token_account.amount, 1);
}
