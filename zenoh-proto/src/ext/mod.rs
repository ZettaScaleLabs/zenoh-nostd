use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Data, DeriveInput, TypeGenerics};

use crate::ext::parse::Extension;

mod parse;

mod body;

pub fn derive_zext(input: DeriveInput) -> TokenStream {
    let ident = &input.ident;
    let data = &input.data;
    let (_, ty_generics, _) = input.generics.split_for_impl();
    let (ty_generics_induced, ty_generics_explicit) = compute_generic_zext(ty_generics);

    let (kind, len, encode, decode) = compute_zext(data);

    let expanded = quote::quote! {
        impl crate::protocol::ext::ZExt for #ident #ty_generics_induced {
            const KIND: crate::protocol::ext::ZExtKind = #kind;

            type Decoded<'a> = #ident #ty_generics_explicit;

            const LEN: fn(&Self) -> usize = |x| {
                let _ = &x;

                #len
            };

            const ENCODE: fn(&mut crate::zbuf::ZBufWriter<'_>, &Self) -> crate::result::ZResult<(), crate::protocol::ZCodecError> = |w, x| {
                let _ = (&w, &x);

                #encode
            };

            const DECODE: for<'a> fn(&mut crate::zbuf::ZBufReader<'a>, usize) -> crate::result::ZResult<Self::Decoded<'a>, crate::protocol::ZCodecError> = |r, l| {
                let _ = (&r, &l);

                use crate::zbuf::BufReaderExt;
                let _start = r.remaining();

                #decode
            };
        }
    };

    expanded.into()
}

fn compute_generic_zext(ty_generics: TypeGenerics) -> (TokenStream, TokenStream) {
    let ty_generics_induced = if ty_generics.to_token_stream().is_empty() {
        quote::quote! {}
    } else {
        quote::quote! { <'_> }
    };

    let ty_generics_explicit = if ty_generics.to_token_stream().is_empty() {
        quote::quote! {}
    } else {
        quote::quote! { <'a> }
    };

    (ty_generics_induced, ty_generics_explicit)
}

pub fn compute_zext(data: &Data) -> (TokenStream, TokenStream, TokenStream, TokenStream) {
    let (ext, named) = Extension::from_data(data);

    let kind = match &ext {
        Extension::Unit => quote::quote! { crate::protocol::ext::ZExtKind::Unit },
        Extension::U64(_) => quote::quote! { crate::protocol::ext::ZExtKind::U64 },
        Extension::ZBuf(_) => quote::quote! { crate::protocol::ext::ZExtKind::ZBuf },
    };

    let (len, encode, decode) = body::compute_body(ext, named);

    (kind, len, encode, decode)
}
