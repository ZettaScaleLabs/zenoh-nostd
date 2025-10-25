use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::DeriveInput;

mod body;
mod kind;

pub fn derive_zext(input: DeriveInput) -> TokenStream {
    let ident = &input.ident;
    let (_, ty_generics, _) = input.generics.split_for_impl();

    let (kind, body) = compute_zext(&input);
    let ty_generics = if ty_generics.to_token_stream().is_empty() {
        quote::quote! {}
    } else {
        quote::quote! { <'_> }
    };

    let expanded = quote::quote! {
        impl crate::protocol::ext::ZExt for #ident #ty_generics {
            const KIND: crate::protocol::ext::ZExtKind = #kind;

            #body
        }

    };

    expanded.into()
}

pub fn compute_zext(input: &DeriveInput) -> (TokenStream, TokenStream) {
    let (token, kind) = kind::infer_kind(&input.data);
    let body = body::infer_body(&kind, input);

    (token, body)
}
