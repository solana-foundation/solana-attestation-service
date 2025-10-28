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
        ChangeSchemaStatusBuilder, CreateCompressedAttestationBuilder, CreateCredentialBuilder,
        CreateSchemaBuilder,
    },
    pdas::{derive_attestation_pda, derive_credential_pda, derive_schema_pda},
    programs::SOLANA_ATTESTATION_SERVICE_ID,
    ALLOWED_ADDRESS_TREE,
};
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

    TestFixtures {
        rpc,
        payer,
        credential: credential_pda,
        schema: schema_pda,
        authority,
        address_tree_pubkey,
        event_auth_pda: Pubkey::default(),
        attestations: vec![],
    }
}

#[tokio::test]
async fn test_create_compressed_attestation_success() {
    let TestFixtures {
        mut rpc,
        payer,
        credential,
        schema,
        authority,
        address_tree_pubkey,
        ..
    } = setup().await;

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

    // Serialize proof to fixed array [u8; 128]

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
        &[&payer, &authority],
    )
    .await
    .unwrap();

    // Verify the compressed attestation was created
    let compressed_account = rpc
        .get_compressed_account(compressed_address, None)
        .await
        .unwrap()
        .value
        .unwrap();

    assert_eq!(compressed_account.address.unwrap(), compressed_address);

    // Compressed account data includes the discriminator
    let attestation =
        Attestation::from_bytes(&compressed_account.data.as_ref().unwrap().data).unwrap();

    // Assert all attestation fields match what was sent
    assert_eq!(attestation.nonce, nonce);
    assert_eq!(attestation.credential, credential);
    assert_eq!(attestation.schema, schema);
    assert_eq!(attestation.data, serialized_attestation_data);
    assert_eq!(attestation.signer, authority.pubkey());
    assert_eq!(attestation.expiry, expiry);
    assert_eq!(attestation.token_account, Pubkey::default());

    // Verify hash computation
    assert_eq!(
        compressed_account.data.as_ref().unwrap().data_hash,
        hash(&attestation),
        "Hash should match stored data_hash"
    );
}

#[tokio::test]
async fn test_create_compressed_attestation_invalid_data() {
    let TestFixtures {
        mut rpc,
        payer,
        credential,
        schema,
        authority,
        address_tree_pubkey,
        ..
    } = setup().await;

    // Prepare INVALID attestation data (missing the location field)
    let mut invalid_data = Vec::new();
    let name = "invalid".to_string();
    invalid_data.extend((name.len() as u32).to_le_bytes());
    invalid_data.extend(name.as_bytes());
    // Missing the u8 location field!

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

    // Serialize proof to fixed array [u8; 128]
    let proof_bytes = rpc_result.proof.0.unwrap().to_array();

    // Get address root index from the validity proof result
    let address_root_index = rpc_result.addresses[0].root_index;

    // Use builder to create instruction with invalid data
    let create_compressed_attestation_ix = CreateCompressedAttestationBuilder::new()
        .payer(payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .schema(schema)
        .output_queue(output_queue)
        .proof(proof_bytes)
        .nonce(nonce)
        .data(invalid_data)
        .expiry(expiry)
        .address_root_index(address_root_index)
        .instruction();

    // Should fail with InvalidAttestationData
    let result = rpc
        .create_and_send_transaction(
            &[create_compressed_attestation_ix],
            &payer.pubkey(),
            &[&payer, &authority],
        )
        .await;
    assert_rpc_error(
        result,
        0,
        SolanaAttestationServiceError::InvalidAttestationData as u32,
    )
    .unwrap();
}

#[tokio::test]
async fn test_create_compressed_attestation_paused_schema() {
    let TestFixtures {
        mut rpc,
        payer,
        credential,
        schema,
        authority,
        address_tree_pubkey,
        ..
    } = setup().await;

    // Pause the schema
    let pause_schema_ix = ChangeSchemaStatusBuilder::new()
        .authority(authority.pubkey())
        .credential(credential)
        .schema(schema)
        .is_paused(true)
        .instruction();

    rpc.create_and_send_transaction(&[pause_schema_ix], &payer.pubkey(), &[&payer, &authority])
        .await
        .unwrap();

    // Prepare valid attestation data
    let attestation_data = TestData {
        name: "test".to_string(),
        location: 5,
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

    // Serialize proof to fixed array [u8; 128]

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
        .data(serialized_attestation_data)
        .expiry(expiry)
        .address_root_index(address_root_index)
        .instruction();

    // Should fail with SchemaPaused
    let result = rpc
        .create_and_send_transaction(
            &[create_compressed_attestation_ix],
            &payer.pubkey(),
            &[&payer, &authority],
        )
        .await;

    assert_rpc_error(
        result,
        0,
        SolanaAttestationServiceError::SchemaPaused as u32,
    )
    .unwrap();
}

#[tokio::test]
async fn test_create_compressed_attestation_unauthorized_signer() {
    let TestFixtures {
        mut rpc,
        payer,
        credential,
        schema,
        authority: _,
        address_tree_pubkey,
        ..
    } = setup().await;

    // Create an unauthorized signer (not in the credential's authorized signers)
    let unauthorized_signer = Keypair::new();

    // Prepare valid attestation data
    let attestation_data = TestData {
        name: "test".to_string(),
        location: 5,
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

    // Serialize proof to fixed array [u8; 128]

    let proof_bytes: [u8; 128] = rpc_result.proof.0.unwrap().to_array();

    // Get address root index from the validity proof result
    let address_root_index = rpc_result.addresses[0].root_index;

    // Use builder to create instruction with unauthorized signer
    let create_compressed_attestation_ix = CreateCompressedAttestationBuilder::new()
        .payer(payer.pubkey())
        .authority(unauthorized_signer.pubkey())
        .credential(credential)
        .schema(schema)
        .output_queue(output_queue)
        .proof(proof_bytes)
        .nonce(nonce)
        .data(serialized_attestation_data)
        .expiry(expiry)
        .address_root_index(address_root_index)
        .instruction();

    // Should fail with SignerNotAuthorized
    let result = rpc
        .create_and_send_transaction(
            &[create_compressed_attestation_ix],
            &payer.pubkey(),
            &[&payer, &unauthorized_signer],
        )
        .await;

    assert_rpc_error(
        result,
        0,
        SolanaAttestationServiceError::SignerNotAuthorized as u32,
    )
    .unwrap();
}

#[tokio::test]
async fn test_create_compressed_attestation_expired() {
    let TestFixtures {
        mut rpc,
        payer,
        credential,
        schema,
        authority,
        address_tree_pubkey,
        ..
    } = setup().await;

    // Prepare valid attestation data
    let attestation_data = TestData {
        name: "test".to_string(),
        location: 5,
    };
    let mut serialized_attestation_data = Vec::new();
    attestation_data
        .serialize(&mut serialized_attestation_data)
        .unwrap();

    // Set expiry to past timestamp
    let clock = rpc.context.get_sysvar::<Clock>();
    let expiry: i64 = clock.unix_timestamp - 60; // 60 seconds ago

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

    // Serialize proof to fixed array [u8; 128]

    let proof_bytes: [u8; 128] = rpc_result.proof.0.unwrap().to_array();

    // Get address root index from the validity proof result
    let address_root_index = rpc_result.addresses[0].root_index;

    // Use builder to create instruction with expired timestamp
    let create_compressed_attestation_ix = CreateCompressedAttestationBuilder::new()
        .payer(payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .schema(schema)
        .output_queue(output_queue)
        .proof(proof_bytes)
        .nonce(nonce)
        .data(serialized_attestation_data)
        .expiry(expiry)
        .address_root_index(address_root_index)
        .instruction();

    // Should fail with InvalidAttestationData (expired timestamp)
    let result = rpc
        .create_and_send_transaction(
            &[create_compressed_attestation_ix],
            &payer.pubkey(),
            &[&payer, &authority],
        )
        .await;

    assert_rpc_error(
        result,
        0,
        SolanaAttestationServiceError::InvalidAttestationData as u32,
    )
    .unwrap();
}

#[tokio::test]
async fn test_create_compressed_attestation_wrong_credential() {
    let TestFixtures {
        mut rpc,
        payer,
        credential: _credential1,
        schema: _schema1,
        authority: authority1,
        address_tree_pubkey,
        ..
    } = setup().await;

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

    // Create schema under SECOND credential
    let schema_name2 = "test_data2";
    let description2 = "schema for test data 2";
    let schema_data2 = TestData::get_serialized_representation();
    let field_names2 = vec!["name".into(), "location".into()];

    let (schema2_pda, _bump) = derive_schema_pda(&credential2_pda, schema_name2, 1);

    let create_schema2_ix = CreateSchemaBuilder::new()
        .payer(payer.pubkey())
        .authority(authority2.pubkey())
        .credential(credential2_pda)
        .schema(schema2_pda)
        .system_program(system_program::ID)
        .description(description2.to_string())
        .name(schema_name2.to_string())
        .layout(schema_data2)
        .field_names(field_names2)
        .instruction();

    rpc.create_and_send_transaction(
        &[create_schema2_ix],
        &payer.pubkey(),
        &[&payer, &authority2],
    )
    .await
    .unwrap();

    // Now try to create attestation using:
    // - credential1 (with authority1 as authorized signer)
    // - schema2 (which belongs to credential2, not credential1)
    // This should pass signer validation but fail schema-credential validation
    let attestation_data = TestData {
        name: "test".to_string(),
        location: 5,
    };
    let mut serialized_attestation_data = Vec::new();
    attestation_data
        .serialize(&mut serialized_attestation_data)
        .unwrap();

    let clock = rpc.context.get_sysvar::<Clock>();
    let expiry: i64 = clock.unix_timestamp + 60;
    let nonce = Pubkey::new_unique();

    // Use credential1 (first credential) but schema2 (from second credential)
    // Get the original credential from setup
    let credential1_pda = _credential1;

    // Derive compressed attestation address - must match what we pass to instruction
    let attestation_pda = Pubkey::find_program_address(
        &[
            b"attestation",
            &credential1_pda.to_bytes(),
            &schema2_pda.to_bytes(),
            &nonce.to_bytes(),
        ],
        &SOLANA_ATTESTATION_SERVICE_ID,
    )
    .0;

    let (compressed_address, _) = derive_address(
        &[attestation_pda.as_ref()],
        &address_tree_pubkey,
        &SOLANA_ATTESTATION_SERVICE_ID,
    );

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

    let proof_bytes: [u8; 128] = rpc_result.proof.0.unwrap().to_array();
    let address_root_index = rpc_result.addresses[0].root_index;

    // Use credential1 with its authorized signer (authority1), but schema2 (from credential2)
    let create_compressed_attestation_ix = CreateCompressedAttestationBuilder::new()
        .payer(payer.pubkey())
        .authority(authority1.pubkey()) // authority1 is authorized for credential1
        .credential(credential1_pda) // credential1
        .schema(schema2_pda) // schema2 (belongs to credential2, NOT credential1!)
        .output_queue(output_queue)
        .proof(proof_bytes)
        .nonce(nonce)
        .data(serialized_attestation_data)
        .expiry(expiry)
        .address_root_index(address_root_index)
        .instruction();

    // Should fail with InvalidCredential (schema2 doesn't belong to credential1)
    let result = rpc
        .create_and_send_transaction(
            &[create_compressed_attestation_ix],
            &payer.pubkey(),
            &[&payer, &authority1],
        )
        .await;

    assert_rpc_error(
        result,
        0,
        SolanaAttestationServiceError::InvalidCredential as u32,
    )
    .unwrap();
}

#[tokio::test]
async fn test_create_compressed_attestation_wrong_address_tree() {
    let TestFixtures {
        mut rpc,
        payer,
        credential,
        schema,
        authority,
        address_tree_pubkey: _,
        ..
    } = setup().await;

    // Prepare valid attestation data
    let attestation_data = TestData {
        name: "test".to_string(),
        location: 5,
    };
    let mut serialized_attestation_data = Vec::new();
    attestation_data
        .serialize(&mut serialized_attestation_data)
        .unwrap();

    let clock = rpc.context.get_sysvar::<Clock>();
    let expiry: i64 = clock.unix_timestamp + 60;
    let nonce = Pubkey::new_unique();

    // Use the allowed address tree constant for proof generation
    let correct_address_tree = ALLOWED_ADDRESS_TREE;

    // Derive compressed attestation address using correct tree for proof
    let attestation_pda = Pubkey::find_program_address(
        &[
            b"attestation",
            &credential.to_bytes(),
            &schema.to_bytes(),
            &nonce.to_bytes(),
        ],
        &SOLANA_ATTESTATION_SERVICE_ID,
    )
    .0;

    let (compressed_address, _) = derive_address(
        &[attestation_pda.as_ref()],
        &correct_address_tree,
        &SOLANA_ATTESTATION_SERVICE_ID,
    );

    // Generate valid proof with correct tree
    let rpc_result = rpc
        .get_validity_proof(
            vec![],
            vec![AddressWithTree {
                address: compressed_address,
                tree: correct_address_tree,
            }],
            None,
        )
        .await
        .unwrap()
        .value;

    // Now use a WRONG address tree in the instruction (different from allowed)
    // The allowed tree is: amt2kaJA14v3urZbZvnc5v2np8jqvc4Z8zDep5wbtzx
    let wrong_address_tree = Pubkey::new_unique();

    let output_queue = rpc.get_random_state_tree_info().unwrap().queue;

    let proof_bytes: [u8; 128] = rpc_result.proof.0.unwrap().to_array();
    let address_root_index = rpc_result.addresses[0].root_index;

    // Create instruction with WRONG address tree
    let create_compressed_attestation_ix = CreateCompressedAttestationBuilder::new()
        .payer(payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .schema(schema)
        .output_queue(output_queue)
        .address_merkle_tree(wrong_address_tree) // Using unauthorized tree
        .proof(proof_bytes)
        .nonce(nonce)
        .data(serialized_attestation_data)
        .expiry(expiry)
        .address_root_index(address_root_index)
        .instruction();

    // Should fail with InvalidAddressTree
    let result = rpc
        .create_and_send_transaction(
            &[create_compressed_attestation_ix],
            &payer.pubkey(),
            &[&payer, &authority],
        )
        .await;

    assert_rpc_error(
        result,
        0,
        SolanaAttestationServiceError::InvalidAddressTree as u32,
    )
    .unwrap();
}
