use proc_macro2::TokenStream;

use crate::ext::parse::{FieldKind, ParsedField, SizeFlavour};

pub fn len_body(fields: &Vec<ParsedField>) -> TokenStream {
    let mut flag_needed = false;
    let mut len_parts = Vec::new();

    for field in fields {
        let access = &field.access;
        let kind = &field.kind;

        match kind {
            FieldKind::U8 => {
                len_parts.push(quote::quote! { 1 });
            }
            FieldKind::U16 | FieldKind::U32 | FieldKind::U64 | FieldKind::Usize => {
                len_parts.push(
                    quote::quote! { crate::protocol::codec::encoded_len_u64(x. #access as u64) },
                );
            }
            FieldKind::Timestamp => {
                len_parts.push(
                    quote::quote! { crate::protocol::codec::encoded_len_timestamp(&x. #access) },
                );
            }
            FieldKind::Array => {
                len_parts
                    .push(quote::quote! { crate::protocol::codec::encoded_len_array(&x. #access) });
            }
            FieldKind::ZBuf(flavour) | FieldKind::Str(flavour) | FieldKind::Zid(flavour) => {
                let encoded_len_fn = match kind {
                    FieldKind::ZBuf(_) => quote::format_ident!("encoded_len_zbuf"),
                    FieldKind::Str(_) => quote::format_ident!("encoded_len_str"),
                    FieldKind::Zid(_) => quote::format_ident!("encoded_len_zid"),
                    _ => unreachable!(),
                };

                match flavour {
                    SizeFlavour::Plain => {
                        len_parts.push(quote::quote! {
                            crate::protocol::codec::encoded_len_u64(crate::protocol::codec::#encoded_len_fn(&x. #access) as u64)
                        });
                    }
                    _ => {}
                }

                len_parts
                    .push(quote::quote! { crate::protocol::codec::#encoded_len_fn(&x. #access) });
            }
            FieldKind::Composite(attr) => {
                let path = &attr.path;
                let ident = &attr.ident;

                let func_ident = quote::format_ident!("encoded_len_{}", ident);
                len_parts.push(quote::quote! { #path :: #func_ident(&x. #access) });
            }
        }

        match kind {
            FieldKind::Zid(flavour) | FieldKind::Str(flavour) | FieldKind::ZBuf(flavour) => {
                match flavour {
                    SizeFlavour::MaybeEmptyFlag(_) | SizeFlavour::NonEmptyFlag(_) => {
                        flag_needed = true;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    let len_body = len_parts
        .into_iter()
        .reduce(|acc, expr| quote::quote! { #acc + #expr })
        .expect("at least one field for zbuf extension");

    if flag_needed {
        quote::quote! {
            1 + (#len_body)
        }
    } else {
        quote::quote! {
            #len_body
        }
    }
}
