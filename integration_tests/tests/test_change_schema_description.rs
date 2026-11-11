use helpers::{expected_schema, fetch_schema, send, setup};
use solana_attestation::{accounts::Schema, instructions::ChangeSchemaDescriptionBuilder};
use solana_signer::Signer;

mod helpers;

#[test]
fn change_schema_description_success() {
    let mut f = setup();

    let description = "new test new test new test";
    let ix = ChangeSchemaDescriptionBuilder::new()
        .payer(f.ctx.payer.pubkey())
        .authority(f.authority.pubkey())
        .credential(f.credential)
        .schema(f.schema)
        .description(description.to_string())
        .instruction();
    send(&mut f.ctx, &[ix], &[&f.authority]).unwrap();

    assert_eq!(
        fetch_schema(&f.ctx, &f.schema),
        Schema { description: description.as_bytes().to_vec(), ..expected_schema(f.credential) }
    );
}
