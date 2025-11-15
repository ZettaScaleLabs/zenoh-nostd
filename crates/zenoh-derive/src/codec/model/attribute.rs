use proc_macro2::{Span, TokenStream};
use syn::{Expr, Ident, meta::ParseNestedMeta, parenthesized, spanned::Spanned};

#[derive(Clone)]
pub struct ZenohAttribute {
    pub span: Span,

    pub flatten: bool,
    pub shift: Option<usize>,
    pub size: SizeAttribute,
    pub maybe_empty: bool,
    pub mandatory: bool,
    pub presence: PresenceAttribute,
    pub header: HeaderAttribute,
    pub ext: ExtAttribute,
    pub default: DefaultAttribute,
}

impl Default for ZenohAttribute {
    fn default() -> Self {
        ZenohAttribute {
            span: Span::call_site(),
            flatten: false,
            shift: None,
            size: SizeAttribute::default(),
            maybe_empty: false,
            mandatory: false,
            presence: PresenceAttribute::default(),
            header: HeaderAttribute::default(),
            ext: ExtAttribute::default(),
            default: DefaultAttribute::default(),
        }
    }
}

impl ZenohAttribute {
    pub fn from_field(field: &syn::Field) -> syn::Result<Self> {
        let mut zattr = ZenohAttribute {
            span: field.ident.span(),
            ..Default::default()
        };

        for attr in &field.attrs {
            if attr.path().is_ident("zenoh") {
                attr.parse_nested_meta(|meta| {
                    let size = SizeAttribute::from_meta(&meta)?;
                    let flatten = flatten_from_meta(&meta)?;
                    let shift = shift_from_meta(&meta)?;
                    let maybe_empty = maybe_empty_from_meta(&meta)?;
                    let mandatory = mandatory_from_meta(&meta)?;
                    let presence = PresenceAttribute::from_meta(&meta)?;
                    let header = HeaderAttribute::from_meta(&meta)?;
                    let default = DefaultAttribute::from_meta(&meta)?;
                    let ext = ExtAttribute::from_meta(&meta)?;

                    if !matches!(size, SizeAttribute::None) {
                        zattr.size = size;
                    }
                    if flatten {
                        zattr.flatten = true;
                    }
                    if let Some(shift) = shift {
                        zattr.shift = Some(shift);
                    }
                    if maybe_empty {
                        zattr.maybe_empty = true;
                    }
                    if mandatory {
                        zattr.mandatory = true;
                    }
                    if !matches!(presence, PresenceAttribute::None) {
                        zattr.presence = presence;
                    }
                    if !matches!(header, HeaderAttribute::None) {
                        zattr.header = header;
                    }
                    if !matches!(ext, ExtAttribute::None) {
                        zattr.ext = ext;
                    }
                    if !matches!(default, DefaultAttribute::None) {
                        zattr.default = default;
                    }

                    Ok(())
                })?;
            }
        }

        Ok(zattr)
    }
}

fn flatten_from_meta(meta: &ParseNestedMeta) -> syn::Result<bool> {
    if meta.path.is_ident("flatten") {
        return Ok(true);
    }

    Ok(false)
}

fn shift_from_meta(meta: &ParseNestedMeta) -> syn::Result<Option<usize>> {
    if meta.path.is_ident("shift") {
        let value = meta.value()?;
        let shift: syn::LitInt = value.parse()?;
        return Ok(Some(shift.base10_parse()?));
    }

    Ok(None)
}

fn maybe_empty_from_meta(meta: &ParseNestedMeta) -> syn::Result<bool> {
    if meta.path.is_ident("maybe_empty") {
        return Ok(true);
    }

    Ok(false)
}

fn mandatory_from_meta(meta: &ParseNestedMeta) -> syn::Result<bool> {
    if meta.path.is_ident("mandatory") {
        return Ok(true);
    }

    Ok(false)
}

fn ident_to_header_path(ident: &Ident) -> TokenStream {
    let ident = syn::Ident::new(&format!("HEADER_SLOT_{}", ident), ident.span());
    quote::quote! { Self::#ident }
}

#[derive(Clone, Default)]
pub enum SizeAttribute {
    #[default]
    None,
    Prefixed,
    Remain,
    Header(TokenStream),
}

impl SizeAttribute {
    fn from_meta(meta: &ParseNestedMeta) -> syn::Result<Self> {
        if meta.path.is_ident("size") {
            let value = meta.value()?;
            let size: syn::Ident = value.parse()?;
            if size == "prefixed" {
                return Ok(SizeAttribute::Prefixed);
            } else if size == "remain" {
                return Ok(SizeAttribute::Remain);
            } else if size == "header" {
                let content;
                parenthesized!(content in value);
                let ident: Ident = content.parse()?;
                return Ok(SizeAttribute::Header(ident_to_header_path(&ident)));
            } else {
                return Err(syn::Error::new_spanned(
                    size,
                    "Invalid size attribute value",
                ));
            }
        }

        Ok(SizeAttribute::None)
    }
}

#[derive(Clone, Default)]
pub enum PresenceAttribute {
    #[default]
    None,
    Prefixed,
    Header(TokenStream),
}

impl PresenceAttribute {
    fn from_meta(meta: &ParseNestedMeta) -> syn::Result<Self> {
        if meta.path.is_ident("presence") {
            let value = meta.value()?;
            let presence: syn::Ident = value.parse()?;
            if presence == "prefixed" {
                return Ok(PresenceAttribute::Prefixed);
            } else if presence == "header" {
                let content;
                parenthesized!(content in value);
                let ident: Ident = content.parse()?;
                return Ok(PresenceAttribute::Header(ident_to_header_path(&ident)));
            } else {
                return Err(syn::Error::new_spanned(
                    presence,
                    "Invalid presence attribute value",
                ));
            }
        }

        Ok(PresenceAttribute::None)
    }
}

#[derive(Clone, Default)]
pub enum HeaderAttribute {
    #[default]
    None,
    Slot(TokenStream),
}

impl HeaderAttribute {
    fn from_meta(meta: &ParseNestedMeta) -> syn::Result<Self> {
        if meta.path.is_ident("header") {
            let ident: Ident = meta.value()?.parse()?;
            return Ok(HeaderAttribute::Slot(ident_to_header_path(&ident)));
        }

        Ok(HeaderAttribute::None)
    }
}

#[derive(Clone, Default)]
pub enum ExtAttribute {
    #[default]
    None,
    Expr(Expr),
}

impl ExtAttribute {
    fn from_meta(meta: &ParseNestedMeta) -> syn::Result<Self> {
        if meta.path.is_ident("ext") {
            let expr: Expr = meta.value()?.parse()?;
            return Ok(ExtAttribute::Expr(expr));
        }

        Ok(ExtAttribute::None)
    }
}

#[derive(Clone, Default)]
pub enum DefaultAttribute {
    #[default]
    None,
    Expr(Expr),
}

impl DefaultAttribute {
    fn from_meta(meta: &ParseNestedMeta) -> syn::Result<Self> {
        if meta.path.is_ident("default") {
            let expr: Expr = meta.value()?.parse()?;
            return Ok(DefaultAttribute::Expr(expr));
        }

        Ok(DefaultAttribute::None)
    }
}
