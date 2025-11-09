use crate::{
    ZBodyDecode, ZBodyEncode, ZBodyLen, ZCodecError, ZCodecResult, ZDecode, ZEncode, ZLen, ZReader,
    ZReaderExt, ZWriter, ZWriterExt,
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
    fn z_body_encode(&self, w: &mut ZWriter) -> ZCodecResult<()> {
        w.write_exact(self.as_bytes())
    }
}

impl ZEncode for &str {
    fn z_encode(&self, w: &mut ZWriter) -> ZCodecResult<()> {
        <Self as ZBodyEncode>::z_body_encode(self, w)
    }
}

impl<'a> ZBodyDecode<'a> for &'a str {
    type Ctx = ();

    fn z_body_decode(r: &mut ZReader<'a>, _: ()) -> ZCodecResult<Self> {
        let bytes = r.read(r.remaining())?;

        core::str::from_utf8(bytes).map_err(|_| ZCodecError::CouldNotParse)
    }
}

impl<'a> ZDecode<'a> for &'a str {
    fn z_decode(r: &mut ZReader<'a>) -> ZCodecResult<Self> {
        <Self as ZBodyDecode>::z_body_decode(r, ())
    }
}
