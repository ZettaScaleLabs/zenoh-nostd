use heapless::Vec;
use zenoh_buffers::{reader::Reader, writer::Writer, zbuf::ZBuf};

use zenoh_protocol::common::{
    iext, imsg::has_flag, ZExtBody, ZExtUnit, ZExtUnknown, ZExtZ64, ZExtZBuf, ZExtZBufHeader,
};
use zenoh_result::{bail, zerr, ZError, ZResult, ZE};

use crate::{RCodec, WCodec, Zenoh080, Zenoh080Bounded, Zenoh080Header};

fn read_inner<R, const N: usize, const L: usize>(
    reader: &mut R,
    _s: &str,
    header: u8,
) -> ZResult<(ZExtUnknown<N, L>, bool)>
where
    R: Reader,
{
    let codec = Zenoh080Header::new(header);
    let (u, has_ext): (ZExtUnknown<N, L>, bool) = codec.read(&mut *reader)?;
    if u.is_mandatory() {
        bail!(ZE::MandatoryFieldMissing);
    }

    Ok((u, has_ext))
}

#[cold]
#[inline(never)]
pub fn read<R, const N: usize, const L: usize>(
    reader: &mut R,
    s: &str,
    header: u8,
) -> ZResult<(ZExtUnknown<N, L>, bool)>
where
    R: Reader,
{
    read_inner(&mut *reader, s, header)
}

fn skip_inner<R, const N: usize, const L: usize>(
    reader: &mut R,
    s: &str,
    header: u8,
) -> ZResult<bool>
where
    R: Reader,
{
    let (_, has_ext): (ZExtUnknown<N, L>, bool) = read_inner(&mut *reader, s, header)?;
    Ok(has_ext)
}

#[cold]
#[inline(never)]
pub fn skip<R, const N: usize, const L: usize>(reader: &mut R, s: &str, header: u8) -> ZResult<bool>
where
    R: Reader,
{
    skip_inner::<_, N, L>(reader, s, header)
}

#[cold]
#[inline(never)]
pub fn skip_all<R, const N: usize, const L: usize>(reader: &mut R, s: &str) -> ZResult<()>
where
    R: Reader,
{
    let codec = Zenoh080::new();
    let mut has_ext = true;
    while has_ext {
        let header: u8 = codec.read(&mut *reader)?;
        has_ext = skip_inner::<_, N, L>(reader, s, header)?;
    }
    Ok(())
}

// ZExtUnit
impl<const ID: u8, W> WCodec<(&ZExtUnit<{ ID }>, bool), &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: (&ZExtUnit<{ ID }>, bool)) -> Self::Output {
        let (x, more) = x;
        let ZExtUnit = x;

        let mut header: u8 = ID;
        if more {
            header |= iext::FLAG_Z;
        }
        self.write(&mut *writer, header)?;
        Ok(())
    }
}

impl<const ID: u8, R> RCodec<(ZExtUnit<{ ID }>, bool), &mut R> for Zenoh080
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<(ZExtUnit<{ ID }>, bool)> {
        let header: u8 = self.read(&mut *reader)?;
        let codec = Zenoh080Header::new(header);
        codec.read(&mut *reader)
    }
}

impl<const ID: u8, R> RCodec<(ZExtUnit<{ ID }>, bool), &mut R> for Zenoh080Header
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, _reader: &mut R) -> ZResult<(ZExtUnit<{ ID }>, bool)> {
        if iext::eid(self.header) != ID {
            bail!(ZE::DidntRead);
        }
        Ok((ZExtUnit::new(), has_flag(self.header, iext::FLAG_Z)))
    }
}

// ZExtZ64
impl<const ID: u8, W> WCodec<(&ZExtZ64<{ ID }>, bool), &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: (&ZExtZ64<{ ID }>, bool)) -> Self::Output {
        let (x, more) = x;
        let ZExtZ64 { value } = x;

        let mut header: u8 = ID;
        if more {
            header |= iext::FLAG_Z;
        }
        self.write(&mut *writer, header)?;
        self.write(&mut *writer, value)?;
        Ok(())
    }
}

impl<const ID: u8, R> RCodec<(ZExtZ64<{ ID }>, bool), &mut R> for Zenoh080
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<(ZExtZ64<{ ID }>, bool)> {
        let header: u8 = self.read(&mut *reader)?;
        let codec = Zenoh080Header::new(header);
        codec.read(&mut *reader)
    }
}

impl<const ID: u8, R> RCodec<(ZExtZ64<{ ID }>, bool), &mut R> for Zenoh080Header
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<(ZExtZ64<{ ID }>, bool)> {
        if iext::eid(self.header) != ID {
            bail!(ZE::DidntRead);
        }

        let value: u64 = self.codec.read(&mut *reader)?;
        Ok((ZExtZ64::new(value), has_flag(self.header, iext::FLAG_Z)))
    }
}

// ZExtZBuf
impl<const ID: u8, W, const N: usize, const L: usize>
    WCodec<(&ZExtZBuf<{ ID }, N, L>, bool), &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: (&ZExtZBuf<{ ID }, N, L>, bool)) -> Self::Output {
        let (x, more) = x;
        let ZExtZBuf { value } = x;

        let mut header: u8 = ID;
        if more {
            header |= iext::FLAG_Z;
        }
        self.write(&mut *writer, header)?;
        let bodec = Zenoh080Bounded::<u32>::new();
        bodec.write(&mut *writer, value)?;
        Ok(())
    }
}

impl<const ID: u8, R, const N: usize, const L: usize> RCodec<(ZExtZBuf<{ ID }, N, L>, bool), &mut R>
    for Zenoh080
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<(ZExtZBuf<{ ID }, N, L>, bool)> {
        let header: u8 = self.read(&mut *reader)?;
        let codec = Zenoh080Header::new(header);
        codec.read(&mut *reader)
    }
}

impl<const ID: u8, R, const N: usize, const L: usize> RCodec<(ZExtZBuf<{ ID }, N, L>, bool), &mut R>
    for Zenoh080Header
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<(ZExtZBuf<{ ID }, N, L>, bool)> {
        if iext::eid(self.header) != ID {
            bail!(ZE::DidntRead);
        }
        let bodec = Zenoh080Bounded::<u32>::new();
        let value: ZBuf<N, L> = bodec.read(&mut *reader)?;
        Ok((ZExtZBuf::new(value), has_flag(self.header, iext::FLAG_Z)))
    }
}

// ZExtZBufHeader
impl<const ID: u8, W> WCodec<(&ZExtZBufHeader<{ ID }>, bool), &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: (&ZExtZBufHeader<{ ID }>, bool)) -> Self::Output {
        let (x, more) = x;
        let ZExtZBufHeader { len } = x;

        let mut header: u8 = ID;
        if more {
            header |= iext::FLAG_Z;
        }
        self.write(&mut *writer, header)?;
        let bodec = Zenoh080Bounded::<u32>::new();
        bodec.write(&mut *writer, *len)?;
        Ok(())
    }
}

impl<const ID: u8, R> RCodec<(ZExtZBufHeader<{ ID }>, bool), &mut R> for Zenoh080
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<(ZExtZBufHeader<{ ID }>, bool)> {
        let header: u8 = self.read(&mut *reader)?;
        let codec = Zenoh080Header::new(header);
        codec.read(&mut *reader)
    }
}

impl<const ID: u8, R> RCodec<(ZExtZBufHeader<{ ID }>, bool), &mut R> for Zenoh080Header
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<(ZExtZBufHeader<{ ID }>, bool)> {
        if iext::eid(self.header) != ID {
            bail!(ZE::DidntRead);
        }

        let bodec = Zenoh080Bounded::<u32>::new();
        let len: usize = bodec.read(&mut *reader)?;
        Ok((
            ZExtZBufHeader::new(len),
            has_flag(self.header, iext::FLAG_Z),
        ))
    }
}

// ZExtUnknown
impl<W, const N: usize, const L: usize> WCodec<(&ZExtUnknown<N, L>, bool), &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: (&ZExtUnknown<N, L>, bool)) -> Self::Output {
        let (x, more) = x;
        let ZExtUnknown { id, body } = x;

        let mut header: u8 = *id;
        if more {
            header |= iext::FLAG_Z;
        }
        match body {
            ZExtBody::Unit => self.write(&mut *writer, header)?,
            ZExtBody::Z64(u64) => {
                self.write(&mut *writer, header)?;
                self.write(&mut *writer, *u64)?
            }
            ZExtBody::ZBuf(zbuf) => {
                self.write(&mut *writer, header)?;
                let bodec = Zenoh080Bounded::<u32>::new();
                bodec.write(&mut *writer, zbuf)?
            }
        }
        Ok(())
    }
}

impl<R, const N: usize, const L: usize> RCodec<(ZExtUnknown<N, L>, bool), &mut R> for Zenoh080
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<(ZExtUnknown<N, L>, bool)> {
        let header: u8 = self.read(&mut *reader)?;
        let codec = Zenoh080Header::new(header);
        codec.read(&mut *reader)
    }
}

impl<R, const N: usize, const L: usize> RCodec<(ZExtUnknown<N, L>, bool), &mut R> for Zenoh080Header
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<(ZExtUnknown<N, L>, bool)> {
        let body = match self.header & iext::ENC_MASK {
            iext::ENC_UNIT => ZExtBody::Unit,
            iext::ENC_Z64 => {
                let u64: u64 = self.codec.read(&mut *reader)?;
                ZExtBody::Z64(u64)
            }
            iext::ENC_ZBUF => {
                let bodec = Zenoh080Bounded::<u32>::new();
                let zbuf: ZBuf<N, L> = bodec.read(&mut *reader)?;
                ZExtBody::ZBuf(zbuf)
            }
            _ => {
                bail!(ZE::DidntRead);
            }
        };

        Ok((
            ZExtUnknown {
                id: self.header & !iext::FLAG_Z,
                body,
            },
            has_flag(self.header, iext::FLAG_Z),
        ))
    }
}

impl<W, const N: usize, const L: usize> WCodec<&[ZExtUnknown<N, L>], &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: &[ZExtUnknown<N, L>]) -> Self::Output {
        let len = x.len();
        for (i, e) in x.iter().enumerate() {
            self.write(&mut *writer, (e, i < len - 1))?;
        }
        Ok(())
    }
}

impl<R, const N: usize, const L: usize, const P: usize> RCodec<Vec<ZExtUnknown<N, L>, P>, &mut R>
    for Zenoh080
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<Vec<ZExtUnknown<N, L>, P>> {
        let mut exts = Vec::new();
        let mut has_ext = reader.can_read();
        while has_ext {
            let (e, more): (ZExtUnknown<N, L>, bool) = self.read(&mut *reader)?;
            exts.push(e).map_err(|_| zerr!(ZE::CapacityExceeded))?;
            has_ext = more;
        }
        Ok(exts)
    }
}

// Macros
#[macro_export]
macro_rules! impl_zextz64 {
    ($ext:ty, $id:expr) => {
        impl<W> WCodec<($ext, bool), &mut W> for Zenoh080
        where
            W: Writer,
        {
            type Output = Result<()>;

            fn write(self, writer: &mut W, x: ($ext, bool)) -> Self::Output {
                let (x, more) = x;
                let ext: ZExtZ64<{ $id }> = x.into();
                self.write(&mut *writer, (&ext, more))
            }
        }

        impl<R> RCodec<($ext, bool), &mut R> for Zenoh080
        where
            R: Reader,
        {
            type Error = ZError;

            fn read(self, reader: &mut R) -> ZResult<($ext, bool)> {
                let header: u8 = self.read(&mut *reader)?;
                let codec = Zenoh080Header::new(header);
                codec.read(reader)
            }
        }

        impl<R> RCodec<($ext, bool), &mut R> for Zenoh080Header
        where
            R: Reader,
        {
            type Error = ZError;

            fn read(self, reader: &mut R) -> ZResult<($ext, bool)> {
                let (ext, more): (ZExtZ64<{ $id }>, bool) = self.read(&mut *reader)?;
                Ok((ext.into(), more))
            }
        }
    };
}
