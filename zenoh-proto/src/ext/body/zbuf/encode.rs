use proc_macro2::TokenStream;
use syn::{Data, Path, parenthesized};

pub fn encode_body(data: &Data) -> TokenStream {
    let fields = match data {
        Data::Struct(s) => &s.fields,
        _ => unreachable!(),
    };

    let iter = match fields {
        syn::Fields::Named(fields_named) => fields_named.named.iter().collect::<Vec<_>>(),
        syn::Fields::Unnamed(fields_unnamed) => fields_unnamed.unnamed.iter().collect::<Vec<_>>(),
        syn::Fields::Unit => unreachable!(),
    };

    let mut encode_parts = Vec::new();

    for (i, field) in iter.iter().enumerate() {
        let attr = &field.attrs[0];

        let access = match &field.ident {
            Some(ident) => quote::quote! { x.#ident },
            None => {
                let index = syn::Index::from(i);
                quote::quote! { x.#index }
            }
        };

        if attr.path().is_ident("u8") {
            encode_parts.push(quote::quote! {
                crate::protocol::codec::encode_u8(w, #access as u8)?;
            });
        } else if attr.path().is_ident("u16") {
            encode_parts.push(quote::quote! {
                crate::protocol::codec::encode_u16(w, #access as u16)?;
            });
        } else if attr.path().is_ident("u32") {
            encode_parts.push(quote::quote! {
                crate::protocol::codec::encode_u32(w, #access as u32)?;
            });
        } else if attr.path().is_ident("u64") {
            encode_parts.push(quote::quote! {
                crate::protocol::codec::encode_u64(w, #access as u64)?;
            });
        } else if attr.path().is_ident("usize") {
            encode_parts.push(quote::quote! {
                crate::protocol::codec::encode_u64(w, #access as u64)?;
            });
        } else if attr.path().is_ident("timestamp") {
            encode_parts.push(quote::quote! {
                crate::protocol::codec::encode_timestamp(w, &#access)?;
            });
        } else if attr.path().is_ident("array") {
            encode_parts.push(quote::quote! {
                crate::protocol::codec::encode_array(w, &#access)?;
            });
        } else if attr.path().is_ident("zid") {
            //TODO
            encode_parts.push(quote::quote! {
                crate::protocol::codec::encode_zid(w, &#access)?;
            });
        } else if attr.path().is_ident("str") {
            //TODO
            encode_parts.push(quote::quote! {
                crate::protocol::codec::encode_str(w, &#access)?;
            });
        } else if attr.path().is_ident("zbuf") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("plain") {
                    encode_parts.push(quote::quote! {
                        crate::protocol::codec::encode_usize(w, crate::protocol::codec::encoded_len_zbuf(&#access))?;
                    });

                    return Ok(());
                }

                if meta.path.is_ident("flag") || meta.path.is_ident("eflag") {
                    let content;
                    parenthesized!(content in meta.input);
                    let _: syn::LitInt = content.parse()?;
                }

                Ok(())
            }).unwrap();

            encode_parts.push(quote::quote! {
                crate::protocol::codec::encode_zbuf(w, &#access)?;
            });
        } else if attr.path().is_ident("composite") {
            let path: Path = attr.parse_args().unwrap();
            let ident = path.get_ident().unwrap();

            let func_ident = quote::format_ident!("encode_{}", ident);

            encode_parts.push(quote::quote! {
                #func_ident(w, &#access)?;
            });
        }
    }

    let expanded = quote::quote! {
        #(#encode_parts)*
    };

    expanded.into()
}
