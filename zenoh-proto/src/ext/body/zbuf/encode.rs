use proc_macro2::TokenStream;

use crate::ext::parse::{ParsedField, SizeFlavour};

pub fn encode_body(fields: &Vec<ParsedField>) -> TokenStream {
    let mut encode_parts = Vec::new();

    for field in fields {
        let access = &field.access;
        let kind = &field.kind;

        match kind {
            crate::ext::parse::FieldKind::U8 => {
                encode_parts.push(quote::quote! {
                    crate::protocol::codec::encode_u8(w, x. #access as u8)?;
                });
            }
            crate::ext::parse::FieldKind::U16 => {
                encode_parts.push(quote::quote! {
                    crate::protocol::codec::encode_u16(w, x. #access as u16)?;
                });
            }
            crate::ext::parse::FieldKind::U32 => {
                encode_parts.push(quote::quote! {
                    crate::protocol::codec::encode_u32(w, x. #access as u32)?;
                });
            }
            crate::ext::parse::FieldKind::U64 => {
                encode_parts.push(quote::quote! {
                    crate::protocol::codec::encode_u64(w, x. #access as u64)?;
                });
            }
            crate::ext::parse::FieldKind::Usize => {
                encode_parts.push(quote::quote! {
                    crate::protocol::codec::encode_u64(w, x. #access as u64)?;
                });
            }
            crate::ext::parse::FieldKind::Timestamp => {
                encode_parts.push(quote::quote! {
                    crate::protocol::codec::encode_timestamp(w, &x. #access)?;
                });
            }
            crate::ext::parse::FieldKind::Array => {
                encode_parts.push(quote::quote! {
                    crate::protocol::codec::encode_array(w, &x. #access)?;
                });
            }
            crate::ext::parse::FieldKind::ZBuf(flavour)
            | crate::ext::parse::FieldKind::Str(flavour)
            | crate::ext::parse::FieldKind::Zid(flavour) => {
                match flavour {
                    SizeFlavour::Plain => {
                        let encoded_len_fn = match kind {
                            crate::ext::parse::FieldKind::ZBuf(_) => {
                                quote::format_ident!("encoded_len_zbuf")
                            }
                            crate::ext::parse::FieldKind::Str(_) => {
                                quote::format_ident!("encoded_len_str")
                            }
                            crate::ext::parse::FieldKind::Zid(_) => {
                                quote::format_ident!("encoded_len_zid")
                            }
                            _ => unreachable!(),
                        };

                        encode_parts.push(quote::quote! {
                            crate::protocol::codec::encode_usize(w, crate::protocol::codec:: #encoded_len_fn(&x. #access))?;
                        });
                    }
                    _ => {}
                }

                let encode_fn = match kind {
                    crate::ext::parse::FieldKind::ZBuf(_) => quote::format_ident!("encode_zbuf"),
                    crate::ext::parse::FieldKind::Str(_) => quote::format_ident!("encode_str"),
                    crate::ext::parse::FieldKind::Zid(_) => quote::format_ident!("encode_zid"),
                    _ => unreachable!(),
                };

                encode_parts.push(quote::quote! {
                    crate::protocol::codec:: #encode_fn (w, &x. #access)?;
                });
            }
            crate::ext::parse::FieldKind::Composite(attr) => {
                let path = &attr.path;
                let ident = &attr.ident;

                let dec_fn = quote::format_ident!("encode_{}", ident);
                encode_parts.push(quote::quote! {
                    #path :: #dec_fn (w, &x. #access)?;
                });
            }
        }
    }

    let expanded = quote::quote! {
        #(#encode_parts)*
    };

    expanded.into()
}
