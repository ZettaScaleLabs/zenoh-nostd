use proc_macro2::TokenStream;

use crate::{
    codec::model::{
        ZenohField, ZenohStruct,
        attribute::{DefaultAttribute, HeaderAttribute, PresenceAttribute, SizeAttribute},
        ty::ZenohType,
    },
    codec::r#struct::enc_len_modifier,
};

pub fn parse(r#struct: &ZenohStruct) -> syn::Result<(TokenStream, bool)> {
    let mut body = Vec::<TokenStream>::new();
    let mut s = Vec::<TokenStream>::new();

    if r#struct.header.is_some() {
        body.push(quote::quote! {
            let mut header: u8 = Self::HEADER_BASE;
        });
    }

    for field in &r#struct.fields {
        match field {
            ZenohField::Regular { field } => {
                let access = &field.access;
                let ty = &field.ty;
                let attr = &field.attr;

                let default = match &attr.default {
                    DefaultAttribute::Expr(expr) => quote::quote! { #expr },
                    _ => quote::quote! {},
                };

                if let HeaderAttribute::Slot(slot) = &attr.header {
                    s.push(access.clone());

                    body.push(quote::quote! {
                        header  |= {
                            let v: u8 = (*#access).into();
                            (v << (#slot .trailing_zeros())) & #slot
                        };
                    });

                    continue;
                }

                if matches!(attr.presence, PresenceAttribute::Header(_))
                    || matches!(attr.size, SizeAttribute::Header(_))
                    || attr.flatten
                {
                    s.push(access.clone());
                }

                let len = if attr.flatten {
                    let shift = attr.shift.unwrap_or(0);

                    body.push(enc_len_modifier(
                        attr,
                        &quote::quote! {
                            header |= < _ as crate::ZHeader>::z_header(#access) << #shift;
                        },
                        access,
                        &default,
                        false,
                    ));

                    quote::quote! { < _ as crate::ZBodyLen>::z_body_len(#access) }
                } else {
                    quote::quote! { < _ as crate::ZLen>::z_len(#access) }
                };

                let check = match ty {
                    ZenohType::Option(_) => quote::quote! { #access.is_some() },
                    _ => quote::quote! { #access  != &#default },
                };

                if let PresenceAttribute::Header(slot) = &attr.presence {
                    body.push(quote::quote! {
                        if #check {
                            header |= #slot;
                        }
                    });
                }

                if let SizeAttribute::Header(slot) = &attr.size {
                    let e: u8 = (!attr.maybe_empty) as u8;
                    body.push(enc_len_modifier(
                        attr,
                        &quote::quote! {
                            header |= {
                                let shift = #slot .trailing_zeros();
                                let len = #len as u8;

                                ((len - #e) << shift) & #slot
                            };
                        },
                        access,
                        &default,
                        false,
                    ));
                }
            }
            ZenohField::ExtBlock { .. } => {
                body.push(quote::quote! {
                    header |= if <_ as crate::ZExtCount>::z_ext_count(self) > 0 {
                        Self::HEADER_SLOT_Z
                    } else {
                        0
                    };
                });
            }
        }
    }

    let expand = if s.is_empty() {
        quote::quote! { .. }
    } else {
        quote::quote! { , .. }
    };

    if body.is_empty() {
        return Ok((quote::quote! {}, false));
    }

    Ok((
        quote::quote! {
            let Self {
                #(#s),*
                #expand
            } = self;

            #(#body)*

            header
        },
        true,
    ))
}
