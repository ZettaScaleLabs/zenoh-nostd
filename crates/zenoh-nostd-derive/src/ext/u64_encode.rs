use proc_macro2::TokenStream;

use crate::model::{ZenohField, ZenohStruct};

pub fn parse(r#struct: &ZenohStruct) -> TokenStream {
    let field = r#struct
        .fields
        .first()
        .expect("At least one field is expected, this should have been caught earlier");

    let field = match field {
        ZenohField::Regular { field } => field,
        ZenohField::ExtBlock { .. } => unreachable!(
            "The single field cannot be an ext block, this should have been caught earlier"
        ),
    };

    let access = &field.access;

    quote::quote! {
        < u64 as crate::ZStructEncode>::z_encode(&(self. #access as u64), w)?;
    }
}
