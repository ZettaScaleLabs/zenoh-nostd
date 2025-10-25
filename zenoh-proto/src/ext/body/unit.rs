use proc_macro2::TokenStream;
use syn::Ident;

pub fn compute_zext_unit(ident: &Ident) -> TokenStream {
    let expanded = quote::quote! {
        type Decoded<'a> = #ident;

        const ENCODE: fn(&mut crate::zbuf::ZBufWriter<'_>, &Self) -> crate::result::ZResult<(), crate::protocol::ZCodecError> = |_, _| { Ok(()) };
        const DECODE: for<'a> fn(&mut crate::zbuf::ZBufReader<'a>) -> crate::result::ZResult<Self::Decoded<'a>, crate::protocol::ZCodecError> = |_| { Ok(Self {}) };
    };

    expanded.into()
}
