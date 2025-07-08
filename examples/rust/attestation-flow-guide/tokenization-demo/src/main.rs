use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    compute_budget::ComputeBudgetInstruction,
    instruction::Instruction,
    message::Message,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};
use solana_system_interface::program::ID as system_program;

use solana_attestation_service_client::{
    accounts::Attestation,
    instructions::{
        ChangeAuthorizedSignersBuilder,
        CloseAttestationBuilder,
        CloseTokenizedAttestationBuilder,
        CreateAttestationBuilder,
        CreateCredentialBuilder,
        CreateSchemaBuilder,
        // Add these three 👇
        CreateTokenizedAttestationBuilder,
        TokenizeSchemaBuilder,
    },
    programs::SOLANA_ATTESTATION_SERVICE_ID,
};

use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_pod::optional_keys::OptionalNonZeroPubkey;
use spl_token_2022::{
    extension::{BaseStateWithExtensions, ExtensionType, StateWithExtensions},
    state::Mint,
    ID as TOKEN_22_PROGRAM_ID,
};
use spl_token_group_interface::state::TokenGroupMember;
use spl_token_metadata_interface::state::TokenMetadata;

struct Config {
    pub rpc_url: String,
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
            rpc_url: "http://127.0.0.1:8899".to_string(),
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

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
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

struct TokenizedConfig {
    // Inherit from base Config
    base: Config,

    // Token metadata
    token_name: String,
    token_symbol: String,
    token_metadata_uri: String,
}

impl Default for TokenizedConfig {
    fn default() -> Self {
        Self {
            base: Config::default(),
            token_name: "Test Identity".to_string(),
            token_symbol: "TESTID".to_string(),
            token_metadata_uri: "https://example.com/metadata.json".to_string(),
        }
    }
}

struct Wallets {
    pub payer: Keypair,
    pub authorized_signer1: Keypair,
    pub authorized_signer2: Keypair,
    pub issuer: Keypair,
    pub test_user: Keypair,
}

impl Wallets {
    fn new() -> Self {
        Self {
            payer: Keypair::new(),
            authorized_signer1: Keypair::new(),
            authorized_signer2: Keypair::new(),
            issuer: Keypair::new(),
            test_user: Keypair::new(),
        }
    }
}

struct SasDemo {
    config: Config,
    rpc_client: RpcClient,
    wallets: Wallets,
}

impl SasDemo {
    fn new() -> Self {
        let config = Config::default();
        let rpc_client =
            RpcClient::new_with_commitment(config.rpc_url.clone(), CommitmentConfig::confirmed());
        let wallets = Wallets::new();

        Self {
            config,
            rpc_client,
            wallets,
        }
    }

    fn derive_credential_pda(&self) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                b"credential",
                &self.wallets.issuer.pubkey().to_bytes(),
                self.config.credential_name.as_bytes(),
            ],
            &SOLANA_ATTESTATION_SERVICE_ID,
        )
    }

    fn derive_schema_pda(&self, credential_pda: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                b"schema",
                &credential_pda.to_bytes(),
                self.config.schema_name.as_bytes(),
                &[self.config.schema_version],
            ],
            &SOLANA_ATTESTATION_SERVICE_ID,
        )
    }

    fn derive_attestation_pda(
        &self,
        credential_pda: &Pubkey,
        schema_pda: &Pubkey,
        nonce: &Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                b"attestation",
                &credential_pda.to_bytes(),
                &schema_pda.to_bytes(),
                &nonce.to_bytes(),
            ],
            &SOLANA_ATTESTATION_SERVICE_ID,
        )
    }

    fn derive_schema_mint_pda(&self, schema_pda: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"schemaMint", &schema_pda.to_bytes()],
            &SOLANA_ATTESTATION_SERVICE_ID,
        )
    }

    fn derive_attestation_mint_pda(&self, attestation_pda: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"attestationMint", &attestation_pda.to_bytes()],
            &SOLANA_ATTESTATION_SERVICE_ID,
        )
    }

    fn derive_sas_authority_address() -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"sas"], &SOLANA_ATTESTATION_SERVICE_ID)
    }

    fn calculate_schema_mint_size(&self) -> usize {
        let expected_acc_size =
            ExtensionType::try_calculate_account_len::<Mint>(&[ExtensionType::GroupPointer])
                .unwrap();
        expected_acc_size
    }

    fn calculate_token_metadata_size(&self, config: &TokenizedConfig) -> Result<usize> {
        // Create dummy addresses for size calculation
        let dummy_key = Pubkey::new_unique();

        // Prepare additional metadata (use placeholder keys for calculating size)
        let additional_metadata = vec![
            ("attestation".to_string(), dummy_key.to_string()),
            ("schema".to_string(), dummy_key.to_string()),
        ];

        // Create TokenMetadata for size calculation
        let token_metadata = TokenMetadata {
            update_authority: OptionalNonZeroPubkey::try_from(Some(dummy_key))?,
            mint: dummy_key,
            name: config.token_name.clone(),
            symbol: config.token_symbol.clone(),
            uri: config.token_metadata_uri.clone(),
            additional_metadata,
        };

        // Calculate the TLV (Type-Length-Value) size
        Ok(token_metadata.tlv_size_of()?)
    }

    fn calculate_attestation_mint_size(&self, config: &TokenizedConfig) -> usize {
        let attestation_extensions = vec![
            ExtensionType::GroupMemberPointer,
            ExtensionType::NonTransferable,
            ExtensionType::MetadataPointer,
            ExtensionType::PermanentDelegate,
            ExtensionType::MintCloseAuthority,
            ExtensionType::TokenGroupMember,
        ];
        let base_size =
            ExtensionType::try_calculate_account_len::<Mint>(&attestation_extensions).unwrap();
        let metadata_size = self.calculate_token_metadata_size(config).unwrap();

        base_size + metadata_size
    }

    async fn send_and_confirm_instruction(
        &self,
        instruction: Instruction,
        signers: &[&Keypair],
        description: &str,
    ) -> Result<Signature> {
        // Simulate transaction to get compute units needed
        let sim_message = Message::new(
            &[
                ComputeBudgetInstruction::set_compute_unit_limit(1_400_000 as u32),
                ComputeBudgetInstruction::set_compute_unit_price(1),
                instruction.clone(),
            ],
            Some(&self.wallets.payer.pubkey()),
        );

        let mut all_signers = vec![&self.wallets.payer];
        all_signers.extend(signers);

        let simulation = Transaction::new(
            &all_signers,
            sim_message,
            self.rpc_client.get_latest_blockhash()?,
        );

        let sim_result = self.rpc_client.simulate_transaction(&simulation)?;
        let compute = sim_result.value.units_consumed.unwrap_or(200_000);

        // Create optimized transaction
        let message = Message::new(
            &[
                ComputeBudgetInstruction::set_compute_unit_limit(compute as u32),
                ComputeBudgetInstruction::set_compute_unit_price(1), // dynamically estimate in production
                instruction,
            ],
            Some(&self.wallets.payer.pubkey()),
        );

        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        let transaction = Transaction::new(&all_signers, message, recent_blockhash);
        let signature = self
            .rpc_client
            .send_and_confirm_transaction_with_spinner(&transaction)?;

        println!("    - {} - Signature: {}", description, signature);
        Ok(signature)
    }

    async fn fund_payer(&self) -> Result<()> {
        println!("1. Funding payer wallet...");

        // Request airdrop for payer
        let airdrop_sig = self
            .rpc_client
            .request_airdrop(&self.wallets.payer.pubkey(), LAMPORTS_PER_SOL)?;

        // Wait for airdrop confirmation
        let _confirmed = self.rpc_client.confirm_transaction_with_spinner(
            &airdrop_sig,
            &self.rpc_client.get_latest_blockhash()?,
            CommitmentConfig::confirmed(),
        )?;

        println!("    - Airdrop completed: {}", airdrop_sig);

        Ok(())
    }
    async fn create_credential(&self) -> Result<Pubkey> {
        println!("\n2. Creating Credential...");

        let (credential_pda, _bump) = self.derive_credential_pda();

        let instruction = CreateCredentialBuilder::new()
            .payer(self.wallets.payer.pubkey())
            .credential(credential_pda)
            .authority(self.wallets.issuer.pubkey())
            .system_program(system_program)
            .name(self.config.credential_name.clone())
            .signers(vec![self.wallets.authorized_signer1.pubkey()])
            .instruction();

        self.send_and_confirm_instruction(
            instruction,
            &[&self.wallets.issuer],
            "Credential created",
        )
        .await?;

        println!("    - Credential PDA: {}", credential_pda);
        Ok(credential_pda)
    }
    async fn create_schema(&self, credential_pda: &Pubkey) -> Result<Pubkey> {
        println!("\n3. Creating Schema...");

        let (schema_pda, _bump) = self.derive_schema_pda(credential_pda);

        let instruction = CreateSchemaBuilder::new()
            .payer(self.wallets.payer.pubkey())
            .authority(self.wallets.issuer.pubkey())
            .credential(*credential_pda)
            .schema(schema_pda)
            .name(self.config.schema_name.clone())
            .description(self.config.schema_description.clone())
            .layout(self.config.schema_layout.clone())
            .field_names(self.config.schema_fields.clone())
            .instruction();

        self.send_and_confirm_instruction(instruction, &[&self.wallets.issuer], "Schema created")
            .await?;

        println!("    - Schema PDA: {}", schema_pda);
        Ok(schema_pda)
    }

    async fn create_attestation(
        &self,
        credential_pda: &Pubkey,
        schema_pda: &Pubkey,
    ) -> Result<Pubkey> {
        println!("\n4. Creating Attestation...");

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
        let (attestation_pda, _bump) =
            self.derive_attestation_pda(credential_pda, schema_pda, &nonce);

        let instruction = CreateAttestationBuilder::new()
            .payer(self.wallets.payer.pubkey())
            .authority(self.wallets.authorized_signer1.pubkey())
            .credential(*credential_pda)
            .schema(*schema_pda)
            .attestation(attestation_pda)
            .data(serialized_data)
            .expiry(expiry)
            .nonce(nonce)
            .instruction();

        self.send_and_confirm_instruction(
            instruction,
            &[&self.wallets.authorized_signer1],
            "Attestation created",
        )
        .await?;

        println!("    - Attestation PDA: {}", attestation_pda);
        Ok(attestation_pda)
    }

    async fn update_authorized_signers(&self, credential_pda: &Pubkey) -> Result<()> {
        println!("\n5. Updating Authorized Signers...");

        let instruction = ChangeAuthorizedSignersBuilder::new()
            .payer(self.wallets.payer.pubkey())
            .authority(self.wallets.issuer.pubkey())
            .credential(*credential_pda)
            .signers(vec![
                self.wallets.authorized_signer1.pubkey(),
                self.wallets.authorized_signer2.pubkey(),
            ])
            .instruction();

        self.send_and_confirm_instruction(
            instruction,
            &[&self.wallets.issuer],
            "Authorized signers updated",
        )
        .await?;

        Ok(())
    }

    async fn verify_attestation(
        &self,
        schema_pda: &Pubkey,
        user_address: &Pubkey,
        credential_pda: &Pubkey,
        user_name: &str,
    ) -> Result<bool> {
        let (attestation_pda, _bump) =
            self.derive_attestation_pda(credential_pda, schema_pda, user_address);

        let is_valid = match self.rpc_client.get_account(&attestation_pda) {
            Ok(account) => match Attestation::from_bytes(&account.data) {
                Ok(attestation) => {
                    let current_timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;

                    current_timestamp < attestation.expiry
                }
                Err(_) => false,
            },
            Err(_) => false,
        };

        println!(
            "    - {} is {}",
            user_name,
            if is_valid { "verified" } else { "not verified" }
        );

        Ok(is_valid)
    }

    async fn close_attestation(
        &self,
        attestation_pda: &Pubkey,
        credential_pda: &Pubkey,
    ) -> Result<()> {
        println!("\n7. Closing Attestation...");

        let instruction = CloseAttestationBuilder::new()
            .payer(self.wallets.payer.pubkey())
            .attestation(*attestation_pda)
            .authority(self.wallets.authorized_signer1.pubkey())
            .credential(*credential_pda)
            .instruction();

        self.send_and_confirm_instruction(
            instruction,
            &[&self.wallets.authorized_signer1],
            "Closed attestation",
        )
        .await?;

        Ok(())
    }

    async fn tokenize_schema(
        &self,
        credential_pda: &Pubkey,
        schema_pda: &Pubkey,
    ) -> Result<Pubkey> {
        println!("\n4. Tokenizing Schema...");

        let (schema_mint_pda, _bump) = self.derive_schema_mint_pda(schema_pda);
        let (sas_authority, _bump) = Self::derive_sas_authority_address();

        let instruction = TokenizeSchemaBuilder::new()
            .payer(self.wallets.payer.pubkey())
            .authority(self.wallets.issuer.pubkey())
            .sas_pda(sas_authority)
            .credential(*credential_pda)
            .schema(*schema_pda)
            .mint(schema_mint_pda)
            .max_size(self.calculate_schema_mint_size() as u64)
            .instruction();

        self.send_and_confirm_instruction(instruction, &[&self.wallets.issuer], "Schema tokenized")
            .await?;

        println!("    - Schema Mint: {}", schema_mint_pda);
        Ok(schema_mint_pda)
    }

    async fn create_tokenized_attestation(
        &self,
        credential_pda: &Pubkey,
        schema_pda: &Pubkey,
        schema_mint_pda: &Pubkey,
        config: &TokenizedConfig,
    ) -> Result<(Pubkey, Pubkey)> {
        println!("\n5. Creating Tokenized Attestation...");

        let (attestation_pda, _bump) = self.derive_attestation_pda(
            credential_pda,
            schema_pda,
            &self.wallets.test_user.pubkey(),
        );
        let (attestation_mint_pda, _bump) = self.derive_attestation_mint_pda(&attestation_pda);
        let (sas_authority, _bump) = Self::derive_sas_authority_address();

        // Calculate associated token account for recipient
        let recipient_token_account = get_associated_token_address_with_program_id(
            &self.wallets.test_user.pubkey(),
            &attestation_mint_pda,
            &TOKEN_22_PROGRAM_ID,
        );

        // Serialize attestation data
        let attestation_data = TestData {
            name: "test-user".to_string(),
            age: 100,
            country: "usa".to_string(),
        };
        let mut serialized_data = Vec::new();
        attestation_data.serialize(&mut serialized_data)?;

        // Calculate expiry timestamp
        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let expiry = current_timestamp + (config.base.attestation_expiry_days * 24 * 60 * 60);

        let instruction = CreateTokenizedAttestationBuilder::new()
            .payer(self.wallets.payer.pubkey())
            .authority(self.wallets.authorized_signer1.pubkey())
            .sas_pda(sas_authority)
            .credential(*credential_pda)
            .schema(*schema_pda)
            .attestation(attestation_pda)
            .schema_mint(*schema_mint_pda)
            .attestation_mint(attestation_mint_pda)
            .recipient(self.wallets.test_user.pubkey())
            .nonce(self.wallets.test_user.pubkey())
            .expiry(expiry)
            .data(serialized_data)
            .name(config.token_name.clone())
            .uri(config.token_metadata_uri.clone())
            .symbol(config.token_symbol.clone())
            .mint_account_space(self.calculate_attestation_mint_size(config) as u16)
            .recipient_token_account(recipient_token_account)
            .instruction();

        self.send_and_confirm_instruction(
            instruction,
            &[&self.wallets.authorized_signer1],
            "Tokenized attestation created",
        )
        .await?;

        println!("    - Attestation PDA: {}", attestation_pda);
        println!("    - Attestation Mint: {}", attestation_mint_pda);
        Ok((attestation_pda, attestation_mint_pda))
    }

    async fn verify_token_attestation(
        &self,
        schema_pda: &Pubkey,
        user_address: &Pubkey,
        credential_pda: &Pubkey,
    ) -> Result<bool> {
        let (attestation_pda, _bump) =
            self.derive_attestation_pda(credential_pda, schema_pda, user_address);
        let (attestation_mint_pda, _bump) = self.derive_attestation_mint_pda(&attestation_pda);
        let (schema_mint_pda, _bump) = self.derive_schema_mint_pda(schema_pda);

        match self.rpc_client.get_account(&attestation_mint_pda) {
            Ok(account) => {
                // Parse the mint account and check extensions
                println!("    - Attestation Mint: {}", attestation_mint_pda);
                let mint_state = StateWithExtensions::<Mint>::unpack(&account.data).unwrap();

                let token_group_member = mint_state.get_extension::<TokenGroupMember>().unwrap();
                assert_eq!(token_group_member.group, schema_mint_pda);

                let token_metadata = &mint_state
                    .get_variable_len_extension::<TokenMetadata>()
                    .unwrap();
                assert_eq!(token_metadata.additional_metadata[0].0, "attestation");
                assert_eq!(
                    token_metadata.additional_metadata[0].1,
                    attestation_pda.to_string()
                );
                assert_eq!(token_metadata.additional_metadata[1].0, "schema");
                assert_eq!(
                    token_metadata.additional_metadata[1].1,
                    schema_pda.to_string()
                );

                Ok(true)
            }
            Err(_) => {
                println!("    - Attestation mint not found");
                Ok(false)
            }
        }
    }

    async fn close_tokenized_attestation(
        &self,
        attestation_pda: &Pubkey,
        attestation_mint_pda: &Pubkey,
        credential_pda: &Pubkey,
    ) -> Result<()> {
        println!("\n8. Closing Tokenized Attestation...");

        let recipient_token_account = get_associated_token_address_with_program_id(
            &self.wallets.test_user.pubkey(),
            attestation_mint_pda,
            &TOKEN_22_PROGRAM_ID,
        );
        let (sas_authority, _bump) = Self::derive_sas_authority_address();

        let instruction = CloseTokenizedAttestationBuilder::new()
            .payer(self.wallets.payer.pubkey())
            .authority(self.wallets.authorized_signer1.pubkey())
            .sas_pda(sas_authority)
            .credential(*credential_pda)
            .attestation(*attestation_pda)
            .attestation_program(SOLANA_ATTESTATION_SERVICE_ID)
            .attestation_mint(*attestation_mint_pda)
            .attestation_token_account(recipient_token_account)
            .instruction();

        self.send_and_confirm_instruction(
            instruction,
            &[&self.wallets.authorized_signer1],
            "Tokenized attestation closed",
        )
        .await?;

        Ok(())
    }

    pub async fn run_demo(&self) -> Result<()> {
        println!("Starting Solana Attestation Service Demo\n");

        // Step 1: Fund payer
        self.fund_payer().await?;

        // Step 2: Create Credential
        let credential_pda = self.create_credential().await?;

        // Step 3: Create Schema
        let schema_pda = self.create_schema(&credential_pda).await?;

        // Step 4: Create Attestation
        let attestation_pda = self
            .create_attestation(&credential_pda, &schema_pda)
            .await?;

        // Step 5: Update Authorized Signers
        self.update_authorized_signers(&credential_pda).await?;

        // Step 6: Verify Attestations
        println!("\n6. Verifying Attestations...");
        let _test_user_result = self
            .verify_attestation(
                &schema_pda,
                &self.wallets.test_user.pubkey(),
                &credential_pda,
                "Test User",
            )
            .await;
        let _random_user_result = self
            .verify_attestation(
                &schema_pda,
                &Keypair::new().pubkey(),
                &credential_pda,
                "Random User",
            )
            .await;

        // Step 7: Close Attestation
        self.close_attestation(&attestation_pda, &credential_pda)
            .await?;

        println!("\nSolana Attestation Service demo completed successfully!");

        Ok(())
    }

    pub async fn run_tokenized_demo(&self) -> Result<()> {
        println!("Starting Solana Attestation Service Tokenized Demo\n");
        let config = TokenizedConfig::default();

        // Steps 1-3: Same as regular demo
        self.fund_payer().await?;
        let credential_pda = self.create_credential().await?;
        let schema_pda = self.create_schema(&credential_pda).await?;

        // Step 4: Tokenize Schema
        let schema_mint_pda = self.tokenize_schema(&credential_pda, &schema_pda).await?;

        // Step 5: Create Tokenized Attestation
        let (attestation_pda, attestation_mint_pda) = self
            .create_tokenized_attestation(&credential_pda, &schema_pda, &schema_mint_pda, &config)
            .await?;

        // Step 6: Verify Attestations (non-tokenized attestation accounts)
        println!("\n6. Verifying Attestations...");
        let _test_user_result = self
            .verify_attestation(
                &schema_pda,
                &self.wallets.test_user.pubkey(),
                &credential_pda,
                "Test User",
            )
            .await;
        let _random_user_result = self
            .verify_attestation(
                &schema_pda,
                &Keypair::new().pubkey(),
                &credential_pda,
                "Random User",
            )
            .await;

        // Step 7: Verify Token Attestation
        println!("\n7. Verifying Token Attestation...");
        let is_token_verified = self
            .verify_token_attestation(
                &schema_pda,
                &self.wallets.test_user.pubkey(),
                &credential_pda,
            )
            .await?;
        println!(
            "    - Test User's token is {}",
            if is_token_verified {
                "verified"
            } else {
                "not verified"
            }
        );

        // Step 8: Close Tokenized Attestation
        self.close_tokenized_attestation(&attestation_pda, &attestation_mint_pda, &credential_pda)
            .await?;

        println!("\nSolana Attestation Service tokenized demo completed successfully!");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let demo = SasDemo::new();

    // Get demo type from command line args or default to standard
    let args: Vec<String> = std::env::args().collect();
    let demo_type = args.get(1).map(|s| s.as_str()).unwrap_or("standard");

    match demo_type {
        "tokenized" => demo.run_tokenized_demo().await,
        _ => demo.run_demo().await,
    }
    .map_err(|e| {
        eprintln!("❌ Demo failed: {}", e);
        std::process::exit(1);
    })
}
