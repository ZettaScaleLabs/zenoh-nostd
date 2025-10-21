use uhlc::{ID, NTP64, Timestamp};

use crate::{
    protocol::ZCodecError,
    result::ZResult,
    zbuf::{BufReaderExt, BufWriterExt, ZBuf, ZBufReader, ZBufWriter},
};

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

pub(crate) fn encoded_len_u64(x: u64) -> usize {
    vle_len(x)
}

pub(crate) fn encode_u64(writer: &mut ZBufWriter<'_>, mut x: u64) -> ZResult<(), ZCodecError> {
    writer.write_slot(VLE_LEN_MAX, |buffer: &mut [u8]| {
        let mut len = 0;

        while (x & !0x7f_u64) != 0 {
            unsafe {
                *buffer.get_unchecked_mut(len) = (x as u8) | 0x80_u8;
            }

            len += 1;
            x >>= 7;
        }

        if len != VLE_LEN_MAX {
            unsafe {
                *buffer.get_unchecked_mut(len) = x as u8;
            }
            len += 1;
        }

        len
    })?;

    Ok(())
}

pub(crate) fn decode_u64(reader: &mut ZBufReader<'_>) -> ZResult<u64, ZCodecError> {
    let mut b = decode_u8(reader)?;

    let mut v = 0;
    let mut i = 0;

    while (b & 0x80_u8) != 0 && i != 7 * (VLE_LEN_MAX - 1) {
        v |= ((b & 0x7f_u8) as u64) << i;
        b = decode_u8(reader)?;
        i += 7;
    }

    v |= (b as u64) << i;

    Ok(v)
}

pub(crate) fn encoded_len_u32(x: u32) -> usize {
    vle_len(x as u64)
}

pub(crate) fn encode_u32(writer: &mut ZBufWriter<'_>, x: u32) -> ZResult<(), ZCodecError> {
    encode_u64(writer, x as u64)
}

pub(crate) fn decode_u32(reader: &mut ZBufReader<'_>) -> ZResult<u32, ZCodecError> {
    decode_u64(reader).and_then(|v| {
        if v <= u32::MAX as u64 {
            Ok(v as u32)
        } else {
            Err(ZCodecError::CouldNotRead)
        }
    })
}

pub(crate) fn encode_u16(writer: &mut ZBufWriter<'_>, x: u16) -> ZResult<(), ZCodecError> {
    encode_u64(writer, x as u64)
}

pub(crate) fn decode_u16(reader: &mut ZBufReader<'_>) -> ZResult<u16, ZCodecError> {
    decode_u64(reader).and_then(|v| {
        if v <= u16::MAX as u64 {
            Ok(v as u16)
        } else {
            Err(ZCodecError::CouldNotRead)
        }
    })
}

pub(crate) fn encode_u8(writer: &mut ZBufWriter<'_>, x: u8) -> ZResult<(), ZCodecError> {
    writer.write_u8(x)?;

    Ok(())
}

pub(crate) fn decode_u8(reader: &mut ZBufReader<'_>) -> ZResult<u8, ZCodecError> {
    Ok(reader.read_u8()?)
}

pub(crate) fn encoded_len_usize(x: usize) -> usize {
    vle_len(x as u64)
}

pub(crate) fn encode_usize(writer: &mut ZBufWriter<'_>, x: usize) -> ZResult<(), ZCodecError> {
    encode_u64(writer, x as u64)
}

pub(crate) fn decode_usize(reader: &mut ZBufReader<'_>) -> ZResult<usize, ZCodecError> {
    decode_u64(reader).and_then(|v| {
        if v <= usize::MAX as u64 {
            Ok(v as usize)
        } else {
            Err(ZCodecError::CouldNotRead)
        }
    })
}

pub(crate) fn encoded_len_zbuf(len: bool, zbuf: ZBuf<'_>) -> usize {
    if len {
        encoded_len_usize(zbuf.len()) + zbuf.len()
    } else {
        zbuf.len()
    }
}

pub(crate) fn encode_zbuf(
    writer: &mut ZBufWriter<'_>,
    len: bool,
    zbuf: ZBuf<'_>,
) -> ZResult<(), ZCodecError> {
    if len {
        encode_usize(writer, zbuf.len())?;
    }

    if zbuf.is_empty() {
        return Ok(());
    }

    writer.write_exact(zbuf)?;

    Ok(())
}

pub(crate) fn decode_zbuf<'a>(
    reader: &mut ZBufReader<'a>,
    len: Option<usize>,
) -> ZResult<ZBuf<'a>, ZCodecError> {
    let len = match len {
        Some(l) => l,
        None => decode_usize(reader)?,
    };

    Ok(reader.read_zbuf(len)?)
}

pub(crate) fn encode_str(
    writer: &mut ZBufWriter<'_>,
    len: bool,
    s: &str,
) -> ZResult<(), ZCodecError> {
    encode_zbuf(writer, len, s.as_bytes())
}

pub(crate) fn decode_str<'a>(
    reader: &mut ZBufReader<'a>,
    len: Option<usize>,
) -> ZResult<&'a str, ZCodecError> {
    let zbuf = decode_zbuf(reader, len)?;
    match core::str::from_utf8(zbuf) {
        Ok(s) => Ok(s),
        Err(_) => Err(ZCodecError::CouldNotParse),
    }
}

pub(crate) fn encoded_len_timestamp(x: &Timestamp) -> usize {
    let id = x.get_id();
    let bytes = &id.to_le_bytes()[..id.size()];

    encoded_len_u64(x.get_time().as_u64()) + encoded_len_zbuf(true, bytes)
}

pub(crate) fn encode_timestamp(
    writer: &mut ZBufWriter<'_>,
    x: &Timestamp,
) -> ZResult<(), ZCodecError> {
    encode_u64(writer, x.get_time().as_u64())?;
    let id = x.get_id();
    let bytes = &id.to_le_bytes()[..id.size()];
    encode_zbuf(writer, true, bytes)?;
    Ok(())
}

pub(crate) fn decode_timestamp(reader: &mut ZBufReader<'_>) -> ZResult<Timestamp, ZCodecError> {
    let time = decode_u64(reader)?;
    let bytes = decode_zbuf(reader, None)?;
    let id = ID::try_from(bytes).map_err(|_| ZCodecError::CouldNotParse)?;

    let time = NTP64(time);
    Ok(Timestamp::new(time, id))
}

pub(crate) fn encode_array<const N: usize>(
    writer: &mut ZBufWriter<'_>,
    x: &[u8; N],
) -> ZResult<(), ZCodecError> {
    writer.write_exact(x)?;

    Ok(())
}

pub(crate) fn decode_array<const N: usize>(
    reader: &'_ mut ZBufReader<'_>,
) -> ZResult<[u8; N], ZCodecError> {
    let mut data = [0u8; N];
    reader.read(&mut data)?;

    Ok(data)
}
