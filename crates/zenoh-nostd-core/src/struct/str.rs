use crate::{
    ZCodecError, ZCodecResult, ZReader, ZReaderExt, ZStructDecode, ZStructEncode, ZWriter,
    ZWriterExt,
};

impl ZStructEncode for &'_ str {
    fn z_len_without_header(&self) -> usize {
        self.len()
    }

    fn z_encode_without_header(&self, w: &mut ZWriter) -> ZCodecResult<()> {
        w.write_exact(self.as_bytes())
    }
}

impl<'a> ZStructDecode<'a> for &'a str {
    fn z_decode_with_header(r: &mut ZReader<'a>, _: u8) -> ZCodecResult<Self> {
        let bytes = r.read(r.remaining())?;

        core::str::from_utf8(bytes).map_err(|_| ZCodecError::CouldNotParse)
    }
}
