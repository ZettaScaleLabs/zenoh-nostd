use crate::{
    Encoding, Timestamp, ZBodyDecode, ZBodyEncode, ZBodyLen, ZDecode, ZEncode, ZEnum, ZExt, ZLen,
    ZReader, ZStruct, ZWriter, ZenohIdProto,
};

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|E|_|ID:5=0x5")]
pub struct Err<'a> {
    // --- Optional attributes ---
    #[zenoh(presence = header(E), default = Encoding::DEFAULT)]
    pub encoding: Encoding<'a>,

    // --- Extension block ---
    #[zenoh(ext = 0x1)]
    pub sinfo: Option<SourceInfo>,

    // --- Body ---
    #[zenoh(size = prefixed)]
    pub payload: &'a [u8],
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|E|T|ID:5=0x1")]
pub struct Put<'a> {
    // --- Optional attributes ---
    #[zenoh(presence = header(T))]
    pub timestamp: Option<Timestamp>,
    #[zenoh(presence = header(E), default = Encoding::DEFAULT)]
    pub encoding: Encoding<'a>,

    // --- Extension block ---
    #[zenoh(ext = 0x1)]
    pub sinfo: Option<SourceInfo>,
    #[zenoh(ext = 0x3)]
    pub attachment: Option<Attachment<'a>>,

    // --- Body ---
    #[zenoh(size = prefixed)]
    pub payload: &'a [u8],
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|P|C|ID:5=0x3")]
pub struct Query<'a> {
    // --- Optional attributes ---
    #[zenoh(presence = header(C), default = ConsolidationMode::default())]
    pub consolidation: ConsolidationMode,
    #[zenoh(presence = header(P), size = prefixed, default = "")]
    pub parameters: &'a str,

    // --- Extension Block ---
    #[zenoh(ext = 0x1)]
    pub sinfo: Option<SourceInfo>,
    #[zenoh(ext = 0x3)]
    pub body: Option<Value<'a>>,
    #[zenoh(ext = 0x5)]
    pub attachment: Option<Attachment<'a>>,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|_|C|ID:5=0x4")]
pub struct Reply<'a> {
    // --- Optional attributes ---
    #[zenoh(presence = header(C), default = ConsolidationMode::default())]
    pub consolidation: ConsolidationMode,

    // --- Body ---
    pub payload: PushBody<'a>,
}

#[derive(ZEnum, Debug, PartialEq)]
pub enum PushBody<'a> {
    Put(Put<'a>),
}

#[derive(ZEnum, Debug, PartialEq)]
pub enum RequestBody<'a> {
    Query(Query<'a>),
}

#[derive(ZEnum, Debug, PartialEq)]
pub enum ResponseBody<'a> {
    Err(Err<'a>),
    Reply(Reply<'a>),
}

#[derive(ZExt, Debug, PartialEq)]
#[zenoh(header = "ID:4|_:4")]
pub struct EntityGlobalId {
    #[zenoh(size = header(ID))]
    pub zid: ZenohIdProto,

    pub eid: u32,
}

#[derive(ZExt, Debug, PartialEq)]
pub struct SourceInfo {
    pub id: EntityGlobalId,
    pub sn: u32,
}

#[derive(ZExt, Debug, PartialEq)]
pub struct Value<'a> {
    pub encoding: Encoding<'a>,

    #[zenoh(size = remain)]
    pub payload: &'a [u8],
}

#[derive(ZExt, Debug, PartialEq)]
pub struct Attachment<'a> {
    #[zenoh(size = remain)]
    pub buffer: &'a [u8],
}

#[repr(u8)]
#[derive(Debug, Default, Clone, PartialEq, Copy)]
pub enum ConsolidationMode {
    #[default]
    Auto = 0,
    None = 1,
    Monotonic = 2,
    Latest = 3,
}

impl ZBodyLen for ConsolidationMode {
    fn z_body_len(&self) -> usize {
        <u64 as ZLen>::z_len(&((*self as u8) as u64))
    }
}

impl ZBodyEncode for ConsolidationMode {
    fn z_body_encode(&self, w: &mut ZWriter) -> crate::ZResult<(), crate::ZCodecError> {
        <u64 as ZEncode>::z_encode(&((*self as u8) as u64), w)
    }
}

impl<'a> ZBodyDecode<'a> for ConsolidationMode {
    type Ctx = ();

    fn z_body_decode(r: &mut ZReader<'a>, _: ()) -> crate::ZResult<Self, crate::ZCodecError> {
        let value = <u64 as ZDecode>::z_decode(r)?;
        match value as u8 {
            0 => Ok(ConsolidationMode::Auto),
            1 => Ok(ConsolidationMode::None),
            2 => Ok(ConsolidationMode::Monotonic),
            3 => Ok(ConsolidationMode::Latest),
            _ => Err(crate::ZCodecError::CouldNotParseField),
        }
    }
}

crate::derive_zstruct_with_body!(ConsolidationMode);
