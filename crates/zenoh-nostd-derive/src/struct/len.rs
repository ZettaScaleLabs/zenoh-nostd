use proc_macro2::TokenStream;

use crate::model::{
    ZenohField, ZenohStruct,
    attribute::{DefaultAttribute, HeaderAttribute, PresenceAttribute, SizeAttribute},
    ty::ZenohType,
};

pub fn parse(r#struct: &ZenohStruct) -> syn::Result<TokenStream> {
    let mut len_parts = Vec::new();

    if r#struct.header.is_some() {
        len_parts.push(quote::quote! { 1usize });
    }

    for field in &r#struct.fields {
        match field {
            ZenohField::Regular { field } => {
                let access = &field.access;
                let ty = &field.ty;
                let attr = &field.attr;

                if !matches!(attr.header, HeaderAttribute::None) {
                    continue;
                }

                match ty {
                    ZenohType::U8
                    | ZenohType::U16
                    | ZenohType::U32
                    | ZenohType::U64
                    | ZenohType::USize
                    | ZenohType::ByteArray => {
                        len_parts.push(quote::quote! {
                            < _ as crate::ZStructEncode>::z_len(&self. #access)
                        });
                    }
                    ZenohType::ByteSlice | ZenohType::Str | ZenohType::ZStruct => {
                        if matches!(attr.size, SizeAttribute::Prefixed) {
                            len_parts.push(quote::quote! {
                                <usize as crate::ZStructEncode>::z_len(&< _ as crate::ZStructEncode>::z_len(&self. #access))
                            });
                        }

                        len_parts.push(quote::quote! {
                            < _ as crate::ZStructEncode>::z_len(&self. #access)
                        });
                    }
                    ZenohType::Option(_) => {
                        if matches!(attr.presence, PresenceAttribute::Prefixed) {
                            len_parts.push(quote::quote! { 1usize });
                        }

                        if matches!(attr.size, SizeAttribute::Prefixed) {
                            len_parts.push(quote::quote! {
                                if let Some(inner) = &self. #access {
                                    <usize as crate::ZStructEncode>::z_len(&< _ as crate::ZStructEncode>::z_len(inner))
                                } else {
                                    0usize
                                }
                            });
                        }

                        len_parts.push(quote::quote! {
                            if let Some(inner) = &self. #access {
                                < _ as crate::ZStructEncode>::z_len(inner)
                            } else {
                                0usize
                            }
                        })
                    }
                }
            }
            ZenohField::ExtBlock { exts } => {
                for field in exts {
                    let access = &field.access;
                    let ty = &field.ty;
                    let attr = &field.attr;

                    match ty {
                        ZenohType::ZStruct => match &attr.default {
                            DefaultAttribute::Expr(expr) => {
                                len_parts.push(quote::quote! {
                                    if &self. #access  != &#expr {
                                        crate::zext_len::<_>(&self. #access)
                                    } else {
                                        0usize
                                    }
                                });
                            }
                            _ => len_parts.push(quote::quote! {
                                crate::zext_len::<_>(&self. #access)
                            }),
                        },
                        ZenohType::Option(_) => {
                            len_parts.push(quote::quote! {
                                if let Some(inner) = &self. #access {
                                    crate::zext_len::<_>(inner)
                                } else {
                                    0usize
                                }
                            });
                        }
                        _ => unreachable!(
                            "Only ZStruct and Option<ZStruct> are allowed in ext blocks, this should have been caught earlier"
                        ),
                    }
                }
            }
        }
    }

    if len_parts.is_empty() {
        len_parts.push(quote::quote! { 0usize });
    }

    let len_body = len_parts
        .into_iter()
        .reduce(|acc, expr| quote::quote! { #acc + #expr })
        .unwrap();

    Ok(quote::quote! {
        #len_body
    })
}
