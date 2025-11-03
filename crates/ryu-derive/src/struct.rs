use proc_macro2::TokenStream;
use syn::DeriveInput;

use crate::r#struct::parse::ZStruct;

pub(crate) mod parse;

pub(crate) mod decode;
pub(crate) mod encode;
pub(crate) mod flag;
pub(crate) mod header;
pub(crate) mod len;

pub fn derive_zstruct(input: DeriveInput) -> TokenStream {
    let ident = &input.ident;
    let data = &input.data;

    let mut lt_params = false;
    for generic in input.generics.params.iter() {
        match generic {
            syn::GenericParam::Lifetime(_) => {
                if lt_params {
                    panic!("ZStruct can only have one lifetime parameter.");
                }

                lt_params = true;
            }
            _ => {
                panic!("ZStruct can only have a lifetime parameter.");
            }
        }
    }

    let (ty_elided, ty_lt) = if lt_params {
        (quote::quote! { <'_> }, quote::quote! { <'a> })
    } else {
        (quote::quote! {}, quote::quote! {})
    };

    let r#struct = ZStruct::from_data(data);

    let (header_enc, header_dec) = header::parse_body(&r#struct);
    let (flag_enc, flag_dec) = flag::parse_body(&r#struct);

    let len_body = len::parse_body(&r#struct);
    let encode_body = encode::parse_body(&r#struct, flag_enc);
    let decode_body = decode::parse_body(&r#struct, flag_dec);

    quote::quote! {
        impl crate::ZStruct for #ident #ty_elided {
            fn z_len(&self) -> usize {
                #len_body
            }

            fn z_encode(&self, w: &mut crate::ByteWriter) -> crate::io::ByteIOResult<()> {
                #header_enc

                #encode_body
            }

            type ZType<'a> = #ident #ty_lt;

            fn z_decode<'a>(r: &mut crate::ByteReader<'a>) -> crate::io::ByteIOResult<Self::ZType<'a>> {
                #header_dec

                #decode_body
            }
        }
    }
}
