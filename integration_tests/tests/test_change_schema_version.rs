use borsh::BorshDeserialize;
use helpers::program_test_context;
use solana_attestation_service_client::{
    accounts::Schema,
    instructions::{ChangeSchemaVersionBuilder, CreateCredentialBuilder, CreateSchemaBuilder},
};
use solana_attestation_service_macros::SchemaStructSerialize;
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    instruction::InstructionError,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::{Transaction, TransactionError},
};
use solana_sdk_ids::system_program;
mod helpers;

#[derive(SchemaStructSerialize)]
struct TestData {
    _name: String,
    _location: u8,
}

#[derive(SchemaStructSerialize)]
struct TestData2 {
    _name: String,
    _location: u8,
    _phone: u64,
}

struct TestFixtures {
    ctx: ProgramTestContext,
    credential: Pubkey,
    schema: Pubkey,
    authority: Keypair,
    schema_name: String,
    schema_description: String,
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
        .signers(vec![authority.pubkey(), ctx.payer.pubkey()])
        .instruction();

    let transaction = Transaction::new_signed_with_payer(
        &[create_credential_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.last_blockhash,
    );
    ctx.banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // Create Schema
    let schema_name = "test_data";
    let description = "schema for test data";
    let schema_layout = TestData::get_serialized_representation();
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
        .layout(schema_layout.clone())
        .field_names(field_names.clone())
        .instruction();
    let transaction = Transaction::new_signed_with_payer(
        &[create_schema_ix],
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
        schema_name: schema_name.to_string(),
        schema_description: description.to_string(),
    }
}

#[tokio::test]
async fn change_schema_version_success() {
    let TestFixtures {
        ctx,
        credential: credential_pda,
        schema: schema_pda,
        authority,
        schema_name,
        schema_description,
    } = setup().await;

    // Update schema for version 2
    let (schema_pda2, _bump) = Pubkey::find_program_address(
        &[
            b"schema",
            &credential_pda.to_bytes(),
            schema_name.as_bytes(),
            &[2],
        ],
        &solana_attestation_service_client::programs::SOLANA_ATTESTATION_SERVICE_ID,
    );
    let schema_layout2 = TestData2::get_serialized_representation();
    let field_names2 = vec!["name".into(), "location".into(), "phone".into()];

    let change_schema_version_ix = ChangeSchemaVersionBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential_pda)
        .existing_schema(schema_pda)
        .new_schema(schema_pda2)
        .system_program(system_program::ID)
        .layout(schema_layout2.clone())
        .field_names(field_names2.clone())
        .instruction();
    let transaction = Transaction::new_signed_with_payer(
        &[change_schema_version_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.last_blockhash,
    );
    ctx.banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // Assert schema account
    let schema_account = ctx
        .banks_client
        .get_account(schema_pda2)
        .await
        .expect("get_account")
        .expect("account not none");
    let schema = Schema::try_from_slice(&schema_account.data).unwrap();
    assert_eq!(schema.credential, credential_pda);
    assert_eq!(schema.layout, schema_layout2);
    assert_eq!(
        schema.field_names,
        // Schema deserialize doesn't include vec length in data.
        borsh::to_vec(&field_names2).unwrap()[4..]
    );
    assert_eq!(schema.description, schema_description.as_bytes());
    assert!(!schema.is_paused);
    assert_eq!(schema.version, 2);
    assert_eq!(schema.name, schema_name.as_bytes());
}

#[tokio::test]
async fn change_schema_version_fail_incorrect_credential() {
    let TestFixtures {
        ctx,
        credential: _credential_pda,
        schema: schema_pda,
        authority,
        schema_name,
        schema_description: _,
    } = setup().await;

    let credential_name = "test-2";
    let (credential_pda_2, _bump) = Pubkey::find_program_address(
        &[
            b"credential",
            &authority.pubkey().to_bytes(),
            credential_name.as_bytes(),
        ],
        &solana_attestation_service_client::programs::SOLANA_ATTESTATION_SERVICE_ID,
    );

    let create_credential_ix = CreateCredentialBuilder::new()
        .payer(ctx.payer.pubkey())
        .credential(credential_pda_2)
        .authority(authority.pubkey())
        .system_program(system_program::ID)
        .name(credential_name.to_string())
        .signers(vec![authority.pubkey(), ctx.payer.pubkey()])
        .instruction();

    let transaction = Transaction::new_signed_with_payer(
        &[create_credential_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.last_blockhash,
    );
    ctx.banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // Update schema for version 2
    let (schema_pda2, _bump) = Pubkey::find_program_address(
        &[
            b"schema",
            &credential_pda_2.to_bytes(),
            schema_name.as_bytes(),
            &[2],
        ],
        &solana_attestation_service_client::programs::SOLANA_ATTESTATION_SERVICE_ID,
    );
    let schema_layout2 = TestData2::get_serialized_representation();
    let field_names2 = vec!["name".into(), "location".into(), "phone".into()];

    let change_schema_version_ix = ChangeSchemaVersionBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential_pda_2)
        .existing_schema(schema_pda)
        .new_schema(schema_pda2)
        .system_program(system_program::ID)
        .layout(schema_layout2.clone())
        .field_names(field_names2.clone())
        .instruction();
    let transaction = Transaction::new_signed_with_payer(
        &[change_schema_version_ix],
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
        TransactionError::InstructionError(0, InstructionError::Custom(1))
    )
}
