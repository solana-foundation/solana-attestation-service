use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// A convenient way to get the serialized representation
/// of a struct for Solana Attestation Service. Adds a
/// `get_serialized_representation` function to the struct.
#[proc_macro_derive(SchemaStructSerialize)]
pub fn schema_struct_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident; // Extract struct name

    let fields = match input.data {
        syn::Data::Struct(ref s) => &s.fields,
        _ => panic!("SchemaStructSerialize only supports structs"),
    };

    let field_bytes = fields.iter().map(|f| {
        let ty = &f.ty;
        if let syn::Type::Path(type_path) = ty {
            let type_ident = type_path.path.segments.last().unwrap().ident.to_string();
            match type_ident.as_str() {
                "u8" => quote! { 0x01 },
                "u16" => quote! { 0x02 },
                "u32" => quote! { 0x03 },
                "u64" => quote! { 0x04 },
                "i8" => quote! { 0x05 },
                "i16" => quote! { 0x06 },
                "i32" => quote! { 0x07 },
                "i64" => quote! { 0x08 },
                "bool" => quote! { 0x09 },
                "String" => quote! { 0x0A },
                _ => panic!("Unsupported type in struct"),
            }
        } else {
            panic!("Unsupported type format");
        }
    });

    let output = quote! {
        impl #struct_name {
            pub fn get_serialized_representation() -> Vec<u8> {
                vec![ #(#field_bytes),* ]
            }
        }
    };

    output.into()
}
