use borsh::BorshDeserialize;
use helpers::program_test_context;
use solana_attestation_service_client::{
    accounts::Schema,
    instructions::{ChangeSchemaStatusBuilder, CreateCredentialBuilder, CreateSchemaBuilder},
};
use solana_attestation_service_macros::SchemaStructSerialize;
use solana_sdk::{
    pubkey::Pubkey, signature::Keypair, signer::Signer, system_program, transaction::Transaction,
};

mod helpers;

#[derive(SchemaStructSerialize)]
struct TestData {
    _name: String,
    _location: u8,
}

#[tokio::test]
async fn pause_and_unpause_schema_success() {
    let ctx = program_test_context().await;
    let authority = Keypair::new();
    let credential_name = "test";
    let (credential_pda, _bump) = Pubkey::find_program_address(
        &[
            b"credential",
            &authority.pubkey().to_bytes(),
            credential_name.as_bytes(),
        ],
        &Pubkey::from(solana_attestation_service_client::programs::SOLANA_ATTESTATION_SERVICE_ID),
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
    let schema_data = TestData::get_serialized_representation();
    let (schema_pda, _bump) = Pubkey::find_program_address(
        &[
            b"schema",
            &credential_pda.to_bytes(),
            schema_name.as_bytes(),
        ],
        &Pubkey::from(solana_attestation_service_client::programs::SOLANA_ATTESTATION_SERVICE_ID),
    );
    let create_schema_ix = CreateSchemaBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential_pda)
        .schema(schema_pda)
        .system_program(system_program::ID)
        .description(description.to_string())
        .name(schema_name.to_string())
        .data(schema_data.clone())
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

    let pause_schema_ix = ChangeSchemaStatusBuilder::new()
        .authority(authority.pubkey())
        .credential(credential_pda)
        .schema(schema_pda)
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

    // Assert schema account
    let schema_account = ctx
        .banks_client
        .get_account(schema_pda)
        .await
        .expect("get_account")
        .expect("account not nonex");
    let schema = Schema::try_from_slice(&schema_account.data).unwrap();
    assert_eq!(schema.credential, credential_pda);
    assert_eq!(schema.data_schema, schema_data);
    assert_eq!(schema.description, description.as_bytes());
    assert_eq!(schema.is_paused, true);
    assert_eq!(schema.name, schema_name.as_bytes());

    let unpause_schema_ix = ChangeSchemaStatusBuilder::new()
        .authority(authority.pubkey())
        .credential(credential_pda)
        .schema(schema_pda)
        .is_paused(false)
        .instruction();
    let transaction = Transaction::new_signed_with_payer(
        &[unpause_schema_ix],
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
        .get_account(schema_pda)
        .await
        .expect("get_account")
        .expect("account not nonex");
    let schema = Schema::try_from_slice(&schema_account.data).unwrap();
    assert_eq!(schema.credential, credential_pda);
    assert_eq!(schema.data_schema, schema_data);
    assert_eq!(schema.description, description.as_bytes());
    assert_eq!(schema.is_paused, false);
    assert_eq!(schema.name, schema_name.as_bytes());
}
