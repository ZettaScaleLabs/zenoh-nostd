use proc_macro2::TokenStream;
use syn::DeriveInput;

use crate::model::ZenohStruct;

pub mod header;
pub mod r#struct;

pub fn derive_zstruct(input: DeriveInput) -> syn::Result<TokenStream> {
    let r#struct = ZenohStruct::from_derive_input(&input)?;
    let ident = &r#struct.ident;

    let generics = &r#struct.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let header = header::parse(&r#struct)?;

    let (len, encode, decode) = r#struct::parse(&r#struct)?;

    Ok(quote::quote! {
        #header

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
