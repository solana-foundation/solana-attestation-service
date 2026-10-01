use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_address::Address;
use solana_commitment_config::CommitmentConfig;
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_native_token::LAMPORTS_PER_SOL;
use solana_rpc_client::rpc_client::RpcClient;
use solana_signer::Signer;
use solana_transaction::Transaction;

use solana_attestation_service_client::{
    accounts::Attestation,
    instructions::{
        ChangeAuthorizedSignersBuilder, CloseAttestationBuilder, CreateAttestationBuilder, CreateCredentialBuilder,
        CreateSchemaBuilder,
    },
    programs::SOLANA_ATTESTATION_SERVICE_ID,
};

const CLOCK_SYSVAR_ID: Address = solana_address::address!("SysvarC1ock11111111111111111111111111111111");

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
        Self { name: "test-user".to_string(), age: 100, country: "usa".to_string() }
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

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

impl SasDemo {
    fn new() -> Self {
        let config = Config::default();
        let rpc_client = RpcClient::new_with_commitment(config.rpc_url.clone(), CommitmentConfig::confirmed());
        let wallets = Wallets::new();

        Self { config, rpc_client, wallets }
    }

    fn derive_credential_pda(&self) -> (Address, u8) {
        Address::find_program_address(
            &[b"credential", self.wallets.issuer.pubkey().as_ref(), self.config.credential_name.as_bytes()],
            &SOLANA_ATTESTATION_SERVICE_ID,
        )
    }

    fn derive_schema_pda(&self, credential_pda: &Address) -> (Address, u8) {
        Address::find_program_address(
            &[b"schema", credential_pda.as_ref(), self.config.schema_name.as_bytes(), &[self.config.schema_version]],
            &SOLANA_ATTESTATION_SERVICE_ID,
        )
    }

    fn derive_attestation_pda(&self, credential_pda: &Address, schema_pda: &Address, nonce: &Address) -> (Address, u8) {
        Address::find_program_address(
            &[b"attestation", credential_pda.as_ref(), schema_pda.as_ref(), nonce.as_ref()],
            &SOLANA_ATTESTATION_SERVICE_ID,
        )
    }

    fn send_and_confirm_instruction(
        &self,
        instruction: Instruction,
        signers: &[&Keypair],
        description: &str,
    ) -> Result<()> {
        let sim_message = Message::new(
            &[
                ComputeBudgetInstruction::set_compute_unit_limit(1_400_000),
                ComputeBudgetInstruction::set_compute_unit_price(1),
                instruction.clone(),
            ],
            Some(&self.wallets.payer.pubkey()),
        );

        let mut all_signers = vec![&self.wallets.payer];
        all_signers.extend(signers);

        let simulation = Transaction::new(&all_signers, sim_message, self.rpc_client.get_latest_blockhash()?);

        let sim_result = self.rpc_client.simulate_transaction(&simulation)?;
        let compute = sim_result.value.units_consumed.unwrap_or(200_000);

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
        let signature = self.rpc_client.send_and_confirm_transaction_with_spinner(&transaction)?;

        println!("    - {} - Signature: {}", description, signature);
        Ok(())
    }

    fn fund_payer(&self) -> Result<()> {
        println!("1. Funding payer wallet...");

        let airdrop_sig = self.rpc_client.request_airdrop(&self.wallets.payer.pubkey(), LAMPORTS_PER_SOL)?;

        self.rpc_client.confirm_transaction_with_spinner(
            &airdrop_sig,
            &self.rpc_client.get_latest_blockhash()?,
            CommitmentConfig::confirmed(),
        )?;

        println!("    - Airdrop completed: {}", airdrop_sig);

        Ok(())
    }

    fn create_credential(&self) -> Result<Address> {
        println!("\n2. Creating Credential...");

        let (credential_pda, _bump) = self.derive_credential_pda();

        let instruction = CreateCredentialBuilder::new()
            .payer(self.wallets.payer.pubkey())
            .credential(credential_pda)
            .authority(self.wallets.issuer.pubkey())
            .name(self.config.credential_name.clone())
            .signers(vec![self.wallets.authorized_signer1.pubkey()])
            .instruction();

        self.send_and_confirm_instruction(instruction, &[&self.wallets.issuer], "Credential created")?;

        println!("    - Credential PDA: {}", credential_pda);
        Ok(credential_pda)
    }

    fn create_schema(&self, credential_pda: &Address) -> Result<Address> {
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

        self.send_and_confirm_instruction(instruction, &[&self.wallets.issuer], "Schema created")?;

        println!("    - Schema PDA: {}", schema_pda);
        Ok(schema_pda)
    }

    fn create_attestation(&self, credential_pda: &Address, schema_pda: &Address) -> Result<Address> {
        println!("\n4. Creating Attestation...");

        let expiry = now() + (self.config.attestation_expiry_days * 24 * 60 * 60);
        let serialized_data = borsh::to_vec(&TestData::get_example_data())?;

        let nonce = self.wallets.test_user.pubkey();
        let (attestation_pda, _bump) = self.derive_attestation_pda(credential_pda, schema_pda, &nonce);

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

        self.send_and_confirm_instruction(instruction, &[&self.wallets.authorized_signer1], "Attestation created")?;

        println!("    - Attestation PDA: {}", attestation_pda);
        Ok(attestation_pda)
    }

    fn update_authorized_signers(&self, credential_pda: &Address) -> Result<()> {
        println!("\n5. Updating Authorized Signers...");

        let instruction = ChangeAuthorizedSignersBuilder::new()
            .payer(self.wallets.payer.pubkey())
            .authority(self.wallets.issuer.pubkey())
            .credential(*credential_pda)
            .signers(vec![self.wallets.authorized_signer1.pubkey(), self.wallets.authorized_signer2.pubkey()])
            .instruction();

        self.send_and_confirm_instruction(instruction, &[&self.wallets.issuer], "Authorized signers updated")
    }

    fn chain_now(&self) -> Option<i64> {
        let clock = self.rpc_client.get_account(&CLOCK_SYSVAR_ID).ok()?;
        // Clock sysvar layout: slot, epoch_start_timestamp, epoch, leader_schedule_epoch, unix_timestamp.
        Some(i64::from_le_bytes(clock.data.get(32..40)?.try_into().ok()?))
    }

    fn verify_attestation(
        &self,
        schema_pda: &Address,
        user_address: &Address,
        credential_pda: &Address,
        label: &str,
    ) -> bool {
        let (attestation_pda, _bump) = self.derive_attestation_pda(credential_pda, schema_pda, user_address);

        let is_valid = self
            .rpc_client
            .get_account(&attestation_pda)
            .ok()
            .and_then(|account| Attestation::from_bytes(&account.data).ok())
            .is_some_and(|attestation| {
                attestation.expiry == 0 || self.chain_now().is_some_and(|now| now < attestation.expiry)
            });

        println!("    - {} is {}", label, if is_valid { "verified" } else { "not verified" });

        is_valid
    }

    fn close_attestation(&self, attestation_pda: &Address, credential_pda: &Address) -> Result<()> {
        println!("\n7. Closing Attestation...");

        let instruction = CloseAttestationBuilder::new()
            .payer(self.wallets.payer.pubkey())
            .attestation(*attestation_pda)
            .authority(self.wallets.authorized_signer1.pubkey())
            .credential(*credential_pda)
            .instruction();

        self.send_and_confirm_instruction(instruction, &[&self.wallets.authorized_signer1], "Closed attestation")
    }

    pub fn run_demo(&self) -> Result<()> {
        println!("Starting Solana Attestation Service Demo\n");

        self.fund_payer()?;
        let credential_pda = self.create_credential()?;
        let schema_pda = self.create_schema(&credential_pda)?;
        let attestation_pda = self.create_attestation(&credential_pda, &schema_pda)?;
        self.update_authorized_signers(&credential_pda)?;

        println!("\n6. Verifying Attestations...");
        self.verify_attestation(&schema_pda, &self.wallets.test_user.pubkey(), &credential_pda, "Test User");
        self.verify_attestation(&schema_pda, &Keypair::new().pubkey(), &credential_pda, "Random User");

        self.close_attestation(&attestation_pda, &credential_pda)?;

        println!("\nSolana Attestation Service demo completed successfully!");

        Ok(())
    }
}

fn main() -> Result<()> {
    SasDemo::new().run_demo()
}
