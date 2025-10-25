use proc_macro2::TokenStream;
use syn::{Data, DeriveInput};

mod body;
mod kind;

pub fn derive_zext(input: DeriveInput) -> TokenStream {
    let ident = &input.ident;
    let data = &input.data;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let (kind, body) = compute_zext(data);

    let expanded = quote::quote! {
        impl #impl_generics crate::protocol::ext::ZExt for #ident #ty_generics #where_clause {
            const KIND: crate::protocol::ext::ZExtKind = #kind;
        }

        #body
    };

    expanded.into()
}

pub fn compute_zext(data: &Data) -> (TokenStream, TokenStream) {
    let (token, kind) = kind::infer_kind(data);
    let body = body::infer_body(&kind, data);

    (token, body)
}
