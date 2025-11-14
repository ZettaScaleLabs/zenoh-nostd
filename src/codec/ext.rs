use crate::{ZCodecError, ZCodecResult, ZDecode, ZEncode, ZLen, ZReader, ZReaderExt, ZWriter};

const KIND_MASK: u8 = 0b0110_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ZExtKind {
    Unit = 0b00 << 5,
    U64 = 0b01 << 5,
    ZStruct = 0b10 << 5,
}

impl From<ZExtKind> for u8 {
    fn from(kind: ZExtKind) -> Self {
        kind as u8
    }
}

impl TryFrom<u8> for ZExtKind {
    type Error = ZCodecError;

    fn try_from(value: u8) -> ZCodecResult<Self> {
        match value & KIND_MASK {
            0b0000_0000 => Ok(ZExtKind::Unit),
            0b0010_0000 => Ok(ZExtKind::U64),
            0b0100_0000 => Ok(ZExtKind::ZStruct),
            _ => Err(ZCodecError::CouldNotParse),
        }
    }
}

pub trait ZExt<'a>: ZLen + ZEncode + ZDecode<'a> {
    const KIND: ZExtKind;
}

const FLAG_MANDATORY: u8 = 1 << 4;
const FLAG_MORE: u8 = 1 << 7;
const ID_MASK: u8 = 0b0000_1111;

pub const fn zext_enc_id<'a, const ID: u8, T: ZExt<'a>>() -> u8 {
    ID | T::KIND as u8
}

pub const fn zext_eheader<'a, const ID: u8, const MANDATORY: bool, T: ZExt<'a>>() -> u8 {
    zext_enc_id::<ID, T>() | if MANDATORY { FLAG_MANDATORY } else { 0 }
}

pub const fn zext_header<'a, const ID: u8, const MANDATORY: bool, T: ZExt<'a>>(more: bool) -> u8 {
    zext_eheader::<ID, MANDATORY, T>() | if more { FLAG_MORE } else { 0 }
}

pub fn zext_len<'a, T: ZExt<'a>>(x: &T) -> usize {
    1 + match T::KIND {
        ZExtKind::Unit | ZExtKind::U64 => <T as ZLen>::z_len(x),
        ZExtKind::ZStruct => <usize as ZLen>::z_len(&<T as ZLen>::z_len(x)) + <T as ZLen>::z_len(x),
    }
}

pub fn zext_encode<'a, T: ZExt<'a>, const ID: u8, const MANDATORY: bool>(
    x: &T,
    w: &mut ZWriter,
    more: bool,
) -> ZCodecResult<()> {
    let header: u8 = zext_header::<ID, MANDATORY, T>(more);

    <u8 as ZEncode>::z_encode(&header, w)?;

    if T::KIND == ZExtKind::ZStruct {
        <usize as ZEncode>::z_encode(&<T as ZLen>::z_len(x), w)?;
    }

    <T as ZEncode>::z_encode(x, w)
}

pub fn zext_decode<'a, T: ZExt<'a>>(r: &mut ZReader<'a>) -> ZCodecResult<T> {
    let _ = <u8 as ZDecode>::z_decode(r)?;

    if T::KIND == ZExtKind::ZStruct {
        let len = <usize as ZDecode>::z_decode(r)?;
        <T as ZDecode>::z_decode(&mut <ZReader as ZReaderExt>::sub(r, len)?)
    } else {
        <T as ZDecode>::z_decode(r)
    }
}

pub fn skip_ext(r: &mut ZReader, kind: ZExtKind) -> ZCodecResult<()> {
    let _ = <u8 as ZDecode>::z_decode(r)?;

    match kind {
        ZExtKind::Unit => {}
        ZExtKind::U64 => {
            let _ = <u64 as ZDecode>::z_decode(r)?;
        }
        ZExtKind::ZStruct => {
            let len = <usize as ZDecode>::z_decode(r)?;
            let _ = <ZReader as ZReaderExt>::sub(r, len)?;
        }
    }

    Ok(())
}

pub fn decode_ext_header(r: &mut ZReader) -> ZCodecResult<(u8, ZExtKind, bool, bool)> {
    let header = r.peek_u8()?;

    let id = header & ID_MASK;
    let kind = ZExtKind::try_from(header & KIND_MASK)?;
    let mandatory = (header & FLAG_MANDATORY) != 0;
    let more = (header & FLAG_MORE) != 0;

    Ok((id, kind, mandatory, more))
}
