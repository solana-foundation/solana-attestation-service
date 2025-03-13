use borsh::{BorshDeserialize, BorshSerialize};
use helpers::program_test_context;
use mpl_core::programs::MPL_CORE_ID;
use mpl_core::types::UpdateAuthority;
use mpl_core::Asset;
use solana_attestation_service_client::{
    accounts::Attestation,
    instructions::{
        CreateAttestationWithTokenBuilder, CreateCredentialBuilder, CreateSchemaBuilder,
    },
};
use solana_attestation_service_macros::SchemaStructSerialize;
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    pubkey::Pubkey, signature::Keypair, signer::Signer, system_program, transaction::Transaction,
};

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
        &Pubkey::from(solana_attestation_service_client::programs::SOLANA_ATTESTATION_SERVICE_ID),
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
        authority: authority,
    }
}

#[tokio::test]
async fn create_attestation_asset_success() {
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
    attestation_data
        .serialize(&mut serialized_attestation_data)
        .unwrap();
    let nonce = Pubkey::new_unique();
    let attestation_pda = Pubkey::find_program_address(
        &[
            b"attestation",
            &credential.to_bytes(),
            &authority.pubkey().to_bytes(),
            &schema.to_bytes(),
            &nonce.to_bytes(),
        ],
        &solana_attestation_service_client::programs::SOLANA_ATTESTATION_SERVICE_ID,
    )
    .0;

    let asset_keypair = Keypair::new();
    let name = "Test Asset".to_string();
    let uri = "https://x.com".to_string();
    let create_attestation_ix = CreateAttestationWithTokenBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .schema(schema)
        .attestation(attestation_pda)
        .system_program(system_program::ID)
        .asset_info(asset_keypair.pubkey())
        .core_program(MPL_CORE_ID)
        .data(serialized_attestation_data.clone())
        .expiry(expiry)
        .nonce(nonce)
        .name(name.clone())
        .uri(uri.clone())
        .instruction();

    let transaction = Transaction::new_signed_with_payer(
        &[create_attestation_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority, &asset_keypair],
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
    assert_eq!(attestation.is_revoked, false);
    assert_eq!(attestation.schema, schema);
    assert_eq!(attestation.signer, authority.pubkey());
    assert_eq!(attestation.nonce, nonce);

    let core_asset_account = ctx
        .banks_client
        .get_account(asset_keypair.pubkey())
        .await
        .unwrap()
        .unwrap();
    let asset = Asset::from_bytes(&core_asset_account.data).unwrap();
    assert_eq!(asset.base.owner, ctx.payer.pubkey());
    assert_eq!(
        asset.base.update_authority,
        UpdateAuthority::Address(ctx.payer.pubkey())
    );
    assert_eq!(asset.base.name, name);
    assert_eq!(asset.base.uri, uri);
    assert!(asset.plugin_header.is_none());

    // Verify plugins not enabled.
    assert!(asset.plugin_list.royalties.is_none());
    assert!(asset.plugin_list.freeze_delegate.is_none());
    assert!(asset.plugin_list.burn_delegate.is_none());
    assert!(asset.plugin_list.transfer_delegate.is_none());
    assert!(asset.plugin_list.update_delegate.is_none());
    assert!(asset.plugin_list.permanent_freeze_delegate.is_none());
    assert!(asset.plugin_list.attributes.is_none());
    assert!(asset.plugin_list.permanent_transfer_delegate.is_none());
    assert!(asset.plugin_list.permanent_burn_delegate.is_none());
    assert!(asset.plugin_list.edition.is_none());
    assert!(asset.plugin_list.master_edition.is_none());
    assert!(asset.plugin_list.add_blocker.is_none());
    assert!(asset.plugin_list.immutable_metadata.is_none());
    assert!(asset.plugin_list.verified_creators.is_none());
    assert!(asset.plugin_list.autograph.is_none());

    // Verify external plugins not enabled.
    assert!(asset
        .external_plugin_adapter_list
        .linked_lifecycle_hooks
        .is_empty());
    assert!(asset.external_plugin_adapter_list.oracles.is_empty());
    assert!(asset.external_plugin_adapter_list.app_data.is_empty());
    assert!(asset
        .external_plugin_adapter_list
        .linked_app_data
        .is_empty());
    assert!(asset.external_plugin_adapter_list.data_sections.is_empty());
}
