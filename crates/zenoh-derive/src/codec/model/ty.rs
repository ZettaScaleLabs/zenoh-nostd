use syn::{Type, TypeArray, TypeReference};

use crate::codec::model::attribute::{
    DefaultAttribute, ExtAttribute, HeaderAttribute, PresenceAttribute, SizeAttribute,
    ZenohAttribute,
};

pub enum ZenohType {
    U8,
    U16,
    U32,
    U64,
    USize,

    ByteArray,

    ByteSlice,
    Str,

    ZStruct,

    Option(Box<ZenohType>),
}

impl ZenohType {
    pub fn check_attribute(&self, attr: &ZenohAttribute) -> syn::Result<()> {
        let (s, f, sh, me, m, p, h, e, d) = (
            !matches!(attr.size, SizeAttribute::None),
            attr.flatten,
            attr.shift.is_some(),
            attr.maybe_empty,
            attr.mandatory,
            !matches!(attr.presence, PresenceAttribute::None),
            !matches!(attr.header, HeaderAttribute::None),
            !matches!(attr.ext, ExtAttribute::None),
            !matches!(attr.default, DefaultAttribute::None),
        );

        match self {
            ZenohType::U8 => {
                if f || sh || s || me || m || e {
                    return Err(syn::Error::new(
                        attr.span,
                        "u8 type does not support flatten, shift, size, maybe_empty, mandatory, presence or ext attributes",
                    ));
                }
                if d && !p || p && !d {
                    return Err(syn::Error::new(
                        attr.span,
                        "types with default attribute requires a presence attribute",
                    ));
                }
                Ok(())
            }
            ZenohType::U16
            | ZenohType::U32
            | ZenohType::U64
            | ZenohType::USize
            | ZenohType::ByteArray => {
                if f || sh || s || me || m || h || e {
                    return Err(syn::Error::new(
                        attr.span,
                        "u16, u32, u64, usize and [u8; N] types do not support flatten, shift, size, maybe_empty, mandatory, header, or ext attributes",
                    ));
                }
                if d && !p || p && !d {
                    return Err(syn::Error::new(
                        attr.span,
                        "types with default attribute requires a presence attribute",
                    ));
                }
                Ok(())
            }
            ZenohType::ByteSlice | ZenohType::Str => {
                if f || sh || m || h || e {
                    return Err(syn::Error::new(
                        attr.span,
                        "string and byte slice types do not support flatten, shift, mandatory, header, ext, or default attributes",
                    ));
                }
                if d && !p || p && !d {
                    return Err(syn::Error::new(
                        attr.span,
                        "types with default attribute requires a presence attribute",
                    ));
                }
                if !s {
                    return Err(syn::Error::new(
                        attr.span,
                        "string and byte slice types require a size attribute",
                    ));
                }
                Ok(())
            }
            ZenohType::ZStruct => {
                if p && !d || (d && !p && !e) {
                    return Err(syn::Error::new(
                        attr.span,
                        "ZStruct types with default attribute requires a presence attribute",
                    ));
                }
                if d && !e && !p {
                    return Err(syn::Error::new(
                        attr.span,
                        "structs with default attribute requires an ext attribute",
                    ));
                }
                if e && !d {
                    return Err(syn::Error::new(
                        attr.span,
                        "ZStruct type with ext attribute requires a default attribute",
                    ));
                }
                if e && s {
                    return Err(syn::Error::new(
                        attr.span,
                        "ZStruct type that are extensions cannot have a size attribute",
                    ));
                }
                if h && f {
                    return Err(syn::Error::new(
                        attr.span,
                        "ZStruct type with header attribute cannot be flattened",
                    ));
                }
                if sh && !f {
                    return Err(syn::Error::new(
                        attr.span,
                        "ZStruct type with shift attribute must be flattened",
                    ));
                }
                Ok(())
            }
            ZenohType::Option(inner_ty) => {
                if d || h {
                    return Err(syn::Error::new(
                        attr.span,
                        "Option type does not support default or header attributes",
                    ));
                }

                if !e && !p {
                    return Err(syn::Error::new(
                        attr.span,
                        "Option type that are not extensions must have a presence attribute",
                    ));
                }

                if e && p {
                    return Err(syn::Error::new(
                        attr.span,
                        "Option type that are extensions cannot have a presence attribute",
                    ));
                }

                if e && s {
                    return Err(syn::Error::new(
                        attr.span,
                        "Option type that are extensions cannot have a size attribute",
                    ));
                }

                let attr = ZenohAttribute {
                    size: attr.size.clone(),
                    flatten: attr.flatten,
                    shift: attr.shift,
                    maybe_empty: attr.maybe_empty,
                    mandatory: attr.mandatory,
                    presence: PresenceAttribute::None,
                    header: HeaderAttribute::None,
                    ext: ExtAttribute::None,
                    default: DefaultAttribute::None,
                    span: attr.span,
                };

                inner_ty.check_attribute(&attr)
            }
        }
    }

    pub fn from_type(ty: &Type) -> syn::Result<Self> {
        match ty {
            Type::Path(type_path) => {
                if type_path.path.segments.first().unwrap().ident == "Option" {
                    if let syn::PathArguments::AngleBracketed(args) =
                        &type_path.path.segments[0].arguments
                        && args.args.len() == 1
                        && let syn::GenericArgument::Type(inner_ty) = &args.args[0]
                    {
                        let zenoh_type = ZenohType::from_type(inner_ty)?;
                        return Ok(ZenohType::Option(Box::new(zenoh_type)));
                    }
                    return Err(syn::Error::new_spanned(
                        ty,
                        "Option must have exactly one type argument",
                    ));
                }

                let ident = &type_path.path.segments.last().unwrap().ident;
                match ident.to_string().as_str() {
                    "u8" => Ok(ZenohType::U8),
                    "u16" => Ok(ZenohType::U16),
                    "u32" => Ok(ZenohType::U32),
                    "u64" => Ok(ZenohType::U64),
                    "usize" => Ok(ZenohType::USize),
                    _ => Ok(ZenohType::ZStruct),
                }
            }
            Type::Reference(TypeReference { elem, .. }) => match &**elem {
                Type::Path(type_path) => {
                    let ident = &type_path.path.segments.last().unwrap().ident;
                    if ident == "str" {
                        Ok(ZenohType::Str)
                    } else {
                        Err(syn::Error::new_spanned(ty, "Unsupported reference type"))
                    }
                }
                Type::Slice(syn::TypeSlice { elem, .. }) => match &**elem {
                    Type::Path(type_path) => {
                        let ident = &type_path.path.segments.last().unwrap().ident;
                        if ident == "u8" {
                            Ok(ZenohType::ByteSlice)
                        } else {
                            Err(syn::Error::new_spanned(
                                ty,
                                "Unsupported slice element type",
                            ))
                        }
                    }
                    _ => Err(syn::Error::new_spanned(
                        ty,
                        "Unsupported slice element type",
                    )),
                },
                _ => Err(syn::Error::new_spanned(ty, "Unsupported reference type")),
            },
            Type::Array(TypeArray { elem, .. }) => match &**elem {
                Type::Path(type_path) => {
                    let ident = &type_path.path.segments.last().unwrap().ident;
                    if ident == "u8" {
                        Ok(ZenohType::ByteArray)
                    } else {
                        Err(syn::Error::new_spanned(
                            ty,
                            "Unsupported array element type",
                        ))
                    }
                }
                _ => Err(syn::Error::new_spanned(
                    ty,
                    "Unsupported array element type",
                )),
            },
            _ => Err(syn::Error::new_spanned(ty, "Unsupported type")),
        }
    }
}
