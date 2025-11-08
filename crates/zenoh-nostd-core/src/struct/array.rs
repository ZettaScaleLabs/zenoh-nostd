use crate::{ZCodecResult, ZReader, ZReaderExt, ZStructDecode, ZStructEncode, ZWriter, ZWriterExt};

impl<const N: usize> ZStructEncode for [u8; N] {
    fn z_len_without_header(&self) -> usize {
        N
    }

    fn z_encode_without_header(&self, w: &mut ZWriter) -> ZCodecResult<()> {
        w.write_exact(self.as_slice())
    }
}

impl<'a, const N: usize> ZStructDecode<'a> for [u8; N] {
    fn z_decode_with_header(r: &mut ZReader<'a>, _: u8) -> ZCodecResult<Self> {
        let mut dst = [0u8; N];
        r.read_into(dst.as_mut_slice())?;
        Ok(dst)
    }
}
