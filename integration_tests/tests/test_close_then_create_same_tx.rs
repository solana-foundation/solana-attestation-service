use borsh::{BorshDeserialize, BorshSerialize};
use helpers::program_test_context;
use solana_attestation_service_client::accounts::Attestation;
use solana_attestation_service_client::instructions::{
    CloseAttestationBuilder, CreateAttestationBuilder, CreateCredentialBuilder, CreateSchemaBuilder,
};
use solana_attestation_service_client::programs::SOLANA_ATTESTATION_SERVICE_ID;
use solana_attestation_service_client::types::CloseAttestationEvent;
use solana_attestation_service_macros::SchemaStructSerialize;
use solana_program_test::ProgramTestContext;
use solana_sdk::clock::Clock;
use solana_sdk::{
    pubkey::Pubkey, signature::Keypair, signer::Signer, system_program, transaction::Transaction,
};

mod helpers;

pub const EVENT_IX_TAG: u64 = 0x1d9acb512ea545e4;
pub const EVENT_IX_TAG_LE: &[u8] = EVENT_IX_TAG.to_le_bytes().as_slice();

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
        &SOLANA_ATTESTATION_SERVICE_ID,
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
        &SOLANA_ATTESTATION_SERVICE_ID,
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
async fn close_then_create_attestation_success() {
    let TestFixtures {
        ctx,
        credential,
        schema,
        authority,
    } = setup().await;

    let clock: Clock = ctx.banks_client.get_sysvar().await.unwrap();
    let expiry_v1: i64 = clock.unix_timestamp + 3600;
    let expiry_v2: i64 = expiry_v1 + 60;

    let nonce = Pubkey::new_unique();
    let attestation_pda = Pubkey::find_program_address(
        &[
            b"attestation",
            &credential.to_bytes(),
            &schema.to_bytes(),
            &nonce.to_bytes(),
        ],
        &SOLANA_ATTESTATION_SERVICE_ID,
    )
    .0;

    let mut v1 = Vec::new();
    TestData {
        name: "v1".to_string(),
        location: 1,
    }
    .serialize(&mut v1)
    .unwrap();
    let mut v2 = Vec::new();
    TestData {
        name: "v2-updated".to_string(),
        location: 2,
    }
    .serialize(&mut v2)
    .unwrap();

    // --- tx 1: initial attestation
    let create_v1 = CreateAttestationBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .schema(schema)
        .attestation(attestation_pda)
        .system_program(system_program::ID)
        .data(v1.clone())
        .expiry(expiry_v1)
        .nonce(nonce)
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[create_v1],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();

    assert!(
        ctx.banks_client
            .get_account(attestation_pda)
            .await
            .unwrap()
            .is_some(),
        "attestation should exist after create",
    );

    // --- tx 2: close + create, SAME transaction, SAME address
    let (event_auth_pda, _bump) =
        Pubkey::find_program_address(&[b"__event_authority"], &SOLANA_ATTESTATION_SERVICE_ID);

    let close_ix = CloseAttestationBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .attestation(attestation_pda)
        .event_authority(event_auth_pda)
        .system_program(system_program::ID)
        .attestation_program(SOLANA_ATTESTATION_SERVICE_ID)
        .instruction();

    let create_v2 = CreateAttestationBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .schema(schema)
        .attestation(attestation_pda)
        .system_program(system_program::ID)
        .data(v2.clone())
        .expiry(expiry_v2)
        .nonce(nonce)
        .instruction();

    let blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();
    let tx2 = Transaction::new_signed_with_payer(
        &[close_ix, create_v2],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        blockhash,
    );

    // Simulate first: the close half must actually run inside the combined
    // transaction, and its event must carry the data being replaced (v1).
    let sim = ctx
        .banks_client
        .simulate_transaction(tx2.clone())
        .await
        .unwrap();
    let inner_ixs = sim.simulation_details.unwrap().inner_instructions.unwrap();
    let mut event_found = false;
    for group in inner_ixs {
        for inner in group {
            let program_id = inner.instruction.program_id(&tx2.message.account_keys);
            if program_id.eq(&SOLANA_ATTESTATION_SERVICE_ID) {
                let data = inner.instruction.data;
                if data.starts_with(EVENT_IX_TAG_LE) {
                    let event = CloseAttestationEvent::try_from_slice(&data[8..]).unwrap();
                    assert_eq!(event.schema, schema);
                    assert_eq!(event.attestation_data, v1);
                    event_found = true;
                }
            }
        }
    }
    assert!(
        event_found,
        "close half did not run inside the combined transaction"
    );

    ctx.banks_client
        .process_transaction(tx2)
        .await
        .expect("close and create should succeed in a single transaction");

    // The attestation is back at the same address holding the new record. Every
    // field is checked, not just the payload: a create that silently wrote a
    // wrong signer or nonce would otherwise pass.
    let account = ctx
        .banks_client
        .get_account(attestation_pda)
        .await
        .unwrap()
        .expect("attestation should exist again after close+create");
    assert_eq!(account.owner, SOLANA_ATTESTATION_SERVICE_ID);
    assert_eq!(
        account.data.len(),
        1 + 32 + 32 + 32 + (4 + v2.len()) + 32 + 8 + 32,
        "account should be reallocated to fit the new payload",
    );

    let attestation = Attestation::from_bytes(&account.data).unwrap();
    assert_eq!(attestation.data, v2);
    assert_eq!(attestation.expiry, expiry_v2);
    assert_eq!(attestation.credential, credential);
    assert_eq!(attestation.schema, schema);
    assert_eq!(attestation.signer, authority.pubkey());
    assert_eq!(attestation.nonce, nonce);
    assert_eq!(attestation.token_account, Pubkey::default());
}

/// The close half is load bearing: creating over a live attestation fails,
/// so the test above is not simply "create succeeded".
#[tokio::test]
async fn create_over_live_attestation_fails() {
    let TestFixtures {
        ctx,
        credential,
        schema,
        authority,
    } = setup().await;

    let clock: Clock = ctx.banks_client.get_sysvar().await.unwrap();
    let expiry: i64 = clock.unix_timestamp + 3600;
    let nonce = Pubkey::new_unique();
    let attestation_pda = Pubkey::find_program_address(
        &[
            b"attestation",
            &credential.to_bytes(),
            &schema.to_bytes(),
            &nonce.to_bytes(),
        ],
        &SOLANA_ATTESTATION_SERVICE_ID,
    )
    .0;

    let mut v1 = Vec::new();
    TestData {
        name: "v1".to_string(),
        location: 1,
    }
    .serialize(&mut v1)
    .unwrap();

    let create = |data: Vec<u8>| {
        CreateAttestationBuilder::new()
            .payer(ctx.payer.pubkey())
            .authority(authority.pubkey())
            .credential(credential)
            .schema(schema)
            .attestation(attestation_pda)
            .system_program(system_program::ID)
            .data(data)
            .expiry(expiry)
            .nonce(nonce)
            .instruction()
    };

    let tx = Transaction::new_signed_with_payer(
        &[create(v1.clone())],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();

    let mut v2 = Vec::new();
    TestData {
        name: "v2-updated".to_string(),
        location: 2,
    }
    .serialize(&mut v2)
    .unwrap();

    let blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();
    let tx2 = Transaction::new_signed_with_payer(
        &[create(v2.clone())],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        blockhash,
    );
    assert!(
        ctx.banks_client.process_transaction(tx2).await.is_err(),
        "creating over a live attestation should fail",
    );

    // And the original record is untouched. Asserting this rather than the raw
    // error code on purpose: the system program's AccountAlreadyInUse and the
    // program's own InvalidCredential are both Custom(0).
    let account = ctx
        .banks_client
        .get_account(attestation_pda)
        .await
        .unwrap()
        .expect("attestation should still exist");
    let attestation = Attestation::from_bytes(&account.data).unwrap();
    assert_eq!(attestation.data, v1);
}

/// The property that makes the pattern safe to use: if the create half fails,
/// the close is rolled back with it and the previous attestation survives
/// untouched. Without this, a failed update would silently destroy the record.
#[tokio::test]
async fn failed_create_rolls_back_the_close() {
    let TestFixtures {
        ctx,
        credential,
        schema,
        authority,
    } = setup().await;

    let clock: Clock = ctx.banks_client.get_sysvar().await.unwrap();
    let expiry: i64 = clock.unix_timestamp + 3600;
    let nonce = Pubkey::new_unique();
    let attestation_pda = Pubkey::find_program_address(
        &[
            b"attestation",
            &credential.to_bytes(),
            &schema.to_bytes(),
            &nonce.to_bytes(),
        ],
        &SOLANA_ATTESTATION_SERVICE_ID,
    )
    .0;

    let mut v1 = Vec::new();
    TestData {
        name: "v1".to_string(),
        location: 1,
    }
    .serialize(&mut v1)
    .unwrap();

    let create_v1 = CreateAttestationBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .schema(schema)
        .attestation(attestation_pda)
        .system_program(system_program::ID)
        .data(v1.clone())
        .expiry(expiry)
        .nonce(nonce)
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[create_v1],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();

    let (event_auth_pda, _bump) =
        Pubkey::find_program_address(&[b"__event_authority"], &SOLANA_ATTESTATION_SERVICE_ID);
    let close_ix = CloseAttestationBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .attestation(attestation_pda)
        .event_authority(event_auth_pda)
        .system_program(system_program::ID)
        .attestation_program(SOLANA_ATTESTATION_SERVICE_ID)
        .instruction();

    // Data that does not match the schema layout, so the create half fails.
    let bad_create = CreateAttestationBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .schema(schema)
        .attestation(attestation_pda)
        .system_program(system_program::ID)
        .data(vec![0xff; 3])
        .expiry(expiry)
        .nonce(nonce)
        .instruction();

    let blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();
    let tx2 = Transaction::new_signed_with_payer(
        &[close_ix, bad_create],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        blockhash,
    );
    assert!(
        ctx.banks_client.process_transaction(tx2).await.is_err(),
        "a create that violates the schema should fail the transaction",
    );

    let account = ctx
        .banks_client
        .get_account(attestation_pda)
        .await
        .unwrap()
        .expect("the previous attestation should survive a failed update");
    let attestation = Attestation::from_bytes(&account.data).unwrap();
    assert_eq!(attestation.data, v1);
    assert_eq!(attestation.expiry, expiry);
}
