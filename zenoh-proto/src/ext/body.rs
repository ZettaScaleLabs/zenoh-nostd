use proc_macro2::TokenStream;
use syn::Data;

use crate::ext::kind::Kind;

pub fn infer_body(kind: &Kind, data: &Data) -> TokenStream {
    let _ = (kind, data);

    TokenStream::new()
}
