use zenoh_buffer::ZBuf;

use crate::{common::extension::ZExtUnknown, core::encoding::Encoding};

/// # Err message
///
/// ```text
/// Flags:
/// - X: Reserved
/// - E: Encoding       If E==1 then the encoding is present
/// - Z: Extension      If Z==1 then at least one extension is present
///
///   7 6 5 4 3 2 1 0
///  +-+-+-+-+-+-+-+-+
///  |Z|E|X|   ERR   |
///  +-+-+-+---------+
///  ~   encoding    ~  if E==1
///  +---------------+
///  ~  [err_exts]   ~  if Z==1
///  +---------------+
///  ~ pl: <u8;z32>  ~  -- Payload
///  +---------------+
/// ```
pub mod flag {
    // pub const X: u8 = 1 << 5; // 0x20 Reserved
    pub const E: u8 = 1 << 6; // 0x40 Encoding      if E==1 then the encoding is present
    pub const Z: u8 = 1 << 7; // 0x80 Extensions        if Z==1 then an extension will follow
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Err<'a> {
    pub encoding: Encoding<'a>,
    pub ext_sinfo: Option<ext::SourceInfoType>,
    pub ext_unknown: &'a [ZExtUnknown<'a>],
    pub payload: ZBuf<'a>,
}

pub mod ext {
    use crate::zextzbuf;

    /// # SourceInfo extension
    /// Used to carry additional information about the source of data
    pub type SourceInfo<'a> = zextzbuf!('a, 0x1, false);
    pub type SourceInfoType = crate::zenoh::ext::SourceInfoType<{ SourceInfo::ID }>;
}
