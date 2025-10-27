mod ext;

/// Derive macro that makes a struct Zenoh Extension compliant.
///
/// # Attributes
///
/// Attributes of type u8, u16, u32, u64, usize, timestamp, array can be used out of the box.
///
/// Attributes of type zbuf, str and zid must precise a flavour:
///     - (flag = <no.bits>) will write the size of the zbuf/str/zid in a flag with the given number of bits. It will assume that the field cannot be empty to optimize the size.
///     - (eflag = <no.bits>) will write the size of the zbuf/str/zid in a flag with the given number of bits. It will assume that the field can be empty.
///         *Note*: this can be used multiple times and it will pack the sizes together in the same flag byte.
///     - (plain) will write the size of the zbuf/str/zid as a plain usize before the actual field.
///     - (deduced) will deduce the size of the zbuf/str/zid from the remaining size of the extension. This can only be used once and only at the end of the struct.
///
/// Composite types can be defined outside of this derive macro. This is to tell the macro to use
/// the composite type's own encoding/decoding implementation through the functions `encoded_len_<type>`,
/// `encode_<type>` and `decode_<type>`. They also must be usable out of the box without the need of extra flags.
#[proc_macro_derive(
    ZExt,
    attributes(
        // --- Raw base types ---
        u8, u16, u32, u64, usize, timestamp, array,
        // --- Unknown size base types ---
        zbuf, str, zid,
        // --- Composite ---
        composite,
    )
)]
pub fn derive_zext(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    ext::derive_zext(input).into()
}
