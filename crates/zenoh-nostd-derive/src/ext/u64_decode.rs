use proc_macro2::TokenStream;

use crate::model::{ZenohField, ZenohStruct, ty::ZenohType};
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
    let ty = &field.ty;
    let ty = match ty {
        ZenohType::U8 => quote::quote! { u8 },
        ZenohType::U16 => quote::quote! { u16 },
        ZenohType::U32 => quote::quote! { u32 },
        ZenohType::U64 => quote::quote! { u64 },
        ZenohType::USize => quote::quote! { usize },
        _ => unreachable!(),
    };

    quote::quote! {
        let #access = < u64 as crate::ZStructDecode>::z_decode(r)? as #ty;
        Ok(Self {
            #access
        })
    }
}
