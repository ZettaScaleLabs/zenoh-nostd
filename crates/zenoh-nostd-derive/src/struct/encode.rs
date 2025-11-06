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
    let mut header = Vec::<TokenStream>::new();

    if r#struct.header.is_some() {
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

                // Lots of checks have been made in the `ty.rs` file so you can merge lots of cases without worrying
                // about invalid combinations
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
                                    "Fields with presence prefixed must have a default attribute, this should have been caught earlier"
                                ),
                            };

                            body.push(quote::quote! {
                                <u8 as crate::ZStructEncode>::z_encode(&((&self. #access  != &#default) as u8), w)?;
                            });

                            match &attr.size {
                                SizeAttribute::Prefixed => {
                                    body.push(quote::quote! {
                                        if &self. #access  != &#default {
                                            <usize as crate::ZStructEncode>::z_encode(&< _ as crate::ZStructEncode>::z_len(&self. #access), w)?;
                                        }
                                    });
                                }
                                SizeAttribute::Header(mask) => {
                                    let e: u8 = (!attr.maybe_empty) as u8;
                                    header.push(quote::quote! {
                                        if &self. #access  != &#default {
                                            header |= {
                                                let shift = #mask .trailing_zeros();
                                                let len = < _ as crate::ZStructEncode>::z_len(&self. #access) as u8;

                                                ((len - #e) << shift) & #mask
                                            };
                                        }
                                    });
                                }
                                _ => {}
                            }

                            body.push(quote::quote! {
                                if &self. #access  != &#default {
                                    < _ as crate::ZStructEncode>::z_encode(&self. #access, w)?;
                                }
                            });
                        }
                        PresenceAttribute::Header(mask) => {
                            let default = match &attr.default {
                                DefaultAttribute::Expr(expr) => expr,
                                _ => unreachable!(
                                    "Fields with presence in header must have a default attribute, this should have been caught earlier"
                                ),
                            };

                            header.push(quote::quote! {
                                if &self. #access  != &#default {
                                    header |= #mask;
                                }
                            });

                            match &attr.size {
                                SizeAttribute::Prefixed => {
                                    body.push(quote::quote! {
                                        if &self. #access  != &#default {
                                            <usize as crate::ZStructEncode>::z_encode(&< _ as crate::ZStructEncode>::z_len(&self. #access), w)?;
                                        }
                                    });
                                }
                                SizeAttribute::Header(mask) => {
                                    let e: u8 = (!attr.maybe_empty) as u8;
                                    header.push(quote::quote! {
                                        if &self. #access  != &#default {
                                            header |= {
                                                let shift = #mask .trailing_zeros();
                                                let len = < _ as crate::ZStructEncode>::z_len(&self. #access) as u8;

                                                ((len - #e) << shift) & #mask
                                            };
                                        }
                                    });
                                }
                                _ => {}
                            }

                            body.push(quote::quote! {
                                if &self. #access  != &#default {
                                    < _ as crate::ZStructEncode>::z_encode(&self. #access, w)?;
                                }
                            });
                        }
                        PresenceAttribute::None => {
                            match &attr.size {
                                SizeAttribute::Prefixed => {
                                    body.push(quote::quote! {
                                            <usize as crate::ZStructEncode>::z_encode(&< _ as crate::ZStructEncode>::z_len(&self. #access), w)?;
                                        });
                                }
                                SizeAttribute::Header(mask) => {
                                    let e: u8 = (!attr.maybe_empty) as u8;
                                    header.push(quote::quote! {
                                            header |= {
                                                let shift = #mask .trailing_zeros();
                                                let len = < _ as crate::ZStructEncode>::z_len(&self. #access) as u8;

                                                ((len - #e) << shift) & #mask
                                            };
                                        });
                                }
                                _ => {}
                            }

                            body.push(quote::quote! {
                                < _ as crate::ZStructEncode>::z_encode(&self. #access, w)?;
                            });
                        }
                    },
                    ZenohType::Option(_) => {
                        match &attr.presence {
                            PresenceAttribute::Prefixed => {
                                body.push(quote::quote! {
                                    <u8 as crate::ZStructEncode>::z_encode(&(self. #access.is_some() as u8), w)?;
                                });
                            }
                            PresenceAttribute::Header(mask) => {
                                header.push(quote::quote! {
                                    if self. #access .is_some() {
                                        header |= #mask;
                                    }
                                });
                            }
                            _ => {}
                        }

                        match &attr.size {
                            SizeAttribute::Prefixed => {
                                body.push(quote::quote! {
                                    if let Some(inner) = &self. #access {
                                        <usize as crate::ZStructEncode>::z_encode(&< _ as crate::ZStructEncode>::z_len(inner), w)?;
                                    }
                                });
                            }
                            SizeAttribute::Header(mask) => {
                                let e: u8 = (!attr.maybe_empty) as u8;
                                header.push(quote::quote! {
                                    if let Some(inner) = &self. #access {
                                        header |= {
                                            let shift = #mask .trailing_zeros();
                                            let len = < _ as crate::ZStructEncode>::z_len(inner) as u8;

                                            ((len - #e) << shift) & #mask
                                        };
                                    }
                                });
                            }
                            _ => {}
                        }

                        body.push(quote::quote! {
                            if let Some(inner) = &self. #access {
                                < _ as crate::ZStructEncode>::z_encode(inner, w)?;
                            }
                        });
                    }
                }
            }
            ZenohField::ExtBlock { exts } => {
                header.push(quote::quote! {
                    let mut n_exts = 0;
                });

                let mut enc_ext = Vec::<TokenStream>::new();

                for field in exts {
                    let access = &field.access;
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

                    match ty {
                        ZenohType::ZStruct => {
                            let expr = match &attr.default {
                                DefaultAttribute::Expr(expr) => expr,
                                _ => unreachable!(
                                    "ExtBlock fields ZStruct must have a default attribute, this should have been caught earlier"
                                ),
                            };

                            header.push(quote::quote! {
                                if &self. #access  != &#expr {
                                    n_exts += 1;
                                }
                            });

                            enc_ext.push(quote::quote! {
                                if &self. #access  != &#expr {
                                    n_exts -= 1;
                                    crate::zext_encode::<_, #id, #mandatory>(&self. #access, w, n_exts != 0)?;
                                }
                            });
                        }
                        ZenohType::Option(_) => {
                            header.push(quote::quote! {
                                if self. #access .is_some() {
                                    n_exts += 1;
                                }
                            });

                            enc_ext.push(quote::quote! {
                                if let Some(inner) = &self. #access {
                                    n_exts -= 1;
                                    crate::zext_encode::<_, #id, #mandatory>(inner, w, n_exts != 0)?;
                                }
                            });
                        }
                        _ => unreachable!(
                            "Only ZStruct and Option<ZStruct> are allowed in ext blocks, this should have been caught earlier"
                        ),
                    }
                }

                header.push(quote::quote! {
                    if n_exts > 0 {
                        header |= Self::HEADER_SLOT_Z;
                    }
                });

                body.push(quote::quote! {
                    #(#enc_ext)*
                });
            }
        }
    }

    if r#struct.header.is_some() {
        body.insert(
            0,
            quote::quote! {
                <u8 as crate::ZStructEncode>::z_encode(&header, w)?;
            },
        );
    }

    Ok(quote::quote! {
        #(#header)*

        #(#body)*
    })
}
