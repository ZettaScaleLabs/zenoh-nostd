use crate::{
    ZBodyDecode, ZBodyEncode, ZBodyLen, ZDecode, ZEncode, ZLen, ZReader, ZReaderExt, ZWriter,
    ZWriterExt,
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
    fn z_body_encode(&self, w: &mut ZWriter) -> crate::ZResult<(), crate::ZCodecError> {
        w.write_exact(self)
    }
}

impl ZEncode for &[u8] {
    fn z_encode(&self, w: &mut ZWriter) -> crate::ZResult<(), crate::ZCodecError> {
        <Self as ZBodyEncode>::z_body_encode(self, w)
    }
}

impl<'a> ZBodyDecode<'a> for &'a [u8] {
    type Ctx = ();

    fn z_body_decode(r: &mut ZReader<'a>, _: ()) -> crate::ZResult<Self, crate::ZCodecError> {
        r.read(r.remaining())
    }
}

impl<'a> ZDecode<'a> for &'a [u8] {
    fn z_decode(r: &mut ZReader<'a>) -> crate::ZResult<Self, crate::ZCodecError> {
        <Self as ZBodyDecode>::z_body_decode(r, ())
    }
}
