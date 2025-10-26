use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Attribute, Data, Generics, Ident};

mod decode;
mod flag;
mod len;

pub fn compute_zext_zbuf(ident: &Ident, generics: &Generics, data: &Data) -> TokenStream {
    let (_, ty_generics, _) = generics.split_for_impl();
    let ty_generics = if ty_generics.to_token_stream().is_empty() {
        quote::quote! {}
    } else {
        quote::quote! { <'a> }
    };

    let flag_needed = flag_needed(data);
    let len_body = len::len_body(data, flag_needed);

    let expanded = quote::quote! {
        type Decoded<'a> = #ident #ty_generics;

        const LEN: fn(&Self) -> usize = |x| {
            #len_body
        };


        const ENCODE: fn(&mut crate::zbuf::ZBufWriter<'_>, &Self) -> crate::result::ZResult<(), crate::protocol::ZCodecError> = |w, x| {
            Err(crate::protocol::ZCodecError::CouldNotWrite)
        };
        const DECODE: for<'a> fn(&mut crate::zbuf::ZBufReader<'a>, usize) -> crate::result::ZResult<Self::Decoded<'a>, crate::protocol::ZCodecError> = |r, _| {
            Err(crate::protocol::ZCodecError::CouldNotRead)
        };
    };

    expanded.into()
}

fn flag_needed(data: &Data) -> bool {
    let fields = match data {
        Data::Struct(s) => &s.fields,
        _ => unreachable!(),
    };

    let iter = match fields {
        syn::Fields::Named(fields_named) => fields_named.named.iter().collect::<Vec<_>>(),
        syn::Fields::Unnamed(fields_unnamed) => fields_unnamed.unnamed.iter().collect::<Vec<_>>(),
        syn::Fields::Unit => unreachable!(),
    };

    let mut total_bits = 0u8;

    for field in iter.iter() {
        for attr in &field.attrs {
            if attr.path().is_ident("zbuf")
                || attr.path().is_ident("str")
                || attr.path().is_ident("zid")
            {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("flag") {
                        let content;
                        syn::parenthesized!(content in meta.input);

                        let value: syn::LitInt = content.parse()?;
                        let flag_size = value.base10_parse::<u8>()?;
                        total_bits += flag_size;
                    }

                    Ok(())
                })
                .unwrap();
            }
        }
    }

    if total_bits > 0 && total_bits <= 8 {
        true
    } else if total_bits > 8 {
        panic!("Total flag size exceeds 1 byte (8 bits)");
    } else {
        false
    }
}

fn composite_ident(attr: &Attribute) -> Option<Ident> {
    let mut ident_opt = None;

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("composite") {
            let content;
            syn::parenthesized!(content in meta.input);

            let value: syn::Ident = content.parse()?;
            ident_opt = Some(value);
        }

        Ok(())
    })
    .unwrap();

    ident_opt
}
