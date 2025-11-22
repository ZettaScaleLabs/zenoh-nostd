use proc_macro2::TokenStream;

use crate::{
    codec::model::{
        ZenohField, ZenohStruct,
        attribute::{DefaultAttribute, HeaderAttribute, PresenceAttribute, SizeAttribute},
    },
    codec::r#struct::enc_len_modifier,
};

pub fn parse(r#struct: &ZenohStruct) -> syn::Result<(TokenStream, TokenStream)> {
    let mut body = Vec::<TokenStream>::new();
    let mut full = quote::quote! { <_ as crate::ZBodyLen>::z_body_len(self) };
    let mut s = Vec::<TokenStream>::new();

    if r#struct.header.is_some() {
        full = quote::quote! { 1 + #full };
    }

    for field in &r#struct.fields {
        match field {
            ZenohField::Regular { field } => {
                let access = &field.access;
                let attr = &field.attr;

                if let HeaderAttribute::Slot(_) = &attr.header {
                    continue;
                }

                s.push(access.clone());

                let default = match &attr.default {
                    DefaultAttribute::Expr(expr) => quote::quote! { #expr },
                    _ => quote::quote! {},
                };

                let len = if attr.flatten {
                    quote::quote! { < _ as crate::ZBodyLen>::z_body_len(#access) }
                } else {
                    quote::quote! { < _ as crate::ZLen>::z_len(#access) }
                };

                match &attr.presence {
                    PresenceAttribute::Prefixed => {
                        body.push(quote::quote! { 1usize });
                    }
                    PresenceAttribute::Header(_) => {}
                    _ => {}
                }

                match &attr.size {
                    SizeAttribute::Prefixed => {
                        body.push(enc_len_modifier(
                            attr,
                            &quote::quote! {
                                <usize as crate::ZLen>::z_len(&#len)
                            },
                            access,
                            &default,
                            true,
                        ));
                    }
                    SizeAttribute::Header(_) => {}
                    _ => {}
                }

                body.push(enc_len_modifier(attr, &len, access, &default, true));
            }
            ZenohField::ExtBlock { exts } => {
                for field in exts {
                    let access = &field.access;
                    s.push(access.clone());
                    let attr = &field.attr;

                    let default = match &attr.default {
                        DefaultAttribute::Expr(expr) => quote::quote! { #expr },
                        _ => quote::quote! {},
                    };

                    body.push(enc_len_modifier(
                        attr,
                        &quote::quote! {
                            crate::zext_len::<_>(#access)
                        },
                        access,
                        &default,
                        true,
                    ));
                }
            }
        }
    }

    let expand = if s.is_empty() {
        quote::quote! { .. }
    } else {
        quote::quote! { , .. }
    };

    Ok((
        quote::quote! {
            let Self {
                #(#s),*
                #expand
            } = self;

            0 #(+ #body )*
        },
        quote::quote! { #full },
    ))
}
