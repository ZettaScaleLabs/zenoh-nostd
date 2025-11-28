use uhlc::NTP64;

use crate::{
    ZBodyDecode, ZBodyEncode, ZBodyLen, ZDecode, ZEncode, ZExt, ZExtKind, ZLen, ZReader,
    ZReaderExt, ZWriter, zbail,
};

use core::{
    convert::{TryFrom, TryInto},
    fmt::{self},
};

pub use uhlc::Timestamp;

mod encoding;
pub use encoding::*;

mod resolution;
pub use resolution::*;

mod whatami;
pub use whatami::*;

mod wire_expr;
pub use wire_expr::*;

mod endpoint;
pub use endpoint::*;

mod ke;
pub use ke::*;

#[derive(PartialEq, Clone)]
#[repr(transparent)]
pub struct ZenohIdProto(pub(crate) uhlc::ID);

impl ZenohIdProto {
    #[inline]
    pub fn size(&self) -> usize {
        self.0.size()
    }

    #[inline]
    pub fn as_le_bytes(&self) -> [u8; uhlc::ID::MAX_SIZE] {
        self.0.to_le_bytes()
    }
}
impl fmt::Debug for ZenohIdProto {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for ZenohIdProto {
    fn default() -> Self {
        Self(uhlc::ID::rand())
    }
}

impl TryFrom<&[u8]> for ZenohIdProto {
    type Error = crate::ZCodecError;

    fn try_from(val: &[u8]) -> crate::ZResult<Self, crate::ZCodecError> {
        match val.try_into() {
            Ok(ok) => Ok(Self(ok)),
            Err(_) => zbail!(crate::ZCodecError::CouldNotParseField),
        }
    }
}

impl ZBodyLen for ZenohIdProto {
    fn z_body_len(&self) -> usize {
        self.size()
    }
}

impl ZBodyEncode for ZenohIdProto {
    fn z_body_encode(&self, w: &mut ZWriter) -> crate::ZResult<(), crate::ZCodecError> {
        let bytes = &self.as_le_bytes()[..self.size()];
        <&[u8] as ZEncode>::z_encode(&bytes, w)
    }
}

impl<'a> ZBodyDecode<'a> for ZenohIdProto {
    type Ctx = ();

    fn z_body_decode(r: &mut ZReader<'a>, _: ()) -> crate::ZResult<Self, crate::ZCodecError> {
        let bytes = <&[u8] as ZDecode>::z_decode(r)?;
        ZenohIdProto::try_from(bytes)
    }
}

crate::derive_zstruct_with_body!(ZenohIdProto);

impl ZBodyLen for Timestamp {
    fn z_body_len(&self) -> usize {
        let bytes = &self.get_id().to_le_bytes()[..self.get_id().size()];
        let time = self.get_time().as_u64();

        <u64 as ZLen>::z_len(&time)
            + <usize as ZLen>::z_len(&bytes.len())
            + <&[u8] as ZLen>::z_len(&bytes)
    }
}

impl ZBodyEncode for Timestamp {
    fn z_body_encode(&self, w: &mut ZWriter) -> crate::ZResult<(), crate::ZCodecError> {
        <u64 as ZEncode>::z_encode(&self.get_time().as_u64(), w)?;
        let bytes = &self.get_id().to_le_bytes()[..self.get_id().size()];
        <usize as ZEncode>::z_encode(&bytes.len(), w)?;
        <&[u8] as ZEncode>::z_encode(&bytes, w)
    }
}

impl<'a> ZBodyDecode<'a> for Timestamp {
    type Ctx = ();

    fn z_body_decode(r: &mut ZReader<'a>, _: ()) -> crate::ZResult<Self, crate::ZCodecError> {
        let time = NTP64(<u64 as ZDecode>::z_decode(r)?);
        let id_len = <usize as ZDecode>::z_decode(r)?;
        let id_bytes = <&[u8] as ZDecode>::z_decode(&mut r.sub(id_len)?)?;
        let id =
            uhlc::ID::try_from(id_bytes).map_err(|_| crate::ZCodecError::CouldNotParseField)?;
        Ok(Timestamp::new(time, id))
    }
}

crate::derive_zstruct_with_body!(Timestamp);

impl<'a> ZExt<'a> for Timestamp {
    const KIND: ZExtKind = ZExtKind::ZStruct;
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Reliability {
    BestEffort = 0,
    #[default]
    Reliable = 1,
}

impl From<Reliability> for u8 {
    fn from(value: Reliability) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for Reliability {
    type Error = crate::ZCodecError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Reliability::BestEffort),
            1 => Ok(Reliability::Reliable),
            _ => Err(crate::ZCodecError::CouldNotParseField),
        }
    }
}
impl Reliability {
    pub const DEFAULT: Self = Self::Reliable;
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
