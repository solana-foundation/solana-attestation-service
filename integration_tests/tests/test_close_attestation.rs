use borsh::BorshDeserialize;
use helpers::{create_attestation, expiry, setup, test_data};
use solana_address::Address;
use solana_attestation_service_client::{
    instructions::CloseAttestationBuilder, programs::SOLANA_ATTESTATION_SERVICE_ID, types::CloseAttestationEvent,
};
use solana_signer::Signer;
use solana_transaction::Transaction;

mod helpers;

const EVENT_IX_TAG: u64 = 0x1d9acb512ea545e4;
const EVENT_IX_TAG_LE: &[u8] = EVENT_IX_TAG.to_le_bytes().as_slice();

#[test]
fn close_attestation_success() {
    let mut f = setup();
    let expiry = expiry(&f.ctx);
    let attestation_pda = create_attestation(&mut f, test_data(), expiry, Address::new_unique()).unwrap();
    let ctx = &mut f.ctx;

    let initial_payer_lamports = ctx.svm.get_account(&ctx.payer.pubkey()).unwrap().lamports;
    let pda_lamports = ctx.svm.get_account(&attestation_pda).unwrap().lamports;

    let close_attestation_ix = CloseAttestationBuilder::new()
        .payer(ctx.payer.pubkey())
        .authority(f.authority.pubkey())
        .credential(f.credential)
        .attestation(attestation_pda)
        .instruction();
    let close_tx = Transaction::new_signed_with_payer(
        &[close_attestation_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &f.authority],
        ctx.svm.latest_blockhash(),
    );

    let simulate_res = ctx.svm.simulate_transaction(close_tx.clone()).unwrap();
    let mut event_found = false;
    for inner_instr in simulate_res.meta.inner_instructions.into_iter().flatten() {
        let program_id = inner_instr.instruction.program_id(&close_tx.message.account_keys);
        let data = inner_instr.instruction.data;
        if program_id.eq(&SOLANA_ATTESTATION_SERVICE_ID) && data.starts_with(EVENT_IX_TAG_LE) {
            let event = CloseAttestationEvent::try_from_slice(&data[8..]).unwrap();
            assert_eq!(event.discriminator, 0);
            assert_eq!(event.schema, f.schema);
            assert_eq!(event.attestation_data, test_data());
            event_found = true;
        }
    }
    assert!(event_found);

    ctx.svm.send_transaction(close_tx).unwrap();

    assert!(ctx.svm.get_account(&attestation_pda).is_none());

    // The payer also pays 10_000 lamports in fees for the two signatures.
    let post_payer_lamports = ctx.svm.get_account(&ctx.payer.pubkey()).unwrap().lamports;
    assert_eq!(initial_payer_lamports + pda_lamports - 10_000, post_payer_lamports)
}
