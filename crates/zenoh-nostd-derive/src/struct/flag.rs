use proc_macro2::{Span, TokenStream};
use syn::Ident;

use crate::r#struct::parse::{
    ZFieldKind, ZPresenceFlavour, ZSizeFlavour, ZStruct, ZStructFlavour, ZStructKind,
};

pub fn parse_body(r#struct: &ZStruct) -> (TokenStream, TokenStream) {
    let mut enc = Vec::new();
    let mut dec = Vec::new();

    let mut flag = false;
    let mut shift = 0u8;

    for field in &r#struct.0 {
        let access = &field.access;
        let kind = &field.kind;

        match kind {
            ZFieldKind::Header => {}
            ZFieldKind::Flag => {
                flag = true;
            }
            ZFieldKind::HeaderStorage { .. } => {}
            ZFieldKind::ZExtBlock { flavour, exts } => {
                if matches!(*flavour, ZPresenceFlavour::Flag) {
                    if !flag {
                        panic!("Flag field must be defined before any field using flag encoding.");
                    }

                    enc.push(quote::quote! {
                        let mut n_exts = 0usize;
                    });

                    for ext in exts {
                        let access = &ext.access;
                        enc.push(quote::quote! {
                            if self.#access.is_some() {
                                n_exts += 1;
                            }
                        });
                    }

                    enc.push(quote::quote! {
                        flag |= ((n_exts > 0) as u8) << ( 8u8 - 1 - #shift );
                    });

                    let paccess = Ident::new(&format!("presence_{}", access), Span::call_site());

                    dec.push(quote::quote! {
                        let mut #paccess = ((flag >> ( 8u8 - 1 - #shift )) & 1) != 0;
                    });

                    shift += 1;
                }
            }
            ZFieldKind::ZExtBlockEnd => {}
            ZFieldKind::ZStruct(ZStructKind { flavour: attr, ty }) => {
                let (presence, size) = match attr {
                    ZStructFlavour::Option { presence, size } => {
                        (matches!(*presence, ZPresenceFlavour::Flag), size)
                    }
                    ZStructFlavour::Size(size) => (false, size),
                };

                let (sized, size, maybe_empty) = match size {
                    ZSizeFlavour::NonEmptyFlag(s) => (true, *s, false),
                    ZSizeFlavour::MaybeEmptyFlag(s) => (true, *s, true),
                    _ => (false, 0, false),
                };

                if !presence && !sized {
                    continue;
                } else if !flag {
                    panic!("Flag field must be defined before any field using flag encoding.");
                }

                let paccess = Ident::new(&format!("presence_{}", access), Span::call_site());

                if presence {
                    enc.push(quote::quote! {
                        if self.#access.is_some() {
                            flag |= 1 << ( 8u8 - 1 - #shift );
                        }
                    });

                    dec.push(quote::quote! {
                        let #paccess = ((flag >> ( 8u8 - 1 - #shift )) & 1) != 0;
                    });

                    shift += 1;
                }

                if sized {
                    let mask = (1usize << size) - 1;

                    let len = quote::quote! { < #ty as crate::ZStruct>::z_len(&self.#access) };

                    let len = match (presence, maybe_empty) {
                        (false, false) => quote::quote! { (((#len - 1) & #mask) as u8)},
                        (false, true) => quote::quote! { ((#len & #mask) as u8)},
                        (true, true) => quote::quote! { ((#len & #mask) as u8)},
                        (true, false) => {
                            quote::quote! { (((#len + self.#access.is_none() as usize - 1) & #mask) as u8)}
                        }
                    };

                    enc.push(quote::quote! { flag |= #len << ( 8u8 - #size - #shift ); });

                    let value = quote::quote! { ((flag >> ( 8u8 - #size - #shift )) as usize) };
                    let len = match (presence, maybe_empty) {
                        (false, false) => {
                            quote::quote! { (#value & #mask) + 1 }
                        }
                        (false, true) => {
                            quote::quote! { #value & #mask }
                        }
                        (true, true) => {
                            quote::quote! { if #paccess { #value & #mask } else { 0 } }
                        }
                        (true, false) => {
                            quote::quote! { if #paccess { (#value & #mask) + 1 } else { 0 } }
                        }
                    };

                    dec.push(quote::quote! { let #access = (#len) as usize; });

                    shift += size;
                }
            }
        }
    }

    if !flag {
        return (quote::quote! {}, quote::quote! {});
    }

    (
        quote::quote! {
            let mut flag: u8 = 0;
            #(#enc)*
            <u8 as crate::ZStruct>::z_encode(&flag, w)?;
        },
        quote::quote! {
            let flag = <u8 as crate::ZStruct>::z_decode(r)?;
            #(#dec)*
        },
    )
}
