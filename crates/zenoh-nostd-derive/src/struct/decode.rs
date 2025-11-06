use proc_macro2::TokenStream;

use crate::model::{
    ZenohField, ZenohStruct,
    attribute::{
        DefaultAttribute, ExtAttribute, HeaderAttribute, PresenceAttribute, SizeAttribute,
    },
    ty::ZenohType,
};

pub fn parse(r#struct: &ZenohStruct) -> syn::Result<TokenStream> {
    let mut body = Vec::<TokenStream>::new();
    let mut declaration = Vec::<TokenStream>::new();

    if r#struct.header.is_some() {
        body.push(quote::quote! {
            let header: u8 = <u8 as crate::ZStructDecode>::z_decode(r)?;
        });
    }

    for field in &r#struct.fields {
        match field {
            ZenohField::Regular { field } => {
                let access = &field.access;
                let ty = &field.ty;
                let attr = &field.attr;

                declaration.push(quote::quote! {
                    #access
                });

                if let HeaderAttribute::Slot(mask) = &attr.header {
                    body.push(quote::quote! {
                            let #access = {
                                let v = header & #mask;
                                <_ as TryFrom<u8>>::try_from(v >> #mask.trailing_zeros()).map_err(|_| crate::ZCodecError::CouldNotParse)?
                            };
                        });
                    continue;
                }

                match ty {
                    ZenohType::U8
                    | ZenohType::U16
                    | ZenohType::U32
                    | ZenohType::U64
                    | ZenohType::USize
                    | ZenohType::ByteArray
                    | ZenohType::ByteSlice
                    | ZenohType::Str
                    | ZenohType::ZStruct => match &attr.presence {
                        PresenceAttribute::Prefixed => {
                            let default = match &attr.default {
                                DefaultAttribute::Expr(expr) => expr,
                                _ => unreachable!(
                                    "Fields with presence attribute must have a default attribute, this should have been caught earlier"
                                ),
                            };

                            body.push(quote::quote! {
                                let #access: bool = <u8 as crate::ZStructDecode>::z_decode(r)? != 0;
                            });

                            match &attr.size {
                                SizeAttribute::Prefixed => {
                                    body.push(quote::quote! {
                                        let #access = if #access {
                                            let #access = < usize as crate::ZStructDecode>::z_decode(r)?;
                                            < _ as crate::ZStructDecode>::z_decode(&mut < crate::ZReader as crate::ZReaderExt>::sub(r, #access)?)?
                                        } else {
                                            #default
                                        };
                                    });
                                }
                                SizeAttribute::Header(mask) => {
                                    let e: u8 = !(attr.maybe_empty) as u8;

                                    body.push(quote::quote! {
                                        let #access = if #access {
                                            let #access = (((header & #mask) >> #mask.trailing_zeros()) + #e) as usize;
                                            < _ as crate::ZStructDecode>::z_decode(&mut < crate::ZReader as crate::ZReaderExt>::sub(r, #access)?)?
                                        } else {
                                            #default
                                        };
                                    });
                                }
                                _ => {
                                    body.push(quote::quote! {
                                        let #access = if #access {
                                            < _ as crate::ZStructDecode>::z_decode(r)?
                                        } else {
                                            #default
                                        };
                                    });
                                }
                            }
                        }
                        PresenceAttribute::Header(mask) => {
                            let default = match &attr.default {
                                DefaultAttribute::Expr(expr) => expr,
                                _ => unreachable!(
                                    "Fields with presence attribute must have a default attribute, this should have been caught earlier"
                                ),
                            };

                            body.push(quote::quote! {
                                let #access: bool = (header & #mask) != 0;
                            });

                            match &attr.size {
                                SizeAttribute::Prefixed => {
                                    body.push(quote::quote! {
                                        let #access = if #access {
                                            let #access = < usize as crate::ZStructDecode>::z_decode(r)?;
                                            < _ as crate::ZStructDecode>::z_decode(&mut < crate::ZReader as crate::ZReaderExt>::sub(r, #access)?)?
                                        } else {
                                            #default
                                        };
                                    });
                                }
                                SizeAttribute::Header(size_mask) => {
                                    let e: u8 = !(attr.maybe_empty) as u8;

                                    body.push(quote::quote! {
                                        let #access = if #access {
                                            let #access = (((header & #size_mask) >> #size_mask.trailing_zeros()) + #e) as usize;
                                            < _ as crate::ZStructDecode>::z_decode(&mut < crate::ZReader as crate::ZReaderExt>::sub(r, #access)?)?
                                        } else {
                                            #default
                                        };
                                    });
                                }
                                _ => {
                                    body.push(quote::quote! {
                                        let #access = if #access {
                                            < _ as crate::ZStructDecode>::z_decode(r)?
                                        } else {
                                            #default
                                        };
                                    });
                                }
                            }
                        }
                        PresenceAttribute::None => match &attr.size {
                            SizeAttribute::Prefixed => {
                                body.push(quote::quote! {
                                                let #access = < usize as crate::ZStructDecode>::z_decode(r)?;
                                                let #access = < _ as crate::ZStructDecode>::z_decode(&mut < crate::ZReader as crate::ZReaderExt>::sub(r, #access)?)?;
                                            });
                            }
                            SizeAttribute::Header(mask) => {
                                let e: u8 = !(attr.maybe_empty) as u8;
                                body.push(quote::quote! {
                                                let #access = (((header & #mask) >> #mask.trailing_zeros()) + #e) as usize;
                                                let #access = < _ as crate::ZStructDecode>::z_decode(&mut < crate::ZReader as crate::ZReaderExt>::sub(r, #access)?)?;
                                            });
                            }
                            _ => {
                                body.push(quote::quote! {
                                    let #access = < _ as crate::ZStructDecode>::z_decode(r)?;
                                });
                            }
                        },
                    },
                    ZenohType::Option(_) => {
                        match &attr.presence {
                            PresenceAttribute::Prefixed => {
                                body.push(quote::quote! {
                                    let #access: bool = <u8 as crate::ZStructDecode>::z_decode(r)? != 0;
                                });
                            }
                            PresenceAttribute::Header(mask) => {
                                body.push(quote::quote! {
                                    let #access: bool = (header & #mask) != 0;
                                });
                            }
                            _ => unreachable!(
                                "Option type must have a presence attribute, this was checked before"
                            ),
                        }

                        match &attr.size {
                            SizeAttribute::Prefixed => {
                                body.push(quote::quote! {
                                    let #access = if #access {
                                        let #access = < usize as crate::ZStructDecode>::z_decode(r)?;
                                        Some(< _ as crate::ZStructDecode>::z_decode(&mut < crate::ZReader as crate::ZReaderExt>::sub(r, #access)?)?)
                                    } else {
                                        None
                                    };
                                });
                            }
                            SizeAttribute::Header(mask) => {
                                let e: u8 = !(attr.maybe_empty) as u8;

                                body.push(quote::quote! {
                                    let #access = if #access {
                                            let #access = (((header & #mask) >> #mask.trailing_zeros()) + #e) as usize;
                                        Some(< _ as crate::ZStructDecode>::z_decode(&mut < crate::ZReader as crate::ZReaderExt>::sub(r, #access)?)?)
                                    } else {
                                        None
                                    };
                                });
                            }
                            _ => {
                                body.push(quote::quote! {
                                    let #access = if #access {
                                        Some(< _ as crate::ZStructDecode>::z_decode(r)?)
                                    } else {
                                        None
                                    };
                                });
                            }
                        }
                    }
                }
            }
            ZenohField::ExtBlock { exts } => {
                body.push(quote::quote! {
                    let mut has_ext: bool = header & Self::HEADER_SLOT_Z != 0;
                });

                let mut ext_body = Vec::<TokenStream>::new();

                for field in exts {
                    let access = &field.access;
                    let ty = &field.ty;
                    let attr = &field.attr;

                    declaration.push(quote::quote! {
                        #access
                    });

                    let id = match &attr.ext {
                        ExtAttribute::Expr(id) => id,
                        _ => unreachable!(
                            "ExtBlock fields must have an ext attribute, this should have been caught earlier"
                        ),
                    };

                    match ty {
                        ZenohType::ZStruct => {
                            let expr = match &attr.default {
                                DefaultAttribute::Expr(expr) => expr,
                                _ => unreachable!(
                                    "ExtBlock fields ZStruct must have a default attribute, this should have been caught earlier"
                                ),
                            };

                            body.push(quote::quote! {
                                let mut #access = #expr;
                            });

                            ext_body.push(quote::quote! {
                                #id => {
                                    #access = crate::zext_decode::< _ >(r)?;
                                }
                            });
                        }
                        ZenohType::Option(_) => {
                            body.push(quote::quote! {
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

                body.push(quote::quote! {
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
        #(#body)*

        Ok(Self { #(#declaration),* })
    })
}
