use crate::{
    ByteIOResult, ByteReader, ByteReaderExt, ByteWriter, ByteWriterExt, r#struct::ZStruct,
};

impl<const N: usize> ZStruct for [u8; N] {
    fn z_len(&self) -> usize {
        N
    }

    fn z_encode(&self, w: &mut ByteWriter) -> ByteIOResult<()> {
        w.write_exact(self.as_slice())
    }

    type ZType<'a> = [u8; N];

    fn z_decode<'a>(r: &mut ByteReader<'a>) -> ByteIOResult<Self::ZType<'a>> {
        let mut dst = [0u8; N];
        r.read_into(dst.as_mut_slice())?;
        Ok(dst)
    }
}
