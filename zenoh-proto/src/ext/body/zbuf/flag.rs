use proc_macro2::TokenStream;
use syn::Data;

pub fn flag_body(data: &Data, flag_needed: bool) -> TokenStream {
    if !flag_needed {
        return quote::quote! {};
    }

    let fields = match data {
        Data::Struct(s) => &s.fields,
        _ => unreachable!(),
    };

    let iter = match fields {
        syn::Fields::Named(fields_named) => fields_named.named.iter().collect::<Vec<_>>(),
        syn::Fields::Unnamed(fields_unnamed) => fields_unnamed.unnamed.iter().collect::<Vec<_>>(),
        syn::Fields::Unit => unreachable!(),
    };

    let mut flag_parts = Vec::<TokenStream>::new();
    let mut shift = 0u8;

    for (i, field) in iter.iter().enumerate() {
        let attr = &field.attrs[0];

        if attr.path().is_ident("zbuf")
            || attr.path().is_ident("str")
            || attr.path().is_ident("zid")
        {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("flag") || meta.path.is_ident("eflag") {
                    let content;
                    syn::parenthesized!(content in meta.input);

                    let value: syn::LitInt = content.parse()?;
                    let flag_size = value.base10_parse::<u8>()?;

                    let access = match &field.ident {
                        Some(ident) => quote::quote! { x.#ident },
                        None => {
                            let index = syn::Index::from(i);
                            quote::quote! { x.#index }
                        }
                    };

                    let len = match attr.path().get_ident().unwrap().to_string().as_str() {
                        "zbuf" => {
                            quote::quote! { crate::protocol::codec::encoded_len_zbuf(&#access) }
                        }
                        "str" => {
                            quote::quote! { crate::protocol::codec::encoded_len_str(&#access) }
                        }
                        "zid" => {
                            quote::quote! { crate::protocol::codec::encoded_len_zid(&#access) }
                        }
                        _ => unreachable!(),
                    };

                    if meta.path.is_ident("flag") {
                        flag_parts.push(quote::quote! {
                            flag |= ((#len as u8 - 1) & ((1 << #flag_size) - 1)) << #shift;
                        });
                    } else {
                        flag_parts.push(quote::quote! {
                            flag |= ((#len as u8) & ((1 << #flag_size) - 1)) << #shift;
                        });
                    }

                    shift += flag_size;
                }

                Ok(())
            })
            .unwrap();
        }
    }

    let expanded = quote::quote! {
        let mut flag: u8 = 0;
        #(#flag_parts)*
        crate::protocol::codec::encode_u8(w, flag)?;
    };

    expanded
}
