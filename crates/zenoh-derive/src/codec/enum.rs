use proc_macro2::TokenStream;
use syn::DeriveInput;

pub fn derive_zenum(input: &DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let variants = match &input.data {
        syn::Data::Enum(data_enum) => &data_enum.variants,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "ZEnum can only be derived for enums",
            ));
        }
    }
    .iter()
    .map(|variant| &variant.ident);

    let body_len = variants.clone().map(|variant| {
        quote::quote! {
            Self::#variant(x) => <#variant as crate::ZBodyLen>::z_body_len(x),
        }
    });

    let len = quote::quote! { 1 + <Self as crate::ZBodyLen>::z_body_len(self) };

    let body_encode = variants.clone().map(|variant| {
        quote::quote! {
            Self::#variant(x) => <#variant as crate::ZBodyEncode>::z_body_encode(x, w),
        }
    });

    let encode = variants.clone().map(|variant| {
        quote::quote! {
            Self::#variant(x) => <#variant as crate::ZEncode>::z_encode(x, w),
        }
    });

    let body_decode = variants.clone().map(|variant| {
        quote::quote! {
            <#variant>::ID => Ok(Self::#variant(<#variant as crate::ZBodyDecode>::z_body_decode(r, header)?)),
        }
    });

    let decode = quote::quote! {
        let header = <u8 as crate::ZDecode>::z_decode(r)?;
        <Self as crate::ZBodyDecode>::z_body_decode(r, header)
    };

    Ok(quote::quote! {
        impl #impl_generics crate::ZBodyLen for #ident #ty_generics #where_clause {
            fn z_body_len(&self) -> usize {
                match self {
                    #(#body_len)*
                }
            }
        }

        impl #impl_generics crate::ZLen for #ident #ty_generics #where_clause {
            fn z_len(&self) -> usize {
                #len
            }
        }

        impl #impl_generics crate::ZBodyEncode for #ident #ty_generics #where_clause {
            fn z_body_encode(&self, w: &mut impl crate::ZWrite) -> core::result::Result<(), crate::CodecError> {
                match self {
                    #(#body_encode)*
                }
            }
        }

        impl #impl_generics crate::ZEncode for #ident #ty_generics #where_clause {
            fn z_encode(&self, w: &mut impl crate::ZWrite) -> core::result::Result<(), crate::CodecError> {
                match self {
                    #(#encode)*
                }
            }
        }

        impl<'a> crate::ZBodyDecode<'a> for #ident #ty_generics #where_clause {
            type Ctx = u8;

            fn z_body_decode(r: &mut impl crate::ZRead<'a>, header: u8) -> core::result::Result<Self, crate::CodecError> {
                let id = header & 0b0001_1111;
                match id {
                    #(#body_decode)*
                    _ => Err(crate::CodecError::CouldNotParseHeader),
                }
            }
        }

        impl<'a> crate::ZDecode<'a> for #ident #ty_generics #where_clause {
            fn z_decode(r: &mut impl crate::ZRead<'a>) -> core::result::Result<Self, crate::CodecError> {
                #decode
            }
        }
    })
}
