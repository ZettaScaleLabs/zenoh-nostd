use crate::{ZCodecResult, ZReader, ZWriter};

mod array;
mod bytes;
mod str;
mod uint;

pub trait ZStructEncode {
    fn z_len(&self) -> usize;

    fn z_encode(&self, w: &mut ZWriter) -> ZCodecResult<()>;
}

pub trait ZStructDecode<'a> {
    fn z_decode(r: &mut ZReader<'a>) -> ZCodecResult<Self>
    where
        Self: Sized;
}
