use crate::{ZCodecResult, ZReader, ZReaderExt, ZStructDecode, ZStructEncode, ZWriter, ZWriterExt};

impl ZStructEncode for &[u8] {
    fn z_len_without_header(&self) -> usize {
        self.len()
    }

    fn z_encode_without_header(&self, w: &mut ZWriter) -> ZCodecResult<()> {
        w.write_exact(self)
    }
}

impl<'a> ZStructDecode<'a> for &'a [u8] {
    fn z_decode_with_header(r: &mut ZReader<'a>, _: u8) -> ZCodecResult<Self> {
        r.read(r.remaining())
    }
}
