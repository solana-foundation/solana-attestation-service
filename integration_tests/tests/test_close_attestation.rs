use borsh::{BorshDeserialize, BorshSerialize};
use helpers::program_test_context;
use solana_attestation_service_client::instructions::{
    CloseAttestationBuilder, CreateAttestationBuilder, CreateCredentialBuilder, CreateSchemaBuilder,
};
use solana_attestation_service_client::programs::SOLANA_ATTESTATION_SERVICE_ID;
use solana_attestation_service_client::types::CloseAttestationEvent;
use solana_attestation_service_macros::SchemaStructSerialize;
use solana_program_test::ProgramTestContext;
use solana_sdk::clock::Clock;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};
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

pub const EVENT_IX_TAG: u64 = 0x1d9acb512ea545e4;
pub const EVENT_IX_TAG_LE: &[u8] = EVENT_IX_TAG.to_le_bytes().as_slice();

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
async fn close_attestation_success() {
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
        &SOLANA_ATTESTATION_SERVICE_ID,
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

    let create_tx = Transaction::new_signed_with_payer(
        &[create_attestation_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.last_blockhash,
    );
    ctx.banks_client
        .process_transaction(create_tx)
        .await
        .unwrap();

    let (event_auth_pda, _bump) =
        Pubkey::find_program_address(&[b"__event_authority"], &SOLANA_ATTESTATION_SERVICE_ID);

    let initial_payer_lamports = ctx
        .banks_client
        .get_account(ctx.payer.pubkey())
        .await
        .unwrap()
        .map(|acc| acc.lamports)
        .unwrap_or(0);

    let pda_lamports = ctx
        .banks_client
        .get_account(attestation_pda)
        .await
        .unwrap()
        .map(|acc| acc.lamports)
        .unwrap_or(0);

    let close_attestation_ix = CloseAttestationBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .attestation(attestation_pda)
        .event_authority(event_auth_pda)
        .system_program(system_program::ID)
        .attestation_program(
            solana_attestation_service_client::programs::SOLANA_ATTESTATION_SERVICE_ID,
        )
        .instruction();
    let close_tx = Transaction::new_signed_with_payer(
        &[close_attestation_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.last_blockhash,
    );

    // Simulate transaction to check if event is emitted correctly.
    let simulate_res = ctx
        .banks_client
        .simulate_transaction(close_tx.clone())
        .await
        .unwrap();
    let inner_ixs = simulate_res
        .simulation_details
        .unwrap()
        .inner_instructions
        .unwrap();

    // Look through transaction instructions to find CloseAttestationEvent in emit_event ix data args.
    let mut event_found = false;
    for inner_instr_group in inner_ixs {
        for inner_instr in inner_instr_group {
            let program_id = inner_instr
                .instruction
                .program_id(&close_tx.message.account_keys);

            if program_id.eq(&SOLANA_ATTESTATION_SERVICE_ID) {
                let data = inner_instr.instruction.data;

                // Check ix discriminator matches emit_event.
                let match_event = data.starts_with(EVENT_IX_TAG_LE);
                if match_event {
                    // Deserialize data in ix args (after discriminator).
                    let event = CloseAttestationEvent::try_from_slice(&data[8..]).unwrap();
                    assert_eq!(event.discriminator, 0);
                    assert_eq!(event.schema, schema);
                    assert_eq!(event.attestation_data, serialized_attestation_data);
                    event_found = true;
                }
            }
        }
    }
    assert!(event_found);

    // Send close attestation transaction.
    ctx.banks_client
        .process_transaction(close_tx)
        .await
        .unwrap();

    // Check that attestation account is closed.
    let attestation_account = ctx
        .banks_client
        .get_account(attestation_pda)
        .await
        .expect("get_account");
    assert!(attestation_account.is_none());

    // Check that lamports are tranferred back to payer (minus 10000 for tx fees).
    let post_payer_lamports = ctx
        .banks_client
        .get_account(ctx.payer.pubkey())
        .await
        .unwrap()
        .map(|acc| acc.lamports)
        .unwrap_or(0);
    assert_eq!(
        initial_payer_lamports + pda_lamports - 10_000,
        post_payer_lamports,
    )
}
