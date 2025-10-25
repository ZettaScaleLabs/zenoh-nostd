use crate::{
    protocol::{
        ZCodecError,
        codec::{decode_u8, decode_u64, decode_usize, decode_zbuf, encode_u8},
        has_flag,
    },
    result::ZResult,
    zbuf::{ZBufReader, ZBufWriter},
};

const EXT_ID_BITS: u8 = 4;
const EXT_ID_MASK: u8 = !(u8::MAX << EXT_ID_BITS);

const FLAG_M: u8 = 1 << 4;
const FLAG_Z: u8 = 1 << 7;

const ENC_UNIT: u8 = 0b00 << 5;
const ENC_U64: u8 = 0b01 << 5;
const ENC_ZBUF: u8 = 0b10 << 5;
const ENC_MASK: u8 = 0b11 << 5;

const fn ext_header(id: u8, mandatory: bool, encoding: u8) -> u8 {
    let mut header = (id & EXT_ID_MASK) | encoding;

    if mandatory {
        header |= FLAG_M;
    } else {
        header &= !FLAG_M;
    }

    header
}

const fn ext_with_more(header: u8, more: bool) -> u8 {
    if more { header | FLAG_Z } else { header }
}

const fn ext_id(header: u8) -> u8 {
    header & EXT_ID_MASK
}

const fn ext_mandatory(header: u8) -> bool {
    has_flag(header, FLAG_M)
}

const fn ext_kind(header: u8) -> ZExtKind {
    match header & ENC_MASK {
        ENC_UNIT => ZExtKind::Unit,
        ENC_U64 => ZExtKind::U64,
        ENC_ZBUF => ZExtKind::ZBuf,
        _ => panic!("Invalid extension encoding"),
    }
}

const fn ext_has_more(header: u8) -> bool {
    has_flag(header, FLAG_Z)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ZExtKind {
    Unit,
    U64,
    ZBuf,
}

pub(crate) trait ZExt {
    const KIND: ZExtKind;

    const ENCODING: u8 = match Self::KIND {
        ZExtKind::Unit => ENC_UNIT,
        ZExtKind::U64 => ENC_U64,
        ZExtKind::ZBuf => ENC_ZBUF,
    };
}

#[macro_export]
macro_rules! zext {
    (impl<$($lt:lifetime),+> $ext:ty, $primitive:ty, $id:expr, $mandatory:expr) => {
        impl<$($lt),+> crate::protocol::ext::ZExtPrimitive<$primitive> for $ext {
            const ID: u8 = $id;
            const MANDATORY: bool = $mandatory;
        }
    };

    ($ext:ident $(<$lt:lifetime>)?, $kind:expr, |$w:ident, $x:ident| $encode:expr, |$r:ident| $decode:expr) => {
        // impl<$($lt)?> crate::protocol::ext::ZExt for $ext<$($lt)?> {
        //     const KIND: crate::protocol::ext::ZExtKind = $kind;
        // }

        paste::paste! {
            pub(crate) fn [<encode_ $ext:snake>]<$($lt,)? Primitive>(writer: &mut crate::zbuf::ZBufWriter<'_>, x: Option<&$ext<$($lt)?>>, more: bool)
            -> crate::result::ZResult<bool, crate::protocol::ZCodecError>
                where $ext<$($lt)?>: crate::protocol::ext::ZExtPrimitive<Primitive>
            {
                if let Some(x) = x {
                    crate::protocol::ext::encode_ext_header::<$ext, Primitive>(writer, more)?;

                    let closure = |$w: &mut crate::zbuf::ZBufWriter<'_>, $x: & $ext<$($lt)?>|
                        -> crate::result::ZResult<(), crate::protocol::ZCodecError> { $encode };

                    closure(writer, x)?;

                    return Ok(true);
                }

                Ok(false)
            }

            pub(crate) fn [<decode_ $ext:snake>]<$($lt,)? Primitive>(reader: &mut crate::zbuf::ZBufReader<$($lt)?>)
            -> crate::result::ZResult<$ext<$($lt)?>, crate::protocol::ZCodecError>
                where $ext<$($lt)?>: crate::protocol::ext::ZExtPrimitive<Primitive>
            {
                let closure = |$r: &mut crate::zbuf::ZBufReader<$($lt)?>| -> crate::result::ZResult<$ext<$($lt)?>, crate::protocol::ZCodecError> { $decode };

                closure(reader)
            }
        }
    };
}

pub(crate) trait ZExtPrimitive<Primitive>: ZExt {
    const ID: u8;
    const MANDATORY: bool;

    const HEADER: u8 = ext_header(Self::ID, Self::MANDATORY, Self::ENCODING);
}

pub(crate) fn skip_ext(reader: &mut ZBufReader<'_>, kind: ZExtKind) -> ZResult<(), ZCodecError> {
    match kind {
        ZExtKind::Unit => {}
        ZExtKind::U64 => {
            let _ = decode_u64(reader)?;
        }
        ZExtKind::ZBuf => {
            let len = decode_usize(reader)?;
            let _ = decode_zbuf(reader, len)?;
        }
    };

    Ok(())
}

pub(crate) fn encode_ext_header<E, P>(
    writer: &mut ZBufWriter<'_>,
    more: bool,
) -> ZResult<(), ZCodecError>
where
    E: ZExtPrimitive<P>,
{
    let header = E::HEADER;
    encode_u8(writer, ext_with_more(header, more))
}

pub(crate) fn decode_ext_header(
    reader: &mut ZBufReader<'_>,
) -> ZResult<(u8, ZExtKind, bool, bool), ZCodecError> {
    let header = decode_u8(reader)?;

    let id = ext_id(header);
    let kind = ext_kind(header);
    let mandatory = ext_mandatory(header);
    let more = ext_has_more(header);

    Ok((id, kind, mandatory, more))
}

#[macro_export]
macro_rules! zext_id {
    ($ext:ty, $p:ty) => {
        <$ext as $crate::protocol::ext::ZExtPrimitive<$p>>::ID
    };
    ($ext:ty) => {
        crate::zext_id!($ext, Self)
    };
}
