

/// The kind of consolidation to apply to a query.
#[repr(u8)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Copy)]
pub enum ConsolidationMode {
    /// Apply automatic consolidation based on queryable's preferences
    #[default]
    Auto,
    /// No consolidation applied: multiple samples may be received for the same key-timestamp.
    None,
    /// Monotonic consolidation immediately forwards samples, except if one with an equal or more recent timestamp
    /// has already been sent with the same key.
    ///
    /// This optimizes latency while potentially reducing bandwidth.
    ///
    /// Note that this doesn't cause re-ordering, but drops the samples for which a more recent timestamp has already
    /// been observed with the same key.
    Monotonic,
    /// Holds back samples to only send the set of samples that had the highest timestamp for their key.
    Latest,
    // Remove the duplicates of any samples based on the their timestamp.
    // Unique,
}

impl ConsolidationMode {
    pub const DEFAULT: Self = Self::Auto;
}

/// # Query message
///
/// ```text
/// Flags:
/// - C: Consolidation  if C==1 then consolidation is present
/// - P: Parameters     If P==1 then the parameters are present
/// - Z: Extension      If Z==1 then at least one extension is present
///
///   7 6 5 4 3 2 1 0
///  +-+-+-+-+-+-+-+-+
///  |Z|P|C|  QUERY  |
///  +-+-+-+---------+
///  % consolidation %  if C==1
///  +---------------+
///  ~ ps: <u8;z16>  ~  if P==1
///  +---------------+
///  ~  [qry_exts]   ~  if Z==1
///  +---------------+
/// ```
pub mod flag {
    pub const C: u8 = 1 << 5; // 0x20 Consolidation if C==1 then consolidation is present
    pub const P: u8 = 1 << 6; // 0x40 Parameters    if P==1 then the parameters are present
    pub const Z: u8 = 1 << 7; // 0x80 Extensions    if Z==1 then an extension will follow
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query<'a> {
    pub consolidation: ConsolidationMode,
    pub parameters: &'a str,
    pub ext_sinfo: Option<ext::SourceInfoType>,
    pub ext_body: Option<ext::QueryBodyType<'a>>,
    pub ext_attachment: Option<ext::AttachmentType<'a>>,
}

pub mod ext {
    use crate::{common::extension::ZExtZBuf, zextzbuf};

    /// # SourceInfo extension
    /// Used to carry additional information about the source of data
    pub type SourceInfo<'a> = zextzbuf!('a, 0x1, false);
    pub type SourceInfoType = crate::zenoh::ext::SourceInfoType<{ SourceInfo::ID }>;

    /// # QueryBody extension
    /// Used to carry a body attached to the query
    /// Shared Memory extension is automatically defined by ValueType extension if
    /// #[cfg(feature = "shared-memory")] is defined.
    pub type QueryBodyType<'a> =
        crate::zenoh::ext::ValueType<'a, { ZExtZBuf::<0x03>::id(false) }, 0x04>;

    /// # User attachment
    pub type Attachment<'a> = zextzbuf!('a, 0x5, false);
    pub type AttachmentType<'a> = crate::zenoh::ext::AttachmentType<'a, { Attachment::ID }>;
}
