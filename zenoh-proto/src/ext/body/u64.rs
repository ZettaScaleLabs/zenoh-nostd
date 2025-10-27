use proc_macro2::TokenStream;

use crate::ext::parse::ParsedField;

pub fn compute_body_u64(
    fields: Vec<ParsedField>,
    named: bool,
) -> (TokenStream, TokenStream, TokenStream) {
    let len_body = fields
        .iter()
        .map(|field| {
            let access = &field.access;
            quote::quote! { crate::protocol::codec::encoded_len_u64(x. #access as u64) }
        })
        .reduce(|acc, expr| quote::quote! { #acc + #expr })
        .expect("at least one field for u64 extension");

    let mut fields_bit_shift = Vec::new();
    let mut total_bits = 0u64;

    for field in &fields {
        let (bits, ty) = match &field.kind {
            crate::ext::parse::FieldKind::U8 => (8, quote::quote! { u8 }),
            crate::ext::parse::FieldKind::U16 => (16, quote::quote! { u16 }),
            crate::ext::parse::FieldKind::U32 => (32, quote::quote! { u32 }),
            crate::ext::parse::FieldKind::U64 => (64, quote::quote! { u64 }),
            crate::ext::parse::FieldKind::Usize => (64, quote::quote! { usize }),
            _ => panic!("each field must have a uint kind for u64 extension"),
        };

        fields_bit_shift.push((field.access.clone(), ty, bits, total_bits));
        total_bits += bits;
    }

    let encode_body = fields_bit_shift
        .iter()
        .map(|(access, _, _, shift)| {
            if *shift == 0 {
                quote::quote! { (x.#access as u64) }
            } else {
                quote::quote! { ((x.#access as u64) << #shift) }
            }
        })
        .reduce(|acc, expr| quote::quote! { #acc | #expr })
        .unwrap();

    let decode_body = fields_bit_shift
        .iter()
        .map(|(access, ty, bits, shift)| {
            let mask = (1u128 << bits) - 1;
            let mask64 = mask as u64;
            let value = quote::quote! {
                (((value >> #shift) & #mask64) as #ty)
            };

            if named {
                quote::quote! { #access: #value }
            } else {
                quote::quote! { #value }
            }
        })
        .collect::<Vec<_>>();

    (
        len_body,
        quote::quote! {
            crate::protocol::codec::encode_u64(w, #encode_body)?;

            Ok(())
        },
        if named {
            quote::quote! {
                let value = crate::protocol::codec::decode_u64(r)?;
                Ok(Self { #(#decode_body),* })
            }
        } else {
            quote::quote! {
                let value = crate::protocol::codec::decode_u64(r)?;
                Ok(Self( #(#decode_body),* ))
            }
        },
    )
}
