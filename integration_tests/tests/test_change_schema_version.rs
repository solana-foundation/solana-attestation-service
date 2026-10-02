use borsh::BorshDeserialize;
use helpers::{create_credential, create_schema, program_error, program_test_context, send, TestContext};
use solana_address::Address;
use solana_attestation::{
    accounts::Schema, errors::SolanaAttestationServiceError, instructions::ChangeSchemaVersionBuilder,
    programs::SOLANA_ATTESTATION_SERVICE_ID,
};
use solana_instruction_error::InstructionError;
use solana_keypair::Keypair;
use solana_signer::Signer;
use solana_transaction_error::TransactionError;

mod helpers;

struct TestFixtures {
    ctx: TestContext,
    credential: Address,
    schema: Address,
    authority: Keypair,
    schema_name: String,
    schema_description: String,
}

fn setup() -> TestFixtures {
    let mut ctx = program_test_context();
    let authority = Keypair::new();
    let signers = vec![authority.pubkey(), ctx.payer.pubkey()];
    let credential = create_credential(&mut ctx, &authority, "test", signers);

    let schema_name = "test_data";
    let description = "schema for test data";
    let schema = create_schema(
        &mut ctx,
        &authority,
        credential,
        schema_name,
        description,
        vec![12, 0],
        vec!["name".into(), "location".into()],
    );

    TestFixtures {
        ctx,
        credential,
        schema,
        authority,
        schema_name: schema_name.to_string(),
        schema_description: description.to_string(),
    }
}

#[test]
fn change_schema_version_success() {
    let TestFixtures {
        mut ctx,
        credential: credential_pda,
        schema: schema_pda,
        authority,
        schema_name,
        schema_description,
    } = setup();

    let (schema_pda2, _bump) = Address::find_program_address(
        &[b"schema", &credential_pda.to_bytes(), schema_name.as_bytes(), &[2]],
        &SOLANA_ATTESTATION_SERVICE_ID,
    );
    let schema_layout2 = vec![12, 0, 3];
    let field_names2 = vec!["name".to_string(), "location".to_string(), "phone".to_string()];

    let change_schema_version_ix = ChangeSchemaVersionBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential_pda)
        .existing_schema(schema_pda)
        .new_schema(schema_pda2)
        .layout(schema_layout2.clone())
        .field_names(field_names2.clone())
        .instruction();
    send(&mut ctx, &[change_schema_version_ix], &[&authority]).unwrap();

    let schema_account = ctx.svm.get_account(&schema_pda2).expect("account not none");
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

#[test]
fn change_schema_version_fail_incorrect_credential() {
    let TestFixtures { mut ctx, schema: schema_pda, authority, schema_name, .. } = setup();

    let signers = vec![authority.pubkey(), ctx.payer.pubkey()];

    let credential_pda_2 = create_credential(&mut ctx, &authority, "test-2", signers);

    let (schema_pda2, _bump) = Address::find_program_address(
        &[b"schema", &credential_pda_2.to_bytes(), schema_name.as_bytes(), &[2]],
        &SOLANA_ATTESTATION_SERVICE_ID,
    );

    let change_schema_version_ix = ChangeSchemaVersionBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential_pda_2)
        .existing_schema(schema_pda)
        .new_schema(schema_pda2)
        .layout(vec![12, 0, 3])
        .field_names(vec!["name".into(), "location".into(), "phone".into()])
        .instruction();
    let tx_err = send(&mut ctx, &[change_schema_version_ix], &[&authority]).expect_err("should error");
    assert_eq!(tx_err, program_error(SolanaAttestationServiceError::InvalidSchema))
}

fn change_schema_version_ix(f: &TestFixtures, new_schema: Address) -> solana_transaction::Instruction {
    ChangeSchemaVersionBuilder::new()
        .payer(f.ctx.payer.pubkey())
        .authority(f.authority.pubkey())
        .credential(f.credential)
        .existing_schema(f.schema)
        .new_schema(new_schema)
        .layout(vec![12, 0, 3])
        .field_names(vec!["name".into(), "location".into(), "phone".into()])
        .instruction()
}

#[test]
fn change_schema_version_fail_wrong_new_schema_pda() {
    let mut f = setup();
    let (skipped_version_pda, _bump) = Address::find_program_address(
        &[b"schema", &f.credential.to_bytes(), f.schema_name.as_bytes(), &[3]],
        &SOLANA_ATTESTATION_SERVICE_ID,
    );

    let ix = change_schema_version_ix(&f, skipped_version_pda);
    let tx_err = send(&mut f.ctx, &[ix], &[&f.authority]).expect_err("should error");
    assert_eq!(tx_err, program_error(SolanaAttestationServiceError::InvalidSchema))
}

#[test]
fn change_schema_version_fail_at_max_version() {
    let mut f = setup();
    let mut schema_account = f.ctx.svm.get_account(&f.schema).unwrap();
    *schema_account.data.last_mut().unwrap() = u8::MAX;
    f.ctx.svm.set_account(f.schema, schema_account).unwrap();

    let ix = change_schema_version_ix(&f, Address::new_unique());
    let tx_err = send(&mut f.ctx, &[ix], &[&f.authority]).expect_err("should error");
    assert_eq!(tx_err, TransactionError::InstructionError(0, InstructionError::ArithmeticOverflow))
}
