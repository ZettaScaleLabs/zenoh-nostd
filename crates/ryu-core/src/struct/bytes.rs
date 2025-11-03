use crate::{
    ByteIOResult, ByteReader, ByteReaderExt, ByteWriter, ByteWriterExt, r#struct::ZStruct,
};

impl ZStruct for &[u8] {
    fn z_len(&self) -> usize {
        self.len()
    }

    fn z_encode(&self, w: &mut ByteWriter) -> ByteIOResult<()> {
        w.write_exact(self)
    }

    type ZType<'a> = &'a [u8];

    fn z_decode<'a>(r: &mut ByteReader<'a>) -> ByteIOResult<Self::ZType<'a>> {
        r.read(r.remaining())
    }
}
