use core::fmt::{self, Debug};

use crate::{
    protocol::{
        ZCodecError,
        common::imsg::has_flag,
        zcodec::{
            decode_u8, decode_u64, decode_usize, decode_zbuf, encode_u8, encode_u64, encode_usize,
            encode_zbuf,
        },
    },
    result::ZResult,
    zbail,
    zbuf::{BufReaderExt, ZBuf, ZBufReader, ZBufWriter},
};

pub(crate) mod iext {
    use core::fmt;

    use crate::protocol::common::imsg::has_flag;

    pub(crate) const ID_BITS: u8 = 4;
    pub(crate) const ID_MASK: u8 = !(u8::MAX << ID_BITS);

    pub(crate) const FLAG_M: u8 = 1 << 4;

    pub(crate) const ENC_UNIT: u8 = 0b00 << 5;
    pub(crate) const ENC_Z64: u8 = 0b01 << 5;
    pub(crate) const ENC_ZBUF: u8 = 0b10 << 5;
    pub(crate) const ENC_MASK: u8 = 0b11 << 5;

    pub(crate) const FLAG_Z: u8 = 1 << 7;

    pub(crate) const fn eheader(header: u8) -> u8 {
        header & !FLAG_Z
    }

    pub(crate) const fn mheader(header: u8) -> u8 {
        header & ID_MASK
    }

    pub(super) const fn header(id: u8, mandatory: bool, encoding: u8) -> u8 {
        let mut header = id & ID_MASK;
        if mandatory {
            header |= FLAG_M;
        } else {
            header &= !FLAG_M;
        }
        header |= encoding;
        header
    }

    pub(super) const fn is_mandatory(id: u8) -> bool {
        has_flag(id, FLAG_M)
    }

    pub(super) fn fmt(f: &mut fmt::DebugStruct, id: u8) {
        f.field("Id", &(id & ID_MASK))
            .field("Mandatory", &is_mandatory(id))
            .field(
                "Encoding",
                match id & ENC_MASK {
                    ENC_UNIT => &"Unit",
                    ENC_Z64 => &"Z64",
                    ENC_ZBUF => &"ZBuf",
                    _ => &"Unknown",
                },
            );
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct ZExtUnit<const ID: u8>;

impl<const ID: u8> ZExtUnit<{ ID }> {
    pub(crate) const ID: u8 = ID;

    pub(crate) const fn id(mandatory: bool) -> u8 {
        iext::header(ID, mandatory, iext::ENC_UNIT)
    }

    pub(crate) fn encode(
        &self,
        more: bool,
        writer: &mut ZBufWriter<'_>,
    ) -> ZResult<usize, ZCodecError> {
        let mut header: u8 = ID;
        if more {
            header |= iext::FLAG_Z;
        }

        encode_u8(writer, header)?;

        Ok(1)
    }

    pub(crate) fn decode(header: u8) -> ZResult<(Self, bool), ZCodecError> {
        if iext::eheader(header) != ID {
            zbail!(ZCodecError::CouldNotRead);
        }

        Ok((ZExtUnit, has_flag(header, iext::FLAG_Z)))
    }

    #[cfg(test)]
    pub(crate) fn rand() -> Self {
        Self
    }
}

impl<const ID: u8> Debug for ZExtUnit<{ ID }> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("ZExtUnit");
        iext::fmt(&mut s, ID);
        s.finish()
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct ZExtZ64<const ID: u8> {
    pub(crate) value: u64,
}

impl<const ID: u8> ZExtZ64<{ ID }> {
    pub(crate) const ID: u8 = ID;

    pub(crate) const fn new(value: u64) -> Self {
        Self { value }
    }

    pub(crate) const fn id(mandatory: bool) -> u8 {
        iext::header(ID, mandatory, iext::ENC_Z64)
    }

    pub(crate) fn encode(
        &self,
        more: bool,
        writer: &mut ZBufWriter<'_>,
    ) -> ZResult<(), ZCodecError> {
        let mut header: u8 = ID;
        if more {
            header |= iext::FLAG_Z;
        }

        encode_u8(writer, header)?;

        encode_u64(writer, self.value)
    }

    pub(crate) fn decode(
        header: u8,
        reader: &mut ZBufReader<'_>,
    ) -> ZResult<(Self, bool), ZCodecError> {
        if iext::eheader(header) != ID {
            zbail!(ZCodecError::CouldNotRead);
        }

        let value = decode_u64(reader)?;

        Ok((ZExtZ64 { value }, has_flag(header, iext::FLAG_Z)))
    }

    #[cfg(test)]
    pub(crate) fn rand() -> Self {
        use rand::Rng;

        let mut rng = rand::thread_rng();
        let value: u64 = rng.r#gen();
        Self { value }
    }
}

impl<const ID: u8> Debug for ZExtZ64<{ ID }> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("ZExtZ64");
        iext::fmt(&mut s, ID);
        s.field("Value", &self.value).finish()
    }
}

#[repr(transparent)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct ZExtZBuf<'a, const ID: u8> {
    pub(crate) value: ZBuf<'a>,
}

impl<'a, const ID: u8> ZExtZBuf<'a, { ID }> {
    pub(crate) const ID: u8 = ID;

    pub(crate) const fn id(mandatory: bool) -> u8 {
        iext::header(ID, mandatory, iext::ENC_ZBUF)
    }

    pub(crate) fn encode(
        &self,
        more: bool,
        writer: &mut ZBufWriter<'_>,
    ) -> ZResult<(), ZCodecError> {
        let mut header: u8 = ID;
        if more {
            header |= iext::FLAG_Z;
        }

        encode_u8(writer, header)?;
        encode_zbuf(writer, true, self.value)?;

        Ok(())
    }

    pub(crate) fn decode(
        header: u8,
        reader: &mut ZBufReader<'a>,
    ) -> ZResult<(Self, bool), ZCodecError> {
        if iext::eheader(header) != ID {
            zbail!(ZCodecError::CouldNotRead);
        }

        let value = decode_zbuf(reader, None)?;

        Ok((ZExtZBuf { value }, has_flag(header, iext::FLAG_Z)))
    }

    #[cfg(test)]
    pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        use rand::Rng;

        use crate::zbuf::BufWriterExt;

        let mut rng = rand::thread_rng();
        let zbuf = zbuf
            .write_slot_return(rng.gen_range(0..=64), |b: &mut [u8]| {
                rng.fill(b);
                b.len()
            })
            .unwrap();
        Self { value: zbuf }
    }
}

impl<const ID: u8> Debug for ZExtZBuf<'_, { ID }> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("ZExtZBuf");
        iext::fmt(&mut s, ID);
        s.field("Value", &self.value).finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct ZExtZBufHeader<const ID: u8> {
    pub(crate) len: usize,
}

impl<const ID: u8> ZExtZBufHeader<{ ID }> {
    pub(crate) const fn new(len: usize) -> Self {
        Self { len }
    }

    pub(crate) fn encode(
        &self,
        more: bool,
        writer: &mut ZBufWriter<'_>,
    ) -> ZResult<(), ZCodecError> {
        let mut header: u8 = ID;
        if more {
            header |= iext::FLAG_Z;
        }

        encode_u8(writer, header)?;
        encode_usize(writer, self.len)?;

        Ok(())
    }

    pub(crate) fn decode(
        header: u8,
        reader: &mut ZBufReader<'_>,
    ) -> ZResult<(Self, bool), ZCodecError> {
        if iext::eheader(header) != ID {
            zbail!(ZCodecError::CouldNotRead);
        }

        let len = decode_usize(reader)?;

        Ok((ZExtZBufHeader { len }, has_flag(header, iext::FLAG_Z)))
    }
}

impl<const ID: u8> Debug for ZExtZBufHeader<{ ID }> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("ZExtZBufHeader");
        iext::fmt(&mut s, ID);
        s.field("Len", &self.len).finish()
    }
}

pub(crate) fn skip(
    _s: &str,
    header: u8,
    reader: &mut ZBufReader<'_>,
) -> ZResult<bool, ZCodecError> {
    let id = header & !iext::FLAG_Z;

    if iext::is_mandatory(id) {
        crate::error!(
            "Mandatory extension {} with id {} not supported.",
            _s,
            iext::mheader(id),
        );

        zbail!(ZCodecError::CouldNotRead);
    }

    match header & iext::ENC_MASK {
        iext::ENC_UNIT => {}
        iext::ENC_Z64 => {
            let _ = decode_u64(reader)?;
        }
        iext::ENC_ZBUF => {
            let _ = decode_zbuf(reader, None)?;
        }
        _ => {
            zbail!(ZCodecError::CouldNotRead);
        }
    };

    Ok(has_flag(header, iext::FLAG_Z))
}

pub(crate) fn skip_all(s: &str, reader: &mut ZBufReader<'_>) -> ZResult<(), ZCodecError> {
    let mut has_ext = reader.can_read();

    while has_ext {
        let header = decode_u8(reader)?;
        has_ext = skip(s, header, reader)?;
    }

    Ok(())
}

#[macro_export]
macro_rules! zextunit {
    ($id:expr, $m:expr) => {
        $crate::protocol::common::extension::ZExtUnit<{ $crate::protocol::common::extension::ZExtUnit::<$id>::id($m) }>
    }
}

#[macro_export]
macro_rules! zextz64 {
    ($id:expr, $m:expr) => {
        $crate::protocol::common::extension::ZExtZ64<{ $crate::protocol::common::extension::ZExtZ64::<$id>::id($m) }>
    }
}

#[macro_export]
macro_rules! zextzbuf {
    ($lt:lifetime, $id:expr, $m:expr) => {
        $crate::protocol::common::extension::ZExtZBuf<$lt, { $crate::protocol::common::extension::ZExtZBuf::<$id>::id($m) }>
    }
}
