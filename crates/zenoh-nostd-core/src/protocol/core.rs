use uhlc::NTP64;

use crate::{
    ZCodecError, ZCodecResult, ZExt, ZExtKind, ZReader, ZReaderExt, ZStructDecode, ZStructEncode,
    ZWriter, zbail,
};

use core::{
    convert::{TryFrom, TryInto},
    fmt::{self},
};

pub use uhlc::Timestamp;

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

impl ZStructEncode for ZenohIdProto {
    fn z_len(&self) -> usize {
        self.size()
    }

    fn z_encode_without_header(&self, w: &mut ZWriter) -> ZCodecResult<()> {
        let bytes = &self.as_le_bytes()[..self.size()];
        <&[u8] as ZStructEncode>::z_encode(&bytes, w)
    }
}

impl<'a> ZStructDecode<'a> for ZenohIdProto {
    fn z_decode(r: &mut ZReader<'a>) -> ZCodecResult<Self> {
        let bytes = <&[u8] as ZStructDecode>::z_decode(r)?;
        ZenohIdProto::try_from(bytes)
    }
}

impl ZStructEncode for Timestamp {
    fn z_len(&self) -> usize {
        let bytes = &self.get_id().to_le_bytes()[..self.get_id().size()];
        let time = self.get_time().as_u64();

        <u64 as ZStructEncode>::z_len(&time)
            + <usize as ZStructEncode>::z_len(&bytes.len())
            + <&[u8] as ZStructEncode>::z_len(&bytes)
    }

    fn z_encode_without_header(&self, w: &mut ZWriter) -> ZCodecResult<()> {
        <u64 as ZStructEncode>::z_encode(&self.get_time().as_u64(), w)?;
        let bytes = &self.get_id().to_le_bytes()[..self.get_id().size()];
        <usize as ZStructEncode>::z_encode(&bytes.len(), w)?;
        <&[u8] as ZStructEncode>::z_encode(&bytes, w)
    }
}
impl<'a> ZStructDecode<'a> for Timestamp {
    fn z_decode(r: &mut ZReader<'a>) -> ZCodecResult<Self> {
        let time = NTP64(<u64 as ZStructDecode>::z_decode(r)?);
        let id_len = <usize as ZStructDecode>::z_decode(r)?;
        let id_bytes = <&[u8] as ZStructDecode>::z_decode(&mut r.sub(id_len)?)?;
        let id = uhlc::ID::try_from(id_bytes).map_err(|_| ZCodecError::CouldNotParse)?;
        Ok(Timestamp::new(time, id))
    }
}

impl<'a> ZExt<'a> for Timestamp {
    const KIND: ZExtKind = ZExtKind::ZStruct;
}
