use crate::zenoh::{err::Err, put::Put, query::Query, reply::Reply};

pub mod err;
pub mod put;
pub mod query;
pub mod reply;

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

// Request
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestBody<'a> {
    Query(Query<'a>),
}

// Response
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResponseBody<'a> {
    Reply(Reply<'a>),
    Err(Err<'a>),
}

pub mod ext {
    use zenoh_buffer::ZBuf;

    use crate::core::{encoding::Encoding, EntityGlobalIdProto};

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
    ///   7 6 5 4 3 2 1 0
    ///  +-+-+-+-+-+-+-+-+
    ///  ~   encoding    ~
    ///  +---------------+
    ///  ~ pl: [u8;z32]  ~  -- Payload
    ///  +---------------+
    /// ```
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ValueType<'a, const VID: u8, const SID: u8> {
        pub encoding: Encoding<'a>,
        pub payload: ZBuf<'a>,
    }

    impl<const VID: u8, const SID: u8> ValueType<'_, { VID }, { SID }> {
        pub const VID: u8 = VID;
        pub const SID: u8 = SID;
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
        pub buffer: ZBuf<'a>,
    }
}
