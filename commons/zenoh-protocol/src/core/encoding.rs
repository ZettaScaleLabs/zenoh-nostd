use core::fmt::Debug;

pub type EncodingId = u16;

/// [`Encoding`] is a metadata that indicates how the data payload should be interpreted.
/// For wire-efficiency and extensibility purposes, Zenoh defines an [`Encoding`] as
/// composed of an unsigned integer prefix and a bytes schema. The actual meaning of the
/// prefix and schema are out-of-scope of the protocol definition. Therefore, Zenoh does not
/// impose any encoding mapping and users are free to use any mapping they like.
/// Nevertheless, it is worth highlighting that Zenoh still provides a default mapping as part
/// of the API as per user convenience. That mapping has no impact on the Zenoh protocol definition.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Encoding {
    pub id: EncodingId,
    pub schema: Option<ZSlice>,
}

/// # Encoding field
///
/// ```text
///  7 6 5 4 3 2 1 0
/// +-+-+-+-+-+-+-+-+
/// ~   id: z16   |S~
/// +---------------+
/// ~schema: <u8;z8>~  -- if S==1
/// +---------------+
/// ```
pub mod flag {
    pub const S: u32 = 1; // 0x01 Suffix    if S==1 then schema is present
}

impl Encoding {
    /// Returns a new [`Encoding`] object with default empty prefix ID.
    pub const fn empty() -> Self {
        Self {
            id: 0,
            schema: None,
        }
    }
}

impl Default for Encoding {
    fn default() -> Self {
        Self::empty()
    }
}
