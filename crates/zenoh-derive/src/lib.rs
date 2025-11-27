pub(crate) mod codec;
pub(crate) mod zerror;

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

#[proc_macro_derive(ZEnum)]
pub fn derive_zenum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    codec::r#enum::derive_zenum(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro]
pub fn declare_zerror(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as zerror::model::DeclaredErrors);

    zerror::declare_zerror(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
