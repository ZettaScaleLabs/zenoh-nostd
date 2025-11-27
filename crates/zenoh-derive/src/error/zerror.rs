use proc_macro2::TokenStream;

use crate::error::model::DeclaredErrors;

pub fn declare_zerror(input: &DeclaredErrors) -> TokenStream {
    let variants_declare = input.values().flat_map(|error_enum| {
        error_enum.variants.iter().map(|variant| {
            let name = &variant.name;
            let code = variant.code;
            let doc = &variant.doc;

            quote::quote! {
                #[doc = #doc]
                #name = #code,
            }
        })
    });

    let variants_display = input.values().flat_map(|error_enum| {
        error_enum.variants.iter().map(|variant| {
            let ename = &error_enum.name;
            let name = &variant.name;
            let err = &variant.err;
            let code = variant.code;

            quote::quote! {
                ZError::#name => write!(f, "[{}::{}({})]: {}", stringify!(#ename), stringify!(#name), #code, #err),
            }
        })
    });

    quote::quote! {
        #[doc = "Base error enum for Zenoh. It contains all possible error codes."]
        #[repr(u8)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum ZError {
            #(#variants_declare)*
        }

        impl ::core::fmt::Display for ZError {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self {
                    #(#variants_display)*
                }
            }
        }

        impl ::core::error::Error for ZError {}
    }
}
