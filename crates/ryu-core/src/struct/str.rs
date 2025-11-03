use crate::{
    ByteIOError, ByteIOResult, ByteReader, ByteReaderExt, ByteWriter, ByteWriterExt,
    r#struct::ZStruct,
};

impl ZStruct for &'_ str {
    fn z_len(&self) -> usize {
        self.len()
    }

    fn z_encode(&self, w: &mut ByteWriter) -> ByteIOResult<()> {
        w.write_exact(self.as_bytes())
    }

    type ZType<'a> = &'a str;

    fn z_decode<'a>(r: &mut ByteReader<'a>) -> ByteIOResult<Self::ZType<'a>> {
        let bytes = r.read(r.remaining())?;

        core::str::from_utf8(bytes).map_err(|_| ByteIOError::CouldNotParse)
    }
}
