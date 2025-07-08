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
        ChangeAuthorizedSignersBuilder, CloseAttestationBuilder, CreateAttestationBuilder,
        CreateCredentialBuilder, CreateSchemaBuilder,
    },
    programs::SOLANA_ATTESTATION_SERVICE_ID,
};

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
    
}

#[tokio::main]
async fn main() -> Result<()> {
    let demo = SasDemo::new();
 
    match demo.run_demo().await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("❌ Demo failed: {}", e);
            std::process::exit(1);
        }
    }
}