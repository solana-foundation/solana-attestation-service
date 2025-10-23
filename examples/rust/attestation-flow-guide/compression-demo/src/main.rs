use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use borsh::BorshSerialize;
use light_client::{
    indexer::{AddressWithTree, Indexer, IndexerRpcConfig},
    rpc::{LightClient, LightClientConfig, Rpc},
};
use light_sdk::address::v2::derive_address;
use solana_attestation_service_client::{
    accounts::Attestation,
    instructions::{
        ChangeAuthorizedSignersBuilder, CloseCompressedAttestationBuilder,
        CreateCompressedAttestationBuilder, CreateCredentialBuilder, CreateSchemaBuilder,
    },
    pdas::{derive_attestation_pda, derive_credential_pda, derive_event_authority_pda, derive_schema_pda},
    programs::SOLANA_ATTESTATION_SERVICE_ID,
    ALLOWED_ADDRESS_TREE,
};
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
};
use solana_system_interface::program::ID as system_program;

// Constants
const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

struct Config {
    pub credential_name: String,
    pub schema_name: String,
    pub schema_version: u8,
    pub schema_description: String,
    pub schema_layout: Vec<u8>,
    pub schema_fields: Vec<String>,
    pub attestation_expiry_days: i64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            credential_name: "TEST-ORGANIZATION".to_string(),
            schema_name: "THE-BASICS".to_string(),
            schema_version: 1,
            schema_description: "Basic user information schema for testing".to_string(),
            schema_layout: vec![12, 0, 12],
            schema_fields: vec!["name".to_string(), "age".to_string(), "country".to_string()],
            attestation_expiry_days: 365,
        }
    }
}

#[derive(BorshSerialize, Clone, Debug)]
pub struct TestData {
    pub name: String,
    pub age: u8,
    pub country: String,
}

impl TestData {
    fn get_example_data() -> Self {
        Self {
            name: "test-user".to_string(),
            age: 100,
            country: "usa".to_string(),
        }
    }
}

struct Wallets {
    pub authorized_signer1: Keypair,
    pub authorized_signer2: Keypair,
    pub issuer: Keypair,
    pub test_user: Keypair,
}

impl Wallets {
    fn new() -> Self {
        Self {
            authorized_signer1: Keypair::new(),
            authorized_signer2: Keypair::new(),
            issuer: Keypair::new(),
            test_user: Keypair::new(),
        }
    }
}

struct CompressionDemo {
    config: Config,
    rpc: LightClient,
    wallets: Wallets,
}

impl CompressionDemo {
    async fn new() -> Result<Self> {
        let config = Config::default();
        let wallets = Wallets::new();

        // Initialize Light client
        let mut rpc = LightClient::new(LightClientConfig::local()).await?;
        rpc.get_latest_active_state_trees().await?;

        Ok(Self {
            config,
            rpc,
            wallets,
        })
    }


    async fn send_and_confirm_instruction(
        &mut self,
        instruction: Instruction,
        signers: &[&Keypair],
        description: &str,
    ) -> Result<String> {
        let payer = self.rpc.get_payer().insecure_clone();
        let mut all_signers = vec![&payer];
        all_signers.extend(signers);

        let signature = self
            .rpc
            .create_and_send_transaction(&[instruction], &payer.pubkey(), &all_signers)
            .await?;

        println!("    - {} - Signature: {}", description, signature);
        Ok(signature.to_string())
    }

    async fn fund_payer(&mut self) -> Result<()> {
        println!("1. Funding payer wallet...");

        let payer_pubkey = self.rpc.get_payer().pubkey();
        self.rpc
            .airdrop_lamports(&payer_pubkey, 10 * LAMPORTS_PER_SOL)
            .await?;

        println!("    - Airdrop completed");

        Ok(())
    }

    async fn create_credential(&mut self) -> Result<Pubkey> {
        println!("\n2. Creating Credential...");

        let (credential_pda, _bump) = derive_credential_pda(&self.wallets.issuer.pubkey(), &self.config.credential_name);
        let payer = self.rpc.get_payer().pubkey();
        let issuer = self.wallets.issuer.insecure_clone();

        let instruction = CreateCredentialBuilder::new()
            .payer(payer)
            .credential(credential_pda)
            .authority(issuer.pubkey())
            .system_program(system_program)
            .name(self.config.credential_name.clone())
            .signers(vec![self.wallets.authorized_signer1.pubkey()])
            .instruction();

        self.send_and_confirm_instruction(
            instruction,
            &[&issuer],
            "Credential created",
        )
        .await?;

        println!("    - Credential PDA: {}", credential_pda);
        Ok(credential_pda)
    }

    async fn create_schema(&mut self, credential_pda: &Pubkey) -> Result<Pubkey> {
        println!("\n3. Creating Schema...");

        let (schema_pda, _bump) = derive_schema_pda(credential_pda, &self.config.schema_name, self.config.schema_version as u8);
        let payer = self.rpc.get_payer().pubkey();
        let issuer = self.wallets.issuer.insecure_clone();

        let instruction = CreateSchemaBuilder::new()
            .payer(payer)
            .authority(issuer.pubkey())
            .credential(*credential_pda)
            .schema(schema_pda)
            .name(self.config.schema_name.clone())
            .description(self.config.schema_description.clone())
            .layout(self.config.schema_layout.clone())
            .field_names(self.config.schema_fields.clone())
            .instruction();
        self.send_and_confirm_instruction(instruction, &[&issuer], "Schema created")
            .await?;

        println!("    - Schema PDA: {}", schema_pda);
        Ok(schema_pda)
    }

    async fn create_compressed_attestation(
        &mut self,
        credential_pda: &Pubkey,
        schema_pda: &Pubkey,
    ) -> Result<([u8; 32], u64)> {
        println!("\n4. Creating Compressed Attestation...");

        let attestation_data = TestData::get_example_data();

        // Calculate expiry timestamp
        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let expiry = current_timestamp + (self.config.attestation_expiry_days * 24 * 60 * 60);

        // Serialize attestation data using Borsh
        let mut serialized_data = Vec::new();
        attestation_data.serialize(&mut serialized_data)?;

        let nonce = self.wallets.test_user.pubkey();

        // Derive attestation PDA (for compressed address derivation)
        let (attestation_pda, _bump) =
            derive_attestation_pda(credential_pda, schema_pda, &nonce);

        // Get address tree
        let address_tree_pubkey = ALLOWED_ADDRESS_TREE;

        // Derive compressed address from PDA
        let (compressed_address, _) = derive_address(
            &[attestation_pda.as_ref()],
            &address_tree_pubkey,
            &SOLANA_ATTESTATION_SERVICE_ID,
        );

        // Get validity proof for new address
        let proof_result = self
            .rpc
            .get_validity_proof(
                vec![], // no input accounts
                vec![AddressWithTree {
                    address: compressed_address,
                    tree: address_tree_pubkey,
                }],
                None,
            )
            .await?
            .value;

        // Serialize proof (128 bytes)
        let proof_bytes = proof_result
            .proof
            .0
            .unwrap()
            .to_array();


        // Get address root index
        let address_root_index = proof_result.addresses[0].root_index;

        let output_queue = self.rpc.get_random_state_tree_info()?.queue;

        let payer = self.rpc.get_payer().pubkey();

        // Build instruction
        let instruction = CreateCompressedAttestationBuilder::new()
            .payer(payer)
            .authority(self.wallets.authorized_signer1.pubkey())
            .credential(*credential_pda)
            .schema(*schema_pda)
            .output_queue(output_queue)
            .proof(proof_bytes)
            .nonce(nonce)
            .data(serialized_data.clone())
            .expiry(expiry)
            .address_root_index(address_root_index)
            .instruction();

        let signer = self.wallets.authorized_signer1.insecure_clone();
        self.send_and_confirm_instruction(
            instruction,
            &[&signer],
            "Compressed attestation created",
        )
        .await?;

        // Get current slot for indexer config
        let slot = self.rpc.get_slot().await?;

        println!("    - Attestation PDA: {}", attestation_pda);
        println!(
            "    - Compressed Address: 0x{}",
            hex::encode(compressed_address)
        );
        println!("    - Slot: {}", slot);

        Ok((compressed_address, slot))
    }

    async fn update_authorized_signers(&mut self, credential_pda: &Pubkey) -> Result<()> {
        println!("\n5. Updating Authorized Signers...");

        let payer = self.rpc.get_payer().pubkey();
        let issuer = self.wallets.issuer.insecure_clone();

        let instruction = ChangeAuthorizedSignersBuilder::new()
            .payer(payer)
            .authority(issuer.pubkey())
            .credential(*credential_pda)
            .signers(vec![
                self.wallets.authorized_signer1.pubkey(),
                self.wallets.authorized_signer2.pubkey(),
            ])
            .instruction();

        self.send_and_confirm_instruction(
            instruction,
            &[&issuer],
            "Authorized signers updated",
        )
        .await?;

        Ok(())
    }

    async fn verify_compressed_attestation(
        &mut self,
        schema_pda: &Pubkey,
        user_address: &Pubkey,
        credential_pda: &Pubkey,
        user_name: &str,
    ) -> Result<bool> {
        // Derive attestation PDA
        let (attestation_pda, _) =
            derive_attestation_pda(credential_pda, schema_pda, user_address);

        // Get address tree
        let address_tree_pubkey = ALLOWED_ADDRESS_TREE;

        // Derive compressed address
        let (compressed_address, _) = derive_address(
            &[attestation_pda.as_ref()],
            &address_tree_pubkey,
            &SOLANA_ATTESTATION_SERVICE_ID,
        );

        // Fetch compressed account
        let compressed_account = self
            .rpc
            .get_compressed_account(compressed_address, None)
            .await?
            .value;

        let is_valid = match compressed_account {
            Some(account) => {
                if let Some(data) = account.data {
                    match Attestation::from_bytes(&data.data) {
                        Ok(attestation) => {
                            let current_timestamp = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs() as i64;

                            current_timestamp < attestation.expiry
                        }
                        Err(_) => false,
                    }
                } else {
                    false
                }
            }
            None => false,
        };

        println!(
            "    - {} is {}",
            user_name,
            if is_valid { "verified" } else { "not verified" }
        );

        Ok(is_valid)
    }

    async fn close_compressed_attestation(
        &mut self,
        compressed_address: [u8; 32],
        credential_pda: &Pubkey,
        slot: u64,
    ) -> Result<()> {
        println!("\n7. Closing Compressed Attestation...");

        // Use IndexerRpcConfig with slot to ensure indexer has synced
        let indexer_config = IndexerRpcConfig::new(slot);

        // Fetch compressed account to get data
        let response = self
            .rpc
            .get_compressed_account(compressed_address, Some(indexer_config))
            .await?;

        let compressed_account = response.value.expect("Compressed account should exist");

        let attestation_account_data = compressed_account
            .data
            .expect("Account data should exist")
            .data;

        let attestation = Attestation::from_bytes(&attestation_account_data)?;

        let leaf_index = compressed_account.leaf_index;
        let account_hash = compressed_account.hash;

        // Get validity proof for input account (closing)
        let proof_result = self
            .rpc
            .get_validity_proof(vec![account_hash], vec![], None)
            .await?
            .value;

        // Extract proof if exists
        let proof = proof_result.proof.0.map(|p| {
            p.to_array()
        });

        // Get root index
        let root_index = proof_result.accounts[0]
            .root_index
            .root_index()
            .unwrap_or_default();

        // Derive event authority
        let event_auth_pda = derive_event_authority_pda();

        let payer = self.rpc.get_payer().pubkey();

        // Build close instruction
        let mut builder = CloseCompressedAttestationBuilder::new();
        builder
            .payer(payer)
            .authority(self.wallets.authorized_signer1.pubkey())
            .credential(*credential_pda)
            .event_authority(event_auth_pda)
            .state_merkle_tree(proof_result.accounts[0].tree_info.tree)
            .output_queue(proof_result.accounts[0].tree_info.queue);

        if let Some(proof_bytes) = proof {
            builder.proof(proof_bytes);
        }

        let instruction = builder
            .nonce(attestation.nonce)
            .schema(attestation.schema)
            .signer(attestation.signer)
            .expiry(attestation.expiry)
            .data(attestation.data)
            .root_index(root_index)
            .leaf_index(leaf_index)
            .address(compressed_address)
            .instruction();

        let signer = self.wallets.authorized_signer1.insecure_clone();
        self.send_and_confirm_instruction(
            instruction,
            &[&signer],
            "Closed compressed attestation",
        )
        .await?;

        // Verify account is nullified
        let after = self
            .rpc
            .get_compressed_account(compressed_address, None)
            .await?
            .value;

        if after.is_none() {
            println!("    - Compressed attestation successfully nullified");
        } else {
            println!("    - WARNING: Compressed attestation still exists");
        }

        Ok(())
    }

    pub async fn run_demo(&mut self) -> Result<()> {
        println!("Starting Compression Attestation Service Demo\n");

        // Step 1: Fund payer
        self.fund_payer().await?;

        // Step 2: Create Credential
        let credential_pda = self.create_credential().await?;

        // Step 3: Create Schema
        let schema_pda = self.create_schema(&credential_pda).await?;

        // Step 4: Create Compressed Attestation
        let (compressed_address, slot) = self
            .create_compressed_attestation(&credential_pda, &schema_pda)
            .await?;

        // Step 5: Update Authorized Signers
        self.update_authorized_signers(&credential_pda).await?;

        // Step 6: Verify Attestations
        println!("\n6. Verifying Compressed Attestations...");
        let _test_user_result = self
            .verify_compressed_attestation(
                &schema_pda,
                &self.wallets.test_user.pubkey(),
                &credential_pda,
                "Test User",
            )
            .await;
        let _random_user_result = self
            .verify_compressed_attestation(
                &schema_pda,
                &Keypair::new().pubkey(),
                &credential_pda,
                "Random User",
            )
            .await;

        // Step 7: Close Compressed Attestation
        self.close_compressed_attestation(compressed_address, &credential_pda, slot)
            .await?;

        println!("\nCompression Attestation Service demo completed successfully!");

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create and run demo
    let mut demo = CompressionDemo::new().await?;

    match demo.run_demo().await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("❌ Demo failed: {}", e);
            std::process::exit(1);
        }
    }
}
