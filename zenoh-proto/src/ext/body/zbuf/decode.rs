use proc_macro2::{Span, TokenStream};
use syn::Ident;

use crate::ext::parse::{FieldKind, ParsedField, SizeFlavour};

pub fn decode_body(fields: &Vec<ParsedField>, named: bool) -> TokenStream {
    let mut decode_parts = Vec::new();
    let mut result_parts = Vec::new();

    for field in fields {
        let access = &field.access;
        let kind = &field.kind;

        let faccess = if named {
            quote::quote! { #access }
        } else {
            let string = access.to_string();
            let ident = Ident::new(&format!("_field_{}", string), Span::call_site());
            quote::quote! { #ident }
        };

        match kind {
            crate::ext::parse::FieldKind::U8 => {
                decode_parts.push(quote::quote! {
                    let #faccess = crate::protocol::codec::decode_u8(r)?;
                });
            }
            crate::ext::parse::FieldKind::U16 => {
                decode_parts.push(quote::quote! {
                    let #faccess = crate::protocol::codec::decode_u16(r)?;
                });
            }
            crate::ext::parse::FieldKind::U32 => {
                decode_parts.push(quote::quote! {
                    let #faccess = crate::protocol::codec::decode_u32(r)?;
                });
            }
            crate::ext::parse::FieldKind::U64 => {
                decode_parts.push(quote::quote! {
                    let #faccess = crate::protocol::codec::decode_u64(r)?;
                });
            }
            crate::ext::parse::FieldKind::Usize => {
                decode_parts.push(quote::quote! {
                    let #faccess = crate::protocol::codec::decode_u64(r)?;
                });
            }
            crate::ext::parse::FieldKind::Timestamp => {
                decode_parts.push(quote::quote! {
                    let #faccess = crate::protocol::codec::decode_timestamp(r)?;
                });
            }
            crate::ext::parse::FieldKind::Array => {
                decode_parts.push(quote::quote! {
                    let #faccess = crate::protocol::codec::decode_array(r)?;
                });
            }
            crate::ext::parse::FieldKind::ZBuf(flavour)
            | crate::ext::parse::FieldKind::Str(flavour)
            | crate::ext::parse::FieldKind::Zid(flavour) => {
                match flavour {
                    SizeFlavour::Plain => {
                        decode_parts.push(
                            quote::quote! { let #faccess = crate::protocol::codec::decode_usize(r)?; },
                        );
                    }
                    SizeFlavour::Deduced => {
                        decode_parts.push(quote::quote! {
                            let #faccess = l - ((_start - r.remaining()) as usize);
                        });
                    }
                    _ => {}
                }

                match kind {
                    crate::ext::parse::FieldKind::ZBuf(_) => {
                        decode_parts.push(
                            quote::quote! { let #faccess = crate::protocol::codec::decode_zbuf(r, #faccess)?; },
                        );
                    }
                    crate::ext::parse::FieldKind::Str(_) => {
                        decode_parts.push(
                            quote::quote! { let #faccess = crate::protocol::codec::decode_str(r, #faccess)?; },
                        );
                    }
                    crate::ext::parse::FieldKind::Zid(_) => {
                        decode_parts.push(
                            quote::quote! { let #faccess = crate::protocol::codec::decode_zid(r, #faccess)?; },
                        );
                    }
                    _ => unreachable!(),
                };
            }
            FieldKind::Composite(attr) => {
                let path = &attr.path;
                let ident = &attr.ident;

                let dec_fn = quote::format_ident!("decode_{}", ident);

                decode_parts.push(quote::quote! {
                    let #faccess = #path :: #dec_fn (r)?;
                });
            }
        }

        result_parts.push(quote::quote! {
            #access: #faccess
        });
    }

    quote::quote! {
        #(#decode_parts)*

        Ok(Self::Decoded { #(#result_parts),* })
    }
}
