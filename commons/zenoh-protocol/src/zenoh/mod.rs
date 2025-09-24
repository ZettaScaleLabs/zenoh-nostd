// pub mod del;
// pub mod err;
pub mod put;
// pub mod query;
// pub mod reply;

// pub use del::Del;
// pub use err::Err;
pub use put::Put;
// pub use query::{ConsolidationMode, Query};
// pub use reply::Reply;

use crate::core::Encoding;

pub mod id {
    pub const OAM: u8 = 0x00;
    pub const PUT: u8 = 0x01;
    pub const DEL: u8 = 0x02;
    pub const QUERY: u8 = 0x03;
    pub const REPLY: u8 = 0x04;
    pub const ERR: u8 = 0x05;
}

// DataInfo
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataInfo {
    pub encoding: Encoding,
}

// Push
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PushBody {
    Put(Put),
    // Del(Del),
}

// impl From<Put> for PushBody {
//     fn from(p: Put) -> PushBody {
//         PushBody::Put(p)
//     }
// }

// impl From<Del> for PushBody {
//     fn from(d: Del) -> PushBody {
//         PushBody::Del(d)
//     }
// }

// // Request
// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum RequestBody {
//     Query(Query),
// }

// impl From<Query> for RequestBody {
//     fn from(q: Query) -> ZRequestBody {
//         RequestBody::Query(q)
//     }
// }

// Response
// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum ResponseBody {
//     Reply(Reply),
//     Err(Err),
// }

// impl From<Reply> for ResponseBody {
//     fn from(r: Reply) -> ZResponseBody {
//         ResponseBody::Reply(r)
//     }
// }

// impl From<Err> for ResponseBody {
//     fn from(r: Err) -> ZResponseBody {
//         ResponseBody::Err(r)
//     }
// }

pub mod ext {
    use zenoh_buffers::zbuf::ZBuf;

    use crate::core::{Encoding, EntityGlobalIdProto};

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
    pub struct ValueType<const VID: u8, const SID: u8> {
        pub encoding: Encoding,
        pub payload: ZBuf<1, 32>,
    }

    impl<const VID: u8, const SID: u8> ValueType<{ VID }, { SID }> {
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
    pub struct AttachmentType<const ID: u8> {
        pub buffer: ZBuf<1, 32>,
    }
}
