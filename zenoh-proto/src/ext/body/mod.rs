use proc_macro2::TokenStream;

use crate::ext::parse::Extension;

mod u64;
mod unit;
mod zbuf;

pub fn compute_body(ext: Extension, named: bool) -> (TokenStream, TokenStream, TokenStream) {
    match ext {
        Extension::Unit => unit::compute_body_unit(),
        Extension::U64(fields) => u64::compute_body_u64(fields, named),
        Extension::ZBuf(fields) => zbuf::compute_body_zbuf(fields, named),
    }
}
