use borsh::BorshDeserialize;
use helpers::program_test_context;
use solana_attestation_service_client::instructions::CreateCredentialBuilder;
use solana_sdk::{
    pubkey::Pubkey, signature::Keypair, signer::Signer, system_program, transaction::Transaction,
};

mod helpers;

// Copy of onchain `Credential` state, but with Borsh.
#[derive(Clone, Debug, PartialEq, BorshDeserialize)]
pub struct Credential {
    /// Admin of this credential
    pub authority: Pubkey,
    /// Name of this credential
    pub name: String,
    /// List of signers that are allowed to "attest"
    pub authorized_signers: Vec<Pubkey>,
}

#[tokio::test]
async fn create_credential_success() {
    let ctx = program_test_context().await;

    let authority = Keypair::new();
    let name = "test";

    let (credential_pda, _bump) = Pubkey::find_program_address(
        &[
            b"credential",
            &authority.pubkey().to_bytes(),
            name.as_bytes(),
        ],
        &Pubkey::from(solana_attestation_service_client::programs::SOLANA_ATTESTATION_SERVICE_ID),
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

    // Assert credential account
    let credential_account = ctx
        .banks_client
        .get_account(credential_pda)
        .await
        .expect("get_account")
        .expect("account not none");
    let credential = Credential::try_from_slice(&credential_account.data).unwrap();
    assert_eq!(credential.authority, authority.pubkey());
    assert_eq!(credential.name, name.to_string());
    assert_eq!(credential.authorized_signers[0], authority.pubkey());
    assert_eq!(credential.authorized_signers[1], ctx.payer.pubkey());
}
