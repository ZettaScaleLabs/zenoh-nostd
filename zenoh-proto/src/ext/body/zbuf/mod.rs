use proc_macro2::TokenStream;

use crate::ext::parse::ParsedField;

mod decode;
mod encode;
mod flag;
mod len;

pub fn compute_body_zbuf(
    fields: Vec<ParsedField>,
    named: bool,
) -> (TokenStream, TokenStream, TokenStream) {
    let len_body = len::len_body(&fields);
    let (flag_body_enc, flag_body_dec) = flag::flag_body(&fields, named);
    let encode_body = encode::encode_body(&fields);
    let decode_body = decode::decode_body(&fields, named);

    (
        len_body,
        quote::quote! {
            #flag_body_enc
            #encode_body
            Ok(())
        },
        quote::quote! {
            #flag_body_dec
            #decode_body
        },
    )
}
