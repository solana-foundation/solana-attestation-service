#![allow(dead_code)]
use light_hasher::sha256::Sha256BE;
use light_hasher::Sha256;
use light_program_test::LightProgramTest;
use solana_attestation_service_client::accounts::Attestation;
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

use borsh::BorshSerialize;
use solana_attestation_service_macros::SchemaStructSerialize;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;

/// Common test data structure used across integration tests
#[derive(BorshSerialize, SchemaStructSerialize)]
pub struct TestData {
    pub name: String,
    pub location: u8,
}

pub struct TestFixtures {
    pub rpc: LightProgramTest,
    pub payer: Keypair,
    pub credential: Pubkey,
    pub schema: Pubkey,
    pub authority: Keypair,
    pub address_tree_pubkey: Pubkey,
    pub event_auth_pda: Pubkey,
    pub attestations: Vec<Pubkey>, // Attestation PDAs created during setup
}

pub fn hash(attestation: &Attestation) -> [u8; 32] {
    use light_hasher::Hasher;
    let mut metadata1 = [0u8; 96]; // 3 * 32 bytes for Pubkey
    metadata1[..32].copy_from_slice(attestation.nonce.as_ref());
    metadata1[32..64].copy_from_slice(attestation.signer.as_ref());
    metadata1[64..96].copy_from_slice(attestation.token_account.as_ref());

    let mut metadata2 = [0u8; 72]; // 2 * 32 bytes for Pubkey + 8 bytes for i64
    metadata2[..32].copy_from_slice(attestation.schema.as_ref());
    metadata2[32..64].copy_from_slice(attestation.credential.as_ref());
    metadata2[64..72].copy_from_slice(&attestation.expiry.to_le_bytes());

    // # SAFETY Sha256BE unwrap cannot fail.
    let metadata_hash = Sha256::hash(&metadata1).unwrap();
    let metadata2_hash = Sha256::hash(&metadata2).unwrap();
    let data_hash = Sha256::hash(&attestation.data).unwrap();
    Sha256BE::hashv(&[
        metadata_hash.as_slice(),
        metadata2_hash.as_slice(),
        data_hash.as_slice(),
    ])
    .unwrap()
}
