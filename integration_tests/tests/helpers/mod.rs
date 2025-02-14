use borsh::BorshSerialize;
use solana_attestation_service_client::accounts::Credential;
use solana_program_test::{ProgramTest, ProgramTestContext};
use solana_sdk::{account::AccountSharedData, signature::Keypair, signer::Signer};

/// Get ProgramTestContext with SAS program loaded.
pub async fn program_test_context() -> ProgramTestContext {
    let mut program_test = ProgramTest::default();
    program_test.add_program(
        "solana_attestation_service",
        solana_attestation_service_client::programs::SOLANA_ATTESTATION_SERVICE_ID,
        None,
    );
    let ctx = program_test.start_with_context().await;
    ctx
}

// pub async fn setup_credential(ctx: &mut ProgramTestContext) {
//     let authority = Keypair::new();
//     let credential = Credential {
//         authority: authority.pubkey(),
//         name: b"test-credential".to_vec(),
//         authorized_signers: vec![authority.pubkey()],
//     };
//     // let buf = credential
//     let serialized_credential = credential.serialize()
//     let account = AccountSharedData::new(lamports, space, owner)
// }
