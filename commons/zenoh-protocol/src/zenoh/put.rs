use heapless::Vec;
use uhlc::Timestamp;
use zenoh_buffers::zslice::ZSlice;

use crate::{common::ZExtUnknown, core::Encoding};

/// # Put message
///
/// ```text
/// Flags:
/// - T: Timestamp      If T==1 then the timestamp if present
/// - E: Encoding       If E==1 then the encoding is present
/// - Z: Extension      If Z==1 then at least one extension is present
///
///   7 6 5 4 3 2 1 0
///  +-+-+-+-+-+-+-+-+
///  |Z|E|T|   PUT   |
///  +-+-+-+---------+
///  ~ ts: <u8;z16>  ~  if T==1
///  +---------------+
///  ~   encoding    ~  if E==1
///  +---------------+
///  ~  [put_exts]   ~  if Z==1
///  +---------------+
///  ~ pl: <u8;z32>  ~  -- Payload
///  +---------------+
/// ```
pub mod flag {
    pub const T: u8 = 1 << 5; // 0x20 Timestamp     if T==0 then the timestamp if present
    pub const E: u8 = 1 << 6; // 0x40 Encoding      if E==1 then the encoding is present
    pub const Z: u8 = 1 << 7; // 0x80 Extensions    if Z==1 then an extension will follow
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Put {
    pub timestamp: Option<Timestamp>,
    pub encoding: Encoding,
    pub ext_sinfo: Option<ext::SourceInfoType>,
    pub ext_attachment: Option<ext::AttachmentType>,
    pub ext_unknown: Vec<ZExtUnknown<1, 32>, 8>,
    pub payload: ZSlice,
}

pub mod ext {
    use crate::{common::ZExtZBuf, zextzbuf};

    /// # SourceInfo extension
    /// Used to carry additional information about the source of data
    pub type SourceInfo = zextzbuf!(0x1, 1, 32, false);
    pub type SourceInfoType = crate::zenoh::ext::SourceInfoType<{ SourceInfo::ID }>;

    /// # User attachment
    pub type Attachment = zextzbuf!(0x3, 1, 32, false);
    pub type AttachmentType = crate::zenoh::ext::AttachmentType<{ Attachment::ID }>;
}
