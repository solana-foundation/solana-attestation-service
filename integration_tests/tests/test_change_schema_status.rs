use helpers::{expected_schema, fetch_schema, send, setup};
use solana_attestation::{accounts::Schema, instructions::ChangeSchemaStatusBuilder};
use solana_signer::Signer;

mod helpers;

#[test]
fn pause_and_unpause_schema_success() {
    let mut f = setup();

    for is_paused in [true, false] {
        let ix = ChangeSchemaStatusBuilder::new()
            .authority(f.authority.pubkey())
            .credential(f.credential)
            .schema(f.schema)
            .is_paused(is_paused)
            .instruction();
        send(&mut f.ctx, &[ix], &[&f.authority]).unwrap();

        assert_eq!(fetch_schema(&f.ctx, &f.schema), Schema { is_paused, ..expected_schema(f.credential) });
    }
}
