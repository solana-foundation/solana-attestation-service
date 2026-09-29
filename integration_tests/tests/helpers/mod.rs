#![allow(dead_code)]

use borsh::BorshSerialize;
use litesvm::LiteSVM;
use solana_address::Address;
use solana_attestation_service_client::{
    errors::SolanaAttestationServiceError,
    instructions::{CreateAttestationBuilder, CreateCredentialBuilder, CreateSchemaBuilder},
    programs::SOLANA_ATTESTATION_SERVICE_ID,
};
use solana_clock::Clock;
use solana_instruction_error::InstructionError;
use solana_keypair::Keypair;
use solana_signer::Signer;
use solana_transaction::{Instruction, Transaction};
use solana_transaction_error::TransactionError;

pub struct TestContext {
    pub svm: LiteSVM,
    pub payer: Keypair,
}

#[derive(BorshSerialize)]
struct TestData {
    name: String,
    location: u8,
}

pub struct TestFixtures {
    pub ctx: TestContext,
    pub credential: Address,
    pub schema: Address,
    pub authority: Keypair,
}

/// Get a LiteSVM instance with the SAS program loaded and a funded payer.
pub fn program_test_context() -> TestContext {
    let mut svm = LiteSVM::new();
    let program_path =
        std::path::Path::new(&std::env::var("SBF_OUT_DIR").expect("SBF_OUT_DIR")).join("solana_attestation_service.so");
    svm.add_program_from_file(SOLANA_ATTESTATION_SERVICE_ID, program_path).unwrap();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 100_000_000_000).unwrap();
    TestContext { svm, payer }
}

pub fn send(ctx: &mut TestContext, ixs: &[Instruction], signers: &[&Keypair]) -> Result<(), TransactionError> {
    let mut all_signers = vec![&ctx.payer];
    all_signers.extend_from_slice(signers);
    let transaction =
        Transaction::new_signed_with_payer(ixs, Some(&ctx.payer.pubkey()), &all_signers, ctx.svm.latest_blockhash());
    ctx.svm.send_transaction(transaction).map(|_| ()).map_err(|e| e.err)
}

pub fn create_credential(ctx: &mut TestContext, authority: &Keypair, name: &str, signers: Vec<Address>) -> Address {
    let (credential, _bump) = Address::find_program_address(
        &[b"credential", &authority.pubkey().to_bytes(), name.as_bytes()],
        &SOLANA_ATTESTATION_SERVICE_ID,
    );
    let ix = CreateCredentialBuilder::new()
        .payer(ctx.payer.pubkey())
        .credential(credential)
        .authority(authority.pubkey())
        .name(name.to_string())
        .signers(signers)
        .instruction();
    send(ctx, &[ix], &[authority]).unwrap();
    credential
}

pub fn create_schema(
    ctx: &mut TestContext,
    authority: &Keypair,
    credential: Address,
    name: &str,
    description: &str,
    layout: Vec<u8>,
    field_names: Vec<String>,
) -> Address {
    let (schema, _bump) = Address::find_program_address(
        &[b"schema", &credential.to_bytes(), name.as_bytes(), &[1]],
        &SOLANA_ATTESTATION_SERVICE_ID,
    );
    let ix = CreateSchemaBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .schema(schema)
        .description(description.to_string())
        .name(name.to_string())
        .layout(layout)
        .field_names(field_names)
        .instruction();
    send(ctx, &[ix], &[authority]).unwrap();
    schema
}

pub fn setup() -> TestFixtures {
    let mut ctx = program_test_context();
    let authority = Keypair::new();
    let credential = create_credential(&mut ctx, &authority, "test", vec![authority.pubkey()]);
    let schema = create_schema(
        &mut ctx,
        &authority,
        credential,
        "test_data",
        "schema for test data",
        vec![12, 0],
        vec!["name".into(), "location".into()],
    );
    TestFixtures { ctx, credential, schema, authority }
}

pub fn attestation_pda(credential: &Address, schema: &Address, nonce: &Address) -> Address {
    Address::find_program_address(
        &[b"attestation", &credential.to_bytes(), &schema.to_bytes(), &nonce.to_bytes()],
        &SOLANA_ATTESTATION_SERVICE_ID,
    )
    .0
}

pub fn create_attestation(
    f: &mut TestFixtures,
    data: Vec<u8>,
    expiry: i64,
    nonce: Address,
) -> Result<Address, TransactionError> {
    let attestation = attestation_pda(&f.credential, &f.schema, &nonce);
    let ix = CreateAttestationBuilder::new()
        .payer(f.ctx.payer.pubkey())
        .authority(f.authority.pubkey())
        .credential(f.credential)
        .schema(f.schema)
        .attestation(attestation)
        .data(data)
        .expiry(expiry)
        .nonce(nonce)
        .instruction();
    send(&mut f.ctx, &[ix], &[&f.authority])?;
    Ok(attestation)
}

pub fn test_data() -> Vec<u8> {
    borsh::to_vec(&TestData { name: "attest".to_string(), location: 11 }).unwrap()
}

pub fn expiry(ctx: &TestContext) -> i64 {
    ctx.svm.get_sysvar::<Clock>().unix_timestamp + 60
}

pub fn program_error(error: SolanaAttestationServiceError) -> TransactionError {
    TransactionError::InstructionError(0, InstructionError::Custom(error as u32))
}
