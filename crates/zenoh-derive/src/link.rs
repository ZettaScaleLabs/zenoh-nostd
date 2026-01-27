use proc_macro2::TokenStream;
use syn::DeriveInput;

pub mod info;
pub mod rx;
pub mod tx;

pub fn derive_zlink(input: &DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let (tx_type, rx_type) = extract_zlink_types(input)?;
    let variants = match &input.data {
        syn::Data::Enum(data_enum) => &data_enum.variants,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "ZLInk can only be derived for enums",
            ));
        }
    }
    .iter()
    .map(|variant| &variant.ident);

    let variants_split = variants.clone().map(|ident| {
        quote::quote! {
            Self:: #ident (link) => {
                let (tx, rx) = zenoh_nostd::platform::ZLink::split(link);
                (Self::Tx:: #ident (tx), Self::Rx:: #ident (rx))
            },
        }
    });

    let ty_generics_link = if input.generics.lifetimes().count() > 0 {
        quote::quote! { <'link> }
    } else {
        quote::quote! {}
    };

    Ok(quote::quote! {
        impl #impl_generics zenoh_nostd::platform::ZLink for #ident #ty_generics #where_clause {
            type Tx<'link> = #tx_type #ty_generics_link where Self: 'link;
            type Rx<'link> = #rx_type #ty_generics_link where Self: 'link;

            fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>) {
                match self {
                    #(#variants_split)*
                }
            }
        }
    })
}

fn extract_zlink_types(input: &DeriveInput) -> syn::Result<(syn::Ident, syn::Ident)> {
    for attr in &input.attrs {
        if !attr.path().is_ident("zenoh") {
            continue;
        }

        let nested = attr.parse_args_with(
            syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
        )?;

        for meta in nested {
            if let syn::Meta::NameValue(nv) = meta {
                if nv.path.is_ident("ZLink") {
                    if let syn::Expr::Tuple(tuple) = nv.value {
                        if tuple.elems.len() == 2 {
                            let tx = match &tuple.elems[0] {
                                syn::Expr::Path(p) => p
                                    .path
                                    .get_ident()
                                    .ok_or_else(|| {
                                        syn::Error::new_spanned(
                                            &tuple.elems[0],
                                            "Expected identifier",
                                        )
                                    })?
                                    .clone(),
                                _ => {
                                    return Err(syn::Error::new_spanned(
                                        &tuple.elems[0],
                                        "Expected identifier",
                                    ));
                                }
                            };
                            let rx = match &tuple.elems[1] {
                                syn::Expr::Path(p) => p
                                    .path
                                    .get_ident()
                                    .ok_or_else(|| {
                                        syn::Error::new_spanned(
                                            &tuple.elems[1],
                                            "Expected identifier",
                                        )
                                    })?
                                    .clone(),
                                _ => {
                                    return Err(syn::Error::new_spanned(
                                        &tuple.elems[1],
                                        "Expected identifier",
                                    ));
                                }
                            };
                            return Ok((tx, rx));
                        }
                    }
                }
            }
        }
    }

    Err(syn::Error::new_spanned(
        input,
        "Missing #[zenoh(ZLink = (TxType, RxType))] attribute",
    ))
}
