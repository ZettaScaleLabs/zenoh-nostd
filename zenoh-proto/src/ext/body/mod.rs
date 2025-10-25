use proc_macro2::TokenStream;
use syn::DeriveInput;

mod u64;
mod unit;
mod zbuf;

use crate::ext::kind::Kind;

pub fn infer_body(kind: &Kind, input: &DeriveInput) -> TokenStream {
    let ident = &input.ident;
    let generics = &input.generics;
    let data = &input.data;

    match kind {
        Kind::Unit => unit::compute_zext_unit(ident),
        Kind::U64 => u64::compute_zext_u64(ident, data),
        Kind::ZBuf => zbuf::compute_zext_zbuf(ident, generics, data),
    }
}
