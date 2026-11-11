use helpers::{expected_schema, fetch_schema, setup};

mod helpers;

#[test]
fn create_schema_success() {
    let f = setup();
    assert_eq!(fetch_schema(&f.ctx, &f.schema), expected_schema(f.credential));
}
