use crate::{ZCodecResult, ZReader, ZWriter};

mod array;
mod bytes;
mod str;
mod uint;

pub trait ZStructEncode {
    fn z_len(&self) -> usize;

    fn z_header(&self) -> Option<u8> {
        None
    }

    fn z_encode_without_header(&self, w: &mut ZWriter) -> ZCodecResult<()>;

    fn z_encode(&self, w: &mut ZWriter) -> ZCodecResult<()> {
        if let Some(header) = self.z_header() {
            <u8 as ZStructEncode>::z_encode(&header, w)?;
        }

        self.z_encode_without_header(w)
    }
}

pub trait ZStructDecode<'a> {
    fn z_decode(r: &mut ZReader<'a>) -> ZCodecResult<Self>
    where
        Self: Sized;
}
