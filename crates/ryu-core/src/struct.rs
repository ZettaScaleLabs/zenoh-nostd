//! A ZStruct is a structure of the Zenoh Protocol. The trait provides methods to encode and decode such structures.
//!
//! A ZExt is a ZStruct with an associated kind, which indicates how the structure is represented in the protocol when it
//! is used as an extension field. The kind can be Unit, U64 or ZStruct.

use crate::{ByteIOError, ByteIOResult, ByteReader, ByteReaderExt, ByteWriter};

mod array;
mod bytes;
mod option;
mod str;
mod uint;

pub trait ZStruct {
    fn z_len(&self) -> usize;

    fn z_encode(&self, w: &mut ByteWriter) -> ByteIOResult<()>;

    type ZType<'a>: Sized;

    fn z_decode<'a>(r: &mut ByteReader<'a>) -> ByteIOResult<Self::ZType<'a>>;
}

const MORE_MASK: /*_*/  u8 = 0b1000_0000;
const KIND_MASK: /*_*/  u8 = 0b0110_0000;
const MAND_MASK: /*_*/  u8 = 0b0001_0000;
const ID_MASK: /*_*/    u8 = 0b0000_1111;

const ZEXT_UNIT: /*_*/      u8 = 0b00 << 5;
const ZEXT_U64: /*_*/       u8 = 0b01 << 5;
const ZEXT_ZSTRUCT: /*_*/   u8 = 0b10 << 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ZExtKind {
    Unit = ZEXT_UNIT,
    U64 = ZEXT_U64,
    ZStruct = ZEXT_ZSTRUCT,
}

impl From<ZExtKind> for u8 {
    fn from(kind: ZExtKind) -> Self {
        kind as u8
    }
}

impl TryFrom<u8> for ZExtKind {
    type Error = ByteIOError;

    fn try_from(value: u8) -> ByteIOResult<Self> {
        match value & KIND_MASK {
            ZEXT_UNIT => Ok(ZExtKind::Unit),
            ZEXT_U64 => Ok(ZExtKind::U64),
            ZEXT_ZSTRUCT => Ok(ZExtKind::ZStruct),
            _ => Err(ByteIOError::CouldNotParse),
        }
    }
}

pub trait ZExt: ZStruct {
    const KIND: ZExtKind;

    fn z_len(&self) -> usize {
        match Self::KIND {
            ZExtKind::Unit | ZExtKind::U64 => <Self as ZStruct>::z_len(self),
            ZExtKind::ZStruct => {
                <usize as ZStruct>::z_len(&<Self as ZStruct>::z_len(self))
                    + <Self as ZStruct>::z_len(self)
            }
        }
    }

    fn z_encode(&self, w: &mut ByteWriter) -> ByteIOResult<()> {
        if Self::KIND == ZExtKind::ZStruct {
            <usize as ZStruct>::z_encode(&<Self as ZStruct>::z_len(self), w)?;
        }

        <Self as ZStruct>::z_encode(self, w)
    }

    fn z_decode<'a>(r: &mut ByteReader<'a>) -> ByteIOResult<Self::ZType<'a>> {
        if Self::KIND == ZExtKind::ZStruct {
            let len = <usize as ZStruct>::z_decode(r)?;
            <Self as ZStruct>::z_decode(&mut <ByteReader as ByteReaderExt>::sub(r, len)?)
        } else {
            <Self as ZStruct>::z_decode(r)
        }
    }
}

pub trait ZExtField<T>: ZExt + ZStruct {
    const ID: u8;
    const MANDATORY: bool;

    const HEADER: u8 = (Self::ID | Self::KIND as u8) | if Self::MANDATORY { MAND_MASK } else { 0 };

    fn z_len(&self) -> usize {
        1 + <Self as ZExt>::z_len(self)
    }

    fn z_encode(&self, w: &mut ByteWriter, more: bool) -> ByteIOResult<()> {
        let header = Self::HEADER | if more { MORE_MASK } else { 0 };

        <u8 as ZStruct>::z_encode(&header, w)?;
        <Self as ZExt>::z_encode(self, w)
    }

    fn z_decode<'a>(r: &mut ByteReader<'a>) -> ByteIOResult<Self::ZType<'a>> {
        let _ = <u8 as ZStruct>::z_decode(r)?;

        <Self as ZExt>::z_decode(r)
    }
}

pub fn skip_ext(r: &mut ByteReader, kind: ZExtKind) -> ByteIOResult<()> {
    let _ = <u8 as ZStruct>::z_decode(r)?;

    match kind {
        ZExtKind::Unit => {}
        ZExtKind::U64 => {
            let _ = <u64 as ZStruct>::z_decode(r)?;
        }
        ZExtKind::ZStruct => {
            let len = <usize as ZStruct>::z_decode(r)?;
            let _ = <ByteReader as ByteReaderExt>::sub(r, len)?;
        }
    }

    Ok(())
}

pub fn decode_ext_header(r: &mut ByteReader) -> ByteIOResult<(u8, ZExtKind, bool, bool)> {
    let header = r.peek_u8()?;

    let id = header & ID_MASK;
    let kind = ZExtKind::try_from(header & KIND_MASK)?;
    let mandatory = (header & MAND_MASK) != 0;
    let more = (header & MORE_MASK) != 0;

    Ok((id, kind, mandatory, more))
}

#[macro_export]
macro_rules! zextfield {
    (impl<'a> $ext:ty, $t:ty, $id:expr, $m:expr) => {
        impl<'a> zenoh_codec::ZExtField<$t> for $ext {
            const ID: u8 = $id;
            const MANDATORY: bool = $m;
        }
    };

    ($ext:ty, $t:ty, $id:expr, $m:expr) => {
        impl zenoh_codec::ZExtField<$t> for $ext {
            const ID: u8 = $id;
            const MANDATORY: bool = $m;
        }
    };
}

pub mod marker {
    #[derive(Debug, PartialEq)]
    pub struct Flag;

    #[derive(Debug, PartialEq)]
    pub struct Header;

    #[derive(Debug, PartialEq)]
    pub struct Phantom;

    #[derive(Debug, PartialEq)]
    pub struct ExtBlockBegin;

    #[derive(Debug, PartialEq)]
    pub struct ExtBlockEnd;
}
