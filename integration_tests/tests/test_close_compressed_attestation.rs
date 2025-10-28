use borsh::BorshSerialize;
use light_program_test::{
    program_test::LightProgramTest, utils::assert::assert_rpc_error, AddressWithTree, Indexer,
    ProgramTestConfig, Rpc,
};
use light_sdk::address::v2::derive_address;
use solana_attestation_service_client::{
    accounts::Attestation,
    errors::SolanaAttestationServiceError,
    instructions::{
        ChangeSchemaStatusBuilder, CloseCompressedAttestationBuilder,
        CreateCompressedAttestationBuilder, CreateCredentialBuilder, CreateSchemaBuilder,
    },
    pdas::{
        derive_attestation_pda, derive_credential_pda, derive_event_authority_pda,
        derive_schema_pda,
    },
    programs::SOLANA_ATTESTATION_SERVICE_ID,
    ALLOWED_ADDRESS_TREE,
};
use solana_sdk::transaction::Transaction;
use solana_sdk::{
    clock::Clock,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use solana_sdk_ids::system_program;

mod helpers;
use helpers::{hash, TestData, TestFixtures};

async fn setup() -> TestFixtures {
    // Initialize Light Protocol test environment with SAS program
    let config = ProgramTestConfig::new_v2(
        true,
        Some(vec![(
            "solana_attestation_service",
            SOLANA_ATTESTATION_SERVICE_ID,
        )]),
    );
    let mut rpc = LightProgramTest::new(config).await.unwrap();
    let payer = rpc.get_payer().insecure_clone();

    let authority = Keypair::new();
    let credential_name = "test";

    // Create credential PDA
    let (credential_pda, _bump) = derive_credential_pda(&authority.pubkey(), credential_name);

    // Create credential
    let create_credential_ix = CreateCredentialBuilder::new()
        .payer(payer.pubkey())
        .credential(credential_pda)
        .authority(authority.pubkey())
        .system_program(system_program::ID)
        .name(credential_name.to_string())
        .signers(vec![authority.pubkey()])
        .instruction();

    rpc.create_and_send_transaction(
        &[create_credential_ix],
        &payer.pubkey(),
        &[&payer, &authority],
    )
    .await
    .unwrap();

    // Create schema
    let schema_name = "test_data";
    let description = "schema for test data";
    let schema_data = TestData::get_serialized_representation();
    let field_names = vec!["name".into(), "location".into()];

    let (schema_pda, _bump) = derive_schema_pda(&credential_pda, schema_name, 1);

    let create_schema_ix = CreateSchemaBuilder::new()
        .payer(payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential_pda)
        .schema(schema_pda)
        .system_program(system_program::ID)
        .description(description.to_string())
        .name(schema_name.to_string())
        .layout(schema_data)
        .field_names(field_names)
        .instruction();

    rpc.create_and_send_transaction(&[create_schema_ix], &payer.pubkey(), &[&payer, &authority])
        .await
        .unwrap();

    // Use the allowed address tree constant
    let address_tree_pubkey = ALLOWED_ADDRESS_TREE;

    let event_auth_pda = derive_event_authority_pda();

    TestFixtures {
        rpc,
        payer,
        credential: credential_pda,
        schema: schema_pda,
        authority,
        address_tree_pubkey,
        event_auth_pda,
        attestations: vec![],
    }
}

struct CompressedAttestationInfo {
    compressed_address: [u8; 32],
    leaf_index: u32,
}

async fn create_compressed_attestation_helper(
    rpc: &mut LightProgramTest,
    payer: &Keypair,
    authority: &Keypair,
    credential: Pubkey,
    schema: Pubkey,
    address_tree_pubkey: Pubkey,
) -> CompressedAttestationInfo {
    // Prepare attestation data
    let attestation_data = TestData {
        name: "attest".to_string(),
        location: 11,
    };
    let mut serialized_attestation_data = Vec::new();
    attestation_data
        .serialize(&mut serialized_attestation_data)
        .unwrap();

    // Get current timestamp for expiry
    let clock = rpc.context.get_sysvar::<Clock>();
    let expiry: i64 = clock.unix_timestamp + 60;

    // Generate nonce for attestation
    let nonce = Pubkey::new_unique();

    // Derive compressed attestation address
    let (attestation_pda, _) = derive_attestation_pda(&credential, &schema, &nonce);

    let (compressed_address, _) = derive_address(
        &[attestation_pda.as_ref()],
        &address_tree_pubkey,
        &SOLANA_ATTESTATION_SERVICE_ID,
    );

    // Get validity proof for the new compressed address
    let rpc_result = rpc
        .get_validity_proof(
            vec![],
            vec![AddressWithTree {
                address: compressed_address,
                tree: address_tree_pubkey,
            }],
            None,
        )
        .await
        .unwrap()
        .value;

    // Get tree pubkeys
    let output_queue = rpc.get_random_state_tree_info().unwrap().queue;
    // Convert proof to fixed array [u8; 128]
    let proof_bytes: [u8; 128] = rpc_result.proof.0.unwrap().to_array();

    // Get address root index from the validity proof result
    let address_root_index = rpc_result.addresses[0].root_index;

    // Use builder to create instruction
    let create_compressed_attestation_ix = CreateCompressedAttestationBuilder::new()
        .payer(payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .schema(schema)
        .output_queue(output_queue)
        .proof(proof_bytes)
        .nonce(nonce)
        .data(serialized_attestation_data.clone())
        .expiry(expiry)
        .address_root_index(address_root_index)
        .instruction();

    rpc.create_and_send_transaction(
        &[create_compressed_attestation_ix],
        &payer.pubkey(),
        &[payer, authority],
    )
    .await
    .unwrap();

    // Get the compressed account to extract leaf_index
    let compressed_account = rpc
        .get_compressed_account(compressed_address, None)
        .await
        .unwrap()
        .value
        .unwrap();

    CompressedAttestationInfo {
        compressed_address,
        leaf_index: compressed_account.leaf_index,
    }
}

#[tokio::test]
async fn test_close_compressed_attestation_success() {
    let TestFixtures {
        mut rpc,
        payer,
        credential,
        schema,
        authority,
        address_tree_pubkey,
        event_auth_pda,
        ..
    } = setup().await;

    // Create a compressed attestation
    let CompressedAttestationInfo {
        compressed_address,
        leaf_index,
    } = create_compressed_attestation_helper(
        &mut rpc,
        &payer,
        &authority,
        credential,
        schema,
        address_tree_pubkey,
    )
    .await;

    // Verify attestation exists before closing
    let compressed_account_before = rpc
        .get_compressed_account(compressed_address, None)
        .await
        .unwrap()
        .value
        .unwrap();

    // Get the full attestation account data with discriminator
    let compressed_data = compressed_account_before.data.as_ref().unwrap();
    let attestation_account_data = &compressed_data.data;
    let stored_data_hash = compressed_data.data_hash;

    // Get validity proof for closing (input account)
    let rpc_result = rpc
        .get_validity_proof(vec![compressed_account_before.hash], vec![], None)
        .await
        .unwrap()
        .value;

    // Extract proof if it exists
    let proof = rpc_result.proof.to_array();

    // Get root_index from the proof result
    let root_index = rpc_result.accounts[0]
        .root_index
        .root_index()
        .unwrap_or_default();

    // Build close instruction
    let mut builder = CloseCompressedAttestationBuilder::new();
    builder
        .payer(payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .event_authority(event_auth_pda)
        .state_merkle_tree(rpc_result.accounts[0].tree_info.tree)
        .output_queue(rpc_result.accounts[0].tree_info.queue);

    // Only set proof if it exists
    if let Some(proof_bytes) = proof {
        builder.proof(proof_bytes);
    }

    // Parse attestation account data
    let attestation = Attestation::from_bytes(attestation_account_data).unwrap();

    // Verify hash computation
    assert_eq!(
        stored_data_hash,
        hash(&attestation),
        "Hash should match stored data_hash"
    );

    let close_compressed_attestation_ix = builder
        .nonce(attestation.nonce)
        .schema(attestation.schema)
        .signer(attestation.signer)
        .expiry(attestation.expiry)
        .data(attestation.data)
        .root_index(root_index)
        .leaf_index(leaf_index)
        .address(compressed_address)
        .instruction();

    // Execute close transaction
    rpc.create_and_send_transaction(
        &[close_compressed_attestation_ix],
        &payer.pubkey(),
        &[&payer, &authority],
    )
    .await
    .unwrap();

    // Verify compressed account is nullified/closed
    let compressed_account_after = rpc
        .get_compressed_account(compressed_address, None)
        .await
        .unwrap()
        .value;

    // The account should be completely removed
    assert!(compressed_account_after.is_none());
}

#[tokio::test]
async fn test_close_compressed_attestation_unauthorized_signer() {
    let TestFixtures {
        mut rpc,
        payer,
        credential,
        schema,
        authority,
        address_tree_pubkey,
        event_auth_pda,
        ..
    } = setup().await;

    // Create a compressed attestation with the authorized authority
    let CompressedAttestationInfo {
        compressed_address,
        leaf_index,
    } = create_compressed_attestation_helper(
        &mut rpc,
        &payer,
        &authority,
        credential,
        schema,
        address_tree_pubkey,
    )
    .await;

    // Create an unauthorized signer (not in the credential's authorized signers)
    let unauthorized_signer = Keypair::new();

    // Get the compressed account data
    let compressed_account_before = rpc
        .get_compressed_account(compressed_address, None)
        .await
        .unwrap()
        .value
        .unwrap();

    let compressed_data = compressed_account_before.data.as_ref().unwrap();
    let attestation_account_data = &compressed_data.data;

    // Get validity proof for closing
    let rpc_result = rpc
        .get_validity_proof(vec![compressed_account_before.hash], vec![], None)
        .await
        .unwrap()
        .value;

    // Extract proof if it exists
    let proof = rpc_result.proof.to_array();

    let root_index = rpc_result.accounts[0]
        .root_index
        .root_index()
        .unwrap_or_default();

    // Build close instruction with UNAUTHORIZED signer
    let mut builder = CloseCompressedAttestationBuilder::new();
    builder
        .payer(payer.pubkey())
        .authority(unauthorized_signer.pubkey()) // Using unauthorized signer
        .credential(credential)
        .event_authority(event_auth_pda)
        .state_merkle_tree(rpc_result.accounts[0].tree_info.tree)
        .output_queue(rpc_result.accounts[0].tree_info.queue);

    if let Some(proof_bytes) = proof {
        builder.proof(proof_bytes);
    }

    // Parse attestation account data
    let attestation = Attestation::from_bytes(attestation_account_data).unwrap();

    let close_compressed_attestation_ix = builder
        .nonce(attestation.nonce)
        .schema(attestation.schema)
        .signer(attestation.signer)
        .expiry(attestation.expiry)
        .data(attestation.data)
        .root_index(root_index)
        .leaf_index(leaf_index)
        .address(compressed_address)
        .instruction();

    // Try to close with unauthorized signer - should fail
    let result = rpc
        .create_and_send_transaction(
            &[close_compressed_attestation_ix],
            &payer.pubkey(),
            &[&payer, &unauthorized_signer],
        )
        .await;

    // Assert fails with SignerNotAuthorized
    assert_rpc_error(
        result,
        0,
        SolanaAttestationServiceError::SignerNotAuthorized as u32,
    )
    .unwrap();
}

#[tokio::test]
async fn test_close_compressed_attestation_wrong_credential() {
    let TestFixtures {
        mut rpc,
        payer,
        credential: credential1,
        schema: schema1,
        authority: authority1,
        address_tree_pubkey,
        event_auth_pda,
        ..
    } = setup().await;

    // Create compressed attestation with credential1 and schema1
    let CompressedAttestationInfo {
        compressed_address,
        leaf_index,
    } = create_compressed_attestation_helper(
        &mut rpc,
        &payer,
        &authority1,
        credential1,
        schema1,
        address_tree_pubkey,
    )
    .await;

    // Create a SECOND credential with different authority
    let authority2 = Keypair::new();
    let credential_name2 = "test2";
    let (credential2_pda, _bump) = derive_credential_pda(&authority2.pubkey(), credential_name2);

    let create_credential2_ix = CreateCredentialBuilder::new()
        .payer(payer.pubkey())
        .credential(credential2_pda)
        .authority(authority2.pubkey())
        .system_program(system_program::ID)
        .name(credential_name2.to_string())
        .signers(vec![authority2.pubkey()])
        .instruction();

    rpc.create_and_send_transaction(
        &[create_credential2_ix],
        &payer.pubkey(),
        &[&payer, &authority2],
    )
    .await
    .unwrap();

    // Get the compressed account data
    let compressed_account_before = rpc
        .get_compressed_account(compressed_address, None)
        .await
        .unwrap()
        .value
        .unwrap();

    let compressed_data = compressed_account_before.data.as_ref().unwrap();
    let attestation_account_data = &compressed_data.data;

    // Get validity proof for closing
    let rpc_result = rpc
        .get_validity_proof(vec![compressed_account_before.hash], vec![], None)
        .await
        .unwrap()
        .value;

    // Extract proof if it exists
    let proof = rpc_result.proof.to_array();

    let root_index = rpc_result.accounts[0]
        .root_index
        .root_index()
        .unwrap_or_default();

    // Try to close attestation1 using WRONG credential2 (attestation belongs to credential1)
    let mut builder = CloseCompressedAttestationBuilder::new();
    builder
        .payer(payer.pubkey())
        .authority(authority2.pubkey()) // Using authority2
        .credential(credential2_pda) // Using credential2 (WRONG - attestation belongs to credential1)
        .event_authority(event_auth_pda)
        .state_merkle_tree(rpc_result.accounts[0].tree_info.tree)
        .output_queue(rpc_result.accounts[0].tree_info.queue);

    if let Some(proof_bytes) = proof {
        builder.proof(proof_bytes);
    }

    // Parse attestation account data
    let attestation = Attestation::from_bytes(attestation_account_data).unwrap();

    let close_compressed_attestation_ix = builder
        .nonce(attestation.nonce)
        .schema(attestation.schema)
        .signer(attestation.signer)
        .expiry(attestation.expiry)
        .data(attestation.data)
        .root_index(root_index)
        .leaf_index(leaf_index)
        .address(compressed_address)
        .instruction();

    // Try to close with wrong credential - should fail
    let result = rpc
        .create_and_send_transaction(
            &[close_compressed_attestation_ix],
            &payer.pubkey(),
            &[&payer, &authority2],
        )
        .await;

    // Assert fails with error code 14307 invalide credential produces invalid hash.
    // Alternatively fails with proof generation failed.
    assert_rpc_error(result, 0, 14307).unwrap();
}

#[tokio::test]
async fn test_close_compressed_attestation_paused_schema_success() {
    let TestFixtures {
        mut rpc,
        payer,
        credential,
        schema,
        authority,
        address_tree_pubkey,
        event_auth_pda,
        ..
    } = setup().await;

    // Create a compressed attestation BEFORE pausing the schema
    let CompressedAttestationInfo {
        compressed_address,
        leaf_index,
    } = create_compressed_attestation_helper(
        &mut rpc,
        &payer,
        &authority,
        credential,
        schema,
        address_tree_pubkey,
    )
    .await;

    // NOW pause the schema
    let pause_schema_ix = ChangeSchemaStatusBuilder::new()
        .authority(authority.pubkey())
        .credential(credential)
        .schema(schema)
        .is_paused(true)
        .instruction();

    rpc.create_and_send_transaction(&[pause_schema_ix], &payer.pubkey(), &[&payer, &authority])
        .await
        .unwrap();

    // Get the compressed account data
    let compressed_account_before = rpc
        .get_compressed_account(compressed_address, None)
        .await
        .unwrap()
        .value
        .unwrap();

    let compressed_data = compressed_account_before.data.as_ref().unwrap();
    let attestation_account_data = &compressed_data.data;
    let stored_data_hash = compressed_data.data_hash;

    // Get validity proof for closing
    let rpc_result = rpc
        .get_validity_proof(vec![compressed_account_before.hash], vec![], None)
        .await
        .unwrap()
        .value;

    // Extract proof if it exists
    let proof = rpc_result.proof.to_array();

    let root_index = rpc_result.accounts[0]
        .root_index
        .root_index()
        .unwrap_or_default();

    // Build close instruction (schema is paused, but close should still work)
    let mut builder = CloseCompressedAttestationBuilder::new();
    builder
        .payer(payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .event_authority(event_auth_pda)
        .state_merkle_tree(rpc_result.accounts[0].tree_info.tree)
        .output_queue(rpc_result.accounts[0].tree_info.queue);

    if let Some(proof_bytes) = proof {
        builder.proof(proof_bytes);
    }

    // Parse attestation account data
    let attestation = Attestation::from_bytes(attestation_account_data).unwrap();

    // Verify hash computation
    assert_eq!(
        stored_data_hash,
        hash(&attestation),
        "Hash should match stored data_hash"
    );

    let close_compressed_attestation_ix = builder
        .nonce(attestation.nonce)
        .schema(attestation.schema)
        .signer(attestation.signer)
        .expiry(attestation.expiry)
        .data(attestation.data)
        .root_index(root_index)
        .leaf_index(leaf_index)
        .address(compressed_address)
        .instruction();

    // Execute close transaction - should SUCCEED despite paused schema
    rpc.create_and_send_transaction(
        &[close_compressed_attestation_ix],
        &payer.pubkey(),
        &[&payer, &authority],
    )
    .await
    .unwrap();

    // Verify compressed account is nullified/closed
    let compressed_account_after = rpc
        .get_compressed_account(compressed_address, None)
        .await
        .unwrap()
        .value;

    // The account should be completely removed (close succeeded despite paused schema)
    assert!(compressed_account_after.is_none());
}

#[tokio::test]
async fn test_close_compressed_attestation_invalid_attestation_data() {
    let TestFixtures {
        mut rpc,
        payer,
        credential,
        schema,
        authority,
        address_tree_pubkey,
        event_auth_pda,
        ..
    } = setup().await;

    // Create a compressed attestation
    let CompressedAttestationInfo {
        compressed_address,
        leaf_index,
    } = create_compressed_attestation_helper(
        &mut rpc,
        &payer,
        &authority,
        credential,
        schema,
        address_tree_pubkey,
    )
    .await;

    // Get the compressed account data
    let compressed_account_before = rpc
        .get_compressed_account(compressed_address, None)
        .await
        .unwrap()
        .value
        .unwrap();

    // Get the CORRECT attestation data
    let correct_attestation_account_data = compressed_account_before.data.unwrap().data;

    // Create WRONG attestation data (modify it)
    let mut wrong_attestation_data = correct_attestation_account_data.clone();

    // Mutate the actual attestation data field (not credential/schema fields)
    // Attestation layout: discriminator(1) + nonce(32) + credential(32) + schema(32) + data_len(4) + data(...)
    let data_field_start = 1 + 32 + 32 + 32 + 4; // = 101
    assert!(
        wrong_attestation_data.len() > data_field_start,
        "Attestation account is only {} bytes, cannot mutate data field at index {}",
        wrong_attestation_data.len(),
        data_field_start
    );
    wrong_attestation_data[data_field_start] =
        wrong_attestation_data[data_field_start].wrapping_add(1);

    // Get validity proof for closing
    let rpc_result = rpc
        .get_validity_proof(vec![compressed_account_before.hash], vec![], None)
        .await
        .unwrap()
        .value;

    // Extract proof if it exists
    let proof = rpc_result.proof.to_array();

    let root_index = rpc_result.accounts[0]
        .root_index
        .root_index()
        .unwrap_or_default();

    // Build close instruction with WRONG attestation data
    let mut builder = CloseCompressedAttestationBuilder::new();
    builder
        .payer(payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .event_authority(event_auth_pda)
        .state_merkle_tree(rpc_result.accounts[0].tree_info.tree)
        .output_queue(rpc_result.accounts[0].tree_info.queue);

    if let Some(proof_bytes) = proof {
        builder.proof(proof_bytes);
    }

    // Parse wrong attestation account data
    let attestation = Attestation::from_bytes(&wrong_attestation_data).unwrap();

    let close_compressed_attestation_ix = builder
        .nonce(attestation.nonce)
        .schema(attestation.schema)
        .signer(attestation.signer)
        .expiry(attestation.expiry)
        .data(attestation.data) // Using WRONG data
        .root_index(root_index)
        .leaf_index(leaf_index)
        .address(compressed_address)
        .instruction();

    // Try to close with wrong attestation data - should fail
    let result = rpc
        .create_and_send_transaction(
            &[close_compressed_attestation_ix],
            &payer.pubkey(),
            &[&payer, &authority],
        )
        .await;

    // Should fail with Light Protocol error 14307 (hash mismatch in merkle tree)
    assert_rpc_error(result, 0, 14307).unwrap();
}

#[tokio::test]
async fn test_close_compressed_attestation_max_data_size() {
    let TestFixtures {
        mut rpc,
        payer,
        credential,
        schema: _,
        authority,
        address_tree_pubkey,
        event_auth_pda,
        ..
    } = setup().await;

    // Create schema with VecU8 layout to test maximum data size
    let schema_name = "large_data_schema";
    let description = "schema for testing max data size";
    let schema_data = vec![13]; // VecU8 type
    let field_names = vec!["large_data".into()];

    let (large_schema_pda, _bump) = derive_schema_pda(&credential, schema_name, 1);

    let create_schema_ix = CreateSchemaBuilder::new()
        .payer(payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .schema(large_schema_pda)
        .system_program(system_program::ID)
        .description(description.to_string())
        .name(schema_name.to_string())
        .layout(schema_data)
        .field_names(field_names)
        .instruction();

    rpc.create_and_send_transaction(&[create_schema_ix], &payer.pubkey(), &[&payer, &authority])
        .await
        .unwrap();

    // Create attestation with large data (testing transaction size limits)
    // MAX_COMPRESSED_ATTESTATION_SIZE: 350 bytes
    let large_data: Vec<u8> = vec![42u8; 346]; // The schema adds 4 more bytes.

    // Serialize the Vec<u8> with length prefix
    let mut serialized_data = Vec::new();
    serialized_data.extend((large_data.len() as u32).to_le_bytes());
    serialized_data.extend(large_data);

    // Get current timestamp for expiry
    let clock = rpc.context.get_sysvar::<Clock>();
    let expiry: i64 = clock.unix_timestamp + 60;

    // Generate nonce for attestation
    let nonce = Pubkey::new_unique();

    // Derive compressed attestation address
    let (attestation_pda, _) = derive_attestation_pda(&credential, &large_schema_pda, &nonce);

    let (compressed_address, _) = derive_address(
        &[attestation_pda.as_ref()],
        &address_tree_pubkey,
        &SOLANA_ATTESTATION_SERVICE_ID,
    );

    // Get validity proof for the new compressed address
    let rpc_result = rpc
        .get_validity_proof(
            vec![],
            vec![AddressWithTree {
                address: compressed_address,
                tree: address_tree_pubkey,
            }],
            None,
        )
        .await
        .unwrap()
        .value;

    let output_queue = rpc.get_random_state_tree_info().unwrap().queue;

    // Convert proof to array
    let proof_bytes: [u8; 128] = rpc_result.proof.0.unwrap().to_array();
    let address_root_index = rpc_result.addresses[0].root_index;

    // Measure transaction size before sending
    // Use separate payer to simulate production (2 signatures)
    let separate_payer = Keypair::new();

    // Fund the separate payer
    rpc.context
        .airdrop(&separate_payer.pubkey(), 10_000_000_000)
        .unwrap();

    // Create compressed attestation with large data
    let create_compressed_attestation_ix = CreateCompressedAttestationBuilder::new()
        .payer(separate_payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .schema(large_schema_pda)
        .output_queue(output_queue)
        .proof(proof_bytes)
        .nonce(nonce)
        .data(serialized_data.clone())
        .expiry(expiry)
        .address_root_index(address_root_index)
        .instruction();

    let recent_blockhash = rpc.get_latest_blockhash().await.unwrap();
    let create_tx = Transaction::new_signed_with_payer(
        std::slice::from_ref(&create_compressed_attestation_ix),
        Some(&separate_payer.pubkey()),
        &[&separate_payer, &authority],
        recent_blockhash.0,
    );

    let create_tx_size = bincode::serialize(&create_tx).unwrap().len();
    println!(
        "Create transaction size (2 signers): {} bytes",
        create_tx_size
    );

    // Mainnet transaction MTU is 1232 bytes
    const MAINNET_TX_MTU: usize = 1232;
    assert!(
        create_tx_size <= MAINNET_TX_MTU,
        "Create transaction too large: {} bytes (max: {} bytes)",
        create_tx_size,
        MAINNET_TX_MTU
    );

    // This should succeed - creating attestation with 350 bytes data
    rpc.create_and_send_transaction(
        &[create_compressed_attestation_ix],
        &separate_payer.pubkey(),
        &[&separate_payer, &authority],
    )
    .await
    .unwrap();

    // Now test closing the large attestation
    let compressed_account = rpc
        .get_compressed_account(compressed_address, None)
        .await
        .unwrap()
        .value
        .unwrap();

    let attestation_account_data = compressed_account.data.unwrap().data;
    let leaf_index = compressed_account.leaf_index;

    // Get validity proof for closing
    let rpc_result = rpc
        .get_validity_proof(vec![compressed_account.hash], vec![], None)
        .await
        .unwrap()
        .value;

    // Extract proof if it exists
    let proof = rpc_result.proof.to_array();

    let root_index = rpc_result.accounts[0]
        .root_index
        .root_index()
        .unwrap_or_default();

    // Build close instruction
    let mut builder = CloseCompressedAttestationBuilder::new();
    builder
        .payer(separate_payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .event_authority(event_auth_pda)
        .state_merkle_tree(rpc_result.accounts[0].tree_info.tree)
        .output_queue(rpc_result.accounts[0].tree_info.queue);

    if let Some(proof_bytes) = proof {
        builder.proof(proof_bytes);
    }

    // Parse attestation account data
    let attestation = Attestation::from_bytes(&attestation_account_data).unwrap();

    let close_compressed_attestation_ix = builder
        .nonce(attestation.nonce)
        .schema(attestation.schema)
        .signer(attestation.signer)
        .expiry(attestation.expiry)
        .data(attestation.data)
        .root_index(root_index)
        .leaf_index(leaf_index)
        .address(compressed_address)
        .instruction();

    // Measure close transaction size (with 2 signers)
    let recent_blockhash = rpc.get_latest_blockhash().await.unwrap();
    let close_tx = Transaction::new_signed_with_payer(
        std::slice::from_ref(&close_compressed_attestation_ix),
        Some(&separate_payer.pubkey()),
        &[&separate_payer, &authority],
        recent_blockhash.0,
    );
    let close_tx_size = bincode::serialize(&close_tx).unwrap().len();
    println!(
        "Close transaction size (2 signers): {} bytes",
        close_tx_size
    );

    assert!(
        close_tx_size <= MAINNET_TX_MTU,
        "Close transaction too large: {} bytes (max: {} bytes)",
        close_tx_size,
        MAINNET_TX_MTU
    );

    // This should succeed - closing attestation with 350 bytes data fits in transaction
    rpc.create_and_send_transaction(
        &[close_compressed_attestation_ix],
        &separate_payer.pubkey(),
        &[&separate_payer, &authority],
    )
    .await
    .unwrap();

    // Verify compressed account is closed
    let compressed_account_after = rpc
        .get_compressed_account(compressed_address, None)
        .await
        .unwrap()
        .value;

    assert!(compressed_account_after.is_none());
}
