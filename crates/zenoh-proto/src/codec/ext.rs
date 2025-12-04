use crate::*;

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
    type Error = crate::CodecError;

    fn try_from(value: u8) -> core::result::Result<Self, crate::CodecError> {
        match value & KIND_MASK {
            0b0000_0000 => Ok(ZExtKind::Unit),
            0b0010_0000 => Ok(ZExtKind::U64),
            0b0100_0000 => Ok(ZExtKind::ZStruct),
            _ => Err(crate::CodecError::CouldNotParseHeader),
        }
    }
}

pub trait ZExt<'a>: ZLen + ZEncode + ZDecode<'a> {
    const KIND: ZExtKind;
}

/// Special trait to solve the 'same ext id issue' in the zenoh protocol
pub trait ZExtResolveKind {
    fn ext_kind(&self) -> ZExtKind;
}

impl<'a, T: ZExt<'a>> ZExtResolveKind for T {
    fn ext_kind(&self) -> ZExtKind {
        T::KIND
    }
}

impl<'a, T: ZExt<'a>> ZExtResolveKind for Option<T> {
    fn ext_kind(&self) -> ZExtKind {
        T::KIND
    }
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
        ZExtKind::Unit | ZExtKind::U64 => x.z_len(),
        ZExtKind::ZStruct => <usize as ZLen>::z_len(&x.z_len()) + x.z_len(),
    }
}

pub fn zext_encode<'a, T: ZExt<'a>, const ID: u8, const MANDATORY: bool>(
    x: &T,
    w: &mut impl crate::ZWrite,
    more: bool,
) -> core::result::Result<(), crate::CodecError> {
    let header: u8 = zext_header::<ID, MANDATORY, T>(more);

    <u8 as ZEncode>::z_encode(&header, w)?;

    if T::KIND == ZExtKind::ZStruct {
        <usize as ZEncode>::z_encode(&x.z_len(), w)?;
    }

    <T as ZEncode>::z_encode(x, w)
}

pub fn zext_decode<'a, T: ZExt<'a>>(
    r: &mut impl crate::ZRead<'a>,
) -> core::result::Result<T, crate::CodecError> {
    let _ = <u8 as ZDecode>::z_decode(r)?;

    if T::KIND == ZExtKind::ZStruct {
        let len = <usize as ZDecode>::z_decode(r)?;
        <T as ZDecode>::z_decode(&mut r.read_slice(len)?)
    } else {
        <T as ZDecode>::z_decode(r)
    }
}

pub fn skip_ext<'a>(
    r: &mut impl crate::ZRead<'a>,
    kind: ZExtKind,
) -> core::result::Result<(), crate::CodecError> {
    let _ = <u8 as ZDecode>::z_decode(r)?;

    match kind {
        ZExtKind::Unit => {}
        ZExtKind::U64 => {
            let _ = <u64 as ZDecode>::z_decode(r)?;
        }
        ZExtKind::ZStruct => {
            let len = <usize as ZDecode>::z_decode(r)?;
            let _ = r.read_slice(len)?;
        }
    }

    Ok(())
}

pub fn decode_ext_header<'a>(
    r: &mut impl crate::ZRead<'a>,
) -> core::result::Result<(u8, ZExtKind, bool, bool), crate::CodecError> {
    let header = r.peek()?;

    let id = header & ID_MASK;
    let kind = ZExtKind::try_from(header & KIND_MASK)?;
    let mandatory = (header & FLAG_MANDATORY) != 0;
    let more = (header & FLAG_MORE) != 0;

    Ok((id, kind, mandatory, more))
}
