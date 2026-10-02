use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_account::from_account;
use solana_address::Address;
use solana_commitment_config::CommitmentConfig;
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_native_token::LAMPORTS_PER_SOL;
use solana_rpc_client::rpc_client::RpcClient;
use solana_signer::Signer;
use solana_sysvar::clock::{self, Clock};
use solana_transaction::Transaction;
use spl_token_2022_interface::{
    extension::{BaseStateWithExtensions, ExtensionType, StateWithExtensions},
    state::Mint,
    ID as TOKEN_22_PROGRAM_ID,
};
use spl_token_group_interface::state::TokenGroupMember;
use spl_token_metadata_interface::state::TokenMetadata;

use solana_attestation::{
    accounts::Attestation,
    instructions::{
        ChangeAuthorizedSignersBuilder, CloseAttestationBuilder, CloseTokenizedAttestationBuilder,
        CreateAttestationBuilder, CreateCredentialBuilder, CreateSchemaBuilder, CreateTokenizedAttestationBuilder,
        TokenizeSchemaBuilder,
    },
    programs::SOLANA_ATTESTATION_SERVICE_ID,
};

const ATA_PROGRAM_ID: Address = solana_address::address!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

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

struct TokenizedConfig {
    base: Config,
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

    fn derive_schema_mint_pda(&self, schema_pda: &Address) -> (Address, u8) {
        Address::find_program_address(&[b"schemaMint", schema_pda.as_ref()], &SOLANA_ATTESTATION_SERVICE_ID)
    }

    fn derive_attestation_mint_pda(&self, attestation_pda: &Address) -> (Address, u8) {
        Address::find_program_address(&[b"attestationMint", attestation_pda.as_ref()], &SOLANA_ATTESTATION_SERVICE_ID)
    }

    fn derive_sas_authority_address() -> (Address, u8) {
        Address::find_program_address(&[b"sas"], &SOLANA_ATTESTATION_SERVICE_ID)
    }

    fn derive_token_account(owner: &Address, mint: &Address) -> Address {
        Address::find_program_address(&[owner.as_ref(), TOKEN_22_PROGRAM_ID.as_ref(), mint.as_ref()], &ATA_PROGRAM_ID).0
    }

    fn calculate_schema_mint_size(&self) -> Result<usize> {
        Ok(ExtensionType::try_calculate_account_len::<Mint>(&[ExtensionType::GroupPointer])?)
    }

    fn calculate_attestation_mint_size(
        &self,
        config: &TokenizedConfig,
        attestation_pda: &Address,
        schema_pda: &Address,
    ) -> Result<usize> {
        let base_size = ExtensionType::try_calculate_account_len::<Mint>(&[
            ExtensionType::GroupMemberPointer,
            ExtensionType::NonTransferable,
            ExtensionType::MetadataPointer,
            ExtensionType::PermanentDelegate,
            ExtensionType::MintCloseAuthority,
            ExtensionType::TokenGroupMember,
        ])?;
        let token_metadata = TokenMetadata {
            name: config.token_name.clone(),
            symbol: config.token_symbol.clone(),
            uri: config.token_metadata_uri.clone(),
            additional_metadata: vec![
                ("attestation".to_string(), attestation_pda.to_string()),
                ("schema".to_string(), schema_pda.to_string()),
            ],
            ..Default::default()
        };

        Ok(base_size + token_metadata.tlv_size_of()?)
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
        let account = self.rpc_client.get_account(&clock::ID).ok()?;
        from_account::<Clock, _>(&account).map(|clock| clock.unix_timestamp)
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

    fn tokenize_schema(&self, credential_pda: &Address, schema_pda: &Address) -> Result<Address> {
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
            .max_size(self.calculate_schema_mint_size()? as u64)
            .instruction();

        self.send_and_confirm_instruction(instruction, &[&self.wallets.issuer], "Schema tokenized")?;

        println!("    - Schema Mint: {}", schema_mint_pda);
        Ok(schema_mint_pda)
    }

    fn create_tokenized_attestation(
        &self,
        credential_pda: &Address,
        schema_pda: &Address,
        schema_mint_pda: &Address,
        config: &TokenizedConfig,
    ) -> Result<(Address, Address)> {
        println!("\n5. Creating Tokenized Attestation...");

        let (attestation_pda, _bump) =
            self.derive_attestation_pda(credential_pda, schema_pda, &self.wallets.test_user.pubkey());
        let (attestation_mint_pda, _bump) = self.derive_attestation_mint_pda(&attestation_pda);
        let (sas_authority, _bump) = Self::derive_sas_authority_address();
        let recipient_token_account =
            Self::derive_token_account(&self.wallets.test_user.pubkey(), &attestation_mint_pda);

        let serialized_data = borsh::to_vec(&TestData::get_example_data())?;
        let expiry = now() + (config.base.attestation_expiry_days * 24 * 60 * 60);
        let mint_account_space = self.calculate_attestation_mint_size(config, &attestation_pda, schema_pda)?;

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
            .mint_account_space(mint_account_space as u16)
            .recipient_token_account(recipient_token_account)
            .instruction();

        self.send_and_confirm_instruction(
            instruction,
            &[&self.wallets.authorized_signer1],
            "Tokenized attestation created",
        )?;

        println!("    - Attestation PDA: {}", attestation_pda);
        println!("    - Attestation Mint: {}", attestation_mint_pda);
        Ok((attestation_pda, attestation_mint_pda))
    }

    fn verify_token_attestation(
        &self,
        schema_pda: &Address,
        user_address: &Address,
        credential_pda: &Address,
    ) -> Result<bool> {
        if !self.verify_attestation(schema_pda, user_address, credential_pda, "Token holder") {
            return Ok(false);
        }
        let (attestation_pda, _bump) = self.derive_attestation_pda(credential_pda, schema_pda, user_address);
        let (attestation_mint_pda, _bump) = self.derive_attestation_mint_pda(&attestation_pda);
        let (schema_mint_pda, _bump) = self.derive_schema_mint_pda(schema_pda);

        let Ok(account) = self.rpc_client.get_account(&attestation_mint_pda) else {
            println!("    - Attestation mint not found");
            return Ok(false);
        };

        println!("    - Attestation Mint: {}", attestation_mint_pda);
        let mint_state = StateWithExtensions::<Mint>::unpack(&account.data)?;

        let token_group_member = mint_state.get_extension::<TokenGroupMember>()?;
        assert_eq!(token_group_member.group, schema_mint_pda);

        let token_metadata = mint_state.get_variable_len_extension::<TokenMetadata>()?;
        assert_eq!(token_metadata.additional_metadata[0].0, "attestation");
        assert_eq!(token_metadata.additional_metadata[0].1, attestation_pda.to_string());
        assert_eq!(token_metadata.additional_metadata[1].0, "schema");
        assert_eq!(token_metadata.additional_metadata[1].1, schema_pda.to_string());

        Ok(true)
    }

    fn close_tokenized_attestation(
        &self,
        attestation_pda: &Address,
        attestation_mint_pda: &Address,
        credential_pda: &Address,
    ) -> Result<()> {
        println!("\n8. Closing Tokenized Attestation...");

        let recipient_token_account =
            Self::derive_token_account(&self.wallets.test_user.pubkey(), attestation_mint_pda);
        let (sas_authority, _bump) = Self::derive_sas_authority_address();

        let instruction = CloseTokenizedAttestationBuilder::new()
            .payer(self.wallets.payer.pubkey())
            .authority(self.wallets.authorized_signer1.pubkey())
            .sas_pda(sas_authority)
            .credential(*credential_pda)
            .attestation(*attestation_pda)
            .attestation_mint(*attestation_mint_pda)
            .attestation_token_account(recipient_token_account)
            .instruction();

        self.send_and_confirm_instruction(
            instruction,
            &[&self.wallets.authorized_signer1],
            "Tokenized attestation closed",
        )
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

    pub fn run_tokenized_demo(&self) -> Result<()> {
        println!("Starting Solana Attestation Service Tokenized Demo\n");
        let config = TokenizedConfig::default();

        self.fund_payer()?;
        let credential_pda = self.create_credential()?;
        let schema_pda = self.create_schema(&credential_pda)?;
        let schema_mint_pda = self.tokenize_schema(&credential_pda, &schema_pda)?;
        let (attestation_pda, attestation_mint_pda) =
            self.create_tokenized_attestation(&credential_pda, &schema_pda, &schema_mint_pda, &config)?;

        println!("\n6. Verifying Attestations...");
        self.verify_attestation(&schema_pda, &self.wallets.test_user.pubkey(), &credential_pda, "Test User");
        self.verify_attestation(&schema_pda, &Keypair::new().pubkey(), &credential_pda, "Random User");

        println!("\n7. Verifying Token Attestation...");
        let is_token_verified =
            self.verify_token_attestation(&schema_pda, &self.wallets.test_user.pubkey(), &credential_pda)?;
        println!("    - Test User's token is {}", if is_token_verified { "verified" } else { "not verified" });

        self.close_tokenized_attestation(&attestation_pda, &attestation_mint_pda, &credential_pda)?;

        println!("\nSolana Attestation Service tokenized demo completed successfully!");
        Ok(())
    }
}

fn main() -> Result<()> {
    let demo = SasDemo::new();
    match std::env::args().nth(1).as_deref() {
        Some("tokenized") => demo.run_tokenized_demo(),
        _ => demo.run_demo(),
    }
}
