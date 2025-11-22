use proc_macro2::TokenStream;
use syn::{Generics, Ident, LitStr};

use crate::codec::model::{
    attribute::{ExtAttribute, ZenohAttribute},
    ty::ZenohType,
};

pub mod attribute;
pub mod ty;

pub struct ZenohFieldInner {
    pub attr: ZenohAttribute,
    pub ty: ZenohType,
    pub access: TokenStream,
}

impl ZenohFieldInner {
    pub fn from_field(field: &syn::Field) -> syn::Result<Self> {
        let attr = ZenohAttribute::from_field(field)?;

        let ident = field
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new_spanned(field, "Expected named field"))?;

        let access = quote::quote! { #ident };

        let ty = ZenohType::from_type(&field.ty)?;
        ty.check_attribute(&attr)?;

        Ok(Self { attr, access, ty })
    }
}

pub struct HeaderDeclaration {
    pub expr: LitStr,
}

pub enum ZenohField {
    Regular { field: Box<ZenohFieldInner> },
    ExtBlock { exts: Vec<ZenohFieldInner> },
}

pub struct ZenohStruct {
    pub ident: Ident,
    pub generics: Generics,
    pub header: Option<HeaderDeclaration>,
    pub fields: Vec<ZenohField>,
}

impl ZenohStruct {
    pub fn from_derive_input(input: &syn::DeriveInput) -> syn::Result<Self> {
        let fields = match &input.data {
            syn::Data::Struct(data_struct) => &data_struct.fields,
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "ZStruct can only be derived for structs",
                ));
            }
        };

        let mut fields_vec = Vec::new();
        let mut found_ext_block = false;
        let mut in_ext_block = false;
        for field in fields {
            let field = ZenohFieldInner::from_field(field)?;
            let is_ext = !matches!(field.attr.ext, ExtAttribute::None);
            if is_ext {
                if !found_ext_block {
                    found_ext_block = true;
                    in_ext_block = true;
                    fields_vec.push(ZenohField::ExtBlock { exts: vec![] });
                } else if !in_ext_block {
                    return Err(syn::Error::new_spanned(
                        field.access.clone(),
                        "Fields with 'ext' attribute must be grouped in a single contiguous block",
                    ));
                }
            } else {
                in_ext_block = false;
            }

            if is_ext {
                match fields_vec
                    .last_mut()
                    .expect("Expected ext block, something went wrong")
                {
                    ZenohField::ExtBlock { exts } => {
                        exts.push(field);
                    }
                    _ => unreachable!("Expected ext block, something went wrong"),
                }
            } else {
                fields_vec.push(ZenohField::Regular {
                    field: Box::new(field),
                });
            }
        }

        let mut header = Option::<HeaderDeclaration>::None;

        for attr in &input.attrs {
            if attr.path().is_ident("zenoh") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("header") {
                        let value = meta.value()?;
                        let expr: LitStr = value.parse()?;
                        header.replace(HeaderDeclaration { expr });
                    }

                    Ok(())
                })?;
            }
        }

        Ok(Self {
            ident: input.ident.clone(),
            generics: input.generics.clone(),
            header,
            fields: fields_vec,
        })
    }
}
