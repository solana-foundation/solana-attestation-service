use borsh::BorshSerialize;
use light_program_test::{
    program_test::LightProgramTest, utils::assert::assert_rpc_error, AddressWithTree, Indexer,
    ProgramTestConfig, Rpc,
};
use light_sdk::address::v2::derive_address;
use solana_attestation_service_client::{
    accounts::Attestation,
    instructions::{
        CompressAttestationsBuilder, CreateAttestationBuilder, CreateCredentialBuilder,
        CreateSchemaBuilder,
    },
    pdas::{
        derive_attestation_pda, derive_credential_pda, derive_event_authority_pda,
        derive_schema_pda,
    },
    programs::SOLANA_ATTESTATION_SERVICE_ID,
    ALLOWED_ADDRESS_TREE,
};
use solana_sdk::{
    clock::Clock,
    instruction::AccountMeta,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use solana_sdk_ids::system_program;

mod helpers;
use helpers::{TestData, TestFixtures};

async fn setup(num_attestations: usize) -> TestFixtures {
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

    // Create attestations
    let mut attestations = Vec::with_capacity(num_attestations);

    for i in 0..num_attestations {
        let attestation_data = TestData {
            name: format!("attest_{}", i),
            location: i as u8,
        };
        let mut serialized_attestation_data = Vec::new();
        attestation_data
            .serialize(&mut serialized_attestation_data)
            .unwrap();

        let clock = rpc.context.get_sysvar::<Clock>();
        let expiry: i64 = clock.unix_timestamp + 60;
        let nonce = Pubkey::new_unique();

        // Derive attestation PDA
        let (attestation_pda, _) = derive_attestation_pda(&credential_pda, &schema_pda, &nonce);

        // Create regular attestation PDA
        let create_attestation_ix = CreateAttestationBuilder::new()
            .payer(payer.pubkey())
            .authority(authority.pubkey())
            .credential(credential_pda)
            .schema(schema_pda)
            .attestation(attestation_pda)
            .system_program(system_program::ID)
            .data(serialized_attestation_data)
            .expiry(expiry)
            .nonce(nonce)
            .instruction();

        rpc.create_and_send_transaction(
            &[create_attestation_ix],
            &payer.pubkey(),
            &[&payer, &authority],
        )
        .await
        .unwrap();

        attestations.push(attestation_pda);
    }

    TestFixtures {
        rpc,
        payer,
        credential: credential_pda,
        schema: schema_pda,
        authority,
        address_tree_pubkey,
        event_auth_pda: derive_event_authority_pda(),
        attestations,
    }
}

#[tokio::test]
async fn test_compress_1_attestation_no_close() {
    let TestFixtures {
        mut rpc,
        payer,
        credential,
        schema,
        authority,
        address_tree_pubkey,
        attestations,
        ..
    } = setup(1).await;

    let attestation_pda = attestations[0];

    // Derive compressed address from PDA
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

    // Derive event authority
    let event_authority = derive_event_authority_pda();

    // Get tree pubkeys
    let output_queue = rpc.get_random_state_tree_info().unwrap().queue;

    // Serialize proof to fixed array [u8; 128]

    let proof_bytes: [u8; 128] = rpc_result.proof.0.unwrap().to_array();

    // Get address root index from the validity proof result
    let address_root_index = rpc_result.addresses[0].root_index;

    // Compress attestation (no close)
    let compress_ix = CompressAttestationsBuilder::new()
        .payer(payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .event_authority(event_authority)
        .output_queue(output_queue)
        .add_remaining_account(AccountMeta::new_readonly(attestation_pda, false)) // Not writable since not closing
        .proof(proof_bytes)
        .close_accounts(false)
        .address_root_index(address_root_index)
        .num_attestations(1)
        .instruction();

    rpc.create_and_send_transaction(&[compress_ix], &payer.pubkey(), &[&payer, &authority])
        .await
        .unwrap();

    // Verify compressed account was created
    let compressed_account = rpc
        .get_compressed_account(compressed_address, None)
        .await
        .unwrap()
        .value
        .unwrap();

    assert_eq!(compressed_account.address.unwrap(), compressed_address);

    // Verify lamports are 0 (compressed accounts don't store lamports)
    assert_eq!(
        compressed_account.lamports, 0,
        "Compressed account should have 0 lamports"
    );

    // Verify discriminator is correct
    assert_eq!(
        compressed_account.data.as_ref().unwrap().discriminator,
        [2, 0, 0, 0, 0, 0, 0, 0],
        "Discriminator should match Attestation::LIGHT_DISCRIMINATOR"
    );

    // Verify address derivation is deterministic
    let (re_derived_address, _) = derive_address(
        &[attestation_pda.as_ref()],
        &address_tree_pubkey,
        &SOLANA_ATTESTATION_SERVICE_ID,
    );
    assert_eq!(
        compressed_address, re_derived_address,
        "Address derivation should be deterministic"
    );

    // Verify compressed account data matches original attestation
    let compressed_attestation =
        Attestation::from_bytes(&compressed_account.data.as_ref().unwrap().data).unwrap();
    assert_eq!(compressed_attestation.credential, credential);
    assert_eq!(compressed_attestation.schema, schema);
    assert_eq!(compressed_attestation.signer, authority.pubkey());
    assert_eq!(compressed_attestation.token_account, Pubkey::default());

    // Verify original PDA still exists (not closed) and has same data
    let pda_account = rpc.get_account(attestation_pda).await.unwrap();
    assert!(pda_account.is_some(), "PDA should still exist");

    let pda_attestation = Attestation::from_bytes(&pda_account.unwrap().data).unwrap();
    assert_eq!(pda_attestation.credential, credential);
    assert_eq!(pda_attestation.schema, schema);
    assert_eq!(pda_attestation.signer, authority.pubkey());

    // Verify both have same data
    assert_eq!(compressed_attestation.nonce, pda_attestation.nonce);
    assert_eq!(compressed_attestation.data, pda_attestation.data);
    assert_eq!(compressed_attestation.expiry, pda_attestation.expiry);
}

#[tokio::test]
async fn test_compress_2_attestations_no_close() {
    let TestFixtures {
        mut rpc,
        payer,
        credential,
        schema,
        authority,
        address_tree_pubkey,
        attestations,
        ..
    } = setup(2).await;

    let attestation_pda_1 = attestations[0];
    let attestation_pda_2 = attestations[1];

    // Derive compressed addresses from PDAs
    let (compressed_address_1, _) = derive_address(
        &[attestation_pda_1.as_ref()],
        &address_tree_pubkey,
        &SOLANA_ATTESTATION_SERVICE_ID,
    );
    let (compressed_address_2, _) = derive_address(
        &[attestation_pda_2.as_ref()],
        &address_tree_pubkey,
        &SOLANA_ATTESTATION_SERVICE_ID,
    );

    // Get validity proof for both new compressed addresses
    let rpc_result = rpc
        .get_validity_proof(
            vec![],
            vec![
                AddressWithTree {
                    address: compressed_address_1,
                    tree: address_tree_pubkey,
                },
                AddressWithTree {
                    address: compressed_address_2,
                    tree: address_tree_pubkey,
                },
            ],
            None,
        )
        .await
        .unwrap()
        .value;

    let event_authority = derive_event_authority_pda();
    let output_queue = rpc.get_random_state_tree_info().unwrap().queue;

    // Serialize proof to fixed array [u8; 128]

    let proof_bytes: [u8; 128] = rpc_result.proof.0.unwrap().to_array();
    let address_root_index = rpc_result.addresses[0].root_index;

    // Compress both attestations (no close)
    let compress_ix = CompressAttestationsBuilder::new()
        .payer(payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .event_authority(event_authority)
        .output_queue(output_queue)
        .add_remaining_account(AccountMeta::new_readonly(attestation_pda_1, false))
        .add_remaining_account(AccountMeta::new_readonly(attestation_pda_2, false))
        .proof(proof_bytes)
        .close_accounts(false)
        .address_root_index(address_root_index)
        .num_attestations(2)
        .instruction();

    rpc.create_and_send_transaction(&[compress_ix], &payer.pubkey(), &[&payer, &authority])
        .await
        .unwrap();

    // Verify both compressed accounts were created
    let compressed_account_1 = rpc
        .get_compressed_account(compressed_address_1, None)
        .await
        .unwrap()
        .value
        .unwrap();
    let compressed_account_2 = rpc
        .get_compressed_account(compressed_address_2, None)
        .await
        .unwrap()
        .value
        .unwrap();

    assert_eq!(compressed_account_1.address.unwrap(), compressed_address_1);
    assert_eq!(compressed_account_2.address.unwrap(), compressed_address_2);

    // Verify compressed account data
    let compressed_attestation_1 =
        Attestation::from_bytes(&compressed_account_1.data.as_ref().unwrap().data).unwrap();
    let compressed_attestation_2 =
        Attestation::from_bytes(&compressed_account_2.data.as_ref().unwrap().data).unwrap();

    assert_eq!(compressed_attestation_1.credential, credential);
    assert_eq!(compressed_attestation_1.schema, schema);
    assert_eq!(compressed_attestation_2.credential, credential);
    assert_eq!(compressed_attestation_2.schema, schema);

    // Verify original PDAs still exist (not closed)
    let pda_account_1 = rpc.get_account(attestation_pda_1).await.unwrap();
    let pda_account_2 = rpc.get_account(attestation_pda_2).await.unwrap();
    assert!(pda_account_1.is_some(), "PDA 1 should still exist");
    assert!(pda_account_2.is_some(), "PDA 2 should still exist");

    let pda_attestation_1 = Attestation::from_bytes(&pda_account_1.unwrap().data).unwrap();
    let pda_attestation_2 = Attestation::from_bytes(&pda_account_2.unwrap().data).unwrap();

    // Verify both PDAs and compressed accounts match
    assert_eq!(compressed_attestation_1.nonce, pda_attestation_1.nonce);
    assert_eq!(compressed_attestation_1.data, pda_attestation_1.data);
    assert_eq!(compressed_attestation_2.nonce, pda_attestation_2.nonce);
    assert_eq!(compressed_attestation_2.data, pda_attestation_2.data);
}

#[tokio::test]
async fn test_compress_1_attestation_with_close() {
    let TestFixtures {
        mut rpc,
        payer,
        credential,
        schema,
        authority,
        address_tree_pubkey,
        attestations,
        ..
    } = setup(1).await;

    let attestation_pda = attestations[0];

    // Derive compressed address from PDA
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

    // Derive event authority
    let event_authority = derive_event_authority_pda();

    // Get tree pubkeys
    let output_queue = rpc.get_random_state_tree_info().unwrap().queue;

    // Serialize proof to fixed array [u8; 128]

    let proof_bytes: [u8; 128] = rpc_result.proof.0.unwrap().to_array();

    // Get address root index from the validity proof result
    let address_root_index = rpc_result.addresses[0].root_index;

    // Compress attestation with close
    let compress_ix = CompressAttestationsBuilder::new()
        .payer(payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .event_authority(event_authority)
        .output_queue(output_queue)
        .add_remaining_account(AccountMeta::new(attestation_pda, false)) // Writable since closing
        .proof(proof_bytes)
        .close_accounts(true)
        .address_root_index(address_root_index)
        .num_attestations(1)
        .instruction();

    rpc.create_and_send_transaction(&[compress_ix], &payer.pubkey(), &[&payer, &authority])
        .await
        .unwrap();

    // Verify compressed account was created
    let compressed_account = rpc
        .get_compressed_account(compressed_address, None)
        .await
        .unwrap()
        .value
        .unwrap();

    assert_eq!(compressed_account.address.unwrap(), compressed_address);

    // Verify compressed account data matches original attestation
    let compressed_attestation =
        Attestation::from_bytes(&compressed_account.data.as_ref().unwrap().data).unwrap();
    assert_eq!(compressed_attestation.credential, credential);
    assert_eq!(compressed_attestation.schema, schema);
    assert_eq!(compressed_attestation.signer, authority.pubkey());
    assert_eq!(compressed_attestation.token_account, Pubkey::default());

    // Verify original PDA is closed
    let pda_account = rpc.get_account(attestation_pda).await.unwrap();
    assert!(pda_account.is_none(), "PDA should be closed");
}

#[tokio::test]
async fn test_compress_2_attestations_with_close() {
    let TestFixtures {
        mut rpc,
        payer,
        credential,
        schema,
        authority,
        address_tree_pubkey,
        attestations,
        ..
    } = setup(2).await;

    let attestation_pda_1 = attestations[0];
    let attestation_pda_2 = attestations[1];

    // Derive compressed addresses from PDAs
    let (compressed_address_1, _) = derive_address(
        &[attestation_pda_1.as_ref()],
        &address_tree_pubkey,
        &SOLANA_ATTESTATION_SERVICE_ID,
    );
    let (compressed_address_2, _) = derive_address(
        &[attestation_pda_2.as_ref()],
        &address_tree_pubkey,
        &SOLANA_ATTESTATION_SERVICE_ID,
    );

    // Get validity proof for both new compressed addresses
    let rpc_result = rpc
        .get_validity_proof(
            vec![],
            vec![
                AddressWithTree {
                    address: compressed_address_1,
                    tree: address_tree_pubkey,
                },
                AddressWithTree {
                    address: compressed_address_2,
                    tree: address_tree_pubkey,
                },
            ],
            None,
        )
        .await
        .unwrap()
        .value;

    let event_authority = derive_event_authority_pda();
    let output_queue = rpc.get_random_state_tree_info().unwrap().queue;

    // Serialize proof to fixed array [u8; 128]

    let proof_bytes: [u8; 128] = rpc_result.proof.0.unwrap().to_array();
    let address_root_index = rpc_result.addresses[0].root_index;

    // Compress both attestations with close
    let compress_ix = CompressAttestationsBuilder::new()
        .payer(payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .event_authority(event_authority)
        .output_queue(output_queue)
        .add_remaining_account(AccountMeta::new(attestation_pda_1, false)) // Writable since closing
        .add_remaining_account(AccountMeta::new(attestation_pda_2, false)) // Writable since closing
        .proof(proof_bytes)
        .close_accounts(true)
        .address_root_index(address_root_index)
        .num_attestations(2)
        .instruction();

    rpc.create_and_send_transaction(&[compress_ix], &payer.pubkey(), &[&payer, &authority])
        .await
        .unwrap();

    // Verify both compressed accounts were created
    let compressed_account_1 = rpc
        .get_compressed_account(compressed_address_1, None)
        .await
        .unwrap()
        .value
        .unwrap();
    let compressed_account_2 = rpc
        .get_compressed_account(compressed_address_2, None)
        .await
        .unwrap()
        .value
        .unwrap();

    assert_eq!(compressed_account_1.address.unwrap(), compressed_address_1);
    assert_eq!(compressed_account_2.address.unwrap(), compressed_address_2);

    // Verify compressed account data
    let compressed_attestation_1 =
        Attestation::from_bytes(&compressed_account_1.data.as_ref().unwrap().data).unwrap();
    let compressed_attestation_2 =
        Attestation::from_bytes(&compressed_account_2.data.as_ref().unwrap().data).unwrap();

    assert_eq!(compressed_attestation_1.credential, credential);
    assert_eq!(compressed_attestation_1.schema, schema);
    assert_eq!(compressed_attestation_2.credential, credential);
    assert_eq!(compressed_attestation_2.schema, schema);

    // Verify original PDAs are closed
    let pda_account_1 = rpc.get_account(attestation_pda_1).await.unwrap();
    let pda_account_2 = rpc.get_account(attestation_pda_2).await.unwrap();
    assert!(pda_account_1.is_none(), "PDA 1 should be closed");
    assert!(pda_account_2.is_none(), "PDA 2 should be closed");
}

#[tokio::test]
async fn test_compress_attestation_unauthorized_signer() {
    let TestFixtures {
        mut rpc,
        payer,
        credential,
        schema: _,
        authority: _,
        address_tree_pubkey,
        attestations,
        ..
    } = setup(1).await;

    let attestation_pda = attestations[0];

    // Create an unauthorized signer (not in the credential's authorized signers)
    let unauthorized_signer = Keypair::new();

    // Derive compressed address from PDA
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

    // Derive event authority
    let event_authority = derive_event_authority_pda();

    // Get tree pubkeys
    let output_queue = rpc.get_random_state_tree_info().unwrap().queue;

    // Serialize proof to fixed array [u8; 128]

    let proof_bytes: [u8; 128] = rpc_result.proof.0.unwrap().to_array();

    // Get address root index from the validity proof result
    let address_root_index = rpc_result.addresses[0].root_index;

    // Try to compress attestation with UNAUTHORIZED signer
    let compress_ix = CompressAttestationsBuilder::new()
        .payer(payer.pubkey())
        .authority(unauthorized_signer.pubkey()) // Using unauthorized signer
        .credential(credential)
        .event_authority(event_authority)
        .output_queue(output_queue)
        .add_remaining_account(AccountMeta::new_readonly(attestation_pda, false))
        .proof(proof_bytes)
        .close_accounts(false)
        .address_root_index(address_root_index)
        .num_attestations(1)
        .instruction();

    // Try to compress with unauthorized signer - should fail
    let result = rpc
        .create_and_send_transaction(
            &[compress_ix],
            &payer.pubkey(),
            &[&payer, &unauthorized_signer],
        )
        .await;

    // Assert fails with error code 5 (SignerNotAuthorized)
    assert_rpc_error(result, 0, 5).unwrap();
}

#[tokio::test]
async fn test_compress_attestation_wrong_credential() {
    let TestFixtures {
        mut rpc,
        payer,
        credential: _credential1,
        schema: _,
        authority: _authority1,
        address_tree_pubkey,
        attestations,
        ..
    } = setup(1).await;

    let attestation_pda = attestations[0];

    // Create a SECOND credential with different authority
    let authority2 = Keypair::new();
    let credential_name2 = "test2";
    let (credential2_pda, _bump) = Pubkey::find_program_address(
        &[
            b"credential",
            &authority2.pubkey().to_bytes(),
            credential_name2.as_bytes(),
        ],
        &SOLANA_ATTESTATION_SERVICE_ID,
    );

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

    // Derive compressed address from PDA
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

    // Derive event authority
    let event_authority = derive_event_authority_pda();

    // Get tree pubkeys
    let output_queue = rpc.get_random_state_tree_info().unwrap().queue;

    // Serialize proof to fixed array [u8; 128]

    let proof_bytes: [u8; 128] = rpc_result.proof.0.unwrap().to_array();

    // Get address root index from the validity proof result
    let address_root_index = rpc_result.addresses[0].root_index;

    // Try to compress attestation1 using WRONG credential2 (attestation belongs to credential1)
    let compress_ix = CompressAttestationsBuilder::new()
        .payer(payer.pubkey())
        .authority(authority2.pubkey()) // Using authority2
        .credential(credential2_pda) // Using credential2 (WRONG - attestation belongs to credential1)
        .event_authority(event_authority)
        .output_queue(output_queue)
        .add_remaining_account(AccountMeta::new_readonly(attestation_pda, false))
        .proof(proof_bytes)
        .close_accounts(false)
        .address_root_index(address_root_index)
        .num_attestations(1)
        .instruction();

    // Try to compress with wrong credential - should fail
    let result = rpc
        .create_and_send_transaction(&[compress_ix], &payer.pubkey(), &[&payer, &authority2])
        .await;

    // Assert fails with error code 0 (InvalidCredential)
    assert_rpc_error(result, 0, 0).unwrap();
}

#[tokio::test]
async fn test_compress_attestation_invalid_address_tree() {
    let TestFixtures {
        mut rpc,
        payer,
        credential,
        schema: _,
        authority,
        address_tree_pubkey: _correct_address_tree,
        attestations,
        ..
    } = setup(1).await;

    let attestation_pda = attestations[0];

    // Use a WRONG address tree (just a random pubkey)
    let wrong_address_tree = Pubkey::new_unique();

    // Derive compressed address from PDA using the correct tree (for proof generation)
    let (compressed_address, _) = derive_address(
        &[attestation_pda.as_ref()],
        &_correct_address_tree,
        &SOLANA_ATTESTATION_SERVICE_ID,
    );

    // Get validity proof using correct tree
    let rpc_result = rpc
        .get_validity_proof(
            vec![],
            vec![AddressWithTree {
                address: compressed_address,
                tree: _correct_address_tree,
            }],
            None,
        )
        .await
        .unwrap()
        .value;

    // Derive event authority
    let event_authority = derive_event_authority_pda();

    // Get tree pubkeys
    let output_queue = rpc.get_random_state_tree_info().unwrap().queue;

    // Serialize proof to fixed array [u8; 128]

    let proof_bytes: [u8; 128] = rpc_result.proof.0.unwrap().to_array();

    // Get address root index from the validity proof result
    let address_root_index = rpc_result.addresses[0].root_index;

    // Try to compress with WRONG address tree - should fail
    let compress_ix = CompressAttestationsBuilder::new()
        .payer(payer.pubkey())
        .authority(authority.pubkey())
        .credential(credential)
        .event_authority(event_authority)
        .output_queue(output_queue)
        .address_merkle_tree(wrong_address_tree) // Using WRONG address tree
        .add_remaining_account(AccountMeta::new_readonly(attestation_pda, false))
        .proof(proof_bytes)
        .close_accounts(false)
        .address_root_index(address_root_index)
        .num_attestations(1)
        .instruction();

    // Try to compress with invalid address tree - should fail
    let result = rpc
        .create_and_send_transaction(&[compress_ix], &payer.pubkey(), &[&payer, &authority])
        .await;

    // Assert fails with error code 12 (InvalidAddressTree)
    assert_rpc_error(result, 0, 12).unwrap();
}
