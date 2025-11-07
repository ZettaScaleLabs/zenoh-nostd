use proc_macro2::TokenStream;

use crate::model::{
    ZenohField, ZenohStruct,
    attribute::{
        DefaultAttribute, ExtAttribute, HeaderAttribute, PresenceAttribute, SizeAttribute,
        ZenohAttribute,
    },
};

fn access_modifier(
    attr: &ZenohAttribute,
    tk: &TokenStream,
    access: &TokenStream,
    default: &TokenStream,
    append: bool,
) -> TokenStream {
    let (p, e, d) = (
        !matches!(attr.presence, PresenceAttribute::None),
        !matches!(attr.ext, ExtAttribute::None),
        !matches!(attr.default, DefaultAttribute::None),
    );

    if !p && !d && !e {
        quote::quote! { #tk }
    } else if (p && d && !e) || (e && !p && d) {
        let res = quote::quote! {
            if #access  != &#default {
                #tk
            }
        };

        if append {
            quote::quote! { #res else { 0usize } }
        } else {
            res
        }
    } else if (p && !d && !e) || (e && !p && !d) {
        let res = quote::quote! {
            if let Some(#access) = #access {
                #tk
            }
        };

        if append {
            quote::quote! { #res else { 0usize } }
        } else {
            res
        }
    } else {
        unreachable!("All cases have been covered, this panic should have been caught earlier.");
    }
}

pub fn parse(r#struct: &ZenohStruct) -> syn::Result<TokenStream> {
    let mut len = Vec::new();
    let mut s = Vec::new();

    if r#struct.header.is_some() {
        len.push(quote::quote! { 1usize });
    }

    for field in &r#struct.fields {
        match field {
            ZenohField::Regular { field } => {
                let access = &field.access;
                let attr = &field.attr;

                if !matches!(attr.header, HeaderAttribute::None) {
                    continue;
                }

                s.push(access.clone());

                let default = match &attr.default {
                    DefaultAttribute::Expr(expr) => quote::quote! { #expr },
                    _ => quote::quote! {},
                };

                if matches!(attr.presence, PresenceAttribute::Prefixed) {
                    len.push(quote::quote! { 1usize });
                }

                if matches!(attr.size, SizeAttribute::Prefixed) {
                    len.push(access_modifier(
                        &attr,
                        &quote::quote! {
                            <usize as crate::ZStructEncode>::z_len(&< _ as crate::ZStructEncode>::z_len(#access))
                        },
                        access,
                        &default,
                        true
                    ));
                }

                len.push(access_modifier(
                    &attr,
                    &quote::quote! {
                        < _ as crate::ZStructEncode>::z_len(#access)
                    },
                    access,
                    &default,
                    true,
                ));
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

                    len.push(access_modifier(
                        &attr,
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

    if len.is_empty() {
        return Ok(quote::quote! { 0usize });
    }

    if s.is_empty() {
        return Ok(quote::quote! { #(#len)+* });
    }

    Ok(quote::quote! {
        let Self {
            #(#s),*
            , ..
        } = self;

        #(#len)+*
    })
}
