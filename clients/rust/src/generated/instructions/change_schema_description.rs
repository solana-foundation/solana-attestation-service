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
pub struct ChangeSchemaDescription {
    pub authority: solana_program::pubkey::Pubkey,
    /// Credential the Schema is associated with
    pub credential: solana_program::pubkey::Pubkey,
    /// Credential the Schema is associated with
    pub schema: solana_program::pubkey::Pubkey,

    pub system_program: solana_program::pubkey::Pubkey,
}

impl ChangeSchemaDescription {
    pub fn instruction(
        &self,
        args: ChangeSchemaDescriptionInstructionArgs,
    ) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(args, &[])
    }
    #[allow(clippy::arithmetic_side_effects)]
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        args: ChangeSchemaDescriptionInstructionArgs,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(4 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.authority,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.credential,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.schema,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.system_program,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let mut data = borsh::to_vec(&ChangeSchemaDescriptionInstructionData::new()).unwrap();
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
pub struct ChangeSchemaDescriptionInstructionData {
    discriminator: u8,
}

impl ChangeSchemaDescriptionInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 4 }
    }
}

impl Default for ChangeSchemaDescriptionInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChangeSchemaDescriptionInstructionArgs {
    pub description: String,
}

/// Instruction builder for `ChangeSchemaDescription`.
///
/// ### Accounts:
///
///   0. `[signer]` authority
///   1. `[]` credential
///   2. `[writable]` schema
///   3. `[optional]` system_program (default to `11111111111111111111111111111111`)
#[derive(Clone, Debug, Default)]
pub struct ChangeSchemaDescriptionBuilder {
    authority: Option<solana_program::pubkey::Pubkey>,
    credential: Option<solana_program::pubkey::Pubkey>,
    schema: Option<solana_program::pubkey::Pubkey>,
    system_program: Option<solana_program::pubkey::Pubkey>,
    description: Option<String>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl ChangeSchemaDescriptionBuilder {
    pub fn new() -> Self {
        Self::default()
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
    /// Credential the Schema is associated with
    #[inline(always)]
    pub fn schema(&mut self, schema: solana_program::pubkey::Pubkey) -> &mut Self {
        self.schema = Some(schema);
        self
    }
    /// `[optional account, default to '11111111111111111111111111111111']`
    #[inline(always)]
    pub fn system_program(&mut self, system_program: solana_program::pubkey::Pubkey) -> &mut Self {
        self.system_program = Some(system_program);
        self
    }
    #[inline(always)]
    pub fn description(&mut self, description: String) -> &mut Self {
        self.description = Some(description);
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
        let accounts = ChangeSchemaDescription {
            authority: self.authority.expect("authority is not set"),
            credential: self.credential.expect("credential is not set"),
            schema: self.schema.expect("schema is not set"),
            system_program: self
                .system_program
                .unwrap_or(solana_program::pubkey!("11111111111111111111111111111111")),
        };
        let args = ChangeSchemaDescriptionInstructionArgs {
            description: self.description.clone().expect("description is not set"),
        };

        accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
    }
}

/// `change_schema_description` CPI accounts.
pub struct ChangeSchemaDescriptionCpiAccounts<'a, 'b> {
    pub authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Credential the Schema is associated with
    pub credential: &'b solana_program::account_info::AccountInfo<'a>,
    /// Credential the Schema is associated with
    pub schema: &'b solana_program::account_info::AccountInfo<'a>,

    pub system_program: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `change_schema_description` CPI instruction.
pub struct ChangeSchemaDescriptionCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,

    pub authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Credential the Schema is associated with
    pub credential: &'b solana_program::account_info::AccountInfo<'a>,
    /// Credential the Schema is associated with
    pub schema: &'b solana_program::account_info::AccountInfo<'a>,

    pub system_program: &'b solana_program::account_info::AccountInfo<'a>,
    /// The arguments for the instruction.
    pub __args: ChangeSchemaDescriptionInstructionArgs,
}

impl<'a, 'b> ChangeSchemaDescriptionCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: ChangeSchemaDescriptionCpiAccounts<'a, 'b>,
        args: ChangeSchemaDescriptionInstructionArgs,
    ) -> Self {
        Self {
            __program: program,
            authority: accounts.authority,
            credential: accounts.credential,
            schema: accounts.schema,
            system_program: accounts.system_program,
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
        let mut accounts = Vec::with_capacity(4 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.authority.key,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.credential.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.schema.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.system_program.key,
            false,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let mut data = borsh::to_vec(&ChangeSchemaDescriptionInstructionData::new()).unwrap();
        let mut args = borsh::to_vec(&self.__args).unwrap();
        data.append(&mut args);

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::SOLANA_ATTESTATION_SERVICE_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(5 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.authority.clone());
        account_infos.push(self.credential.clone());
        account_infos.push(self.schema.clone());
        account_infos.push(self.system_program.clone());
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

/// Instruction builder for `ChangeSchemaDescription` via CPI.
///
/// ### Accounts:
///
///   0. `[signer]` authority
///   1. `[]` credential
///   2. `[writable]` schema
///   3. `[]` system_program
#[derive(Clone, Debug)]
pub struct ChangeSchemaDescriptionCpiBuilder<'a, 'b> {
    instruction: Box<ChangeSchemaDescriptionCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> ChangeSchemaDescriptionCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(ChangeSchemaDescriptionCpiBuilderInstruction {
            __program: program,
            authority: None,
            credential: None,
            schema: None,
            system_program: None,
            description: None,
            __remaining_accounts: Vec::new(),
        });
        Self { instruction }
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
    /// Credential the Schema is associated with
    #[inline(always)]
    pub fn schema(
        &mut self,
        schema: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.schema = Some(schema);
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
    pub fn description(&mut self, description: String) -> &mut Self {
        self.instruction.description = Some(description);
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
        let args = ChangeSchemaDescriptionInstructionArgs {
            description: self
                .instruction
                .description
                .clone()
                .expect("description is not set"),
        };
        let instruction = ChangeSchemaDescriptionCpi {
            __program: self.instruction.__program,

            authority: self.instruction.authority.expect("authority is not set"),

            credential: self.instruction.credential.expect("credential is not set"),

            schema: self.instruction.schema.expect("schema is not set"),

            system_program: self
                .instruction
                .system_program
                .expect("system_program is not set"),
            __args: args,
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct ChangeSchemaDescriptionCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    authority: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    credential: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    schema: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    system_program: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    description: Option<String>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}
