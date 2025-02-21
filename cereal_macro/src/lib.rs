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
                "u8" => quote! { 0 },
                "u16" => quote! { 1 },
                "u32" => quote! { 2 },
                "u64" => quote! { 3 },
                "u128" => quote! { 4 },
                "i8" => quote! { 5 },
                "i16" => quote! { 6 },
                "i32" => quote! { 7 },
                "i64" => quote! { 8 },
                "i128" => quote! { 9 },
                "bool" => quote! { 10 },
                "char" => quote! { 11 },
                "String" => quote! { 12 },
                "Vec<u8>" => quote! { 13 },
                "Vec<u16>" => quote! { 14 },
                "Vec<u32>" => quote! { 15 },
                "Vec<u64>" => quote! { 16 },
                "Vec<u128>" => quote! { 17 },
                "Vec<i8>" => quote! { 18 },
                "Vec<i16>" => quote! { 19 },
                "Vec<i32>" => quote! { 20 },
                "Vec<i64>" => quote! { 21 },
                "Vec<i128>" => quote! { 22 },
                "Vec<bool>" => quote! { 23 },
                "Vec<char>" => quote! { 24 },
                "Vec<String>" => quote! { 25 },
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
