use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::DeriveInput;

use crate::{
    model::{ZenohField, ZenohStruct, ty::ZenohType},
    r#struct::{decode, encode, header, len},
};

mod u64_decode;
mod u64_encode;
mod u64_len;

pub fn derive_zext(input: DeriveInput) -> syn::Result<TokenStream> {
    let r#struct = ZenohStruct::from_derive_input(&input)?;
    let ident = &r#struct.ident;

    let generics = &r#struct.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let kind = infer_kind(&r#struct)?;
    if matches!(kind, InferredKind::U64) {
        let len = u64_len::parse(&r#struct);
        let encode = u64_encode::parse(&r#struct);
        let decode = u64_decode::parse(&r#struct);

        return Ok(quote::quote! {
            impl<'a> crate::ZExt<'a> for #ident #ty_generics #where_clause {
                const KIND: crate::ZExtKind = #kind;
            }

            impl #impl_generics crate::ZStructEncode for #ident #ty_generics #where_clause {
                fn z_len(&self) -> usize {
                    #len
                }

                fn z_encode(&self, w: &mut crate::ZWriter) -> crate::ZCodecResult<()> {
                    #encode

                    Ok(())
                }
            }

            impl<'a> crate::ZStructDecode<'a> for #ident #ty_generics #where_clause {
                fn z_decode(r: &mut crate::ZReader<'a>) -> crate::ZCodecResult<Self> {
                    #decode
                }
            }
        });
    }

    let header = header::parse(&r#struct)?;

    let len = len::parse(&r#struct)?;
    let encode = encode::parse(&r#struct)?;
    let decode = decode::parse(&r#struct)?;

    Ok(quote::quote! {
        #header

        impl<'a> crate::ZExt<'a> for #ident #ty_generics #where_clause {
            const KIND: crate::ZExtKind = #kind;
        }

        impl #impl_generics crate::ZStructEncode for #ident #ty_generics #where_clause {
            fn z_len(&self) -> usize {
                #len
            }

            fn z_encode(&self, w: &mut crate::ZWriter) -> crate::ZCodecResult<()> {
                #encode

                Ok(())
            }
        }

        impl<'a> crate::ZStructDecode<'a> for #ident #ty_generics #where_clause {
            fn z_decode(r: &mut crate::ZReader<'a>) -> crate::ZCodecResult<Self> {
                #decode
            }
        }
    })
}

enum InferredKind {
    Unit,
    U64,
    ZStruct,
}

impl ToTokens for InferredKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let kind_token = match self {
            InferredKind::Unit => quote::quote! { crate::ZExtKind::Unit },
            InferredKind::U64 => quote::quote! { crate::ZExtKind::U64 },
            InferredKind::ZStruct => quote::quote! { crate::ZExtKind::ZStruct },
        };

        tokens.extend(kind_token);
    }
}

fn infer_kind(ext: &ZenohStruct) -> syn::Result<InferredKind> {
    if ext.fields.is_empty() {
        Ok(InferredKind::Unit)
    } else if ext.fields.len() == 1 {
        let field = &ext.fields.first().unwrap();

        match field {
            ZenohField::ExtBlock { .. } => Err(syn::Error::new(
                Span::call_site(),
                "Cannot infer ZExtKind from only one ext block field",
            )),
            ZenohField::Regular { field } => match field.ty {
                ZenohType::U8
                | ZenohType::U16
                | ZenohType::U32
                | ZenohType::U64
                | ZenohType::USize => Ok(InferredKind::U64),
                _ => Ok(InferredKind::ZStruct),
            },
        }
    } else {
        Ok(InferredKind::ZStruct)
    }
}
