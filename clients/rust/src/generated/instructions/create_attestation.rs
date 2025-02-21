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
pub struct CreateAttestation {
    pub payer: solana_program::pubkey::Pubkey,
    /// Authorized signer of the Schema's Credential
    pub authority: solana_program::pubkey::Pubkey,
    /// Schema the Attestation is associated with
    pub schema: solana_program::pubkey::Pubkey,
    /// Credential the Schema is associated with
    pub credential: solana_program::pubkey::Pubkey,

    pub attestation: solana_program::pubkey::Pubkey,
}

impl CreateAttestation {
    pub fn instruction(
        &self,
        args: CreateAttestationInstructionArgs,
    ) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(args, &[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        args: CreateAttestationInstructionArgs,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(5 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.payer, true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.authority,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.schema,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.credential,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.attestation,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let mut data = borsh::to_vec(&CreateAttestationInstructionData::new()).unwrap();
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
pub struct CreateAttestationInstructionData {
    discriminator: u8,
}

impl CreateAttestationInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 4 }
    }
}

impl Default for CreateAttestationInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateAttestationInstructionArgs {
    pub data: Vec<u8>,
    pub expiry: i64,
}

/// Instruction builder for `CreateAttestation`.
///
/// ### Accounts:
///
///   0. `[writable, signer]` payer
///   1. `[signer]` authority
///   2. `[]` schema
///   3. `[]` credential
///   4. `[writable]` attestation
#[derive(Clone, Debug, Default)]
pub struct CreateAttestationBuilder {
    payer: Option<solana_program::pubkey::Pubkey>,
    authority: Option<solana_program::pubkey::Pubkey>,
    schema: Option<solana_program::pubkey::Pubkey>,
    credential: Option<solana_program::pubkey::Pubkey>,
    attestation: Option<solana_program::pubkey::Pubkey>,
    data: Option<Vec<u8>>,
    expiry: Option<i64>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl CreateAttestationBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    #[inline(always)]
    pub fn payer(&mut self, payer: solana_program::pubkey::Pubkey) -> &mut Self {
        self.payer = Some(payer);
        self
    }
    /// Authorized signer of the Schema's Credential
    #[inline(always)]
    pub fn authority(&mut self, authority: solana_program::pubkey::Pubkey) -> &mut Self {
        self.authority = Some(authority);
        self
    }
    /// Schema the Attestation is associated with
    #[inline(always)]
    pub fn schema(&mut self, schema: solana_program::pubkey::Pubkey) -> &mut Self {
        self.schema = Some(schema);
        self
    }
    /// Credential the Schema is associated with
    #[inline(always)]
    pub fn credential(&mut self, credential: solana_program::pubkey::Pubkey) -> &mut Self {
        self.credential = Some(credential);
        self
    }
    #[inline(always)]
    pub fn attestation(&mut self, attestation: solana_program::pubkey::Pubkey) -> &mut Self {
        self.attestation = Some(attestation);
        self
    }
    #[inline(always)]
    pub fn data(&mut self, data: Vec<u8>) -> &mut Self {
        self.data = Some(data);
        self
    }
    #[inline(always)]
    pub fn expiry(&mut self, expiry: i64) -> &mut Self {
        self.expiry = Some(expiry);
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
        let accounts = CreateAttestation {
            payer: self.payer.expect("payer is not set"),
            authority: self.authority.expect("authority is not set"),
            schema: self.schema.expect("schema is not set"),
            credential: self.credential.expect("credential is not set"),
            attestation: self.attestation.expect("attestation is not set"),
        };
        let args = CreateAttestationInstructionArgs {
            data: self.data.clone().expect("data is not set"),
            expiry: self.expiry.clone().expect("expiry is not set"),
        };

        accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
    }
}

/// `create_attestation` CPI accounts.
pub struct CreateAttestationCpiAccounts<'a, 'b> {
    pub payer: &'b solana_program::account_info::AccountInfo<'a>,
    /// Authorized signer of the Schema's Credential
    pub authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Schema the Attestation is associated with
    pub schema: &'b solana_program::account_info::AccountInfo<'a>,
    /// Credential the Schema is associated with
    pub credential: &'b solana_program::account_info::AccountInfo<'a>,

    pub attestation: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `create_attestation` CPI instruction.
pub struct CreateAttestationCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,

    pub payer: &'b solana_program::account_info::AccountInfo<'a>,
    /// Authorized signer of the Schema's Credential
    pub authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Schema the Attestation is associated with
    pub schema: &'b solana_program::account_info::AccountInfo<'a>,
    /// Credential the Schema is associated with
    pub credential: &'b solana_program::account_info::AccountInfo<'a>,

    pub attestation: &'b solana_program::account_info::AccountInfo<'a>,
    /// The arguments for the instruction.
    pub __args: CreateAttestationInstructionArgs,
}

impl<'a, 'b> CreateAttestationCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: CreateAttestationCpiAccounts<'a, 'b>,
        args: CreateAttestationInstructionArgs,
    ) -> Self {
        Self {
            __program: program,
            payer: accounts.payer,
            authority: accounts.authority,
            schema: accounts.schema,
            credential: accounts.credential,
            attestation: accounts.attestation,
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
        let mut accounts = Vec::with_capacity(5 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.payer.key,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.authority.key,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.schema.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.credential.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.attestation.key,
            false,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let mut data = borsh::to_vec(&CreateAttestationInstructionData::new()).unwrap();
        let mut args = borsh::to_vec(&self.__args).unwrap();
        data.append(&mut args);

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::SOLANA_ATTESTATION_SERVICE_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(6 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.payer.clone());
        account_infos.push(self.authority.clone());
        account_infos.push(self.schema.clone());
        account_infos.push(self.credential.clone());
        account_infos.push(self.attestation.clone());
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

/// Instruction builder for `CreateAttestation` via CPI.
///
/// ### Accounts:
///
///   0. `[writable, signer]` payer
///   1. `[signer]` authority
///   2. `[]` schema
///   3. `[]` credential
///   4. `[writable]` attestation
#[derive(Clone, Debug)]
pub struct CreateAttestationCpiBuilder<'a, 'b> {
    instruction: Box<CreateAttestationCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> CreateAttestationCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(CreateAttestationCpiBuilderInstruction {
            __program: program,
            payer: None,
            authority: None,
            schema: None,
            credential: None,
            attestation: None,
            data: None,
            expiry: None,
            __remaining_accounts: Vec::new(),
        });
        Self { instruction }
    }
    #[inline(always)]
    pub fn payer(&mut self, payer: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.payer = Some(payer);
        self
    }
    /// Authorized signer of the Schema's Credential
    #[inline(always)]
    pub fn authority(
        &mut self,
        authority: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.authority = Some(authority);
        self
    }
    /// Schema the Attestation is associated with
    #[inline(always)]
    pub fn schema(
        &mut self,
        schema: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.schema = Some(schema);
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
    pub fn attestation(
        &mut self,
        attestation: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.attestation = Some(attestation);
        self
    }
    #[inline(always)]
    pub fn data(&mut self, data: Vec<u8>) -> &mut Self {
        self.instruction.data = Some(data);
        self
    }
    #[inline(always)]
    pub fn expiry(&mut self, expiry: i64) -> &mut Self {
        self.instruction.expiry = Some(expiry);
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
        let args = CreateAttestationInstructionArgs {
            data: self.instruction.data.clone().expect("data is not set"),
            expiry: self.instruction.expiry.clone().expect("expiry is not set"),
        };
        let instruction = CreateAttestationCpi {
            __program: self.instruction.__program,

            payer: self.instruction.payer.expect("payer is not set"),

            authority: self.instruction.authority.expect("authority is not set"),

            schema: self.instruction.schema.expect("schema is not set"),

            credential: self.instruction.credential.expect("credential is not set"),

            attestation: self
                .instruction
                .attestation
                .expect("attestation is not set"),
            __args: args,
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct CreateAttestationCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    payer: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    authority: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    schema: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    credential: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    attestation: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    data: Option<Vec<u8>>,
    expiry: Option<i64>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}
