use proc_macro2::TokenStream;

use crate::r#struct::parse::{
    ZFieldKind, ZPresenceFlavour, ZSizeFlavour, ZStruct, ZStructFlavour, ZStructKind,
};

pub fn parse_body(r#struct: &ZStruct) -> TokenStream {
    let mut len_parts = Vec::new();

    for field in &r#struct.0 {
        let access = &field.access;
        let kind = &field.kind;

        match kind {
            ZFieldKind::Flag | ZFieldKind::Header => {
                len_parts.push(quote::quote! {
                    1usize
                });
            }
            ZFieldKind::HeaderStorage { .. } => {}
            ZFieldKind::ZExtBlock { flavour, exts } => {
                if matches!(*flavour, ZPresenceFlavour::Plain) {
                    len_parts.push(quote::quote! { 1usize });
                }

                for ext in exts {
                    let access = &ext.access;
                    let ty = &ext.ty;
                    len_parts.push(quote::quote! {
                        if let Some(ext) = &self.#access {
                            < #ty as crate::ZExtAttribute<Self>>::z_len(ext)
                        } else {
                            0usize
                        }
                    });
                }
            }
            ZFieldKind::ZExtBlockEnd => {}
            ZFieldKind::ZStruct(ZStructKind { flavour: attr, ty }) => {
                let (presence, size) = match attr {
                    ZStructFlavour::Option { presence, size } => (
                        matches!(*presence, ZPresenceFlavour::Plain),
                        matches!(*size, ZSizeFlavour::Plain),
                    ),
                    ZStructFlavour::Size(size) => (false, matches!(*size, ZSizeFlavour::Plain)),
                };

                if presence {
                    len_parts.push(quote::quote! { 1usize });
                }

                if size {
                    let len = quote::quote! {
                        <usize as crate::ZStruct>::z_len(&< #ty as crate::ZStruct>::z_len(&self.#access))
                    };

                    if presence {
                        len_parts.push(quote::quote! {
                            if self.#access.is_some() {
                                #len
                            } else {
                                0usize
                            }
                        });
                    } else {
                        len_parts.push(len);
                    }
                }

                len_parts.push(quote::quote! {
                    < #ty as crate::ZStruct>::z_len(&self.#access)
                });
            }
        }
    }

    if len_parts.is_empty() {
        len_parts.push(quote::quote! { 0usize });
    }

    let len_body = len_parts
        .into_iter()
        .reduce(|acc, expr| quote::quote! { #acc + #expr })
        .unwrap();

    quote::quote! {
        #len_body
    }
}
