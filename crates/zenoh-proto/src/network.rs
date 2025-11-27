use crate::{
    WireExpr, ZBodyDecode, ZBodyEncode, ZBodyLen, ZDecode, ZEncode, ZEnum, ZExt, ZExtKind, ZHeader,
    ZLen, ZReader, ZStruct, ZWriter, zbail,
    zenoh::{EntityGlobalId, PushBody, RequestBody, ResponseBody},
};

use core::time::Duration;
use uhlc::Timestamp;

impl InterestInner<'_> {
    const HEADER_SLOT_FULL: u8 = 0b1111_1111;
}

#[derive(ZStruct, Debug)]
#[zenoh(header = "A|M|N|R|T|Q|S|K")]
pub struct InterestInner<'a> {
    #[zenoh(header = FULL)]
    pub options: u8,

    #[zenoh(presence = header(R), flatten, shift = 5)]
    pub wire_expr: Option<WireExpr<'a>>,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|_:7")]
pub struct InterestExt {
    #[zenoh(ext = 0x1, default = QoS::DEFAULT)]
    pub qos: QoS,
    #[zenoh(ext = 0x2)]
    pub timestamp: Option<Timestamp>,
    #[zenoh(ext = 0x3, default = NodeId::DEFAULT, mandatory)]
    pub nodeid: NodeId,
}

#[derive(Debug, PartialEq)]
// #[zenoh(header = "Z|MODE|_:5=0x19")]
pub struct Interest<'a> {
    pub id: u32,
    pub mode: InterestMode,

    pub inner: InterestInner<'a>,
    // #[zenoh(flatten)]
    pub ext: InterestExt,
}

impl Interest<'_> {
    const HEADER_BASE: u8 = 25u8;
    pub const ID: u8 = 25u8;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterestMode {
    Final,
    Current,
    Future,
    CurrentFuture,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct InterestOptions {
    pub options: u8,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|_|I|ID:5=0x1e")]
pub struct Declare<'a> {
    #[zenoh(presence = header(I))]
    pub id: Option<u32>,

    #[zenoh(ext = 0x1, default = QoS::DEFAULT)]
    pub qos: QoS,
    #[zenoh(ext = 0x2)]
    pub timestamp: Option<Timestamp>,
    #[zenoh(ext = 0x3, default = NodeId::DEFAULT, mandatory)]
    pub nodeid: NodeId,

    pub body: DeclareBody<'a>,
}

#[derive(ZEnum, Debug, PartialEq)]
pub enum DeclareBody<'a> {
    DeclareKeyExpr(DeclareKeyExpr<'a>),
    UndeclareKeyExpr(UndeclareKeyExpr),
    DeclareSubscriber(DeclareSubscriber<'a>),
    UndeclareSubscriber(UndeclareSubscriber<'a>),
    DeclareQueryable(DeclareQueryable<'a>),
    UndeclareQueryable(UndeclareQueryable<'a>),
    DeclareToken(DeclareToken<'a>),
    UndeclareToken(UndeclareToken<'a>),
    DeclareFinal(DeclareFinal),
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "_|M|N|ID:5=0x00")]
pub struct DeclareKeyExpr<'a> {
    pub id: u16,
    #[zenoh(flatten, shift = 5)]
    pub wire_expr: WireExpr<'a>,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "_:3|ID:5=0x01")]
pub struct UndeclareKeyExpr {
    pub id: u16,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "_|M|N|ID:5=0x02")]
pub struct DeclareSubscriber<'a> {
    pub id: u32,
    #[zenoh(flatten, shift = 5)]
    pub wire_expr: WireExpr<'a>,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|_:2|ID:5=0x03")]
pub struct UndeclareSubscriber<'a> {
    pub id: u32,
    #[zenoh(ext = 0x0f)]
    pub wire_expr: Option<WireExpr<'a>>,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|M|N|ID:5=0x04")]
pub struct DeclareQueryable<'a> {
    pub id: u32,
    #[zenoh(flatten, shift = 5)]
    pub wire_expr: WireExpr<'a>,

    #[zenoh(ext = 0x01, default = QueryableInfo::DEFAULT)]
    pub qinfo: QueryableInfo,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|_:2|ID:5=0x05")]
pub struct UndeclareQueryable<'a> {
    pub id: u32,
    #[zenoh(ext = 0x0f)]
    pub wire_expr: Option<WireExpr<'a>>,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|M|N|ID:5=0x06")]
pub struct DeclareToken<'a> {
    pub id: u32,
    #[zenoh(flatten, shift = 5)]
    pub wire_expr: WireExpr<'a>,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|_:2|ID:5=0x07")]
pub struct UndeclareToken<'a> {
    pub id: u32,
    #[zenoh(ext = 0x0f)]
    pub wire_expr: Option<WireExpr<'a>>,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|_:2|ID:5=0x1A")]
pub struct DeclareFinal {}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|M|N|ID:5=0x1d")]
pub struct Push<'a> {
    #[zenoh(flatten, shift = 5)]
    pub wire_expr: WireExpr<'a>,

    // --- Extension block ---
    #[zenoh(ext = 0x1, default = QoS::DEFAULT)]
    pub qos: QoS,
    #[zenoh(ext = 0x2)]
    pub timestamp: Option<Timestamp>,
    #[zenoh(ext = 0x3, default = NodeId::DEFAULT, mandatory)]
    pub nodeid: NodeId,

    // --- Body ---
    pub payload: PushBody<'a>,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|M|N|ID:5=0x1c")]
pub struct Request<'a> {
    pub id: u32,

    #[zenoh(flatten, shift = 5)]
    pub wire_expr: WireExpr<'a>,

    // --- Extension block ---
    #[zenoh(ext = 0x1, default = QoS::DEFAULT)]
    pub qos: QoS,
    #[zenoh(ext = 0x2)]
    pub timestamp: Option<Timestamp>,
    #[zenoh(ext = 0x3, default = NodeId::DEFAULT, mandatory)]
    pub nodeid: NodeId,
    #[zenoh(ext = 0x4, default = QueryTarget::DEFAULT, mandatory)]
    pub target: QueryTarget,
    #[zenoh(ext = 0x5)]
    pub budget: Option<Budget>,
    #[zenoh(ext = 0x6)]
    pub timeout: Option<Duration>,

    // --- Body ---
    pub payload: RequestBody<'a>,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|M|N|ID:5=0x1b")]
pub struct Response<'a> {
    pub rid: u32,

    #[zenoh(flatten, shift = 5)]
    pub wire_expr: WireExpr<'a>,

    // --- Extension block ---
    #[zenoh(ext = 0x1, default = QoS::DEFAULT)]
    pub qos: QoS,
    #[zenoh(ext = 0x2)]
    pub timestamp: Option<Timestamp>,
    #[zenoh(ext = 0x3)]
    pub respid: Option<EntityGlobalId>,

    // --- Body ---
    pub payload: ResponseBody<'a>,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|_:2|ID:5=0x1a")]
pub struct ResponseFinal {
    pub rid: u32,

    // --- Extension block ---
    #[zenoh(ext = 0x1, default = QoS::DEFAULT)]
    pub qos: QoS,
    #[zenoh(ext = 0x2)]
    pub timestamp: Option<Timestamp>,
}

#[derive(ZEnum, Debug, PartialEq)]
pub enum NetworkBody<'a> {
    Push(Push<'a>),
    Request(Request<'a>),
    Response(Response<'a>),
    ResponseFinal(ResponseFinal),
    Interest(Interest<'a>),
    Declare(Declare<'a>),
}

#[derive(ZExt, Debug, PartialEq)]
pub struct QoS {
    pub inner: u8,
}

impl QoS {
    const D_FLAG: u8 = 0b00001000;
    const E_FLAG: u8 = 0b00010000;

    pub const DEFAULT: Self = Self::new(Priority::DEFAULT, CongestionControl::DEFAULT, false);
    pub const DECLARE: Self =
        Self::new(Priority::DEFAULT, CongestionControl::DEFAULT_DECLARE, false);

    pub const fn new(
        priority: Priority,
        congestion_control: CongestionControl,
        is_express: bool,
    ) -> Self {
        let mut inner = priority as u8;
        if matches!(congestion_control, CongestionControl::Block) {
            inner |= Self::D_FLAG;
        }
        if is_express {
            inner |= Self::E_FLAG;
        }
        Self { inner }
    }
}

#[derive(ZExt, Debug, PartialEq)]
pub struct NodeId {
    pub node_id: u16,
}
impl NodeId {
    pub const DEFAULT: Self = Self { node_id: 0 };
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum QueryTarget {
    #[default]
    BestMatching = 0,
    All = 1,
    AllComplete = 2,
}

impl QueryTarget {
    pub const DEFAULT: Self = Self::BestMatching;
}

impl ZBodyLen for QueryTarget {
    fn z_body_len(&self) -> usize {
        <u64 as ZLen>::z_len(&((*self as u8) as u64))
    }
}

impl ZBodyEncode for QueryTarget {
    fn z_body_encode(&self, w: &mut ZWriter) -> crate::ZResult<(), crate::ZCodecError> {
        <u64 as ZEncode>::z_encode(&((*self as u8) as u64), w)
    }
}

impl ZBodyDecode<'_> for QueryTarget {
    type Ctx = ();

    fn z_body_decode(r: &mut ZReader<'_>, _: ()) -> crate::ZResult<Self, crate::ZCodecError> {
        let value = <u64 as ZDecode>::z_decode(r)?;

        match value as u8 {
            0 => Ok(QueryTarget::BestMatching),
            1 => Ok(QueryTarget::All),
            2 => Ok(QueryTarget::AllComplete),
            _ => Err(crate::ZCodecError::CouldNotParseField),
        }
    }
}

crate::derive_zstruct_with_body!(QueryTarget);

impl<'a> ZExt<'a> for QueryTarget {
    const KIND: ZExtKind = ZExtKind::U64;
}

#[derive(ZExt, Debug, PartialEq)]
pub struct Budget {
    pub budget: u32,
}

impl ZHeader for Duration {
    fn z_header(&self) -> u8 {
        let header = 0u8;
        match self.as_millis() % 1_000 {
            0 => header | 0b0000_0001,
            _ => header,
        }
    }
}

impl ZBodyLen for Duration {
    fn z_body_len(&self) -> usize {
        let v = match self.as_millis() % 1_000 {
            0 => self.as_millis() / 1_000,
            _ => self.as_millis(),
        } as u64;

        <u64 as ZLen>::z_len(&v)
    }
}

impl ZLen for Duration {
    fn z_len(&self) -> usize {
        <u64 as ZLen>::z_len(&(self.as_millis() as u64))
    }
}

impl ZBodyEncode for Duration {
    fn z_body_encode(&self, w: &mut ZWriter) -> crate::ZResult<(), crate::ZCodecError> {
        let v = match self.as_millis() % 1_000 {
            0 => self.as_millis() / 1_000,
            _ => self.as_millis(),
        } as u64;

        <u64 as ZEncode>::z_encode(&v, w)
    }
}

impl ZEncode for Duration {
    fn z_encode(&self, w: &mut ZWriter) -> crate::ZResult<(), crate::ZCodecError> {
        <u64 as ZEncode>::z_encode(&(self.as_millis() as u64), w)
    }
}

impl<'a> ZBodyDecode<'a> for Duration {
    type Ctx = u8;

    fn z_body_decode(r: &mut ZReader<'a>, h: u8) -> crate::ZResult<Self, crate::ZCodecError> {
        let is_seconds = (h & 0b0000_0001) != 0;
        let value = <u64 as ZDecode>::z_decode(r)?;
        if is_seconds {
            Ok(Duration::from_secs(value))
        } else {
            Ok(Duration::from_millis(value))
        }
    }
}

impl<'a> ZDecode<'a> for Duration {
    fn z_decode(r: &mut ZReader<'a>) -> crate::ZResult<Self, crate::ZCodecError> {
        let value = <u64 as ZDecode>::z_decode(r)?;
        Ok(Duration::from_millis(value))
    }
}

impl<'a> ZExt<'a> for Duration {
    const KIND: ZExtKind = ZExtKind::U64;
}

#[derive(Debug, PartialEq)]
pub struct QueryableInfo {
    pub complete: bool,
    pub distance: u16,
}

impl ZBodyLen for QueryableInfo {
    fn z_body_len(&self) -> usize {
        <u64 as ZLen>::z_len(&self.as_u64())
    }
}

impl ZBodyEncode for QueryableInfo {
    fn z_body_encode(&self, w: &mut ZWriter) -> crate::ZResult<(), crate::ZCodecError> {
        <u64 as ZEncode>::z_encode(&self.as_u64(), w)
    }
}

impl ZBodyDecode<'_> for QueryableInfo {
    type Ctx = ();

    fn z_body_decode(r: &mut ZReader<'_>, _: ()) -> crate::ZResult<Self, crate::ZCodecError> {
        let value = <u64 as ZDecode>::z_decode(r)?;
        let complete = (value & 0b0000_0001) != 0;
        let distance = ((value >> 8) & 0xFFFF) as u16;
        Ok(QueryableInfo { complete, distance })
    }
}

crate::derive_zstruct_with_body!(QueryableInfo);

impl<'a> ZExt<'a> for QueryableInfo {
    const KIND: ZExtKind = ZExtKind::U64;
}

impl QueryableInfo {
    pub const DEFAULT: Self = Self {
        complete: false,
        distance: 0,
    };

    fn as_u64(&self) -> u64 {
        let mut flags: u8 = 0;
        if self.complete {
            flags |= 0b0000_0001;
        }
        (flags as u64) | ((self.distance as u64) << 8)
    }
}

#[repr(u8)]
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum Mapping {
    #[default]
    Receiver = 0,
    Sender = 1,
}

impl From<Mapping> for u8 {
    fn from(val: Mapping) -> u8 {
        val as u8
    }
}

impl TryFrom<u8> for Mapping {
    type Error = crate::ZCodecError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Mapping::Receiver),
            1 => Ok(Mapping::Sender),
            _ => Err(crate::ZCodecError::CouldNotParseField),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, PartialEq)]
pub enum Priority {
    #[default]
    Data = 5,
}

impl Priority {
    pub const DEFAULT: Self = Self::Data;
}

#[derive(Debug, Default, PartialEq)]
#[repr(u8)]
pub enum CongestionControl {
    #[default]
    Drop = 0,
    Block = 1,
}

impl CongestionControl {
    pub const DEFAULT: Self = Self::Drop;
    pub const DEFAULT_DECLARE: Self = Self::Block;
}

impl PartialEq for InterestInner<'_> {
    fn eq(&self, other: &Self) -> bool {
        let options = InterestOptions::options(self.options);
        let other_options = InterestOptions::options(other.options);

        options == other_options && self.wire_expr == other.wire_expr
    }
}

impl ZHeader for Interest<'_> {
    fn z_header(&self) -> u8 {
        let mut header: u8 = Self::HEADER_BASE;

        header |= match self.mode {
            InterestMode::Final => 0b00,
            InterestMode::Current => 0b01,
            InterestMode::Future => 0b10,
            InterestMode::CurrentFuture => 0b11,
        } << 5;

        header |= <_ as ZHeader>::z_header(&self.ext);

        header
    }
}

impl ZBodyLen for Interest<'_> {
    fn z_body_len(&self) -> usize {
        <u32 as ZLen>::z_len(&self.id)
            + if self.mode != InterestMode::Final {
                <_ as ZLen>::z_len(&self.inner)
            } else {
                0usize
            }
            + self.ext.z_body_len()
    }
}

impl ZBodyEncode for Interest<'_> {
    fn z_body_encode(&self, w: &mut ZWriter) -> crate::ZResult<(), crate::ZCodecError> {
        <u32 as ZBodyEncode>::z_body_encode(&self.id, w)?;

        if self.mode != InterestMode::Final {
            <_ as ZEncode>::z_encode(&self.inner, w)?;
        }

        <_ as ZBodyEncode>::z_body_encode(&self.ext, w)?;

        Ok(())
    }
}

impl<'a> ZBodyDecode<'a> for Interest<'a> {
    type Ctx = u8;

    fn z_body_decode(
        r: &mut crate::ZReader<'a>,
        header: u8,
    ) -> crate::ZResult<Self, crate::ZCodecError> {
        let id = <u32 as ZDecode>::z_decode(r)?;

        let mode = match (header >> 5) & 0b11 {
            0b00 => InterestMode::Final,
            0b01 => InterestMode::Current,
            0b10 => InterestMode::Future,
            0b11 => InterestMode::CurrentFuture,
            _ => zbail!(crate::ZCodecError::CouldNotParseField),
        };

        let inner = if mode != InterestMode::Final {
            <_ as ZDecode>::z_decode(r)?
        } else {
            InterestInner {
                options: 0,
                wire_expr: None,
            }
        };

        let ext = <_ as ZBodyDecode>::z_body_decode(r, header)?;

        Ok(Self {
            id,
            mode,
            inner,
            ext,
        })
    }
}

impl ZLen for Interest<'_> {
    fn z_len(&self) -> usize {
        1usize + <Self as ZBodyLen>::z_body_len(self)
    }
}

impl ZEncode for Interest<'_> {
    fn z_encode(&self, w: &mut ZWriter) -> crate::ZResult<(), crate::ZCodecError> {
        let header = <Self as ZHeader>::z_header(self);
        <u8 as ZEncode>::z_encode(&header, w)?;
        <Self as ZBodyEncode>::z_body_encode(self, w)
    }
}

impl<'a> ZDecode<'a> for Interest<'a> {
    fn z_decode(r: &mut crate::ZReader<'a>) -> crate::ZResult<Self, crate::ZCodecError> {
        let header = <u8 as ZDecode>::z_decode(r)?;
        <Self as ZBodyDecode>::z_body_decode(r, header)
    }
}

impl PartialEq for InterestOptions {
    fn eq(&self, other: &Self) -> bool {
        self.keyexprs() == other.keyexprs()
            && self.subscribers() == other.subscribers()
            && self.queryables() == other.queryables()
            && self.tokens() == other.tokens()
            && self.aggregate() == other.aggregate()
    }
}

impl InterestOptions {
    pub const KEYEXPRS: InterestOptions = InterestOptions::options(1);
    pub const SUBSCRIBERS: InterestOptions = InterestOptions::options(1 << 1);
    pub const QUERYABLES: InterestOptions = InterestOptions::options(1 << 2);
    pub const TOKENS: InterestOptions = InterestOptions::options(1 << 3);

    pub const AGGREGATE: InterestOptions = InterestOptions::options(1 << 7);

    const fn options(options: u8) -> Self {
        Self { options }
    }

    pub const fn keyexprs(&self) -> bool {
        self.options & Self::KEYEXPRS.options != 0
    }

    pub const fn subscribers(&self) -> bool {
        self.options & Self::SUBSCRIBERS.options != 0
    }

    pub const fn queryables(&self) -> bool {
        self.options & Self::QUERYABLES.options != 0
    }

    pub const fn tokens(&self) -> bool {
        self.options & Self::TOKENS.options != 0
    }

    pub const fn aggregate(&self) -> bool {
        self.options & Self::AGGREGATE.options != 0
    }
}
