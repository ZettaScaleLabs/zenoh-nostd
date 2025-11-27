use proc_macro2::TokenStream;

use crate::error::model::DeclaredErrors;

pub(crate) mod model;

pub(crate) mod children;
pub(crate) mod zerror;

pub fn make_error(input: &DeclaredErrors) -> syn::Result<TokenStream> {
    let zerror = zerror::declare_zerror(input);
    let children = children::declare_children(input);

    Ok(quote::quote! {
        #zerror

        #children
    })
}
