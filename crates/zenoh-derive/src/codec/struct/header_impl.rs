use proc_macro2::{Span, TokenStream};
use syn::Ident;

use crate::codec::model::ZenohStruct;

pub fn parse(r#struct: &ZenohStruct) -> syn::Result<TokenStream> {
    if let Some(header) = &r#struct.header {
        let ident = &r#struct.ident;
        let generics = &r#struct.generics;
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let mut shift = 8u8;
        let content = header.expr.value();
        let mut const_defs = Vec::new();
        let mut base_header = Vec::new();

        for part in content.split('|') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            const_defs.push(parse_part(
                part,
                &mut shift,
                &mut base_header,
                header.expr.span(),
            )?);
        }

        let base_header = if base_header.is_empty() {
            quote::quote! { 0u8 }
        } else {
            base_header
                .into_iter()
                .reduce(|acc, expr| {
                    quote::quote! { (#acc) | (#expr) }
                })
                .unwrap()
        };

        if shift != 0 {
            return Err(syn::Error::new(
                header.expr.span(),
                "Header declaration does not use all 8 bits",
            ));
        }

        Ok(quote::quote! {
            impl #impl_generics #ident #ty_generics #where_clause {
                const HEADER_BASE: u8 = #base_header;

                #(#const_defs)*
            }
        })
    } else {
        Ok(quote::quote! {})
    }
}

fn parse_part(
    part: &str,
    shift: &mut u8,
    base_header: &mut Vec<TokenStream>,
    span: Span,
) -> syn::Result<TokenStream> {
    if part == "_" {
        *shift = shift.saturating_sub(1);
        return Ok(quote::quote! {});
    }

    if part == "Z" && *shift != 8 {
        return Err(syn::Error::new(
            span,
            "The special 'Z' placeholder must be the first part in header declaration",
        ));
    }

    let mut split = part.split('=');
    let left = split.next().unwrap();
    let value_opt = split.next();
    let mut left_split = left.split(':');
    let name_str = left_split.next().unwrap();
    let size_opt = left_split.next();
    let name = Ident::new(name_str, proc_macro2::Span::call_site());

    if let Some(size_str) = size_opt {
        let size: u8 = size_str.parse().map_err(|_| {
            syn::Error::new(
                span,
                format!("Invalid size '{}' in header declaration", size_str),
            )
        })?;
        if let Some(value_str) = value_opt {
            let value: u8 = if let Some(stripped) = value_str.strip_prefix("0x") {
                u8::from_str_radix(stripped, 16).map_err(|_| {
                    syn::Error::new(
                        span,
                        format!("Invalid hex value '{}' in header declaration", value_str),
                    )
                })?
            } else {
                value_str.parse().map_err(|_| {
                    syn::Error::new(
                        span,
                        format!("Invalid value '{}' in header declaration", value_str),
                    )
                })?
            };

            let x = syn::LitInt::new(&format!("0b{:b}", (1 << size) - 1), Span::call_site());
            let y = *shift - size;

            let slot = quote::quote! { #x << #y };
            let shifted_value = quote::quote! { #value << #y };
            let value = quote::quote! { #value };

            *shift = shift.checked_sub(size).ok_or_else(|| {
                syn::Error::new(span, "Not enough bits left in header declaration")
            })?;

            base_header.push(shifted_value);

            if name == "_" {
                return Ok(quote::quote! {});
            }

            let name_slot = Ident::new(&format!("HEADER_SLOT_{}", name_str), Span::call_site());
            Ok(quote::quote! {
                pub const #name: u8 = #value;
                const #name_slot: u8 = #slot;
            })
        } else {
            let x = syn::LitInt::new(&format!("0b{:b}", (1 << size) - 1), Span::call_site());
            let y = *shift - size;

            *shift = shift.checked_sub(size).ok_or_else(|| {
                syn::Error::new(span, "Not enough bits left in header declaration")
            })?;

            if name == "_" {
                return Ok(quote::quote! {});
            }

            let name = Ident::new(&format!("HEADER_SLOT_{}", name_str), Span::call_site());
            Ok(quote::quote! {
                const #name: u8 = #x << #y;
            })
        }
    } else if value_opt.is_some() {
        Err(syn::Error::new(
            span,
            "Affectation without size is not allowed in header declaration",
        ))
    } else {
        *shift = shift
            .checked_sub(1)
            .ok_or_else(|| syn::Error::new(span, "Not enough bits left in header declaration"))?;

        let name = Ident::new(&format!("HEADER_SLOT_{}", name_str), Span::call_site());
        Ok(quote::quote! {
            const #name: u8 = 0b1 << #shift;
        })
    }
}
