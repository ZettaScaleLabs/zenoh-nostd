use uhlc::NTP64;

use crate::{ByteIOError, ByteIOResult, ByteReaderExt, ByteWriter, ZExt, ZExtKind, bail};
use core::{
    convert::{TryFrom, TryInto},
    fmt::{self},
};

use crate::ZStruct;

pub mod encoding;
pub mod resolution;
pub mod whatami;

#[derive(PartialEq)]
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
    pub fn rand() -> ZenohIdProto {
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
    type Error = ByteIOError;

    fn try_from(val: &[u8]) -> ByteIOResult<Self> {
        match val.try_into() {
            Ok(ok) => Ok(Self(ok)),
            Err(_) => bail!(ByteIOError::CouldNotParse),
        }
    }
}

impl ZStruct for ZenohIdProto {
    fn z_len(&self) -> usize {
        self.size()
    }

    fn z_encode(&self, w: &mut ByteWriter) -> crate::ByteIOResult<()> {
        let bytes = &self.as_le_bytes()[..self.size()];
        <&[u8] as ZStruct>::z_encode(&bytes, w)
    }

    type ZType<'a> = ZenohIdProto;
    fn z_decode<'a>(r: &mut crate::ByteReader<'a>) -> crate::ByteIOResult<Self::ZType<'a>> {
        let bytes = <&[u8] as ZStruct>::z_decode(r)?;
        Ok(ZenohIdProto::try_from(bytes)?)
    }
}

impl ZStruct for uhlc::Timestamp {
    fn z_len(&self) -> usize {
        let bytes = &self.get_id().to_le_bytes()[..self.get_id().size()];
        let time = self.get_time().as_u64();

        <u64 as ZStruct>::z_len(&time)
            + <usize as ZStruct>::z_len(&bytes.len())
            + <&[u8] as ZStruct>::z_len(&bytes)
    }

    fn z_encode(&self, w: &mut ByteWriter) -> ByteIOResult<()> {
        <u64 as ZStruct>::z_encode(&self.get_time().as_u64(), w)?;
        let bytes = &self.get_id().to_le_bytes()[..self.get_id().size()];
        <usize as ZStruct>::z_encode(&bytes.len(), w)?;
        <&[u8] as ZStruct>::z_encode(&bytes, w)
    }

    type ZType<'a> = uhlc::Timestamp;
    fn z_decode<'a>(r: &mut crate::ByteReader<'a>) -> ByteIOResult<Self::ZType<'a>> {
        let time = NTP64(<u64 as ZStruct>::z_decode(r)?);
        let id_len = <usize as ZStruct>::z_decode(r)?;
        let id_bytes = <&[u8] as ZStruct>::z_decode(&mut r.sub(id_len)?)?;
        let id = uhlc::ID::try_from(id_bytes).map_err(|_| ByteIOError::CouldNotParse)?;
        Ok(uhlc::Timestamp::new(time, id))
    }
}

impl ZExt for uhlc::Timestamp {
    const KIND: crate::ZExtKind = ZExtKind::ZStruct;
}
