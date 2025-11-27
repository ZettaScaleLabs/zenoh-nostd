use crate::{
    ZBodyDecode, ZBodyEncode, ZBodyLen, ZDecode, ZEncode, ZLen, ZReader, ZReaderExt, ZWriter,
    ZWriterExt,
};

impl ZBodyLen for &str {
    fn z_body_len(&self) -> usize {
        self.len()
    }
}

impl ZLen for &str {
    fn z_len(&self) -> usize {
        <Self as ZBodyLen>::z_body_len(self)
    }
}

impl ZBodyEncode for &str {
    fn z_body_encode(&self, w: &mut ZWriter) -> crate::ZResult<(), crate::ZCodecError> {
        w.write_exact(self.as_bytes())
    }
}

impl ZEncode for &str {
    fn z_encode(&self, w: &mut ZWriter) -> crate::ZResult<(), crate::ZCodecError> {
        <Self as ZBodyEncode>::z_body_encode(self, w)
    }
}

impl<'a> ZBodyDecode<'a> for &'a str {
    type Ctx = ();

    fn z_body_decode(r: &mut ZReader<'a>, _: ()) -> crate::ZResult<Self, crate::ZCodecError> {
        let bytes = r.read(r.remaining())?;

        core::str::from_utf8(bytes).map_err(|_| crate::ZCodecError::CouldNotParseField)
    }
}

impl<'a> ZDecode<'a> for &'a str {
    fn z_decode(r: &mut ZReader<'a>) -> crate::ZResult<Self, crate::ZCodecError> {
        <Self as ZBodyDecode>::z_body_decode(r, ())
    }
}
