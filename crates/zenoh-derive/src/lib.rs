pub(crate) mod codec;

#[proc_macro_derive(ZStruct, attributes(zenoh))]
pub fn derive_zstruct(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    codec::r#struct::derive_zstruct(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(ZExt, attributes(zenoh))]
pub fn derive_zext(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    codec::ext::derive_zext(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
