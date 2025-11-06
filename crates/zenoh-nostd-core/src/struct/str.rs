use crate::{
    ZCodecError, ZCodecResult, ZReader, ZReaderExt, ZStructDecode, ZStructEncode, ZWriter,
    ZWriterExt,
};

impl ZStructEncode for &'_ str {
    fn z_len(&self) -> usize {
        self.len()
    }

    fn z_encode(&self, w: &mut ZWriter) -> ZCodecResult<()> {
        w.write_exact(self.as_bytes())
    }
}

impl<'a> ZStructDecode<'a> for &'a str {
    fn z_decode(r: &mut ZReader<'a>) -> ZCodecResult<Self> {
        let bytes = r.read(r.remaining())?;

        core::str::from_utf8(bytes).map_err(|_| ZCodecError::CouldNotParse)
    }
}
