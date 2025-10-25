use proc_macro2::TokenStream;
use syn::{Attribute, Data, DataStruct, Fields};

pub enum Kind {
    Unit,
    U64,
    ZBuf,
}

pub fn infer_kind(data: &Data) -> (TokenStream, Kind) {
    let kind = match data {
        Data::Struct(DataStruct { fields, .. }) => match fields {
            Fields::Named(named) if named.named.is_empty() => Kind::Unit,
            Fields::Unnamed(unnamed) if unnamed.unnamed.is_empty() => Kind::Unit,
            Fields::Unit => Kind::Unit,
            Fields::Named(named) => infer_from_fields(named.named.iter().map(|f| &f.attrs)),
            Fields::Unnamed(unnamed) => infer_from_fields(unnamed.unnamed.iter().map(|f| &f.attrs)),
        },
        _ => panic!("infer_kind only supports structs"),
    };

    let token = match kind {
        Kind::Unit => quote::quote! { crate::protocol::ext::ZExtKind::Unit },
        Kind::U64 => quote::quote! { crate::protocol::ext::ZExtKind::U64 },
        Kind::ZBuf => quote::quote! { crate::protocol::ext::ZExtKind::ZBuf },
    };

    (token, kind)
}

enum FieldKind {
    Integer(u32),
    ZBufLike,
}

fn infer_from_fields<'a>(fields: impl Iterator<Item = &'a Vec<Attribute>>) -> Kind {
    use FieldKind::*;
    use Kind::*;

    let mut total_bits = 0u32;

    for attrs in fields {
        let kind = match infer_field_kind(attrs) {
            Some(k) => k,
            None => panic!(
                "Each field must have exactly one attribute like #[u32], #[zid], #[zstr], etc."
            ),
        };

        match kind {
            Integer(bits) => total_bits += bits,
            ZBufLike => return ZBuf,
        }
    }

    if total_bits == 0 {
        Unit
    } else if total_bits <= 64 {
        U64
    } else {
        ZBuf
    }
}

fn infer_field_kind(attrs: &[Attribute]) -> Option<FieldKind> {
    use FieldKind::*;

    if attrs.len() != 1 {
        return None;
    }

    let attr = &attrs[0];

    if let Some(ident) = attr.path().get_ident() {
        match ident.to_string().as_str() {
            "u8" => return Some(Integer(8)),
            "u16" => return Some(Integer(16)),
            "u32" => return Some(Integer(32)),
            "u64" => return Some(Integer(64)),
            "usize" => return Some(Integer(64)),

            "zid" | "zbuf" | "zstr" | "timestamp" | "array" | "composite" => {
                return Some(ZBufLike);
            }

            _ => return None,
        }
    }

    None
}
