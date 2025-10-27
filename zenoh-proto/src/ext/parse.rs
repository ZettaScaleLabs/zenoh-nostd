use proc_macro2::TokenStream;

use syn::{
    Attribute, Data, DataStruct, Field, Fields, Ident, LitInt, Path, Result, Token,
    parse::{Parse, ParseStream},
};

pub enum SizeFlavor {
    Plain,
    Deduced,
    NonEmptyFlag(u8),
    MaybeEmptyFlag(u8),
}

impl SizeFlavor {
    fn from_attr(attr: &Attribute) -> SizeFlavor {
        let mut flavor = Option::<SizeFlavor>::None;

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("plain") {
                flavor = Some(SizeFlavor::Plain);
            } else if meta.path.is_ident("deduced") {
                flavor = Some(SizeFlavor::Deduced);
            } else if meta.path.is_ident("flag") {
                let value = meta.value().expect("Expected value for flag flavor");
                let lit: LitInt = value.parse()?;
                let flag_index = lit.base10_parse::<u8>()?;

                flavor = Some(SizeFlavor::NonEmptyFlag(flag_index));
            } else if meta.path.is_ident("eflag") {
                let value = meta.value().expect("Expected value for eflag flavor");
                let lit: LitInt = value.parse()?;
                let flag_index = lit.base10_parse::<u8>()?;

                flavor = Some(SizeFlavor::MaybeEmptyFlag(flag_index));
            }

            Ok(())
        })
        .expect("Failed to parse size flavor attribute");

        flavor.expect("Field expected to have a size flavor attribute")
    }
}

pub struct CompositeAttr {
    pub path: Path,
    pub ident: Ident,
}

impl Parse for CompositeAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let path: Path = input.parse()?;
        let _: Token![,] = input.parse()?;
        let ident: Ident = input.parse()?;

        Ok(CompositeAttr { path, ident })
    }
}

pub enum FieldKind {
    U8,
    U16,
    U32,
    U64,
    Usize,
    Timestamp,
    Array,

    ZBuf(SizeFlavor),
    Str(SizeFlavor),
    Zid(SizeFlavor),

    Composite(CompositeAttr),
}

impl FieldKind {
    fn from_attr(attr: &Attribute) -> FieldKind {
        if attr.path().is_ident("u8") {
            FieldKind::U8
        } else if attr.path().is_ident("u16") {
            FieldKind::U16
        } else if attr.path().is_ident("u32") {
            FieldKind::U32
        } else if attr.path().is_ident("u64") {
            FieldKind::U64
        } else if attr.path().is_ident("usize") {
            FieldKind::Usize
        } else if attr.path().is_ident("timestamp") {
            FieldKind::Timestamp
        } else if attr.path().is_ident("array") {
            FieldKind::Array
        } else if attr.path().is_ident("zbuf") {
            let flavor = SizeFlavor::from_attr(attr);
            FieldKind::ZBuf(flavor)
        } else if attr.path().is_ident("str") {
            let flavor = SizeFlavor::from_attr(attr);
            FieldKind::Str(flavor)
        } else if attr.path().is_ident("zid") {
            let flavor = SizeFlavor::from_attr(attr);
            FieldKind::Zid(flavor)
        } else if attr.path().is_ident("composite") {
            let composite_attr: CompositeAttr = attr
                .parse_args()
                .expect("Failed to parse composite attribute");

            FieldKind::Composite(composite_attr)
        } else {
            panic!("Unknown field attribute");
        }
    }
}

pub struct ParsedField {
    pub kind: FieldKind,
    pub access: TokenStream,
}

pub enum Extension {
    Unit,
    U64(Vec<ParsedField>),
    ZBuf(Vec<ParsedField>),
}

impl Extension {
    fn from_fields<'a>(fields: impl Iterator<Item = &'a Field>) -> Extension {
        let mut parsed_fields = Vec::<ParsedField>::new();
        let mut is_zbuf = false;
        let mut is_deduced = false;

        let mut total_flag_bits = 0u8;
        let mut total_u64_bits = 0u8;

        for (i, field) in fields.enumerate() {
            if is_deduced {
                panic!("Deduced size flavor must appear at the end of the struct");
            }

            let attrs = &field.attrs;

            if attrs.len() != 1 {
                panic!(
                    "Each field must have exactly one attribute like #[u32], #[zid], #[zstr], etc."
                );
            }

            let attr = &attrs[0];
            let kind = FieldKind::from_attr(attr);

            match &kind {
                FieldKind::ZBuf(flavor) | FieldKind::Str(flavor) | FieldKind::Zid(flavor) => {
                    is_zbuf = true;

                    match flavor {
                        SizeFlavor::NonEmptyFlag(size) => {
                            total_flag_bits += *size;
                        }
                        SizeFlavor::MaybeEmptyFlag(size) => {
                            total_flag_bits += *size;
                        }
                        SizeFlavor::Deduced => {
                            if is_deduced {
                                panic!("Only one field can have deduced size flavor");
                            }

                            is_deduced = true;
                        }
                        _ => {}
                    }
                }
                FieldKind::U8 => total_u64_bits += 8,
                FieldKind::U16 => total_u64_bits += 16,
                FieldKind::U32 => total_u64_bits += 32,
                FieldKind::U64 | FieldKind::Usize => total_u64_bits += 64,
                FieldKind::Timestamp => is_zbuf = true,
                FieldKind::Array => is_zbuf = true,
                FieldKind::Composite(_) => is_zbuf = true,
            }

            let access = match &field.ident {
                Some(ident) => quote::quote! { #ident },
                None => {
                    let index = syn::Index::from(i);
                    quote::quote! { #index }
                }
            };

            parsed_fields.push(ParsedField { kind, access });
        }

        if is_zbuf {
            if total_flag_bits > 8 {
                panic!("Total flag bits exceed 8 bits");
            }

            Extension::ZBuf(parsed_fields)
        } else {
            if total_u64_bits > 64 {
                Extension::ZBuf(parsed_fields)
            } else {
                Extension::U64(parsed_fields)
            }
        }
    }

    pub fn from_data(data: &Data) -> (Extension, bool) {
        match data {
            Data::Struct(DataStruct { fields, .. }) => match fields {
                Fields::Named(named) if named.named.is_empty() => (Extension::Unit, false),
                Fields::Unnamed(unnamed) if unnamed.unnamed.is_empty() => (Extension::Unit, true),
                Fields::Unit => (Extension::Unit, false),
                Fields::Named(named) => (Self::from_fields(named.named.iter()), true),
                Fields::Unnamed(unnamed) => (Self::from_fields(unnamed.unnamed.iter()), false),
            },
            _ => panic!("infer_kind only supports structs"),
        }
    }
}
