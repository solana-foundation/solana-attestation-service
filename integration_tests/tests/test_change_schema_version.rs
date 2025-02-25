use borsh::BorshDeserialize;
use helpers::program_test_context;
use solana_attestation_service_client::{
    accounts::Schema,
    instructions::{ChangeSchemaVersionBuilder, CreateCredentialBuilder, CreateSchemaBuilder},
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

#[derive(SchemaStructSerialize)]
struct TestData2 {
    _name: String,
    _location: u8,
    _phone: u64,
}

#[tokio::test]
async fn change_schema_version_success() {
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
    let schema_layout = TestData::get_serialized_representation();
    let field_names = vec!["name".into(), "location".into()];
    let (schema_pda, _bump) = Pubkey::find_program_address(
        &[
            b"schema",
            &credential_pda.to_bytes(),
            schema_name.as_bytes(),
            &[1],
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

    // Update schema for version 2
    let (schema_pda2, _bump) = Pubkey::find_program_address(
        &[
            b"schema",
            &credential_pda.to_bytes(),
            schema_name.as_bytes(),
            &[2],
        ],
        &Pubkey::from(solana_attestation_service_client::programs::SOLANA_ATTESTATION_SERVICE_ID),
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
    assert_eq!(schema.description, description.as_bytes());
    assert_eq!(schema.is_paused, false);
    assert_eq!(schema.version, 2);
    assert_eq!(schema.name, schema_name.as_bytes());
}
