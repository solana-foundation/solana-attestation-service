//! This code was AUTOGENERATED using the codama library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun codama to update it.
//!
//! <https://github.com/codama-idl/codama>
//!

use borsh::BorshDeserialize;
use borsh::BorshSerialize;

/// Accounts.
#[derive(Debug)]
pub struct TokenizeSchema {
    pub payer: solana_program::pubkey::Pubkey,

    pub authority: solana_program::pubkey::Pubkey,
    /// Credential the Schema is associated with
    pub credential: solana_program::pubkey::Pubkey,

    pub schema: solana_program::pubkey::Pubkey,
    /// Mint of Schema Token
    pub mint: solana_program::pubkey::Pubkey,
    /// Program derived address used as program signer authority
    pub sas_pda: solana_program::pubkey::Pubkey,

    pub system_program: solana_program::pubkey::Pubkey,

    pub token_program: solana_program::pubkey::Pubkey,
}

impl TokenizeSchema {
    pub fn instruction(
        &self,
        args: TokenizeSchemaInstructionArgs,
    ) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(args, &[])
    }
    #[allow(clippy::arithmetic_side_effects)]
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        args: TokenizeSchemaInstructionArgs,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(8 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.payer, true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.authority,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.credential,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.schema,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.mint, false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.sas_pda,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.system_program,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.token_program,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let mut data = borsh::to_vec(&TokenizeSchemaInstructionData::new()).unwrap();
        let mut args = borsh::to_vec(&args).unwrap();
        data.append(&mut args);

        solana_program::instruction::Instruction {
            program_id: crate::SOLANA_ATTESTATION_SERVICE_ID,
            accounts,
            data,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TokenizeSchemaInstructionData {
    discriminator: u8,
}

impl TokenizeSchemaInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 9 }
    }
}

impl Default for TokenizeSchemaInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TokenizeSchemaInstructionArgs {
    pub max_size: u64,
}

/// Instruction builder for `TokenizeSchema`.
///
/// ### Accounts:
///
///   0. `[writable, signer]` payer
///   1. `[signer]` authority
///   2. `[]` credential
///   3. `[]` schema
///   4. `[writable]` mint
///   5. `[]` sas_pda
///   6. `[optional]` system_program (default to `11111111111111111111111111111111`)
///   7. `[optional]` token_program (default to `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`)
#[derive(Clone, Debug, Default)]
pub struct TokenizeSchemaBuilder {
    payer: Option<solana_program::pubkey::Pubkey>,
    authority: Option<solana_program::pubkey::Pubkey>,
    credential: Option<solana_program::pubkey::Pubkey>,
    schema: Option<solana_program::pubkey::Pubkey>,
    mint: Option<solana_program::pubkey::Pubkey>,
    sas_pda: Option<solana_program::pubkey::Pubkey>,
    system_program: Option<solana_program::pubkey::Pubkey>,
    token_program: Option<solana_program::pubkey::Pubkey>,
    max_size: Option<u64>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl TokenizeSchemaBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    #[inline(always)]
    pub fn payer(&mut self, payer: solana_program::pubkey::Pubkey) -> &mut Self {
        self.payer = Some(payer);
        self
    }
    #[inline(always)]
    pub fn authority(&mut self, authority: solana_program::pubkey::Pubkey) -> &mut Self {
        self.authority = Some(authority);
        self
    }
    /// Credential the Schema is associated with
    #[inline(always)]
    pub fn credential(&mut self, credential: solana_program::pubkey::Pubkey) -> &mut Self {
        self.credential = Some(credential);
        self
    }
    #[inline(always)]
    pub fn schema(&mut self, schema: solana_program::pubkey::Pubkey) -> &mut Self {
        self.schema = Some(schema);
        self
    }
    /// Mint of Schema Token
    #[inline(always)]
    pub fn mint(&mut self, mint: solana_program::pubkey::Pubkey) -> &mut Self {
        self.mint = Some(mint);
        self
    }
    /// Program derived address used as program signer authority
    #[inline(always)]
    pub fn sas_pda(&mut self, sas_pda: solana_program::pubkey::Pubkey) -> &mut Self {
        self.sas_pda = Some(sas_pda);
        self
    }
    /// `[optional account, default to '11111111111111111111111111111111']`
    #[inline(always)]
    pub fn system_program(&mut self, system_program: solana_program::pubkey::Pubkey) -> &mut Self {
        self.system_program = Some(system_program);
        self
    }
    /// `[optional account, default to 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA']`
    #[inline(always)]
    pub fn token_program(&mut self, token_program: solana_program::pubkey::Pubkey) -> &mut Self {
        self.token_program = Some(token_program);
        self
    }
    #[inline(always)]
    pub fn max_size(&mut self, max_size: u64) -> &mut Self {
        self.max_size = Some(max_size);
        self
    }
    /// Add an additional account to the instruction.
    #[inline(always)]
    pub fn add_remaining_account(
        &mut self,
        account: solana_program::instruction::AccountMeta,
    ) -> &mut Self {
        self.__remaining_accounts.push(account);
        self
    }
    /// Add additional accounts to the instruction.
    #[inline(always)]
    pub fn add_remaining_accounts(
        &mut self,
        accounts: &[solana_program::instruction::AccountMeta],
    ) -> &mut Self {
        self.__remaining_accounts.extend_from_slice(accounts);
        self
    }
    #[allow(clippy::clone_on_copy)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let accounts = TokenizeSchema {
            payer: self.payer.expect("payer is not set"),
            authority: self.authority.expect("authority is not set"),
            credential: self.credential.expect("credential is not set"),
            schema: self.schema.expect("schema is not set"),
            mint: self.mint.expect("mint is not set"),
            sas_pda: self.sas_pda.expect("sas_pda is not set"),
            system_program: self
                .system_program
                .unwrap_or(solana_program::pubkey!("11111111111111111111111111111111")),
            token_program: self.token_program.unwrap_or(solana_program::pubkey!(
                "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
            )),
        };
        let args = TokenizeSchemaInstructionArgs {
            max_size: self.max_size.clone().expect("max_size is not set"),
        };

        accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
    }
}

/// `tokenize_schema` CPI accounts.
pub struct TokenizeSchemaCpiAccounts<'a, 'b> {
    pub payer: &'b solana_program::account_info::AccountInfo<'a>,

    pub authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Credential the Schema is associated with
    pub credential: &'b solana_program::account_info::AccountInfo<'a>,

    pub schema: &'b solana_program::account_info::AccountInfo<'a>,
    /// Mint of Schema Token
    pub mint: &'b solana_program::account_info::AccountInfo<'a>,
    /// Program derived address used as program signer authority
    pub sas_pda: &'b solana_program::account_info::AccountInfo<'a>,

    pub system_program: &'b solana_program::account_info::AccountInfo<'a>,

    pub token_program: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `tokenize_schema` CPI instruction.
pub struct TokenizeSchemaCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,

    pub payer: &'b solana_program::account_info::AccountInfo<'a>,

    pub authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Credential the Schema is associated with
    pub credential: &'b solana_program::account_info::AccountInfo<'a>,

    pub schema: &'b solana_program::account_info::AccountInfo<'a>,
    /// Mint of Schema Token
    pub mint: &'b solana_program::account_info::AccountInfo<'a>,
    /// Program derived address used as program signer authority
    pub sas_pda: &'b solana_program::account_info::AccountInfo<'a>,

    pub system_program: &'b solana_program::account_info::AccountInfo<'a>,

    pub token_program: &'b solana_program::account_info::AccountInfo<'a>,
    /// The arguments for the instruction.
    pub __args: TokenizeSchemaInstructionArgs,
}

impl<'a, 'b> TokenizeSchemaCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: TokenizeSchemaCpiAccounts<'a, 'b>,
        args: TokenizeSchemaInstructionArgs,
    ) -> Self {
        Self {
            __program: program,
            payer: accounts.payer,
            authority: accounts.authority,
            credential: accounts.credential,
            schema: accounts.schema,
            mint: accounts.mint,
            sas_pda: accounts.sas_pda,
            system_program: accounts.system_program,
            token_program: accounts.token_program,
            __args: args,
        }
    }
    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(&[], &[])
    }
    #[inline(always)]
    pub fn invoke_with_remaining_accounts(
        &self,
        remaining_accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(&[], remaining_accounts)
    }
    #[inline(always)]
    pub fn invoke_signed(
        &self,
        signers_seeds: &[&[&[u8]]],
    ) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(signers_seeds, &[])
    }
    #[allow(clippy::arithmetic_side_effects)]
    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::vec_init_then_push)]
    pub fn invoke_signed_with_remaining_accounts(
        &self,
        signers_seeds: &[&[&[u8]]],
        remaining_accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> solana_program::entrypoint::ProgramResult {
        let mut accounts = Vec::with_capacity(8 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.payer.key,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.authority.key,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.credential.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.schema.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.mint.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.sas_pda.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.system_program.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.token_program.key,
            false,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let mut data = borsh::to_vec(&TokenizeSchemaInstructionData::new()).unwrap();
        let mut args = borsh::to_vec(&self.__args).unwrap();
        data.append(&mut args);

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::SOLANA_ATTESTATION_SERVICE_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(9 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.payer.clone());
        account_infos.push(self.authority.clone());
        account_infos.push(self.credential.clone());
        account_infos.push(self.schema.clone());
        account_infos.push(self.mint.clone());
        account_infos.push(self.sas_pda.clone());
        account_infos.push(self.system_program.clone());
        account_infos.push(self.token_program.clone());
        remaining_accounts
            .iter()
            .for_each(|remaining_account| account_infos.push(remaining_account.0.clone()));

        if signers_seeds.is_empty() {
            solana_program::program::invoke(&instruction, &account_infos)
        } else {
            solana_program::program::invoke_signed(&instruction, &account_infos, signers_seeds)
        }
    }
}

/// Instruction builder for `TokenizeSchema` via CPI.
///
/// ### Accounts:
///
///   0. `[writable, signer]` payer
///   1. `[signer]` authority
///   2. `[]` credential
///   3. `[]` schema
///   4. `[writable]` mint
///   5. `[]` sas_pda
///   6. `[]` system_program
///   7. `[]` token_program
#[derive(Clone, Debug)]
pub struct TokenizeSchemaCpiBuilder<'a, 'b> {
    instruction: Box<TokenizeSchemaCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> TokenizeSchemaCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(TokenizeSchemaCpiBuilderInstruction {
            __program: program,
            payer: None,
            authority: None,
            credential: None,
            schema: None,
            mint: None,
            sas_pda: None,
            system_program: None,
            token_program: None,
            max_size: None,
            __remaining_accounts: Vec::new(),
        });
        Self { instruction }
    }
    #[inline(always)]
    pub fn payer(&mut self, payer: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.payer = Some(payer);
        self
    }
    #[inline(always)]
    pub fn authority(
        &mut self,
        authority: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.authority = Some(authority);
        self
    }
    /// Credential the Schema is associated with
    #[inline(always)]
    pub fn credential(
        &mut self,
        credential: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.credential = Some(credential);
        self
    }
    #[inline(always)]
    pub fn schema(
        &mut self,
        schema: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.schema = Some(schema);
        self
    }
    /// Mint of Schema Token
    #[inline(always)]
    pub fn mint(&mut self, mint: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.mint = Some(mint);
        self
    }
    /// Program derived address used as program signer authority
    #[inline(always)]
    pub fn sas_pda(
        &mut self,
        sas_pda: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.sas_pda = Some(sas_pda);
        self
    }
    #[inline(always)]
    pub fn system_program(
        &mut self,
        system_program: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.system_program = Some(system_program);
        self
    }
    #[inline(always)]
    pub fn token_program(
        &mut self,
        token_program: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.token_program = Some(token_program);
        self
    }
    #[inline(always)]
    pub fn max_size(&mut self, max_size: u64) -> &mut Self {
        self.instruction.max_size = Some(max_size);
        self
    }
    /// Add an additional account to the instruction.
    #[inline(always)]
    pub fn add_remaining_account(
        &mut self,
        account: &'b solana_program::account_info::AccountInfo<'a>,
        is_writable: bool,
        is_signer: bool,
    ) -> &mut Self {
        self.instruction
            .__remaining_accounts
            .push((account, is_writable, is_signer));
        self
    }
    /// Add additional accounts to the instruction.
    ///
    /// Each account is represented by a tuple of the `AccountInfo`, a `bool` indicating whether the account is writable or not,
    /// and a `bool` indicating whether the account is a signer or not.
    #[inline(always)]
    pub fn add_remaining_accounts(
        &mut self,
        accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> &mut Self {
        self.instruction
            .__remaining_accounts
            .extend_from_slice(accounts);
        self
    }
    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed(&[])
    }
    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::vec_init_then_push)]
    pub fn invoke_signed(
        &self,
        signers_seeds: &[&[&[u8]]],
    ) -> solana_program::entrypoint::ProgramResult {
        let args = TokenizeSchemaInstructionArgs {
            max_size: self
                .instruction
                .max_size
                .clone()
                .expect("max_size is not set"),
        };
        let instruction = TokenizeSchemaCpi {
            __program: self.instruction.__program,

            payer: self.instruction.payer.expect("payer is not set"),

            authority: self.instruction.authority.expect("authority is not set"),

            credential: self.instruction.credential.expect("credential is not set"),

            schema: self.instruction.schema.expect("schema is not set"),

            mint: self.instruction.mint.expect("mint is not set"),

            sas_pda: self.instruction.sas_pda.expect("sas_pda is not set"),

            system_program: self
                .instruction
                .system_program
                .expect("system_program is not set"),

            token_program: self
                .instruction
                .token_program
                .expect("token_program is not set"),
            __args: args,
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct TokenizeSchemaCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    payer: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    authority: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    credential: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    schema: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    mint: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    sas_pda: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    system_program: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    token_program: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    max_size: Option<u64>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}
