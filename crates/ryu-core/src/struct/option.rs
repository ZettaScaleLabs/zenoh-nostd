use crate::{ByteIOResult, ByteReader, ByteWriter, r#struct::ZStruct};

impl<T: ZStruct> ZStruct for Option<T> {
    fn z_len(&self) -> usize {
        match self {
            Some(value) => value.z_len(),
            None => 0,
        }
    }

    fn z_encode(&self, w: &mut ByteWriter) -> ByteIOResult<()> {
        if let Some(value) = self {
            value.z_encode(w)?;
        }
        Ok(())
    }

    type ZType<'a> = T::ZType<'a>;

    fn z_decode<'a>(r: &mut ByteReader<'a>) -> ByteIOResult<Self::ZType<'a>> {
        T::z_decode(r)
    }
}
