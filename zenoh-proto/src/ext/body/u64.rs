use proc_macro2::TokenStream;
use syn::{Data, Fields, Ident};

pub fn compute_zext_u64(ident: &Ident, data: &Data) -> TokenStream {
    let fields = match data {
        Data::Struct(s) => &s.fields,
        _ => panic!("only structs are supported"),
    };

    let mut field_infos = Vec::new();
    let mut total_bits = 0u64;

    let iter = match fields {
        Fields::Named(fields_named) => fields_named.named.iter().collect::<Vec<_>>(),
        Fields::Unnamed(fields_unnamed) => fields_unnamed.unnamed.iter().collect::<Vec<_>>(),
        Fields::Unit => unreachable!(),
    };

    for (i, field) in iter.iter().enumerate() {
        if field.attrs.len() != 1 {
            panic!("each field must have exactly one size attribute (#[u8], #[u16], ...)");
        }

        let attr = &field.attrs[0];
        let bits = if attr.path().is_ident("u8") {
            8
        } else if attr.path().is_ident("u16") {
            16
        } else if attr.path().is_ident("u32") {
            32
        } else if attr.path().is_ident("u64") {
            64
        } else if attr.path().is_ident("usize") {
            32
        } else {
            panic!("each field must have a size attribute (#[u8], #[u16], ...)");
        };

        let access = match field.ident {
            Some(ref ident) => quote::quote! { x.#ident },
            None => {
                let idx = iter.len() - 1 - i;
                let index = syn::Index::from(idx);
                quote::quote! { x.#index }
            }
        };

        let ty = &field.ty;

        field_infos.push((access, bits, total_bits, ty.clone()));
        total_bits += bits;
    }

    let encode_parts = field_infos.iter().map(|(access, _, shift, _)| {
        if *shift == 0 {
            quote::quote! { (#access as u64) }
        } else {
            quote::quote! { ((#access as u64) << #shift) }
        }
    });

    let encode_body = encode_parts
        .reduce(|acc, expr| quote::quote! { #acc | #expr })
        .unwrap();

    let decode_parts = field_infos
        .iter()
        .enumerate()
        .map(|(i, (_, bits, shift, ty))| {
            let mask = (1u128 << bits) - 1;
            let mask64 = mask as u64;
            let value_expr = quote::quote! {
                (((value >> #shift) & #mask64) as #ty)
            };

            match fields {
                Fields::Named(_) => {
                    let field_ident = &fields.iter().collect::<Vec<_>>()[i].ident;
                    quote::quote! { #field_ident: #value_expr }
                }
                Fields::Unnamed(_) => quote::quote! { #value_expr },
                Fields::Unit => unreachable!(),
            }
        });

    let decode_body = match fields {
        Fields::Named(_) => quote::quote! { Self { #(#decode_parts),* } },
        Fields::Unnamed(_) => quote::quote! { Self(#(#decode_parts),*) },
        Fields::Unit => unreachable!(),
    };

    let expanded = quote::quote! {
        type Decoded<'a> = #ident;

        const ENCODE: fn(&mut crate::zbuf::ZBufWriter<'_>, &Self) -> crate::result::ZResult<(), crate::protocol::ZCodecError> = |w, x| {
            let v: u64 = #encode_body;
            crate::protocol::codec::encode_u64(w, v)
        };
        const DECODE: for<'a> fn(&mut crate::zbuf::ZBufReader<'a>) -> crate::result::ZResult<Self::Decoded<'a>, crate::protocol::ZCodecError> = |r| {
            let value = crate::protocol::codec::decode_u64(r)?;
            Ok(#decode_body)
        };
    };

    expanded.into()
}
