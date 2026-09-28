extern crate alloc;

use alloc::vec::Vec;
use pinocchio::{
    cpi::{invoke_signed, Signer},
    instruction::{InstructionAccount, InstructionView},
    AccountView, Address, ProgramResult,
};
use pinocchio_token_2022::ID as TOKEN_2022_PROGRAM_ID;

use super::to_serialized_vec;

/// Initializes the Token-2022 TokenMetadata extension on a mint.
pub struct InitializeTokenMetadata<'a> {
    pub metadata: &'a AccountView,
    pub update_authority: &'a AccountView,
    pub mint: &'a AccountView,
    pub mint_authority: &'a AccountView,
    pub name: &'a [u8],
    pub symbol: &'a [u8],
    pub uri: &'a [u8],
}

impl InitializeTokenMetadata<'_> {
    const DISCRIMINATOR: [u8; 8] = [210, 225, 30, 162, 88, 184, 77, 141];

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let data: Vec<u8> = [
            Self::DISCRIMINATOR.as_slice(),
            &to_serialized_vec(self.name),
            &to_serialized_vec(self.symbol),
            &to_serialized_vec(self.uri),
        ]
        .concat();

        invoke_signed(
            &InstructionView {
                program_id: &TOKEN_2022_PROGRAM_ID,
                accounts: &[
                    InstructionAccount::writable(self.metadata.address()),
                    InstructionAccount::readonly(self.update_authority.address()),
                    InstructionAccount::readonly(self.mint.address()),
                    InstructionAccount::readonly_signer(self.mint_authority.address()),
                ],
                data: &data,
            },
            &[self.metadata, self.update_authority, self.mint, self.mint_authority],
            signers,
        )
    }
}

/// Sets a custom `Field::Key` entry in the TokenMetadata extension.
pub struct UpdateField<'a> {
    pub metadata: &'a AccountView,
    pub update_authority: &'a AccountView,
    pub key: &'a str,
    pub value: &'a str,
}

impl UpdateField<'_> {
    const DISCRIMINATOR: [u8; 8] = [221, 233, 49, 45, 181, 202, 220, 200];
    const FIELD_KEY_VARIANT: u8 = 3;

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let data: Vec<u8> = [
            Self::DISCRIMINATOR.as_slice(),
            &[Self::FIELD_KEY_VARIANT],
            &to_serialized_vec(self.key.as_bytes()),
            &to_serialized_vec(self.value.as_bytes()),
        ]
        .concat();

        invoke_signed(
            &InstructionView {
                program_id: &TOKEN_2022_PROGRAM_ID,
                accounts: &[
                    InstructionAccount::writable(self.metadata.address()),
                    InstructionAccount::readonly_signer(self.update_authority.address()),
                ],
                data: &data,
            },
            &[self.metadata, self.update_authority],
            signers,
        )
    }
}

/// Initializes the Token-2022 TokenGroup extension on a mint.
pub struct InitializeGroup<'a> {
    pub group: &'a AccountView,
    pub mint: &'a AccountView,
    pub mint_authority: &'a AccountView,
    pub update_authority: &'a Address,
    pub max_size: u64,
}

impl InitializeGroup<'_> {
    const DISCRIMINATOR: [u8; 8] = [121, 113, 108, 39, 54, 51, 0, 4];

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let data: Vec<u8> =
            [Self::DISCRIMINATOR.as_slice(), self.update_authority.as_ref(), &self.max_size.to_le_bytes()].concat();

        invoke_signed(
            &InstructionView {
                program_id: &TOKEN_2022_PROGRAM_ID,
                accounts: &[
                    InstructionAccount::writable(self.group.address()),
                    InstructionAccount::readonly(self.mint.address()),
                    InstructionAccount::readonly_signer(self.mint_authority.address()),
                ],
                data: &data,
            },
            &[self.group, self.mint, self.mint_authority],
            signers,
        )
    }
}

/// Initializes the Token-2022 TokenGroupMember extension on a mint and adds it to `group`.
pub struct InitializeMember<'a> {
    pub member: &'a AccountView,
    pub member_mint: &'a AccountView,
    pub member_mint_authority: &'a AccountView,
    pub group: &'a AccountView,
    pub group_update_authority: &'a AccountView,
}

impl InitializeMember<'_> {
    const DISCRIMINATOR: [u8; 8] = [152, 32, 222, 176, 223, 237, 116, 134];

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        invoke_signed(
            &InstructionView {
                program_id: &TOKEN_2022_PROGRAM_ID,
                accounts: &[
                    InstructionAccount::writable(self.member.address()),
                    InstructionAccount::readonly(self.member_mint.address()),
                    InstructionAccount::readonly_signer(self.member_mint_authority.address()),
                    InstructionAccount::writable(self.group.address()),
                    InstructionAccount::readonly_signer(self.group_update_authority.address()),
                ],
                data: &Self::DISCRIMINATOR,
            },
            &[self.member, self.member_mint, self.member_mint_authority, self.group, self.group_update_authority],
            signers,
        )
    }
}
