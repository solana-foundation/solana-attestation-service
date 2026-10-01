use borsh::BorshDeserialize;
use helpers::{attestation_pda, expiry, send, setup, test_data, TestContext, TestFixtures};
use solana_address::{address, Address};
use solana_attestation_service_client::{
    accounts::Attestation,
    instructions::{CloseTokenizedAttestationBuilder, CreateTokenizedAttestationBuilder, TokenizeSchemaBuilder},
    programs::SOLANA_ATTESTATION_SERVICE_ID,
};
use solana_instruction_error::InstructionError;
use solana_keypair::Keypair;
use solana_program_option::COption;
use solana_program_pack::Pack;
use solana_signer::Signer;
use solana_transaction::Instruction;
use solana_transaction_error::TransactionError;
use spl_token_2022_interface::{
    error::TokenError,
    extension::{
        group_member_pointer::GroupMemberPointer, group_pointer::GroupPointer, metadata_pointer::MetadataPointer,
        mint_close_authority::MintCloseAuthority, non_transferable::NonTransferable,
        permanent_delegate::PermanentDelegate, BaseStateWithExtensions, ExtensionType, StateWithExtensions,
    },
    instruction::{burn_checked, close_account},
    state::{Account, Mint},
    ID as TOKEN_2022_PROGRAM_ID,
};
use spl_token_group_interface::state::{TokenGroup, TokenGroupMember};
use spl_token_metadata_interface::state::TokenMetadata;

mod helpers;

const ATA_PROGRAM_ID: Address = address!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
const NAME: &str = "Test Asset";
const URI: &str = "https://x.com";
const SYMBOL: &str = "VAT";
const MINT_ACCOUNT_SPACE: u16 = 686;

struct TokenizationFixtures {
    ctx: TestContext,
    credential: Address,
    schema: Address,
    authority: Keypair,
    schema_mint_pda: Address,
    sas_pda: Address,
    attestation_pda: Address,
    attestation_mint_pda: Address,
    recipient: Address,
    recipient_keypair: Keypair,
    recipient_token_account: Address,
    nonce: Address,
    expiry: i64,
}

fn setup_tokenization() -> TokenizationFixtures {
    let TestFixtures { ctx, credential, schema, authority } = setup();

    let (sas_pda, _bump) = Address::find_program_address(&[b"sas"], &SOLANA_ATTESTATION_SERVICE_ID);
    let (schema_mint_pda, _bump) =
        Address::find_program_address(&[b"schemaMint", &schema.to_bytes()], &SOLANA_ATTESTATION_SERVICE_ID);

    let nonce = Address::new_unique();
    let attestation_pda = attestation_pda(&credential, &schema, &nonce);
    let (attestation_mint_pda, _bump) = Address::find_program_address(
        &[b"attestationMint", &attestation_pda.to_bytes()],
        &SOLANA_ATTESTATION_SERVICE_ID,
    );

    let recipient_keypair = Keypair::new();
    let recipient = recipient_keypair.pubkey();
    let (recipient_token_account, _bump) = Address::find_program_address(
        &[recipient.as_ref(), TOKEN_2022_PROGRAM_ID.as_ref(), attestation_mint_pda.as_ref()],
        &ATA_PROGRAM_ID,
    );

    let expiry = expiry(&ctx);
    TokenizationFixtures {
        ctx,
        credential,
        schema,
        authority,
        sas_pda,
        schema_mint_pda,
        attestation_pda,
        attestation_mint_pda,
        recipient,
        recipient_keypair,
        recipient_token_account,
        nonce,
        expiry,
    }
}

fn tokenize_schema_ix(f: &TokenizationFixtures, max_size: u64) -> Instruction {
    TokenizeSchemaBuilder::new()
        .payer(f.ctx.payer.pubkey())
        .authority(f.authority.pubkey())
        .credential(f.credential)
        .schema(f.schema)
        .mint(f.schema_mint_pda)
        .sas_pda(f.sas_pda)
        .max_size(max_size)
        .instruction()
}

fn create_tokenized_attestation_ix(f: &TokenizationFixtures) -> Instruction {
    CreateTokenizedAttestationBuilder::new()
        .payer(f.ctx.payer.pubkey())
        .authority(f.authority.pubkey())
        .credential(f.credential)
        .schema(f.schema)
        .attestation(f.attestation_pda)
        .schema_mint(f.schema_mint_pda)
        .attestation_mint(f.attestation_mint_pda)
        .sas_pda(f.sas_pda)
        .recipient_token_account(f.recipient_token_account)
        .recipient(f.recipient)
        .data(test_data())
        .expiry(f.expiry)
        .nonce(f.nonce)
        .name(NAME.to_string())
        .uri(URI.to_string())
        .symbol(SYMBOL.to_string())
        .mint_account_space(MINT_ACCOUNT_SPACE)
        .instruction()
}

fn close_tokenized_attestation_ix(f: &TokenizationFixtures) -> Instruction {
    CloseTokenizedAttestationBuilder::new()
        .payer(f.ctx.payer.pubkey())
        .authority(f.authority.pubkey())
        .credential(f.credential)
        .attestation(f.attestation_pda)
        .attestation_mint(f.attestation_mint_pda)
        .sas_pda(f.sas_pda)
        .attestation_token_account(f.recipient_token_account)
        .instruction()
}

fn send_ixs(f: &mut TokenizationFixtures, ixs: &[Instruction]) {
    send(&mut f.ctx, ixs, &[&f.authority]).unwrap();
}

#[test]
fn tokenize_schema_success() {
    let mut f = setup_tokenization();
    let TokenizationFixtures { sas_pda, schema_mint_pda, .. } = f;

    let max_size = 100;
    let ix = tokenize_schema_ix(&f, max_size);
    send_ixs(&mut f, &[ix]);

    let mint_account = f.ctx.svm.get_account(&schema_mint_pda).unwrap();

    let expected_acc_size =
        ExtensionType::try_calculate_account_len::<Mint>(&[ExtensionType::GroupPointer, ExtensionType::TokenGroup])
            .unwrap();
    assert_eq!(mint_account.data.len(), expected_acc_size);
    assert_eq!(mint_account.owner, TOKEN_2022_PROGRAM_ID);

    let mint_state = StateWithExtensions::<Mint>::unpack(&mint_account.data).unwrap();
    assert!(mint_state.base.is_initialized);
    assert_eq!(mint_state.base.decimals, 0);
    assert_eq!(mint_state.base.supply, 0);
    assert_eq!(mint_state.base.mint_authority, COption::Some(sas_pda));
    assert_eq!(mint_state.base.freeze_authority, COption::Some(sas_pda));

    let group_pointer = mint_state.get_extension::<GroupPointer>().unwrap();
    assert_eq!(group_pointer.authority.get(), Some(sas_pda));
    assert_eq!(group_pointer.group_address.get(), Some(sas_pda));

    let token_group = mint_state.get_extension::<TokenGroup>().unwrap();
    assert_eq!(token_group.update_authority.get(), Some(sas_pda));
    assert_eq!(token_group.mint, schema_mint_pda);
    assert_eq!(u64::from(token_group.size), 0);
    assert_eq!(u64::from(token_group.max_size), max_size);
}

#[test]
fn create_tokenized_attestation_success() {
    let mut f = setup_tokenization();
    let TokenizationFixtures {
        credential,
        schema,
        sas_pda,
        schema_mint_pda,
        attestation_pda,
        attestation_mint_pda,
        recipient_token_account,
        nonce,
        expiry,
        ..
    } = f;

    let ix = tokenize_schema_ix(&f, 100);
    send_ixs(&mut f, &[ix]);
    let ix = create_tokenized_attestation_ix(&f);
    send_ixs(&mut f, &[ix]);

    let attestation_account = f.ctx.svm.get_account(&attestation_pda).unwrap();
    let attestation = Attestation::try_from_slice(&attestation_account.data).unwrap();
    assert_eq!(attestation.data, test_data());
    assert_eq!(attestation.credential, credential);
    assert_eq!(attestation.expiry, expiry);
    assert_eq!(attestation.schema, schema);
    assert_eq!(attestation.signer, f.authority.pubkey());
    assert_eq!(attestation.nonce, nonce);
    assert_eq!(attestation.token_account, recipient_token_account);

    let attestation_mint_account = f.ctx.svm.get_account(&attestation_mint_pda).unwrap();

    let expected_lamports = f.ctx.svm.minimum_balance_for_rent_exemption(MINT_ACCOUNT_SPACE.into());
    assert_eq!(attestation_mint_account.lamports, expected_lamports);
    assert!(attestation_mint_account.data.len() <= MINT_ACCOUNT_SPACE.into());

    assert_eq!(attestation_mint_account.owner, TOKEN_2022_PROGRAM_ID);

    let mint_state = StateWithExtensions::<Mint>::unpack(&attestation_mint_account.data).unwrap();
    assert!(mint_state.base.is_initialized);
    assert_eq!(mint_state.base.decimals, 0);
    assert_eq!(mint_state.base.supply, 1);
    assert_eq!(mint_state.base.mint_authority, COption::Some(sas_pda));
    assert_eq!(mint_state.base.freeze_authority, COption::Some(sas_pda));

    let group_member_pointer = mint_state.get_extension::<GroupMemberPointer>().unwrap();
    assert_eq!(group_member_pointer.authority.get(), Some(sas_pda));
    assert_eq!(group_member_pointer.member_address.get(), Some(attestation_mint_pda));

    mint_state.get_extension::<NonTransferable>().unwrap();

    let token_group_member = mint_state.get_extension::<TokenGroupMember>().unwrap();
    assert_eq!(token_group_member.mint, attestation_mint_pda);
    assert_eq!(token_group_member.group, schema_mint_pda);
    assert_eq!(u64::from(token_group_member.member_number), 1);

    let permanent_delegate = mint_state.get_extension::<PermanentDelegate>().unwrap();
    assert_eq!(permanent_delegate.delegate.get(), Some(sas_pda));

    let close_authority = mint_state.get_extension::<MintCloseAuthority>().unwrap();
    assert_eq!(close_authority.close_authority.get(), Some(sas_pda));

    let metadata_pointer = mint_state.get_extension::<MetadataPointer>().unwrap();
    assert_eq!(metadata_pointer.authority.get(), Some(sas_pda));
    assert_eq!(metadata_pointer.metadata_address.get(), Some(attestation_mint_pda));

    let token_metadata = &mint_state.get_variable_len_extension::<TokenMetadata>().unwrap();
    assert_eq!(token_metadata.update_authority.get(), Some(sas_pda));
    assert_eq!(token_metadata.mint, attestation_mint_pda);
    assert_eq!(token_metadata.name, NAME);
    assert_eq!(token_metadata.uri, URI);
    assert_eq!(token_metadata.symbol, SYMBOL);
    assert_eq!(token_metadata.additional_metadata.len(), 2);
    assert_eq!(token_metadata.additional_metadata[0].0, "attestation");
    assert_eq!(token_metadata.additional_metadata[0].1, attestation_pda.to_string());
    assert_eq!(token_metadata.additional_metadata[1].0, "schema");
    assert_eq!(token_metadata.additional_metadata[1].1, schema.to_string());

    let recipient_token_account_data = f.ctx.svm.get_account(&recipient_token_account).unwrap();
    let token_account = Account::unpack(&recipient_token_account_data.data[..Account::LEN]).unwrap();
    assert_eq!(token_account.mint, attestation_mint_pda);
    assert_eq!(token_account.amount, 1);
}

#[test]
fn close_tokenized_attestation_success() {
    let mut f = setup_tokenization();
    let TokenizationFixtures { attestation_pda, attestation_mint_pda, recipient_token_account, .. } = f;

    let ixs = [tokenize_schema_ix(&f, 100), create_tokenized_attestation_ix(&f)];
    send_ixs(&mut f, &ixs);
    let ix = close_tokenized_attestation_ix(&f);
    send_ixs(&mut f, &[ix]);

    assert!(f.ctx.svm.get_account(&attestation_pda).is_none());
    assert!(f.ctx.svm.get_account(&attestation_mint_pda).is_none());

    let recipient_token_account_data = f.ctx.svm.get_account(&recipient_token_account).unwrap();
    let token_account = Account::unpack(&recipient_token_account_data.data[..Account::LEN]).unwrap();
    assert_eq!(token_account.mint, attestation_mint_pda);
    assert_eq!(token_account.amount, 0);
}

fn holder_burn_ix(f: &TokenizationFixtures) -> Instruction {
    burn_checked(&TOKEN_2022_PROGRAM_ID, &f.recipient_token_account, &f.attestation_mint_pda, &f.recipient, &[], 1, 0)
        .unwrap()
}

fn holder_close_token_account_ix(f: &TokenizationFixtures) -> Instruction {
    close_account(&TOKEN_2022_PROGRAM_ID, &f.recipient_token_account, &f.recipient, &f.recipient, &[]).unwrap()
}

fn assert_closed_after_holder_actions(holder_ixs: fn(&TokenizationFixtures) -> Vec<Instruction>) {
    let mut f = setup_tokenization();
    let TokenizationFixtures { attestation_pda, attestation_mint_pda, .. } = f;

    let ixs = [tokenize_schema_ix(&f, 100), create_tokenized_attestation_ix(&f)];
    send_ixs(&mut f, &ixs);
    let ixs = holder_ixs(&f);
    let recipient_keypair = f.recipient_keypair.insecure_clone();
    send(&mut f.ctx, &ixs, &[&recipient_keypair]).unwrap();
    let ix = close_tokenized_attestation_ix(&f);
    send_ixs(&mut f, &[ix]);

    assert!(f.ctx.svm.get_account(&attestation_pda).is_none());
    assert!(f.ctx.svm.get_account(&attestation_mint_pda).is_none());
}

#[test]
fn close_tokenized_attestation_after_holder_burn() {
    assert_closed_after_holder_actions(|f| vec![holder_burn_ix(f)]);
}

#[test]
fn close_tokenized_attestation_after_holder_burn_and_close() {
    assert_closed_after_holder_actions(|f| vec![holder_burn_ix(f), holder_close_token_account_ix(f)]);
}

#[test]
fn recreate_tokenized_attestation_after_close() {
    let mut f = setup_tokenization();
    let TokenizationFixtures {
        schema_mint_pda, attestation_pda, attestation_mint_pda, recipient_token_account, ..
    } = f;

    let ixs = [tokenize_schema_ix(&f, 100), create_tokenized_attestation_ix(&f)];
    send_ixs(&mut f, &ixs);
    let ix = close_tokenized_attestation_ix(&f);
    send_ixs(&mut f, &[ix]);
    let ix = create_tokenized_attestation_ix(&f);
    send_ixs(&mut f, &[ix]);

    let attestation_account = f.ctx.svm.get_account(&attestation_pda).unwrap();
    let attestation = Attestation::try_from_slice(&attestation_account.data).unwrap();
    assert_eq!(attestation.token_account, recipient_token_account);

    let attestation_mint_account = f.ctx.svm.get_account(&attestation_mint_pda).unwrap();
    let mint_state = StateWithExtensions::<Mint>::unpack(&attestation_mint_account.data).unwrap();
    assert_eq!(mint_state.base.supply, 1);

    let token_group_member = mint_state.get_extension::<TokenGroupMember>().unwrap();
    assert_eq!(token_group_member.mint, attestation_mint_pda);
    assert_eq!(token_group_member.group, schema_mint_pda);
    // TokenGroup size cannot decrement, so the re-created member is number 2.
    assert_eq!(u64::from(token_group_member.member_number), 2);

    let recipient_token_account_data = f.ctx.svm.get_account(&recipient_token_account).unwrap();
    let token_account = Account::unpack(&recipient_token_account_data.data[..Account::LEN]).unwrap();
    assert_eq!(token_account.mint, attestation_mint_pda);
    assert_eq!(token_account.amount, 1);
}

#[test]
fn create_tokenized_attestation_fail_non_utf8_name() {
    let mut f = setup_tokenization();
    let tokenize_ix = tokenize_schema_ix(&f, 100);
    send_ixs(&mut f, &[tokenize_ix]);

    let mut ix = create_tokenized_attestation_ix(&f);
    let name_start = ix.data.windows(NAME.len()).position(|w| w == NAME.as_bytes()).unwrap();
    ix.data[name_start..name_start + NAME.len()].fill(0xff);

    let tx_err = send(&mut f.ctx, &[ix], &[&f.authority]).expect_err("should error");
    assert_eq!(
        tx_err,
        TransactionError::InstructionError(0, InstructionError::Custom(TokenError::InvalidInstruction as u32))
    )
}
