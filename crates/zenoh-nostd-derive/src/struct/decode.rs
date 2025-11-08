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

pub fn parse(r#struct: &ZenohStruct) -> syn::Result<TokenStream> {
    let mut dec = Vec::<TokenStream>::new();
    let mut d = Vec::<TokenStream>::new();

    if r#struct.header.is_some() {
        dec.push(quote::quote! {
            let header: u8 = <u8 as crate::ZStructDecode>::z_decode(r)?;
        });
    }

    for field in &r#struct.fields {
        match field {
            ZenohField::Regular { field } => {
                let access = &field.access;
                d.push(access.clone());
                let attr = &field.attr;

                if let HeaderAttribute::Slot(mask) = &attr.header {
                    dec.push(quote::quote! {
                        let #access = {
                            let v = header & #mask;
                            <_ as TryFrom<u8>>::try_from(v >> #mask.trailing_zeros()).map_err(|_| crate::ZCodecError::CouldNotParse)?
                        };
                    });

                    continue;
                }

                let default = match &attr.default {
                    DefaultAttribute::Expr(expr) => quote::quote! { #expr },
                    _ => quote::quote! {},
                };

                match &attr.presence {
                    PresenceAttribute::Prefixed => {
                        dec.push(quote::quote! {
                            let #access: bool = <u8 as crate::ZStructDecode>::z_decode(r)? != 0;
                        });
                    }
                    PresenceAttribute::Header(mask) => {
                        dec.push(quote::quote! {
                            let #access: bool = (header & #mask) != 0;
                        });
                    }
                    _ => {}
                };

                match &attr.size {
                    SizeAttribute::Prefixed => {
                        dec.push(access_modifier(
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
                        let e: u8 = !(attr.maybe_empty) as u8;

                        dec.push(access_modifier(
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
                        dec.push(access_modifier(
                            &attr,
                            &quote::quote! {
                                < _ as crate::ZStructDecode>::z_decode(r)?
                            },
                            access,
                            &default,
                        ));
                    }
                };
            }
            ZenohField::ExtBlock { exts } => {
                dec.push(quote::quote! {
                    let mut has_ext: bool = header & Self::HEADER_SLOT_Z != 0;
                });

                let mut ext_body = Vec::<TokenStream>::new();

                for field in exts {
                    let access = &field.access;
                    d.push(access.clone());
                    let ty = &field.ty;
                    let attr = &field.attr;

                    let id = match &attr.ext {
                        ExtAttribute::Expr(id) => id,
                        _ => unreachable!(
                            "ExtBlock fields must have an ext attribute, this should have been caught earlier"
                        ),
                    };

                    let default = match &attr.default {
                        DefaultAttribute::Expr(expr) => quote::quote! { #expr },
                        _ => quote::quote! {},
                    };

                    match ty {
                        ZenohType::ZStruct => {
                            dec.push(quote::quote! {
                                let mut #access = #default;
                            });

                            ext_body.push(quote::quote! {
                                #id => {
                                    #access = crate::zext_decode::< _ >(r)?;
                                }
                            });
                        }
                        ZenohType::Option(_) => {
                            dec.push(quote::quote! {
                                let mut #access: _ = None;
                            });

                            ext_body.push(quote::quote! {
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

                dec.push(quote::quote! {
                    while has_ext {
                        let (ext_id, ext_kind, mandatory, more) = crate::decode_ext_header(r)?;
                        has_ext = more;

                        match ext_id {
                            #(#ext_body,)*
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

    Ok(quote::quote! {
        #(#dec)*

        Ok(Self { #(#d),* })
    })
}
