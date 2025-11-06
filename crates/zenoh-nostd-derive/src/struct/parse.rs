use std::panic;

use proc_macro2::TokenStream;
use syn::{
    AngleBracketedGenericArguments, Attribute, Data, DataStruct, Expr, Field, Fields,
    GenericArgument, LitInt, Path, PathArguments, Type, meta::ParseNestedMeta,
};

#[derive(PartialEq)]
pub(crate) enum ZSizeFlavour {
    Plain,
    Deduced,
    NonEmptyFlag(u8),
    MaybeEmptyFlag(u8),
    None,
}

#[derive(Clone)]
pub(crate) enum ZPresenceFlavour {
    Flag,
    Plain,
    Header(Expr),
}

pub(crate) enum ZStructFlavour {
    Option {
        presence: ZPresenceFlavour,
        size: ZSizeFlavour,
    },
    Size(ZSizeFlavour),
}

pub(crate) enum ZHStorageFlavour {
    Value(Expr),
    U8 { mask: Expr, shift: Expr },
}

pub(crate) struct ZStructKind {
    pub flavour: ZStructFlavour,
    pub ty: TokenStream,
}

pub(crate) struct ZExtKind {
    pub ty: TokenStream,
    pub access: TokenStream,
}

pub(crate) enum ZFieldKind {
    Header,
    Flag,

    HeaderStorage {
        flavour: ZHStorageFlavour,
        ty: TokenStream,
    },

    ZExtBlock {
        flavour: ZPresenceFlavour,
        exts: Vec<ZExtKind>,
    },
    ZExtBlockEnd,
    ZStruct(ZStructKind),
}

pub(crate) struct ZField {
    pub kind: ZFieldKind,
    pub access: TokenStream,
}

pub struct ZStruct(pub Vec<ZField>);

impl ZSizeFlavour {
    fn from_meta(meta: &ParseNestedMeta, flavour: &mut Option<ZSizeFlavour>) -> syn::Result<()> {
        if meta.path.is_ident("plain") {
            flavour.replace(ZSizeFlavour::Plain);
        } else if meta.path.is_ident("deduced") {
            flavour.replace(ZSizeFlavour::Deduced);
        } else if meta.path.is_ident("flag") {
            let value = meta.value().expect("Expected value for flag flavour");
            let lit: LitInt = value.parse()?;
            let flag_index = lit.base10_parse::<u8>()?;

            flavour.replace(ZSizeFlavour::NonEmptyFlag(flag_index));
        } else if meta.path.is_ident("eflag") {
            let value = meta.value().expect("Expected value for eflag flavour");
            let lit: LitInt = value.parse()?;
            let flag_index = lit.base10_parse::<u8>()?;

            flavour.replace(ZSizeFlavour::MaybeEmptyFlag(flag_index));
        } else if meta.path.is_ident("none") {
            flavour.replace(ZSizeFlavour::None);
        }

        Ok(())
    }
}

impl ZPresenceFlavour {
    fn from_meta(
        meta: &ParseNestedMeta,
        flavour: &mut Option<ZPresenceFlavour>,
    ) -> syn::Result<()> {
        if meta.path.is_ident("flag") {
            flavour.replace(ZPresenceFlavour::Flag);
        } else if meta.path.is_ident("plain") {
            flavour.replace(ZPresenceFlavour::Plain);
        } else if meta.path.is_ident("header") {
            let value = meta.value().expect("Expected value for header flavour");
            let expr: Expr = value.parse()?;

            flavour.replace(ZPresenceFlavour::Header(expr));
        }

        Ok(())
    }
}

impl ZStructFlavour {
    fn from_attr(attr: &Attribute) -> ZStructFlavour {
        let mut struct_attr = Option::<ZStructFlavour>::None;

        if attr.path().is_ident("option") {
            let mut presence_flavour = Option::<ZPresenceFlavour>::None;
            let mut size_flavour = Option::<ZSizeFlavour>::None;

            attr.parse_nested_meta(|meta| {
                ZPresenceFlavour::from_meta(&meta, &mut presence_flavour)?;

                if meta.path.is_ident("size") {
                    meta.parse_nested_meta(|size_meta| {
                        ZSizeFlavour::from_meta(&size_meta, &mut size_flavour)
                    })?;
                }

                Ok(())
            })
            .expect("Failed to parse struct attribute");

            struct_attr.replace(ZStructFlavour::Option {
                presence: presence_flavour
                    .expect("Option struct expected to have a presence flavour"),
                size: size_flavour.unwrap_or(ZSizeFlavour::None),
            });
        } else if attr.path().is_ident("size") {
            let mut size_flavour = Option::<ZSizeFlavour>::None;

            attr.parse_nested_meta(|meta| ZSizeFlavour::from_meta(&meta, &mut size_flavour))
                .expect("Failed to parse struct attribute");

            struct_attr.replace(ZStructFlavour::Size(
                size_flavour.expect("Struct expected to have a size flavour"),
            ));
        }

        struct_attr.expect("Struct expected to have either option or size attribute")
    }
}

impl ZHStorageFlavour {
    fn from_attr(attr: &Attribute) -> ZHStorageFlavour {
        if attr.path().is_ident("hstore") {
            let mut value_expr = Option::<Expr>::None;
            let mut mask_expr = Option::<Expr>::None;
            let mut shift_expr = Option::<Expr>::None;

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("value") {
                    let value = meta.value().expect("Expected value for hstore value");
                    let expr: Expr = value.parse()?;
                    value_expr.replace(expr);
                } else if meta.path.is_ident("mask") {
                    let value = meta.value().expect("Expected value for hstore mask");
                    let expr: Expr = value.parse()?;
                    mask_expr.replace(expr);
                } else if meta.path.is_ident("shift") {
                    let value = meta.value().expect("Expected value for hstore shift");
                    let expr: Expr = value.parse()?;
                    shift_expr.replace(expr);
                }

                Ok(())
            })
            .expect("Failed to parse HeaderStorage attribute");

            if let Some(expr) = value_expr {
                ZHStorageFlavour::Value(expr)
            } else if let (Some(mask), Some(shift)) = (mask_expr, shift_expr) {
                ZHStorageFlavour::U8 { mask, shift }
            } else {
                panic!(
                    "HeaderStorage must have either a value expression or both mask and shift expressions"
                );
            }
        } else {
            panic!("Expected hstore attribute for HeaderStorage field");
        }
    }
}

fn ty_to_ext_path(ty: &Type) -> TokenStream {
    match ty {
        Type::Path(tp) => {
            let tp = tp
                .path
                .segments
                .first()
                .expect("Expected type path segment");
            if tp.ident != "Option" {
                panic!("Extension type must be Option<T>");
            }

            match &tp.arguments {
                PathArguments::AngleBracketed(ab) => {
                    let ab = ab.args.first().expect("Expected generic argument");
                    match ab {
                        GenericArgument::Type(ty) => match ty {
                            Type::Path(tp) => {
                                let path = remove_lt_from_path(tp.path.clone());
                                quote::quote! { #path }
                            }
                            _ => panic!("Extension type must be a path"),
                        },
                        _ => panic!("Extension type must be Option<T>"),
                    }
                }
                _ => panic!("Extension type must be Option<T>"),
            }
        }
        _ => panic!("Extension type must be a path"),
    }
}

fn remove_lt_from_path(mut path: Path) -> Path {
    match &mut path.segments.last_mut().unwrap().arguments {
        PathArguments::None => path,
        PathArguments::Parenthesized(_) => panic!("Parenthesized arguments are not supported"),
        PathArguments::AngleBracketed(aba) => {
            let mut new_args = AngleBracketedGenericArguments {
                colon2_token: aba.colon2_token,
                lt_token: aba.lt_token,
                args: syn::punctuated::Punctuated::new(),
                gt_token: aba.gt_token,
            };

            for arg in &aba.args {
                if let GenericArgument::Type(ty) = arg {
                    match ty {
                        Type::Reference(tr) => {
                            let mut new_tr = tr.clone();
                            new_tr.lifetime = None;
                            new_tr.mutability = None;
                            new_args
                                .args
                                .push(GenericArgument::Type(Type::Reference(new_tr)));
                        }
                        Type::Path(path) => {
                            let new_path = remove_lt_from_path(path.path.clone());

                            new_args
                                .args
                                .push(GenericArgument::Type(Type::Path(syn::TypePath {
                                    qself: None,
                                    path: new_path,
                                })));
                        }
                        _ => {
                            new_args.args.push(arg.clone());
                        }
                    }
                }
            }

            path.segments.last_mut().unwrap().arguments = PathArguments::AngleBracketed(new_args);
            path
        }
    }
}

impl ZField {
    fn from_field(field: &Field) -> ZField {
        let ty = &field.ty;
        let attrs = &field.attrs;
        let access = match &field.ident {
            Some(ident) => quote::quote! { #ident },
            None => {
                panic!("ZStruct fields must be named");
            }
        };

        if let syn::Type::Path(tp) = ty {
            let path = tp.path.segments.last().expect("Expected type path segment");

            if path.ident == "Flag" {
                return ZField {
                    kind: ZFieldKind::Flag,
                    access,
                };
            } else if path.ident == "Header" {
                return ZField {
                    kind: ZFieldKind::Header,
                    access,
                };
            } else if path.ident == "ExtBlockBegin" {
                let attr = attrs
                    .iter()
                    .find(|a| a.path().is_ident("option"))
                    .expect("ZExtBlockBegin must have option attribute");

                let mut presence = Option::<ZPresenceFlavour>::None;
                attr.parse_nested_meta(|meta| {
                    ZPresenceFlavour::from_meta(&meta, &mut presence)
                        .expect("Failed to parse presence flavour");
                    Ok(())
                })
                .expect("Failed to parse ZExtBlockBegin attribute");

                return ZField {
                    kind: ZFieldKind::ZExtBlock {
                        flavour: presence.expect("ZExtBlockBegin must have a presence flavour"),
                        exts: Vec::new(),
                    },
                    access,
                };
            } else if path.ident == "ExtBlockEnd" {
                return ZField {
                    kind: ZFieldKind::ZExtBlockEnd,
                    access,
                };
            }
        }

        let ty = match ty {
            Type::Array(ty) => {
                let len = &ty.len;
                quote::quote! {
                    [u8; #len]
                }
            }
            Type::Reference(ty) => {
                let mut ty = ty.clone();
                ty.lifetime = None;
                ty.mutability = None;
                quote::quote! {
                    #ty
                }
            }
            Type::Path(ty) => {
                let path = remove_lt_from_path(ty.path.clone());

                quote::quote! {
                    #path
                }
            }
            _ => panic!("Unsupported field type in ZStruct"),
        };

        if let Some(attr) = attrs.iter().find(|a| a.path().is_ident("hstore")) {
            let flavour = ZHStorageFlavour::from_attr(attr);
            return ZField {
                kind: ZFieldKind::HeaderStorage { flavour, ty },
                access,
            };
        }

        let attr = attrs
            .iter()
            .find(|a| a.path().is_ident("option") || a.path().is_ident("size"))
            .map(ZStructFlavour::from_attr)
            .unwrap_or(ZStructFlavour::Size(ZSizeFlavour::None));

        ZField {
            kind: ZFieldKind::ZStruct(ZStructKind { flavour: attr, ty }),
            access,
        }
    }
}

impl ZStruct {
    fn from_fields<'a>(fields: impl Iterator<Item = &'a Field>) -> ZStruct {
        let mut parsed_fields = Vec::<ZField>::new();
        let mut is_deduced = false;
        let mut flag = false;
        let mut header = false;
        let mut total_flag_bits = 0u8;

        let mut ext_block = false;

        for field in fields {
            if is_deduced {
                panic!("Deduced size flavour must appear once at the end of the struct");
            }

            let zfield = ZField::from_field(field);

            match &zfield.kind {
                ZFieldKind::Header => {
                    if header {
                        panic!("Only one header field is supported per struct");
                    }

                    if !parsed_fields.is_empty() {
                        panic!("Header field must be defined at the beginning of the struct");
                    }

                    header = true;
                }
                ZFieldKind::HeaderStorage { .. } => {
                    if !header {
                        panic!("HStorage field must be defined after the header field");
                    }
                }
                ZFieldKind::Flag => {
                    if flag {
                        panic!("Only one Flag field is supported per struct");
                    }
                    flag = true;
                }
                ZFieldKind::ZExtBlock { .. } => {
                    if ext_block {
                        panic!("Nested ZExtBlockBegin is not supported");
                    }
                    ext_block = true;
                }
                ZFieldKind::ZExtBlockEnd => {
                    if !ext_block {
                        panic!("ZExtBlockEnd found without a matching ZExtBlockBegin");
                    }
                    ext_block = false;
                }
                ZFieldKind::ZStruct(ZStructKind { flavour, .. }) => {
                    if ext_block {
                        if !matches!(flavour, ZStructFlavour::Size(ZSizeFlavour::None)) {
                            panic!("Fields inside ZExtBlock must have no size or option flavour");
                        }

                        let kind = &mut parsed_fields.last_mut().unwrap().kind;
                        match kind {
                            ZFieldKind::ZExtBlock { exts, .. } => {
                                let ty = ty_to_ext_path(&field.ty);
                                let access = zfield.access.clone();
                                exts.push(ZExtKind { ty, access });
                            }
                            _ => panic!("Expected ZExtBlock kind"),
                        }

                        continue;
                    } else {
                        if let ZStructFlavour::Option {
                            presence: ZPresenceFlavour::Flag,
                            ..
                        } = flavour
                        {
                            if !flag {
                                panic!(
                                    "Flag field must be defined before using flag presence flavour"
                                );
                            }

                            total_flag_bits += 1;
                        }

                        match flavour {
                            ZStructFlavour::Size(flavour)
                            | ZStructFlavour::Option { size: flavour, .. } => match flavour {
                                ZSizeFlavour::Deduced => {
                                    is_deduced = true;
                                }
                                ZSizeFlavour::NonEmptyFlag(size)
                                | ZSizeFlavour::MaybeEmptyFlag(size) => {
                                    if !flag {
                                        panic!(
                                            "Flag field must be defined before using flag size flavours"
                                        );
                                    }
                                    total_flag_bits += *size;
                                }
                                _ => {}
                            },
                        }
                    }
                }
            }

            parsed_fields.push(zfield);
        }

        if ext_block {
            panic!("ZExtBlockBegin without a matching ZExtBlockEnd");
        }

        if total_flag_bits > 8 {
            panic!("Total flag bits used in struct exceed 8 bits");
        }

        ZStruct(parsed_fields)
    }

    pub fn from_data(data: &Data) -> ZStruct {
        match data {
            Data::Struct(DataStruct { fields, .. }) => match fields {
                Fields::Named(named) => Self::from_fields(named.named.iter()),
                _ => panic!("ZStruct only supports named fields"),
            },
            _ => panic!("infer_kind only supports structs"),
        }
    }
}
