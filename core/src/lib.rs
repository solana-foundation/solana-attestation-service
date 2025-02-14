#[repr(u8)]
pub enum PrimitiveDataTypes {
    U8 = 1,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    BOOL, // 9,
}

#[repr(u8)]
pub enum VariableDataTypes {
    STRING = 10,
    VEC(PrimitiveDataTypes),
}

#[cfg(test)]
mod test {
    use solana_attestation_service_macros::SchemaStructSerialize;

    #[derive(SchemaStructSerialize)]
    struct CustomData {
        _field1: u64,
        _field2: i8,
        _field3: String,
    }

    #[test]
    fn test_serialization() {
        assert_eq!(CustomData::get_serialized_representation(), vec![4, 5, 10]);
    }
}
