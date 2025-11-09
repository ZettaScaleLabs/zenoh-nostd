use crate::{
    ZBodyDecode, ZBodyEncode, ZBodyLen, ZCodecResult, ZDecode, ZEncode, ZLen, ZReader, ZReaderExt,
    ZWriter, ZWriterExt,
};

impl ZBodyLen for &[u8] {
    fn z_body_len(&self) -> usize {
        self.len()
    }
}

impl ZLen for &[u8] {
    fn z_len(&self) -> usize {
        <Self as ZBodyLen>::z_body_len(self)
    }
}

impl ZBodyEncode for &[u8] {
    fn z_body_encode(&self, w: &mut ZWriter) -> ZCodecResult<()> {
        w.write_exact(self)
    }
}

impl ZEncode for &[u8] {
    fn z_encode(&self, w: &mut ZWriter) -> ZCodecResult<()> {
        <Self as ZBodyEncode>::z_body_encode(self, w)
    }
}

impl<'a> ZBodyDecode<'a> for &'a [u8] {
    type Ctx = ();

    fn z_body_decode(r: &mut ZReader<'a>, _: ()) -> ZCodecResult<Self> {
        r.read(r.remaining())
    }
}

impl<'a> ZDecode<'a> for &'a [u8] {
    fn z_decode(r: &mut ZReader<'a>) -> ZCodecResult<Self> {
        <Self as ZBodyDecode>::z_body_decode(r, ())
    }
}
