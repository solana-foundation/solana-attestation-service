use borsh::{BorshDeserialize, BorshSerialize};
use helpers::program_test_context;
use solana_attestation_service_client::{
    accounts::Attestation,
    instructions::{
        ChangeSchemaStatusBuilder, CreateAttestationBuilder, CreateCredentialBuilder,
        CreateSchemaBuilder,
    },
};
use solana_attestation_service_macros::SchemaStructSerialize;
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    clock::Clock,
    instruction::InstructionError,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::{Transaction, TransactionError},
};
use solana_sdk_ids::system_program;

mod helpers;

#[derive(BorshSerialize, SchemaStructSerialize)]
struct TestData {
    name: String,
    location: u8,
}

struct TestFixtures {
    ctx: ProgramTestContext,
    credential: Pubkey,
    schema: Pubkey,
    authority: Keypair,
}

async fn setup() -> TestFixtures {
    let ctx = program_test_context().await;

    let authority = Keypair::new();
    let credential_name = "test";
    let (credential_pda, _bump) = Pubkey::find_program_address(
        &[
            b"credential",
            &authority.pubkey().to_bytes(),
            credential_name.as_bytes(),
        ],
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
    let schema_data = TestData::get_serialized_representation();
    let field_names = vec!["name".into(), "location".into()];
    let (schema_pda, _bump) = Pubkey::find_program_address(
        &[
            b"schema",
            &credential_pda.to_bytes(),
            schema_name.as_bytes(),
            &[1],
        ],
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
        ctx.last_blockhash,
    );
    ctx.banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    TestFixtures {
        ctx,
        credential: credential_pda,
        schema: schema_pda,
        authority,
    }
}

#[tokio::test]
async fn create_attestation_success() {
    let TestFixtures {
        ctx,
        credential,
        schema,
        authority,
    } = setup().await;
    // Create Attestation
    let attestation_data = TestData {
        name: "attest".to_string(),
        location: 11,
    };
    let clock: Clock = ctx.banks_client.get_sysvar().await.unwrap();
    let expiry: i64 = clock.unix_timestamp + 60;
    let mut serialized_attestation_data = Vec::new();
    attestation_data
        .serialize(&mut serialized_attestation_data)
        .unwrap();
    let nonce = Pubkey::new_unique();
    let attestation_pda = Pubkey::find_program_address(
        &[
            b"attestation",
            &credential.to_bytes(),
            &schema.to_bytes(),
            &nonce.to_bytes(),
        ],
        &solana_attestation_service_client::programs::SOLANA_ATTESTATION_SERVICE_ID,
    )
    .0;
    let create_attestation_ix = CreateAttestationBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .schema(schema)
        .attestation(attestation_pda)
        .system_program(system_program::ID)
        .data(serialized_attestation_data.clone())
        .expiry(expiry)
        .nonce(nonce)
        .instruction();

    let transaction = Transaction::new_signed_with_payer(
        &[create_attestation_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.last_blockhash,
    );
    ctx.banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // Assert attestation
    let attestation_account = ctx
        .banks_client
        .get_account(attestation_pda)
        .await
        .unwrap()
        .unwrap();
    let attestation = Attestation::try_from_slice(&attestation_account.data).unwrap();
    assert_eq!(attestation.data, serialized_attestation_data);
    assert_eq!(attestation.credential, credential);
    assert_eq!(attestation.expiry, expiry);
    assert_eq!(attestation.schema, schema);
    assert_eq!(attestation.signer, authority.pubkey());
    assert_eq!(attestation.nonce, nonce);
    assert_eq!(attestation.token_account, Pubkey::default());
}

#[tokio::test]
async fn create_attestation_fail_bad_data() {
    let TestFixtures {
        ctx,
        credential,
        schema,
        authority,
    } = setup().await;
    // Create Attestation
    let attestation_data = TestData {
        name: "attest".to_string(),
        location: 11,
    };
    let expiry: i64 = 1000;
    let mut serialized_attestation_data = Vec::new();
    serialized_attestation_data.extend([1, 2, 3, 4, 5, 6, 7]);
    attestation_data
        .serialize(&mut serialized_attestation_data)
        .unwrap();
    let nonce = Pubkey::new_unique();
    let attestation_pda = Pubkey::find_program_address(
        &[
            b"attestation",
            &credential.to_bytes(),
            &schema.to_bytes(),
            &nonce.to_bytes(),
        ],
        &solana_attestation_service_client::programs::SOLANA_ATTESTATION_SERVICE_ID,
    )
    .0;
    let create_attestation_ix = CreateAttestationBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .schema(schema)
        .attestation(attestation_pda)
        .system_program(system_program::ID)
        .data(serialized_attestation_data.clone())
        .expiry(expiry)
        .nonce(nonce)
        .instruction();

    let transaction = Transaction::new_signed_with_payer(
        &[create_attestation_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.last_blockhash,
    );
    let tx_err = ctx
        .banks_client
        .process_transaction(transaction)
        .await
        .expect_err("should error")
        .unwrap();
    assert_eq!(
        tx_err,
        TransactionError::InstructionError(0, InstructionError::Custom(6))
    )
}

#[tokio::test]
async fn create_attestation_fail_schema_paused() {
    let TestFixtures {
        ctx,
        credential,
        schema,
        authority,
    } = setup().await;
    // Pause Schema
    let pause_schema_ix = ChangeSchemaStatusBuilder::new()
        .authority(authority.pubkey())
        .credential(credential)
        .schema(schema)
        .is_paused(true)
        .instruction();
    let transaction = Transaction::new_signed_with_payer(
        &[pause_schema_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.last_blockhash,
    );
    ctx.banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // Create Attestation
    let attestation_data = TestData {
        name: "attest".to_string(),
        location: 11,
    };
    let expiry: i64 = 1000;
    let mut serialized_attestation_data = Vec::new();
    attestation_data
        .serialize(&mut serialized_attestation_data)
        .unwrap();
    let nonce = Pubkey::new_unique();
    let attestation_pda = Pubkey::find_program_address(
        &[
            b"attestation",
            &credential.to_bytes(),
            &schema.to_bytes(),
            &nonce.to_bytes(),
        ],
        &solana_attestation_service_client::programs::SOLANA_ATTESTATION_SERVICE_ID,
    )
    .0;
    let create_attestation_ix = CreateAttestationBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .schema(schema)
        .attestation(attestation_pda)
        .system_program(system_program::ID)
        .data(serialized_attestation_data.clone())
        .expiry(expiry)
        .nonce(nonce)
        .instruction();

    let transaction = Transaction::new_signed_with_payer(
        &[create_attestation_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.last_blockhash,
    );
    let tx_err = ctx
        .banks_client
        .process_transaction(transaction)
        .await
        .expect_err("should error")
        .unwrap();
    assert_eq!(
        tx_err,
        TransactionError::InstructionError(0, InstructionError::Custom(11))
    )
}
