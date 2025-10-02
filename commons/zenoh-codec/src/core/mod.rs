use core::str::FromStr;

use heapless::String;
use zenoh_buffer::{ZBuf, ZBufReader, ZBufWriter};
use zenoh_protocol::core::ZenohIdProto;
use zenoh_result::{zbail, zctx, zerr, WithContext, ZResult, ZE};

use crate::{LCodec, RCodec, WCodec, ZCodec};

pub mod encoding;
pub mod locator;
pub mod timestamp;
pub mod wire_expr;

const VLE_LEN_MAX: usize = vle_len(u64::MAX);

const fn vle_len(x: u64) -> usize {
    const B1: u64 = u64::MAX << 7;
    const B2: u64 = u64::MAX << (7 * 2);
    const B3: u64 = u64::MAX << (7 * 3);
    const B4: u64 = u64::MAX << (7 * 4);
    const B5: u64 = u64::MAX << (7 * 5);
    const B6: u64 = u64::MAX << (7 * 6);
    const B7: u64 = u64::MAX << (7 * 7);
    const B8: u64 = u64::MAX << (7 * 8);

    if (x & B1) == 0 {
        1
    } else if (x & B2) == 0 {
        2
    } else if (x & B3) == 0 {
        3
    } else if (x & B4) == 0 {
        4
    } else if (x & B5) == 0 {
        5
    } else if (x & B6) == 0 {
        6
    } else if (x & B7) == 0 {
        7
    } else if (x & B8) == 0 {
        8
    } else {
        9
    }
}

impl<'a> LCodec<'a, u8> for ZCodec {
    fn w_len(&self, _message: u8) -> usize {
        1
    }
}

impl<'a> WCodec<'a, u8> for ZCodec {
    fn write(&self, message: u8, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        writer.write_u8(message)?;

        Ok(())
    }
}

impl<'a> RCodec<'a, u8> for ZCodec {
    fn read(&self, reader: &mut ZBufReader<'a>) -> ZResult<u8> {
        reader.read_u8()
    }
}

impl<'a> LCodec<'a, u64> for ZCodec {
    fn w_len(&self, message: u64) -> usize {
        vle_len(message)
    }
}

impl<'a> WCodec<'a, u64> for ZCodec {
    fn write(&self, mut message: u64, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        writer.write_slot(VLE_LEN_MAX, |buffer: &mut [u8]| {
            let mut len = 0;
            while (message & !0x7f_u64) != 0 {
                // SAFETY: buffer is guaranteed to be VLE_LEN long where VLE_LEN is
                //         the maximum number of bytes a VLE can take once encoded.
                //         I.e.: x is shifted 7 bits to the right every iteration,
                //         the loop is at most VLE_LEN iterations.
                unsafe {
                    *buffer.get_unchecked_mut(len) = (message as u8) | 0x80_u8;
                }
                len += 1;
                message >>= 7;
            }
            // In case len == VLE_LEN then all the bits have already been written in the latest iteration.
            // Else we haven't written all the necessary bytes yet.
            if len != VLE_LEN_MAX {
                // SAFETY: buffer is guaranteed to be VLE_LEN long where VLE_LEN is
                //         the maximum number of bytes a VLE can take once encoded.
                //         I.e.: x is shifted 7 bits to the right every iteration,
                //         the loop is at most VLE_LEN iterations.
                unsafe {
                    *buffer.get_unchecked_mut(len) = message as u8;
                }
                len += 1;
            }
            // The number of written bytes
            len
        })
    }
}

impl<'a> RCodec<'a, u64> for ZCodec {
    fn read(&self, reader: &mut ZBufReader<'a>) -> ZResult<u64> {
        let mut b = reader.read_u8()?;

        let mut v = 0;
        let mut i = 0;
        // 7 * VLE_LEN is beyond the maximum number of shift bits
        while (b & 0x80_u8) != 0 && i != 7 * (VLE_LEN_MAX - 1) {
            v |= ((b & 0x7f_u8) as u64) << i;
            b = reader.read_u8()?;
            i += 7;
        }
        v |= (b as u64) << i;
        Ok(v)
    }
}

impl<'a> LCodec<'a, u32> for ZCodec {
    fn w_len(&self, message: u32) -> usize {
        self.w_len(message as u64)
    }
}

impl<'a> WCodec<'a, u32> for ZCodec {
    fn write(&self, message: u32, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        self.write(message as u64, writer).ctx(zctx!())
    }
}

impl<'a> RCodec<'a, u32> for ZCodec {
    fn read(&self, reader: &mut ZBufReader<'a>) -> ZResult<u32> {
        let value: u64 = self.read(reader).ctx(zctx!())?;

        if value > u32::MAX as u64 {
            zbail!(ZE::CapacityExceeded);
        }

        Ok(value as u32)
    }
}

impl<'a> LCodec<'a, u16> for ZCodec {
    fn w_len(&self, message: u16) -> usize {
        self.w_len(message as u64)
    }
}

impl<'a> WCodec<'a, u16> for ZCodec {
    fn write(&self, message: u16, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        self.write(message as u64, writer).ctx(zctx!())
    }
}

impl<'a> RCodec<'a, u16> for ZCodec {
    fn read(&self, reader: &mut ZBufReader<'a>) -> ZResult<u16> {
        let value: u64 = self.read(reader).ctx(zctx!())?;

        if value > u16::MAX as u64 {
            zbail!(ZE::CapacityExceeded);
        }

        Ok(value as u16)
    }
}

impl<'a> LCodec<'a, usize> for ZCodec {
    fn w_len(&self, message: usize) -> usize {
        self.w_len(message as u64)
    }
}

impl<'a> WCodec<'a, usize> for ZCodec {
    fn write(&self, message: usize, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        self.write(message as u64, writer).ctx(zctx!())
    }
}

impl<'a> RCodec<'a, usize> for ZCodec {
    fn read(&self, reader: &mut ZBufReader<'a>) -> ZResult<usize> {
        let value: u64 = self.read(reader).ctx(zctx!())?;

        if value > usize::MAX as u64 {
            zbail!(ZE::CapacityExceeded);
        }

        Ok(value as usize)
    }
}

impl<'a, const N: usize> LCodec<'a, &[u8; N]> for ZCodec {
    fn w_len(&self, _: &[u8; N]) -> usize {
        N
    }
}

impl<'a, const N: usize> WCodec<'a, &[u8; N]> for ZCodec {
    fn write(&self, message: &[u8; N], writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        writer.write_exact(message)?;

        Ok(())
    }
}

impl<'a, const N: usize> WCodec<'a, [u8; N]> for ZCodec {
    fn write(&self, message: [u8; N], writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        writer.write_exact(&message)?;

        Ok(())
    }
}

impl<'a, const N: usize> RCodec<'a, [u8; N]> for ZCodec {
    fn read(&self, reader: &mut ZBufReader<'a>) -> ZResult<[u8; N]> {
        if reader.remaining() < N {
            zbail!(ZE::CapacityExceeded);
        }

        let mut array = [0u8; N];
        reader.read_exact(&mut array)?;

        Ok(array)
    }
}

impl<'a> LCodec<'a, &ZBuf<'_>> for ZCodec {
    fn w_len(&self, message: &ZBuf<'_>) -> usize {
        self.w_len(message.len()) + message.len()
    }
}

impl<'a> WCodec<'a, &ZBuf<'_>> for ZCodec {
    fn write(&self, message: &ZBuf<'_>, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        if message.is_empty() {
            zbail!(ZE::WriteFailure);
        }

        let len = message.len();
        self.write(len, writer).ctx(zctx!())?;
        writer.write_exact(message.as_bytes())?;

        Ok(())
    }

    fn write_without_length(&self, message: &ZBuf<'_>, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        if message.is_empty() {
            zbail!(ZE::WriteFailure);
        }

        writer.write_exact(message.as_bytes())?;

        Ok(())
    }
}

impl<'a> LCodec<'a, ZBuf<'_>> for ZCodec {
    fn w_len(&self, message: ZBuf<'_>) -> usize {
        self.w_len(&message)
    }
}

impl<'a> WCodec<'a, ZBuf<'_>> for ZCodec {
    fn write(&self, message: ZBuf<'_>, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        self.write(&message, writer).ctx(zctx!())
    }

    fn write_without_length(&self, message: ZBuf<'_>, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        self.write_without_length(&message, writer).ctx(zctx!())
    }
}

impl<'a> RCodec<'a, ZBuf<'a>> for ZCodec {
    fn read(&self, reader: &mut ZBufReader<'a>) -> ZResult<ZBuf<'a>> {
        let len: usize = self.read(reader).ctx(zctx!())?;

        reader.read_zbuf(len)
    }

    fn read_knowing_length(&self, reader: &mut ZBufReader<'a>, length: usize) -> ZResult<ZBuf<'a>> {
        reader.read_zbuf(length)
    }
}

impl<'a> LCodec<'a, &str> for ZCodec {
    fn w_len(&self, message: &str) -> usize {
        self.w_len(message.len()) + message.len()
    }
}

impl<'a> WCodec<'a, &str> for ZCodec {
    fn write(&self, message: &str, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        let zbuf = ZBuf(message.as_bytes());
        self.write(zbuf, writer).ctx(zctx!())
    }
}

impl<'a> RCodec<'a, &'a str> for ZCodec {
    fn read(&self, reader: &mut ZBufReader<'a>) -> ZResult<&'a str> {
        let zbuf: ZBuf<'a> = self.read(reader).ctx(zctx!())?;

        zbuf.as_str()
    }
}

impl<'a, const N: usize> LCodec<'a, &String<N>> for ZCodec {
    fn w_len(&self, message: &String<N>) -> usize {
        self.w_len(message.len()) + message.len()
    }
}

impl<'a, const N: usize> WCodec<'a, &String<N>> for ZCodec {
    fn write(&self, message: &String<N>, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        let zbuf = ZBuf(message.as_bytes());
        self.write(zbuf, writer).ctx(zctx!())
    }
}

impl<'a, const N: usize> RCodec<'a, String<N>> for ZCodec {
    fn read(&self, reader: &mut ZBufReader<'a>) -> ZResult<String<N>> {
        let s: &'a str = self.read(reader).ctx(zctx!())?;

        String::from_str(s).map_err(|_| zerr!(ZE::CapacityExceeded))
    }
}

impl<'a> LCodec<'a, &ZenohIdProto> for ZCodec {
    fn w_len(&self, message: &ZenohIdProto) -> usize {
        message.size()
    }
}

impl<'a> WCodec<'a, &ZenohIdProto> for ZCodec {
    fn write(&self, message: &ZenohIdProto, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        let bytes = &message.to_le_bytes()[..message.size()];

        self.write(ZBuf(bytes), writer).ctx(zctx!())
    }

    fn write_without_length(
        &self,
        message: &ZenohIdProto,
        writer: &mut ZBufWriter<'a>,
    ) -> ZResult<()> {
        writer
            .write_exact(&message.to_le_bytes()[..message.size()])
            .map(|_| ())
    }
}

impl<'a> WCodec<'a, ZenohIdProto> for ZCodec {
    fn write(&self, message: ZenohIdProto, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        self.write(&message, writer).ctx(zctx!())
    }

    fn write_without_length(
        &self,
        message: ZenohIdProto,
        writer: &mut ZBufWriter<'a>,
    ) -> ZResult<()> {
        self.write_without_length(&message, writer).ctx(zctx!())
    }
}

impl<'a> RCodec<'a, ZenohIdProto> for ZCodec {
    fn read(&self, reader: &mut ZBufReader<'a>) -> ZResult<ZenohIdProto> {
        let zbuf: ZBuf<'a> = self.read(reader).ctx(zctx!())?;

        ZenohIdProto::try_from(zbuf.as_bytes())
    }

    fn read_knowing_length(
        &self,
        reader: &mut ZBufReader<'a>,
        length: usize,
    ) -> ZResult<ZenohIdProto> {
        if length > ZenohIdProto::MAX_SIZE {
            zbail!(ZE::ReadFailure);
        }

        let mut id = [0; ZenohIdProto::MAX_SIZE];
        reader.read(&mut id, length)?;

        ZenohIdProto::try_from(&id[..length])
    }
}
