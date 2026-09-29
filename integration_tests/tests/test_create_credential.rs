use borsh::BorshDeserialize;
use helpers::{create_credential, program_test_context};
use solana_attestation_service_client::accounts::Credential;
use solana_keypair::Keypair;
use solana_signer::Signer;

mod helpers;

#[test]
fn create_credential_success() {
    let mut ctx = program_test_context();
    let authority = Keypair::new();
    let name = "test";

    let signers = vec![authority.pubkey(), ctx.payer.pubkey()];

    let credential_pda = create_credential(&mut ctx, &authority, name, signers);

    let credential_account = ctx.svm.get_account(&credential_pda).expect("account not none");
    let credential = Credential::try_from_slice(&credential_account.data).unwrap();
    assert_eq!(credential.authority, authority.pubkey());
    assert_eq!(credential.name, name.as_bytes());
    assert_eq!(credential.authorized_signers[0], authority.pubkey());
    assert_eq!(credential.authorized_signers[1], ctx.payer.pubkey());
}
