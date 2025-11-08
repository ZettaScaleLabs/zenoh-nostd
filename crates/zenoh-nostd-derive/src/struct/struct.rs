use proc_macro2::TokenStream;

use crate::model::{
    ZenohField, ZenohStruct,
    attribute::{
        DefaultAttribute, ExtAttribute, HeaderAttribute, PresenceAttribute, SizeAttribute,
        ZenohAttribute,
    },
    ty::ZenohType,
};

fn enc_modifier(
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

fn dec_modifier(
    attr: &ZenohAttribute,
    tk: &TokenStream,
    access: &TokenStream,
    default: &TokenStream,
) -> TokenStream {
    let (p, d) = (
        !matches!(attr.presence, PresenceAttribute::None),
        !matches!(attr.default, DefaultAttribute::None),
    );

    if !p && !d {
        quote::quote! {
            let #access = {
                #tk
            };
        }
    } else if p && d {
        quote::quote! {
            let #access = if #access {
                #tk
            } else {
                #default
            };
        }
    } else if p && !d {
        quote::quote! {
            let #access = if #access {
                Some( { #tk })
            } else {
                None
            };
        }
    } else {
        unreachable!("All cases have been covered, this panic should have been caught earlier.");
    }
}

pub fn parse(
    r#struct: &ZenohStruct,
) -> syn::Result<(TokenStream, TokenStream, TokenStream, TokenStream)> {
    let mut len = Vec::new();
    let mut header = Vec::new();
    let mut enc = Vec::new();
    let mut dec = Vec::new();

    let mut s = Vec::new();
    let mut d = Vec::new();
    let mut h = Vec::new();

    if r#struct.header.is_some() {
        len.push(quote::quote! { 1usize });

        header.push(quote::quote! {
            let mut header: u8 = Self::HEADER_BASE;
        });

        dec.push(quote::quote! {
            let header: u8 = <u8 as crate::ZStructDecode>::z_decode(r)?;
        });
    }

    for field in &r#struct.fields {
        match field {
            ZenohField::Regular { field } => {
                let access = &field.access;
                d.push(access.clone());
                let ty = &field.ty;
                let attr = &field.attr;

                if let HeaderAttribute::Slot(mask) = &attr.header {
                    header.push(quote::quote! {
                        header  |= {
                            let v: u8 = self. #access.into();
                            (v << (#mask .trailing_zeros())) & #mask
                        };
                    });

                    dec.push(quote::quote! {
                        let #access = {
                            let v = header & #mask;
                            <_ as TryFrom<u8>>::try_from(v >> #mask.trailing_zeros()).map_err(|_| crate::ZCodecError::CouldNotParse)?
                        };
                    });
                    continue;
                }

                if matches!(attr.presence, PresenceAttribute::Header(_))
                    || matches!(attr.size, SizeAttribute::Header(_))
                {
                    h.push(access.clone());
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
                        dec.push(quote::quote! {
                            let #access: bool = <u8 as crate::ZStructDecode>::z_decode(r)? != 0;
                        });
                    }
                    PresenceAttribute::Header(slot) => {
                        header.push(quote::quote! {
                            if #check {
                                header |= #slot;
                            }
                        });
                        dec.push(quote::quote! {
                            let #access: bool = (header & #slot) != 0;
                        });
                    }
                    _ => {}
                }

                match &attr.size {
                    SizeAttribute::Prefixed => {
                        len.push(enc_modifier(
                            &attr,
                            &quote::quote! {
                                <usize as crate::ZStructEncode>::z_len(&< _ as crate::ZStructEncode>::z_len(#access))
                            },
                            access,
                            &default,
                            true
                        ));

                        enc.push(enc_modifier(
                            &attr,
                            &quote::quote! {
                                <usize as crate::ZStructEncode>::z_encode(&< _ as crate::ZStructEncode>::z_len( #access), w)?;
                            },
                            access,
                            &default,
                            false,
                        ));

                        dec.push(dec_modifier(
                            &attr,
                            &quote::quote! {
                                let #access = < usize as crate::ZStructDecode>::z_decode(r)?;
                                < _ as crate::ZStructDecode>::z_decode(&mut < crate::ZReader as crate::ZReaderExt>::sub(r, #access)?)?
                            },
                            access,
                            &default,
                        ));
                    }
                    SizeAttribute::Header(slot) => {
                        let e: u8 = (!attr.maybe_empty) as u8;
                        header.push(enc_modifier(
                            &attr,
                            &quote::quote! {
                                header |= {
                                    let shift = #slot .trailing_zeros();
                                    let len = < _ as crate::ZStructEncode>::z_len(#access) as u8;

                                    ((len - #e) << shift) & #slot
                                };
                            },
                            access,
                            &default,
                            false,
                        ));

                        dec.push(dec_modifier(
                            &attr,
                            &quote::quote! {
                                let #access = (((header & #slot) >> #slot.trailing_zeros()) + #e) as usize;
                                < _ as crate::ZStructDecode>::z_decode(&mut < crate::ZReader as crate::ZReaderExt>::sub(r, #access)?)?
                            },
                            access,
                            &default,
                        ));
                    }
                    _ => {
                        dec.push(dec_modifier(
                            &attr,
                            &quote::quote! {
                                < _ as crate::ZStructDecode>::z_decode(r)?
                            },
                            access,
                            &default,
                        ));
                    }
                }

                len.push(enc_modifier(
                    &attr,
                    &quote::quote! {
                        < _ as crate::ZStructEncode>::z_len(#access)
                    },
                    access,
                    &default,
                    true,
                ));
                enc.push(enc_modifier(
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

                enc.push(quote::quote! {
                    let mut n_exts = 0;
                });

                dec.push(quote::quote! {
                    let mut has_ext: bool = header & Self::HEADER_SLOT_Z != 0;
                });

                let mut enc_ext = Vec::<TokenStream>::new();
                let mut dec_ext = Vec::<TokenStream>::new();

                for field in exts {
                    let access = &field.access;
                    s.push(access.clone());
                    d.push(access.clone());
                    h.push(access.clone());
                    let ty = &field.ty;
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

                    len.push(enc_modifier(
                        &attr,
                        &quote::quote! {
                            crate::zext_len::<_>(#access)
                        },
                        access,
                        &default,
                        true,
                    ));

                    header.push(enc_modifier(
                        &attr,
                        &quote::quote! {
                            let _ = #access;
                            n_exts += 1;
                        },
                        access,
                        &default,
                        false,
                    ));

                    enc.push(enc_modifier(
                        &attr,
                        &quote::quote! {
                            let _ = #access;
                            n_exts += 1;
                        },
                        access,
                        &default,
                        false,
                    ));

                    enc_ext.push(enc_modifier(
                        &attr,
                        &quote::quote! {
                            n_exts -= 1;
                            crate::zext_encode::<_, #id, #mandatory>(#access, w, n_exts != 0)?;
                        },
                        access,
                        &default,
                        false,
                    ));

                    match ty {
                        ZenohType::ZStruct => {
                            dec.push(quote::quote! {
                                let mut #access = #default;
                            });

                            dec_ext.push(quote::quote! {
                                #id => {
                                    #access = crate::zext_decode::< _ >(r)?;
                                }
                            });
                        }
                        ZenohType::Option(_) => {
                            dec.push(quote::quote! {
                                let mut #access: _ = None;
                            });

                            dec_ext.push(quote::quote! {
                                #id => {
                                    #access = Some(crate::zext_decode::< _ >(r)?);
                                }
                            });
                        }
                        _ => unreachable!(
                            "ExtBlock fields must be ZStruct or Option<ZStruct>, this should have been caught earlier"
                        ),
                    }
                }

                header.push(quote::quote! {
                    if n_exts > 0 {
                        header |= Self::HEADER_SLOT_Z;
                    }
                });

                enc.push(quote::quote! {
                    #(#enc_ext)*
                });

                dec.push(quote::quote! {
                    while has_ext {
                        let (ext_id, ext_kind, mandatory, more) = crate::decode_ext_header(r)?;
                        has_ext = more;

                        match ext_id {
                            #(#dec_ext,)*
                            _ => {
                                if mandatory {
                                    return Err(crate::ZCodecError::UnsupportedMandatoryExtension);
                                }

                                crate::skip_ext(r, ext_kind)?;
                            }
                        }
                    }
                });
            }
        }
    }

    if r#struct.header.is_some() {
        header.push(quote::quote! {
            // <u8 as crate::ZStructEncode>::z_encode(&header, w)?;
            Some(header)
        });
    } else {
        header.push(quote::quote! {
            None
        });
    }

    if len.is_empty() {
        return Ok((
            quote::quote! { 0usize },
            quote::quote! { #(#header)* },
            quote::quote! {},
            quote::quote! { Ok(Self {}) },
        ));
    }

    if s.is_empty() {
        return Ok((
            quote::quote! { #(#len)+* },
            quote::quote! { #(#header)* },
            quote::quote! {
                #(#enc)*
            },
            quote::quote! {
                #(#dec)*

                Ok(Self { #(#d),* })
            },
        ));
    }

    let header = if h.is_empty() {
        quote::quote! { #(#header)* }
    } else {
        quote::quote! {
            let Self {
                #(#h),*
                , ..
            } = self;

            #(#header)*
        }
    };

    Ok((
        quote::quote! {
            let Self {
                #(#s),*
                , ..
            } = self;

            #(#len)+*
        },
        header,
        quote::quote! {
            let Self {
                #(#s),*
                , ..
            } = self;

            #(#enc)*
        },
        quote::quote! {
            #(#dec)*

            Ok(Self { #(#d),* })
        },
    ))
}
