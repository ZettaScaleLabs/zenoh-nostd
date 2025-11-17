use uhlc::NTP64;

#[cfg(test)]
use rand::{Rng, thread_rng};

use crate::{
    ZBodyDecode, ZBodyEncode, ZBodyLen, ZCodecError, ZCodecResult, ZDecode, ZEncode, ZExt,
    ZExtKind, ZLen, ZReader, ZReaderExt, ZWriter, zbail,
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

#[derive(PartialEq, Clone)]
#[repr(transparent)]
pub struct ZenohIdProto(uhlc::ID);

impl ZenohIdProto {
    #[inline]
    pub fn size(&self) -> usize {
        self.0.size()
    }

    #[inline]
    pub fn as_le_bytes(&self) -> [u8; uhlc::ID::MAX_SIZE] {
        self.0.to_le_bytes()
    }

    #[cfg(test)]
    pub fn rand(_: &mut ZWriter) -> ZenohIdProto {
        ZenohIdProto(uhlc::ID::rand())
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
    type Error = ZCodecError;

    fn try_from(val: &[u8]) -> ZCodecResult<Self> {
        match val.try_into() {
            Ok(ok) => Ok(Self(ok)),
            Err(_) => zbail!(ZCodecError::CouldNotParse),
        }
    }
}

impl ZBodyLen for ZenohIdProto {
    fn z_body_len(&self) -> usize {
        self.size()
    }
}

impl ZBodyEncode for ZenohIdProto {
    fn z_body_encode(&self, w: &mut ZWriter) -> ZCodecResult<()> {
        let bytes = &self.as_le_bytes()[..self.size()];
        <&[u8] as ZEncode>::z_encode(&bytes, w)
    }
}

impl<'a> ZBodyDecode<'a> for ZenohIdProto {
    type Ctx = ();

    fn z_body_decode(r: &mut ZReader<'a>, _: ()) -> ZCodecResult<Self> {
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
    fn z_body_encode(&self, w: &mut ZWriter) -> ZCodecResult<()> {
        <u64 as ZEncode>::z_encode(&self.get_time().as_u64(), w)?;
        let bytes = &self.get_id().to_le_bytes()[..self.get_id().size()];
        <usize as ZEncode>::z_encode(&bytes.len(), w)?;
        <&[u8] as ZEncode>::z_encode(&bytes, w)
    }
}

impl<'a> ZBodyDecode<'a> for Timestamp {
    type Ctx = ();

    fn z_body_decode(r: &mut ZReader<'a>, _: ()) -> ZCodecResult<Self> {
        let time = NTP64(<u64 as ZDecode>::z_decode(r)?);
        let id_len = <usize as ZDecode>::z_decode(r)?;
        let id_bytes = <&[u8] as ZDecode>::z_decode(&mut r.sub(id_len)?)?;
        let id = uhlc::ID::try_from(id_bytes).map_err(|_| ZCodecError::CouldNotParse)?;
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
    type Error = ZCodecError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Reliability::BestEffort),
            1 => Ok(Reliability::Reliable),
            _ => Err(ZCodecError::CouldNotParse),
        }
    }
}

impl Reliability {
    pub const DEFAULT: Self = Self::Reliable;

    #[cfg(test)]
    pub fn rand(_: &mut ZWriter) -> Self {
        if thread_rng().gen_bool(0.5) {
            Reliability::Reliable
        } else {
            Reliability::BestEffort
        }
    }
}
