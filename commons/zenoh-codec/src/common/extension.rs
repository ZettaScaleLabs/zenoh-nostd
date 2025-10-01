use heapless::Vec;
use zenoh_buffer::{ZBuf, ZBufReader, ZBufWriter};
use zenoh_protocol::common::{
    extension::{iext, ZExtBody, ZExtUnit, ZExtUnknown, ZExtZ64, ZExtZBuf, ZExtZBufHeader},
    imsg::has_flag,
};
use zenoh_result::{zbail, zerr, ZResult, ZE};

use crate::{RCodec, WCodec, Zenoh080};

pub fn read<'a>(
    reader: &mut ZBufReader<'a>,
    _s: &str,
    header: u8,
) -> ZResult<(ZExtUnknown<'a>, bool)> {
    let codec = Zenoh080;
    let (u, has_ext): (ZExtUnknown, bool) = codec.read_knowing_header(reader, header)?;

    if u.is_mandatory() {
        zenoh_log::error!("Unknown {} ext: {:?}", _s, u);
    }

    Ok((u, has_ext))
}

pub fn skip<'a>(reader: &mut ZBufReader<'a>, s: &str, header: u8) -> ZResult<bool> {
    let (_, has_ext): (ZExtUnknown, bool) = read(reader, s, header)?;
    Ok(has_ext)
}

pub fn skip_all<'a>(reader: &mut ZBufReader<'a>, s: &str) -> ZResult<()> {
    let codec = Zenoh080;
    let mut has_ext = reader.can_read();

    while has_ext {
        let header: u8 = codec.read(reader)?;
        has_ext = skip(reader, s, header)?;
    }

    Ok(())
}

// ZExtUnit
impl<'a, const ID: u8> WCodec<'a, (&ZExtUnit<{ ID }>, bool)> for Zenoh080 {
    fn write(
        &self,
        message: (&ZExtUnit<{ ID }>, bool),
        writer: &mut ZBufWriter<'a>,
    ) -> ZResult<()> {
        let (x, more) = message;
        let ZExtUnit = x;

        let mut header: u8 = ID;
        if more {
            header |= iext::FLAG_Z;
        }

        self.write(header, writer)
    }
}

impl<'a, const ID: u8> RCodec<'a, (ZExtUnit<{ ID }>, bool)> for Zenoh080 {
    fn read_knowing_header(
        &self,
        _: &mut ZBufReader<'a>,
        header: u8,
    ) -> ZResult<(ZExtUnit<{ ID }>, bool)> {
        if iext::eid(header) != ID {
            zbail!(zenoh_result::ZE::InvalidBits);
        }

        Ok((ZExtUnit, has_flag(header, iext::FLAG_Z)))
    }

    fn read(&self, reader: &mut zenoh_buffer::ZBufReader<'a>) -> ZResult<(ZExtUnit<{ ID }>, bool)> {
        let header: u8 = self.read(reader)?;

        self.read_knowing_header(reader, header)
    }
}

// ZExt64
impl<'a, const ID: u8> WCodec<'a, (&ZExtZ64<ID>, bool)> for Zenoh080 {
    fn write(&self, message: (&ZExtZ64<{ ID }>, bool), writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        let (x, more) = message;
        let ZExtZ64 { value } = *x;

        let mut header: u8 = ID;
        if more {
            header |= iext::FLAG_Z;
        }

        self.write(header, writer)?;
        self.write(value, writer)
    }
}

impl<'a, const ID: u8> RCodec<'a, (ZExtZ64<{ ID }>, bool)> for Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut ZBufReader<'a>,
        header: u8,
    ) -> ZResult<(ZExtZ64<{ ID }>, bool)> {
        if iext::eid(header) != ID {
            zbail!(zenoh_result::ZE::InvalidBits);
        }

        let value: u64 = self.read(reader)?;

        Ok((ZExtZ64 { value }, has_flag(header, iext::FLAG_Z)))
    }

    fn read(&self, reader: &mut zenoh_buffer::ZBufReader<'a>) -> ZResult<(ZExtZ64<{ ID }>, bool)> {
        let header: u8 = self.read(reader)?;
        self.read_knowing_header(reader, header)
    }
}

// ZExtZBuf
impl<'a, const ID: u8> WCodec<'a, (&ZExtZBuf<'_, ID>, bool)> for Zenoh080 {
    fn write(
        &self,
        message: (&ZExtZBuf<'_, { ID }>, bool),
        writer: &mut ZBufWriter<'a>,
    ) -> ZResult<()> {
        let (x, more) = message;
        let ZExtZBuf { value } = x;

        let mut header: u8 = ID;
        if more {
            header |= iext::FLAG_Z;
        }

        self.write(header, writer)?;
        self.write(value, writer)
    }
}

impl<'a, const ID: u8> RCodec<'a, (ZExtZBuf<'a, ID>, bool)> for Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut ZBufReader<'a>,
        header: u8,
    ) -> ZResult<(ZExtZBuf<'a, ID>, bool)> {
        if iext::eid(header) != ID {
            zbail!(zenoh_result::ZE::InvalidBits);
        }

        let value: ZBuf<'a> = self.read(reader)?;

        Ok((ZExtZBuf { value }, has_flag(header, iext::FLAG_Z)))
    }

    fn read(&self, reader: &mut zenoh_buffer::ZBufReader<'a>) -> ZResult<(ZExtZBuf<'a, ID>, bool)> {
        let header: u8 = self.read(reader)?;
        self.read_knowing_header(reader, header)
    }
}

// ZExtZBufHeader
impl<'a, const ID: u8> WCodec<'a, (&ZExtZBufHeader<{ ID }>, bool)> for Zenoh080 {
    fn write(
        &self,
        message: (&ZExtZBufHeader<{ ID }>, bool),
        writer: &mut ZBufWriter<'a>,
    ) -> ZResult<()> {
        let (x, more) = message;
        let ZExtZBufHeader { len } = *x;

        let mut header: u8 = ID;
        if more {
            header |= iext::FLAG_Z;
        }

        self.write(header, writer)?;
        self.write(len, writer)
    }
}

impl<'a, const ID: u8> RCodec<'a, (ZExtZBufHeader<{ ID }>, bool)> for Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut ZBufReader<'a>,
        header: u8,
    ) -> ZResult<(ZExtZBufHeader<{ ID }>, bool)> {
        if iext::eid(header) != ID {
            zbail!(zenoh_result::ZE::InvalidBits);
        }

        let len: usize = self.read(reader)?;

        Ok((ZExtZBufHeader { len }, has_flag(header, iext::FLAG_Z)))
    }

    fn read(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
    ) -> ZResult<(ZExtZBufHeader<{ ID }>, bool)> {
        let header: u8 = self.read(reader)?;

        self.read_knowing_header(reader, header)
    }
}

// ZExtUnknown
impl<'a> WCodec<'a, (&ZExtUnknown<'_>, bool)> for Zenoh080 {
    fn write(&self, message: (&ZExtUnknown<'_>, bool), writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        let (x, more) = message;
        let ZExtUnknown { id, body } = x;

        let mut header: u8 = *id;
        if more {
            header |= iext::FLAG_Z;
        }

        match body {
            ZExtBody::Unit => self.write(header, writer),
            ZExtBody::Z64(v) => {
                self.write(header, writer)?;
                self.write(*v, writer)
            }
            ZExtBody::ZBuf(v) => {
                self.write(header, writer)?;
                self.write(v, writer)
            }
        }
    }
}

impl<'a> RCodec<'a, (ZExtUnknown<'a>, bool)> for Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut ZBufReader<'a>,
        header: u8,
    ) -> ZResult<(ZExtUnknown<'a>, bool)> {
        let body = match header & iext::ENC_MASK {
            iext::ENC_UNIT => ZExtBody::Unit,
            iext::ENC_Z64 => {
                let u64: u64 = self.read(reader)?;
                ZExtBody::Z64(u64)
            }
            iext::ENC_ZBUF => {
                let zbuf: ZBuf<'a> = self.read(reader)?;
                ZExtBody::ZBuf(zbuf)
            }
            _ => {
                zbail!(ZE::InvalidBits);
            }
        };

        Ok((
            ZExtUnknown {
                id: header & !iext::FLAG_Z,
                body,
            },
            has_flag(header, iext::FLAG_Z),
        ))
    }

    fn read(&self, reader: &mut zenoh_buffer::ZBufReader<'a>) -> ZResult<(ZExtUnknown<'a>, bool)> {
        let header: u8 = self.read(reader)?;
        self.read_knowing_header(reader, header)
    }
}

// &[ZExtUnknown]
impl<'a> WCodec<'a, &'_ [ZExtUnknown<'_>]> for Zenoh080 {
    fn write(&self, message: &'_ [ZExtUnknown<'_>], writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        let len = message.len();

        for (i, e) in message.iter().enumerate() {
            self.write((e, i < len - 1), writer)?;
        }

        Ok(())
    }
}

impl<'a, const N: usize> RCodec<'a, Vec<ZExtUnknown<'a>, N>> for Zenoh080 {
    fn read(&self, reader: &mut zenoh_buffer::ZBufReader<'a>) -> ZResult<Vec<ZExtUnknown<'a>, N>> {
        let mut exts = Vec::<ZExtUnknown<'a>, N>::new();
        let mut has_ext = reader.can_read();

        while has_ext {
            let (e, more): (ZExtUnknown, bool) = self.read(&mut *reader)?;
            exts.push(e).map_err(|_| zerr!(ZE::CapacityExceeded))?;

            has_ext = more;
        }

        Ok(exts)
    }
}
