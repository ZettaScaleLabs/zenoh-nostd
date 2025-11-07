use proc_macro2::TokenStream;

use crate::model::{
    ZenohField, ZenohStruct,
    attribute::{
        DefaultAttribute, ExtAttribute, HeaderAttribute, PresenceAttribute, SizeAttribute,
        ZenohAttribute,
    },
    ty::ZenohType,
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

pub fn parse(r#struct: &ZenohStruct) -> syn::Result<(TokenStream, TokenStream)> {
    let mut len = Vec::new();
    let mut enc = Vec::new();
    let mut header = Vec::new();
    let mut s = Vec::new();

    if r#struct.header.is_some() {
        len.push(quote::quote! { 1usize });

        header.push(quote::quote! {
            let mut header: u8 = Self::HEADER_BASE;
        });
    }

    for field in &r#struct.fields {
        match field {
            ZenohField::Regular { field } => {
                let access = &field.access;
                let ty = &field.ty;
                let attr = &field.attr;

                if let HeaderAttribute::Slot(mask) = &attr.header {
                    header.push(quote::quote! { header  |= {
                        let v: u8 = self. #access.into();
                        (v << (#mask .trailing_zeros())) & #mask
                    }; });
                    continue;
                }

                s.push(access.clone());

                let default = match &attr.default {
                    DefaultAttribute::Expr(expr) => quote::quote! { #expr },
                    _ => quote::quote! {},
                };

                let check = match ty {
                    ZenohType::Option(_) => quote::quote! { #access.is_some() },
                    _ => quote::quote! { #access  != &#default },
                };

                match &attr.presence {
                    PresenceAttribute::Prefixed => {
                        len.push(quote::quote! { 1usize });
                        enc.push(quote::quote! {
                            <u8 as crate::ZStructEncode>::z_encode(&((#check) as u8), w)?;
                        });
                    }
                    PresenceAttribute::Header(mask) => {
                        header.push(quote::quote! {
                            if #check {
                                header |= #mask;
                            }
                        });
                    }
                    _ => {}
                }

                match &attr.size {
                    SizeAttribute::Prefixed => {
                        len.push(access_modifier(
                            &attr,
                            &quote::quote! {
                                <usize as crate::ZStructEncode>::z_len(&< _ as crate::ZStructEncode>::z_len(#access))
                            },
                            access,
                            &default,
                            true
                        ));

                        enc.push(access_modifier(
                            &attr,
                            &quote::quote! {
                                <usize as crate::ZStructEncode>::z_encode(&< _ as crate::ZStructEncode>::z_len( #access), w)?;
                            },
                            access,
                            &default,
                            false,
                        ));
                    }
                    SizeAttribute::Header(mask) => {
                        let e: u8 = (!attr.maybe_empty) as u8;
                        header.push(access_modifier(
                            &attr,
                            &quote::quote! {
                                header |= {
                                    let shift = #mask .trailing_zeros();
                                    let len = < _ as crate::ZStructEncode>::z_len(#access) as u8;

                                    ((len - #e) << shift) & #mask
                                };
                            },
                            access,
                            &default,
                            false,
                        ));
                    }
                    _ => {}
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
                enc.push(access_modifier(
                    &attr,
                    &quote::quote! {
                        < _ as crate::ZStructEncode>::z_encode(#access, w)?;
                    },
                    access,
                    &default,
                    false,
                ));
            }
            ZenohField::ExtBlock { exts } => {
                header.push(quote::quote! {
                    let mut n_exts = 0;
                });

                let mut enc_ext = Vec::<TokenStream>::new();

                for field in exts {
                    let access = &field.access;
                    s.push(access.clone());
                    let attr = &field.attr;

                    let id = match &attr.ext {
                        ExtAttribute::Expr(id) => id,
                        _ => unreachable!(
                            "ExtBlock fields must have an ext attribute, this should have been caught earlier"
                        ),
                    };

                    let mandatory = match &field.attr.mandatory {
                        true => quote::quote! { true },
                        false => quote::quote! { false },
                    };

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

                    header.push(access_modifier(
                        &attr,
                        &quote::quote! {
                            let _ = #access;
                            n_exts += 1;
                        },
                        access,
                        &default,
                        false,
                    ));

                    enc_ext.push(access_modifier(
                        &attr,
                        &quote::quote! {
                            n_exts -= 1;
                            crate::zext_encode::<_, #id, #mandatory>(#access, w, n_exts != 0)?;
                        },
                        access,
                        &default,
                        false,
                    ));
                }

                header.push(quote::quote! {
                    if n_exts > 0 {
                        header |= Self::HEADER_SLOT_Z;
                    }
                });

                enc.push(quote::quote! {
                    #(#enc_ext)*
                });
            }
        }
    }

    if r#struct.header.is_some() {
        header.push(quote::quote! {
            <u8 as crate::ZStructEncode>::z_encode(&header, w)?;
        });
    }

    if len.is_empty() {
        return Ok((quote::quote! { 0usize }, quote::quote! {}));
    }

    if s.is_empty() {
        return Ok((
            quote::quote! { #(#len)+* },
            quote::quote! {
                #(#header)*
                #(#enc)*
            },
        ));
    }

    Ok((
        quote::quote! {
            let Self {
                #(#s),*
                , ..
            } = self;

            #(#len)+*
        },
        quote::quote! {
            let Self {
                #(#s),*
                , ..
            } = self;

            #(#header)*

            #(#enc)*
        },
    ))
}
