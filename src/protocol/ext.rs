use crate::{
    protocol::{
        ZCodecError,
        codec::{decode_u8, decode_u64, decode_zbuf},
        has_flag,
    },
    result::ZResult,
    zbail,
    zbuf::{BufReaderExt, ZBufReader},
};

const EXT_ID_BITS: u8 = 4;
const EXT_ID_MASK: u8 = !(u8::MAX << EXT_ID_BITS);

const FLAG_M: u8 = 1 << 4;
const FLAG_Z: u8 = 1 << 7;

const ENC_UNIT: u8 = 0b00 << 5;
const ENC_Z64: u8 = 0b01 << 5;
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

const fn ext_encoding(header: u8) -> u8 {
    header & ENC_MASK
}

const fn ext_has_more(header: u8) -> bool {
    has_flag(header, FLAG_Z)
}

pub(crate) enum ZExtKind {
    Unit,
    Z64,
    ZBuf,
}

pub(crate) trait ZExt {
    const KIND: ZExtKind;

    const ENCODING: u8 = match Self::KIND {
        ZExtKind::Unit => ENC_UNIT,
        ZExtKind::Z64 => ENC_Z64,
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

    ($ext:ty, $kind:expr) => {
        impl crate::protocol::ext::ZExt for $ext {
            const KIND: crate::protocol::ext::ZExtKind = $kind;
        }
    };
}

pub(crate) trait ZExtPrimitive<Primitive>: ZExt {
    const ID: u8;
    const MANDATORY: bool;

    const HEADER: u8 = ext_header(Self::ID, Self::MANDATORY, Self::ENCODING);
}

pub(crate) fn skip(s: &str, reader: &mut ZBufReader<'_>, header: u8) -> ZResult<bool, ZCodecError> {
    let id = ext_id(header);

    if ext_mandatory(header) {
        crate::error!("Mandatory extension {} with id {} not supported.", s, id,);

        zbail!(ZCodecError::CouldNotRead);
    }

    match ext_encoding(header) {
        ENC_UNIT => {}
        ENC_Z64 => {
            let _ = decode_u64(reader)?;
        }
        ENC_ZBUF => {
            let _ = decode_zbuf(reader, None)?;
        }
        _ => {
            zbail!(ZCodecError::CouldNotRead);
        }
    };

    Ok(ext_has_more(header))
}

pub(crate) fn skip_all(s: &str, reader: &mut ZBufReader<'_>) -> ZResult<(), ZCodecError> {
    let mut has_ext = reader.can_read();

    while has_ext {
        let header = decode_u8(reader)?;
        has_ext = skip(s, reader, header)?;
    }

    Ok(())
}
