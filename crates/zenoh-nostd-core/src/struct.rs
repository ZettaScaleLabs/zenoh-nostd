use crate::{ZCodecResult, ZReader, ZWriter};

mod array;
mod bytes;
mod str;
mod uint;

pub trait ZStructEncode {
    fn z_len_without_header(&self) -> usize;

    fn z_len(&self) -> usize {
        let header_len = if <Self as ZStructEncode>::z_header(&self).is_some() {
            1
        } else {
            0
        };
        header_len + <Self as ZStructEncode>::z_len_without_header(&self)
    }

    fn z_header(&self) -> Option<u8> {
        None
    }

    fn z_encode_without_header(&self, w: &mut ZWriter) -> ZCodecResult<()>;

    fn z_encode(&self, w: &mut ZWriter) -> ZCodecResult<()> {
        if let Some(header) = <Self as ZStructEncode>::z_header(&self) {
            <u8 as ZStructEncode>::z_encode(&header, w)?;
        }

        <Self as ZStructEncode>::z_encode_without_header(self, w)
    }
}

pub trait ZStructDecode<'a> {
    fn z_decode_with_header(r: &mut ZReader<'a>, h: u8) -> ZCodecResult<Self>
    where
        Self: Sized;

    fn z_decode(r: &mut ZReader<'a>) -> ZCodecResult<Self>
    where
        Self: Sized,
    {
        <Self as ZStructDecode>::z_decode_with_header(r, 0) // Assumes no header for most types
    }
}
