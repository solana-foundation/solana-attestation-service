use helpers::{
    create_credential, expected_schema, fetch_schema, field_names_bytes, program_error, send, setup, TestFixtures,
    SCHEMA_NAME,
};
use solana_address::Address;
use solana_attestation::{
    accounts::Schema, errors::SolanaAttestationServiceError, instructions::ChangeSchemaVersionBuilder,
    programs::SOLANA_ATTESTATION_SERVICE_ID,
};
use solana_instruction_error::InstructionError;
use solana_signer::Signer;
use solana_transaction_error::TransactionError;

mod helpers;

#[test]
fn change_schema_version_success() {
    let mut f = setup();

    let (schema_pda2, _bump) = Address::find_program_address(
        &[b"schema", &f.credential.to_bytes(), SCHEMA_NAME.as_bytes(), &[2]],
        &SOLANA_ATTESTATION_SERVICE_ID,
    );
    let ix = change_schema_version_ix(&f, schema_pda2);
    send(&mut f.ctx, &[ix], &[&f.authority]).unwrap();

    assert_eq!(
        fetch_schema(&f.ctx, &schema_pda2),
        Schema {
            layout: vec![12, 0, 3],
            field_names: field_names_bytes(&["name", "location", "phone"]),
            version: 2,
            ..expected_schema(f.credential)
        }
    );
}

#[test]
fn change_schema_version_fail_incorrect_credential() {
    let TestFixtures { mut ctx, schema: schema_pda, authority, .. } = setup();

    let signers = vec![authority.pubkey(), ctx.payer.pubkey()];

    let credential_pda_2 = create_credential(&mut ctx, &authority, "test-2", signers);

    let (schema_pda2, _bump) = Address::find_program_address(
        &[b"schema", &credential_pda_2.to_bytes(), SCHEMA_NAME.as_bytes(), &[2]],
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
        &[b"schema", &f.credential.to_bytes(), SCHEMA_NAME.as_bytes(), &[3]],
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
