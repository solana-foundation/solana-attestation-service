use std::io::{Cursor, Write};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct CreateCredentialArgs {
    pub name: String,
    pub signers: Vec<Pubkey>,
}

/// CreateCredential Instruction for use in Solana SDK.
pub fn create_credential_instruction(
    payer: &Pubkey,
    authority: &Pubkey,
    name: String,
    signers: Vec<Pubkey>,
) -> Instruction {
    let (credential_pda, _bump) = Pubkey::find_program_address(
        &[b"credential", &authority.to_bytes(), name.as_bytes()],
        &Pubkey::from(solana_attestation_service::ID),
    );

    // Manually construct IX data
    // TODO find a better way to do this. Would be nice to use borsh
    // or something on the client side.
    let ix: Vec<u8> = Vec::new();
    let mut ix = Cursor::new(ix);
    ix.write(&[0]).unwrap(); // discriminator for AttestationServiceInstruction::CreateCredential
    let name_len = name.len() as u32;
    ix.write(&name_len.to_le_bytes()).unwrap();
    ix.write(name.as_bytes()).unwrap();
    let signers_len = signers.len() as u32;
    ix.write(&signers_len.to_le_bytes()).unwrap();
    ix.write(
        &signers
            .iter()
            .map(|p| p.to_bytes())
            .flatten()
            .collect::<Vec<u8>>(),
    )
    .unwrap();
    Instruction {
        program_id: solana_attestation_service::ID.into(),
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(credential_pda, false),
            AccountMeta::new_readonly(*authority, true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: ix.into_inner(),
    }
}
