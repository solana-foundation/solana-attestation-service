use borsh::BorshDeserialize;
use helpers::{create_attestation, expiry, program_error, send, setup, test_data};
use solana_address::Address;
use solana_attestation_service_client::{
    accounts::Attestation, errors::SolanaAttestationServiceError, instructions::ChangeSchemaStatusBuilder,
};
use solana_signer::Signer;

mod helpers;

#[test]
fn create_attestation_success() {
    let mut f = setup();
    let expiry = expiry(&f.ctx);
    let nonce = Address::new_unique();

    let attestation_pda = create_attestation(&mut f, test_data(), expiry, nonce).unwrap();

    let attestation_account = f.ctx.svm.get_account(&attestation_pda).unwrap();
    let attestation = Attestation::try_from_slice(&attestation_account.data).unwrap();
    assert_eq!(attestation.data, test_data());
    assert_eq!(attestation.credential, f.credential);
    assert_eq!(attestation.expiry, expiry);
    assert_eq!(attestation.schema, f.schema);
    assert_eq!(attestation.signer, f.authority.pubkey());
    assert_eq!(attestation.nonce, nonce);
    assert_eq!(attestation.token_account, Address::default());
}

#[test]
fn create_attestation_fail_bad_data() {
    let mut f = setup();
    let expiry = expiry(&f.ctx);
    let mut data = vec![1, 2, 3, 4, 5, 6, 7];
    data.extend(test_data());

    let tx_err = create_attestation(&mut f, data, expiry, Address::new_unique()).expect_err("should error");
    assert_eq!(tx_err, program_error(SolanaAttestationServiceError::InvalidAttestationData))
}

#[test]
fn create_attestation_fail_schema_paused() {
    let mut f = setup();
    let pause_schema_ix = ChangeSchemaStatusBuilder::new()
        .authority(f.authority.pubkey())
        .credential(f.credential)
        .schema(f.schema)
        .is_paused(true)
        .instruction();
    send(&mut f.ctx, &[pause_schema_ix], &[&f.authority]).unwrap();
    let expiry = expiry(&f.ctx);

    let tx_err = create_attestation(&mut f, test_data(), expiry, Address::new_unique()).expect_err("should error");
    assert_eq!(tx_err, program_error(SolanaAttestationServiceError::SchemaPaused))
}
