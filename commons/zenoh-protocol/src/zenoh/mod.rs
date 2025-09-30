pub mod put;
pub use put::Put;

pub mod id {
    pub const OAM: u8 = 0x00;
    pub const PUT: u8 = 0x01;
    pub const DEL: u8 = 0x02;
    pub const QUERY: u8 = 0x03;
    pub const REPLY: u8 = 0x04;
    pub const ERR: u8 = 0x05;
}

// Push
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PushBody<'a> {
    Put(Put<'a>),
}

pub mod ext {
    use crate::core::EntityGlobalIdProto;

    /// ```text
    ///  7 6 5 4 3 2 1 0
    /// +-+-+-+-+-+-+-+-+
    /// |zid_len|X|X|X|X|
    /// +-------+-+-+---+
    /// ~      zid      ~
    /// +---------------+
    /// %      eid      %  -- Counter decided by the Zenoh Node
    /// +---------------+
    /// %      sn       %
    /// +---------------+
    /// ```
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SourceInfoType<const ID: u8> {
        pub id: EntityGlobalIdProto,
        pub sn: u32,
    }

    /// ```text
    /// 7 6 5 4 3 2 1 0
    /// +-+-+-+-+-+-+-+-+
    /// %   num elems   %
    /// +-------+-+-+---+
    /// ~ key: <u8;z16> ~
    /// +---------------+
    /// ~ val: <u8;z32> ~
    /// +---------------+
    ///       ...         -- N times (key, value) tuples
    /// ```
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct AttachmentType<'a, const ID: u8> {
        pub buffer: &'a [u8],
    }
}
