use proc_macro2::TokenStream;
use syn::DeriveInput;

use crate::r#struct::{
    decode, encode, flag, header, len,
    parse::{ZFieldKind, ZStruct, ZStructKind},
};

pub fn derive_zext(input: DeriveInput) -> TokenStream {
    let ident = &input.ident;
    let data = &input.data;

    let mut lt_params = false;
    for generic in input.generics.params.iter() {
        match generic {
            syn::GenericParam::Lifetime(_) => {
                if lt_params {
                    panic!("ZStruct can only have one lifetime parameter.");
                }

                lt_params = true;
            }
            _ => {
                panic!("ZStruct can only have a lifetime parameter.");
            }
        }
    }

    let (ty_elided, ty_lt) = if lt_params {
        (quote::quote! { <'_> }, quote::quote! { <'a> })
    } else {
        (quote::quote! {}, quote::quote! {})
    };

    let r#struct = ZStruct::from_data(data);
    let kind = infer_kind(&r#struct);

    let (header_enc, header_dec) = header::parse_body(&r#struct);
    let (flag_enc, flag_dec) = flag::parse_body(&r#struct);

    let len_body = len::parse_body(&r#struct);
    let encode_body = encode::parse_body(&r#struct, flag_enc);
    let decode_body = decode::parse_body(&r#struct, flag_dec);

    quote::quote! {
        impl crate::ZStruct for #ident #ty_elided {
            fn z_len(&self) -> usize {
                #len_body
            }

            fn z_encode(&self, w: &mut crate::ByteWriter) -> crate::io::ByteIOResult<()> {
                #header_enc

                #encode_body
            }

            type ZType<'a> = #ident #ty_lt;

            fn z_decode<'a>(r: &mut crate::ByteReader<'a>) -> crate::io::ByteIOResult<Self::ZType<'a>> {
                #header_dec

                #decode_body
            }
        }

        impl crate::ZExt for #ident #ty_elided {
            const KIND: crate::ZExtKind = #kind;
        }
    }
}

fn infer_kind(ext: &ZStruct) -> TokenStream {
    if ext.0.is_empty() {
        quote::quote! { crate::ZExtKind::Unit }
    } else if ext.0.len() == 1 {
        let kind = &ext.0.first().unwrap().kind;

        match kind {
            ZFieldKind::Flag
            | ZFieldKind::Header
            | ZFieldKind::HeaderStorage { .. }
            | ZFieldKind::ZExtBlockEnd
            | ZFieldKind::ZExtBlock { .. } => {
                panic!("ZExt cannot infer kind from one marker field.")
            }
            ZFieldKind::ZStruct(ZStructKind { ty, .. }) => {
                let ty = ty.to_string();

                if ty == "u8" {
                    panic!("For now, U64 kind inference is limited to u16, u32, u64 and usize");
                } else if ty == "u16" || ty == "u32" || ty == "u64" || ty == "usize" {
                    quote::quote! { crate::ZExtKind::U64 }
                } else {
                    quote::quote! { crate::ZExtKind::ZStruct }
                }
            }
        }
    } else {
        quote::quote! { crate::ZExtKind::ZStruct }
    }
}
