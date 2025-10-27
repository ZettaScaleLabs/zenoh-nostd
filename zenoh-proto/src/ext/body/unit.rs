use proc_macro2::TokenStream;

pub fn compute_body_unit() -> (TokenStream, TokenStream, TokenStream) {
    (
        quote::quote! {
            0
        },
        quote::quote! {
            Ok(())
        },
        quote::quote! {
            Ok(Self {})
        },
    )
}
