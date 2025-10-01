use core::str::FromStr;

use heapless::String;
use zenoh_buffer::{ZBuf, ZBufReader, ZBufWriter};
use zenoh_result::{zbail, zerr, ZResult, ZE};

use crate::{RCodec, WCodec, Zenoh080};

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

impl<'a> WCodec<'a, u8> for Zenoh080 {
    fn write(&self, message: u8, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        writer.write_u8(message)?;

        Ok(())
    }
}

impl<'a> RCodec<'a, u8> for Zenoh080 {
    fn read(&self, reader: &mut ZBufReader<'a>) -> ZResult<u8> {
        reader.read_u8()
    }
}

impl<'a> WCodec<'a, u64> for Zenoh080 {
    fn write(&self, message: u64, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        let mut value = message;
        let mut buf = [0u8; VLE_LEN_MAX];
        let mut i = 0;

        loop {
            let byte = (value & 0x7F) as u8;
            value >>= 7;

            if value != 0 {
                unsafe {
                    *buf.get_unchecked_mut(i) = byte | 0x80;
                }
            } else {
                unsafe {
                    *buf.get_unchecked_mut(i) = byte;
                }
                break;
            }

            i += 1;
            if i >= VLE_LEN_MAX {
                zbail!(ZE::WriteFailure);
            }
        }

        writer.write_exact(&buf[..=i])?;

        Ok(())
    }
}

impl<'a> RCodec<'a, u64> for Zenoh080 {
    fn read(&self, reader: &mut ZBufReader<'a>) -> ZResult<u64> {
        let mut value: u64 = 0;
        let mut shift = 0;
        let mut byte: u8;

        loop {
            if shift >= 64 {
                zbail!(ZE::MalformedVLE);
            }

            byte = reader.read_u8()?;
            value |= ((byte & 0x7F) as u64) << shift;

            if (byte & 0x80) == 0 {
                break;
            }

            shift += 7;
        }

        Ok(value)
    }
}

impl<'a> WCodec<'a, u32> for Zenoh080 {
    fn write(&self, message: u32, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        self.write(message as u64, writer)
    }
}

impl<'a> RCodec<'a, u32> for Zenoh080 {
    fn read(&self, reader: &mut ZBufReader<'a>) -> ZResult<u32> {
        let value: u64 = self.read(reader)?;

        if value > u32::MAX as u64 {
            zbail!(ZE::CapacityExceeded);
        }

        Ok(value as u32)
    }
}

impl<'a> WCodec<'a, u16> for Zenoh080 {
    fn write(&self, message: u16, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        self.write(message as u64, writer)
    }
}

impl<'a> RCodec<'a, u16> for Zenoh080 {
    fn read(&self, reader: &mut ZBufReader<'a>) -> ZResult<u16> {
        let value: u64 = self.read(reader)?;

        if value > u16::MAX as u64 {
            zbail!(ZE::CapacityExceeded);
        }

        Ok(value as u16)
    }
}

impl<'a> WCodec<'a, usize> for Zenoh080 {
    fn write(&self, message: usize, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        self.write(message as u64, writer)
    }
}

impl<'a> RCodec<'a, usize> for Zenoh080 {
    fn read(&self, reader: &mut ZBufReader<'a>) -> ZResult<usize> {
        let value: u64 = self.read(reader)?;

        if value > usize::MAX as u64 {
            zbail!(ZE::CapacityExceeded);
        }

        Ok(value as usize)
    }
}

impl<'a, const N: usize> WCodec<'a, &[u8; N]> for Zenoh080 {
    fn write(&self, message: &[u8; N], writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        writer.write_exact(message)?;

        Ok(())
    }
}

impl<'a, const N: usize> RCodec<'a, [u8; N]> for Zenoh080 {
    fn read(&self, reader: &mut ZBufReader<'a>) -> ZResult<[u8; N]> {
        if reader.remaining() < N {
            zbail!(ZE::CapacityExceeded);
        }

        let mut array = [0u8; N];
        reader.read_exact(&mut array)?;

        Ok(array)
    }
}

impl<'a> WCodec<'a, ZBuf<'_>> for Zenoh080 {
    fn write(&self, message: ZBuf<'_>, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        if message.is_empty() {
            zbail!(ZE::WriteFailure);
        }

        let len = message.len();
        self.write(len, writer)?;
        writer.write_exact(message.as_bytes())?;

        Ok(())
    }
}

impl<'a> RCodec<'a, ZBuf<'a>> for Zenoh080 {
    fn read(&self, reader: &mut ZBufReader<'a>) -> ZResult<ZBuf<'a>> {
        let len: usize = self.read(reader)?;

        reader.read_zbuf(len)
    }
}

impl<'a> WCodec<'a, &str> for Zenoh080 {
    fn write(&self, message: &str, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        let zbuf = ZBuf(message.as_bytes());
        self.write(zbuf, writer)
    }
}

impl<'a> RCodec<'a, &'a str> for Zenoh080 {
    fn read(&self, reader: &mut ZBufReader<'a>) -> ZResult<&'a str> {
        let zbuf: ZBuf<'a> = self.read(reader)?;

        zbuf.as_str()
    }
}

impl<'a, const N: usize> WCodec<'a, &String<N>> for Zenoh080 {
    fn write(&self, message: &String<N>, writer: &mut ZBufWriter<'a>) -> ZResult<()> {
        let zbuf = ZBuf(message.as_bytes());
        self.write(zbuf, writer)
    }
}

impl<'a, const N: usize> RCodec<'a, String<N>> for Zenoh080 {
    fn read(&self, reader: &mut ZBufReader<'a>) -> ZResult<String<N>> {
        let s: &'a str = self.read(reader)?;

        String::from_str(s).map_err(|_| zerr!(ZE::CapacityExceeded))
    }
}
