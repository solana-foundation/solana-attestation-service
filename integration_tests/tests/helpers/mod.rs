use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_signer::Signer;

pub struct TestContext {
    pub svm: LiteSVM,
    pub payer: Keypair,
}

/// Get a LiteSVM instance with the SAS program loaded and a funded payer.
pub fn program_test_context() -> TestContext {
    let mut svm = LiteSVM::new();
    let program_path =
        std::path::Path::new(&std::env::var("SBF_OUT_DIR").expect("SBF_OUT_DIR")).join("solana_attestation_service.so");
    svm.add_program_from_file(solana_attestation_service_client::programs::SOLANA_ATTESTATION_SERVICE_ID, program_path)
        .unwrap();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 100_000_000_000).unwrap();
    TestContext { svm, payer }
}
