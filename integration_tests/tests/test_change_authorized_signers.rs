use borsh::BorshDeserialize;
use helpers::program_test_context;
use solana_address::Address;
use solana_attestation_service_client::{
    accounts::Credential,
    instructions::{ChangeAuthorizedSignersBuilder, CreateCredentialBuilder},
};
use solana_keypair::Keypair;
use solana_sdk_ids::system_program;
use solana_signer::Signer;
use solana_transaction::Transaction;

mod helpers;

#[test]
fn change_authorized_signers_success() {
    let mut ctx = program_test_context();

    let authority = Keypair::new();
    let name = "test";

    let (credential_pda, _bump) = Address::find_program_address(
        &[b"credential", &authority.pubkey().to_bytes(), name.as_bytes()],
        &solana_attestation_service_client::programs::SOLANA_ATTESTATION_SERVICE_ID,
    );

    let ix = CreateCredentialBuilder::new()
        .payer(ctx.payer.pubkey())
        .credential(credential_pda)
        .authority(authority.pubkey())
        .system_program(system_program::ID)
        .name(name.to_string())
        .signers(vec![authority.pubkey(), ctx.payer.pubkey()])
        .instruction();

    let transaction = Transaction::new_signed_with_payer(
        &[ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(transaction).unwrap();

    // Test downsizing authorized_signers.
    let new_signers = vec![Keypair::new().pubkey()];
    let ix = ChangeAuthorizedSignersBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential_pda)
        .system_program(system_program::ID)
        .signers(new_signers.clone())
        .instruction();

    let transaction = Transaction::new_signed_with_payer(
        &[ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(transaction).unwrap();

    // Assert credential account
    let credential_account = ctx.svm.get_account(&credential_pda).expect("account not none");

    let credential = Credential::try_from_slice(&credential_account.data).unwrap();
    assert_eq!(credential.authority, authority.pubkey());
    assert_eq!(credential.name, name.as_bytes());
    assert_eq!(credential.authorized_signers.len(), new_signers.len());
    for (i, signer) in credential.authorized_signers.iter().enumerate() {
        assert_eq!(*signer, new_signers[i]);
    }

    // Test upsizing authorized_signers.
    let new_signers = vec![Keypair::new().pubkey(), Keypair::new().pubkey(), Keypair::new().pubkey()];
    let ix = ChangeAuthorizedSignersBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential_pda)
        .system_program(system_program::ID)
        .signers(new_signers.clone())
        .instruction();

    let transaction = Transaction::new_signed_with_payer(
        &[ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(transaction).unwrap();

    // Assert credential account
    let credential_account = ctx.svm.get_account(&credential_pda).expect("account not none");

    let credential = Credential::try_from_slice(&credential_account.data).unwrap();
    assert_eq!(credential.authority, authority.pubkey());
    assert_eq!(credential.name, name.as_bytes());
    assert_eq!(credential.authorized_signers.len(), new_signers.len());
    for (i, signer) in credential.authorized_signers.iter().enumerate() {
        assert_eq!(*signer, new_signers[i]);
    }

    // Test updating with same number of authorized_signers.
    let new_signers = vec![Keypair::new().pubkey(), Keypair::new().pubkey()];
    let ix = ChangeAuthorizedSignersBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential_pda)
        .system_program(system_program::ID)
        .signers(new_signers.clone())
        .instruction();

    let transaction = Transaction::new_signed_with_payer(
        &[ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(transaction).unwrap();

    // Assert credential account
    let credential_account = ctx.svm.get_account(&credential_pda).expect("account not none");

    let credential = Credential::try_from_slice(&credential_account.data).unwrap();
    assert_eq!(credential.authority, authority.pubkey());
    assert_eq!(credential.name, name.as_bytes());
    assert_eq!(credential.authorized_signers.len(), new_signers.len());
    for (i, signer) in credential.authorized_signers.iter().enumerate() {
        assert_eq!(*signer, new_signers[i]);
    }
}
