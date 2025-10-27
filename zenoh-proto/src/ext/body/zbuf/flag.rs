use proc_macro2::{Span, TokenStream};
use syn::Ident;

use crate::ext::parse::{FieldKind, ParsedField, SizeFlavour};

pub fn flag_body(fields: &Vec<ParsedField>, named: bool) -> (TokenStream, TokenStream) {
    let mut enc_flag_parts = Vec::new();
    let mut dec_flag_parts = Vec::new();
    let mut shift = 0u8;

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
            FieldKind::ZBuf(flavour) | FieldKind::Str(flavour) | FieldKind::Zid(flavour) => {
                let (flag_size, maybe_empty) = match flavour {
                    SizeFlavour::NonEmptyFlag(size) => (*size, false),
                    SizeFlavour::MaybeEmptyFlag(size) => (*size, true),
                    _ => continue,
                };

                let len = match kind {
                    FieldKind::ZBuf(_) => {
                        quote::quote! { crate::protocol::codec::encoded_len_zbuf(&x. #access) }
                    }
                    FieldKind::Str(_) => {
                        quote::quote! { crate::protocol::codec::encoded_len_str(&x. #access) }
                    }
                    FieldKind::Zid(_) => {
                        quote::quote! { crate::protocol::codec::encoded_len_zid(&x. #access) }
                    }
                    _ => unreachable!(),
                };

                if maybe_empty {
                    enc_flag_parts.push(quote::quote! {
                        flag |= ((#len as u8) & ((1 << #flag_size) - 1)) << #shift;
                    });

                    dec_flag_parts.push(quote::quote! {
                        let #faccess =
                            ((flag >> #shift) & ((1 << #flag_size) - 1)) as usize;
                    });
                } else {
                    enc_flag_parts.push(quote::quote! {
                        flag |= ((#len as u8 - 1) & ((1 << #flag_size) - 1)) << #shift;
                    });

                    dec_flag_parts.push(quote::quote! {
                        let #faccess =
                            (((flag >> #shift) & ((1 << #flag_size) - 1)) as usize) + 1;
                    });
                }

                shift += flag_size;
            }
            _ => {}
        }
    }

    if enc_flag_parts.is_empty() {
        return (quote::quote! {}, quote::quote! {});
    }

    (
        quote::quote! {
            let mut flag: u8 = 0;
            #(#enc_flag_parts)*
            crate::protocol::codec::encode_u8(w, flag)?;
        },
        quote::quote! {
            let flag = crate::protocol::codec::decode_u8(r)?;
            #(#dec_flag_parts)*
        },
    )
}
