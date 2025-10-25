use proc_macro2::TokenStream;
use syn::{Data, Generics, Ident};

pub fn compute_zext_zbuf(ident: &Ident, generics: &Generics, data: &Data) -> TokenStream {
    let _ = (ident, generics, data);

    TokenStream::new()
}
