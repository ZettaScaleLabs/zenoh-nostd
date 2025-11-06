use crate::{ZCodecResult, ZReader, ZReaderExt, ZStructDecode, ZStructEncode, ZWriter, ZWriterExt};

impl ZStructEncode for &[u8] {
    fn z_len(&self) -> usize {
        self.len()
    }

    fn z_encode(&self, w: &mut ZWriter) -> ZCodecResult<()> {
        w.write_exact(self)
    }
}

impl<'a> ZStructDecode<'a> for &'a [u8] {
    fn z_decode(r: &mut ZReader<'a>) -> ZCodecResult<Self> {
        r.read(r.remaining())
    }
}
