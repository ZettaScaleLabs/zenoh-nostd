use proc_macro2::TokenStream;
use syn::{Data, Path};

pub fn len_body(data: &Data, flag_needed: bool) -> TokenStream {
    let fields = match data {
        Data::Struct(s) => &s.fields,
        _ => unreachable!(),
    };

    let iter = match fields {
        syn::Fields::Named(fields_named) => fields_named.named.iter().collect::<Vec<_>>(),
        syn::Fields::Unnamed(fields_unnamed) => fields_unnamed.unnamed.iter().collect::<Vec<_>>(),
        syn::Fields::Unit => unreachable!(),
    };

    let mut len_parts = Vec::new();

    if flag_needed {
        len_parts.push(quote::quote! { 1 });
    }

    for (i, field) in iter.iter().enumerate() {
        let access = match &field.ident {
            Some(ident) => quote::quote! { x.#ident },
            None => {
                let index = syn::Index::from(i);
                quote::quote! { x.#index }
            }
        };

        let attr = &field.attrs[0];

        if attr.path().is_ident("u8") {
            len_parts
                .push(quote::quote! { crate::protocol::codec::encoded_len_u64(#access as u64) });
        } else if attr.path().is_ident("u16") {
            len_parts
                .push(quote::quote! { crate::protocol::codec::encoded_len_u64(#access as u64) });
        } else if attr.path().is_ident("u32") {
            len_parts
                .push(quote::quote! { crate::protocol::codec::encoded_len_u64(#access as u64) });
        } else if attr.path().is_ident("u64") {
            len_parts
                .push(quote::quote! { crate::protocol::codec::encoded_len_u64(#access as u64) });
        } else if attr.path().is_ident("usize") {
            len_parts
                .push(quote::quote! { crate::protocol::codec::encoded_len_u64(#access as u64) });
        } else if attr.path().is_ident("timestamp") {
            len_parts
                .push(quote::quote! { crate::protocol::codec::encoded_len_timestamp(&#access) });
        } else if attr.path().is_ident("array") {
            len_parts.push(quote::quote! { crate::protocol::codec::encoded_len_array(&#access) });
        } else if attr.path().is_ident("zid") {
            len_parts.push(quote::quote! { crate::protocol::codec::encoded_len_zid(&#access) });
        } else if attr.path().is_ident("str") {
            len_parts.push(quote::quote! { crate::protocol::codec::encoded_len_str(&#access) });
        } else if attr.path().is_ident("zbuf") {
            len_parts.push(quote::quote! { crate::protocol::codec::encoded_len_zbuf(&#access) });
        } else if attr.path().is_ident("composite") {
            let path: Path = attr.parse_args().unwrap();
            let ident = path.get_ident().unwrap();

            let func_ident = quote::format_ident!("encoded_len_{}", ident);
            len_parts.push(quote::quote! { #func_ident(&#access) });
        }
    }

    let len_body = len_parts
        .into_iter()
        .reduce(|acc, expr| quote::quote! { #acc + #expr })
        .unwrap();

    len_body
}
