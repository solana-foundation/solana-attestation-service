use solana_program_test::{ProgramTest, ProgramTestContext};

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
