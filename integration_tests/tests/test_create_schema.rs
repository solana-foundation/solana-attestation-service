use borsh::BorshDeserialize;
use helpers::{create_credential, create_schema, program_test_context};
use solana_attestation_service_client::accounts::Schema;
use solana_keypair::Keypair;
use solana_signer::Signer;

mod helpers;

#[test]
fn create_schema_success() {
    let mut ctx = program_test_context();
    let authority = Keypair::new();
    let signers = vec![authority.pubkey(), ctx.payer.pubkey()];
    let credential_pda = create_credential(&mut ctx, &authority, "test", signers);

    let schema_name = "test_data";
    let description = "schema for test data";
    let schema_layout = vec![12, 0];
    let field_names = vec!["name".to_string(), "location".to_string()];
    let schema_pda = create_schema(
        &mut ctx,
        &authority,
        credential_pda,
        schema_name,
        description,
        schema_layout.clone(),
        field_names.clone(),
    );

    let schema_account = ctx.svm.get_account(&schema_pda).expect("account not none");
    let schema = Schema::try_from_slice(&schema_account.data).unwrap();
    assert_eq!(schema.credential, credential_pda);
    assert_eq!(schema.layout, schema_layout);
    assert_eq!(
        schema.field_names,
        // Schema deserialize doesn't include vec length in data.
        borsh::to_vec(&field_names).unwrap()[4..]
    );
    assert_eq!(schema.description, description.as_bytes());
    assert!(!schema.is_paused);
    assert_eq!(schema.version, 1);
    assert_eq!(schema.name, schema_name.as_bytes());
}
