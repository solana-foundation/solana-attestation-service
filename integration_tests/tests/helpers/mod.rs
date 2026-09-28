use solana_program_test::{ProgramTest, ProgramTestContext};
use solana_sdk::rent::Rent;

/// Get ProgramTestContext with SAS program loaded.
pub async fn program_test_context() -> ProgramTestContext {
    let mut program_test = ProgramTest::default();
    program_test.add_program(
        "solana_attestation_service",
        solana_attestation_service_client::programs::SOLANA_ATTESTATION_SERVICE_ID,
        None,
    );
    let ctx = program_test.start_with_context().await;
    // Post-SIMD-0194 rent, as on mainnet: pinocchio reads the first field as lamports per byte.
    ctx.set_sysvar(&Rent { lamports_per_byte_year: 6960, exemption_threshold: 1.0, ..Rent::default() });
    ctx
}
