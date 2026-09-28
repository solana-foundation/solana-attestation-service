use borsh::BorshDeserialize;
use helpers::program_test_context;
use solana_attestation_service_client::{accounts::Credential, instructions::CreateCredentialBuilder};
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_sdk_ids::system_program;
use solana_signer::Signer;
use solana_transaction::Transaction;

mod helpers;

#[test]
fn create_credential_success() {
    let mut ctx = program_test_context();

    let authority = Keypair::new();
    let name = "test";

    let (credential_pda, _bump) = Pubkey::find_program_address(
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

    // Assert credential account
    let credential_account = ctx.svm.get_account(&credential_pda).expect("account not none");

    let credential = Credential::try_from_slice(&credential_account.data).unwrap();
    assert_eq!(credential.authority, authority.pubkey());
    assert_eq!(credential.name, name.as_bytes());
    assert_eq!(credential.authorized_signers[0], authority.pubkey());
    assert_eq!(credential.authorized_signers[1], ctx.payer.pubkey());
}
