use borsh::BorshDeserialize;
use helpers::program_test_context;
use solana_attestation_service_client::{
    accounts::Credential,
    instructions::{ChangeAuthorizedSignersBuilder, CreateCredentialBuilder},
};
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};
use solana_sdk_ids::system_program;

mod helpers;

#[tokio::test]
async fn change_authorized_signers_success() {
    let ctx = program_test_context().await;

    let authority = Keypair::new();
    let name = "test";

    let (credential_pda, _bump) = Pubkey::find_program_address(
        &[
            b"credential",
            &authority.pubkey().to_bytes(),
            name.as_bytes(),
        ],
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
        ctx.last_blockhash,
    );
    ctx.banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

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
        ctx.last_blockhash,
    );
    ctx.banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // Assert credential account
    let credential_account = ctx
        .banks_client
        .get_account(credential_pda)
        .await
        .expect("get_account")
        .expect("account not none");

    let credential = Credential::try_from_slice(&credential_account.data).unwrap();
    assert_eq!(credential.authority, authority.pubkey());
    assert_eq!(credential.name, name.as_bytes());
    assert_eq!(credential.authorized_signers.len(), new_signers.len());
    for (authority, new_signer) in credential.authorized_signers.iter().zip(new_signers.iter()) {
        assert_eq!(authority, new_signer);
    }

    // Test upsizing authorized_signers.
    let new_signers = vec![
        Keypair::new().pubkey(),
        Keypair::new().pubkey(),
        Keypair::new().pubkey(),
    ];
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
        ctx.last_blockhash,
    );
    ctx.banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // Assert credential account
    let credential_account = ctx
        .banks_client
        .get_account(credential_pda)
        .await
        .expect("get_account")
        .expect("account not none");

    let credential = Credential::try_from_slice(&credential_account.data).unwrap();
    assert_eq!(credential.authority, authority.pubkey());
    assert_eq!(credential.name, name.as_bytes());
    assert_eq!(credential.authorized_signers.len(), new_signers.len());
    for (authority, new_signer) in credential.authorized_signers.iter().zip(new_signers.iter()) {
        assert_eq!(authority, new_signer);
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
        ctx.last_blockhash,
    );
    ctx.banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // Assert credential account
    let credential_account = ctx
        .banks_client
        .get_account(credential_pda)
        .await
        .expect("get_account")
        .expect("account not none");

    let credential = Credential::try_from_slice(&credential_account.data).unwrap();
    assert_eq!(credential.authority, authority.pubkey());
    assert_eq!(credential.name, name.as_bytes());
    assert_eq!(credential.authorized_signers.len(), new_signers.len());
    for (authority, new_signer) in credential.authorized_signers.iter().zip(new_signers.iter()) {
        assert_eq!(authority, new_signer);
    }
}
