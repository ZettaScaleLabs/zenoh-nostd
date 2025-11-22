use proc_macro2::TokenStream;
use syn::DeriveInput;

use crate::codec::model::{
    ZenohStruct,
    attribute::{DefaultAttribute, ExtAttribute, PresenceAttribute, ZenohAttribute},
};

pub mod header_impl;

pub mod decode;
pub mod encode;
pub mod ext_count;
pub mod header;
pub mod len;

pub fn derive_zstruct(input: &DeriveInput) -> syn::Result<TokenStream> {
    let r#struct = ZenohStruct::from_derive_input(input)?;
    let ident = &r#struct.ident;

    let generics = &r#struct.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let header_impl = header_impl::parse(&r#struct)?;
    let (header, h) = header::parse(&r#struct)?;

    let (header, ctx, ctx_p) = if h {
        (
            quote::quote! {
                impl #impl_generics crate::ZHeader for #ident #ty_generics #where_clause {
                    fn z_header(&self) -> u8 {
                        #header
                    }
                }
            },
            quote::quote! { u8 },
            quote::quote! { header: u8 },
        )
    } else {
        (header, quote::quote! { () }, quote::quote! { _: () })
    };

    let (ext_count, e) = ext_count::parse(&r#struct)?;
    let ext_count = if e {
        quote::quote! {
            impl #impl_generics crate::ZExtCount for #ident #ty_generics #where_clause {
                fn z_ext_count(&self) -> usize {
                    #ext_count
                }
            }
        }
    } else {
        quote::quote! {}
    };

    let (len_body, len) = len::parse(&r#struct)?;
    let (encode_body, encode) = encode::parse(&r#struct)?;
    let (decode_body, decode) = decode::parse(&r#struct)?;

    Ok(quote::quote! {
        #header_impl
        #header

        #ext_count

        impl #impl_generics crate::ZBodyLen for #ident #ty_generics #where_clause {
            fn z_body_len(&self) -> usize {
                #len_body
            }
        }

        impl #impl_generics crate::ZLen for #ident #ty_generics #where_clause {
            fn z_len(&self) -> usize {
                #len
            }
        }

        impl #impl_generics crate::ZBodyEncode for #ident #ty_generics #where_clause {
            fn z_body_encode(&self, w: &mut crate::ZWriter) -> crate::ZCodecResult<()> {
                #encode_body

                Ok(())
            }
        }

        impl #impl_generics crate::ZEncode for #ident #ty_generics #where_clause {
            fn z_encode(&self, w: &mut crate::ZWriter) -> crate::ZCodecResult<()> {
                #encode

                Ok(())
            }
        }

        impl<'a> crate::ZBodyDecode<'a> for #ident #ty_generics #where_clause {
            type Ctx = #ctx;

            fn z_body_decode(r: &mut crate::ZReader<'a>, #ctx_p) -> crate::ZCodecResult<Self> {
                #decode_body
            }
        }

        impl<'a> crate::ZDecode<'a> for #ident #ty_generics #where_clause {
            fn z_decode(r: &mut crate::ZReader<'a>) -> crate::ZCodecResult<Self> {
                #decode
            }
        }
    })
}

#[allow(clippy::nonminimal_bool)]
pub fn enc_len_modifier(
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

pub fn dec_modifier(
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
