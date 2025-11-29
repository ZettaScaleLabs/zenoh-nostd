use crate::*;

impl<const N: usize> ZBodyLen for [u8; N] {
    fn z_body_len(&self) -> usize {
        N
    }
}

impl<const N: usize> ZBodyEncode for [u8; N] {
    fn z_body_encode(&self, w: &mut impl crate::ZWrite) -> crate::ZResult<(), crate::ZCodecError> {
        Ok(w.write_exact(self.as_slice())?)
    }
}

impl<'a, const N: usize> ZBodyDecode<'a> for [u8; N] {
    type Ctx = ();

    fn z_body_decode(
        r: &mut impl crate::ZRead<'a>,
        _: (),
    ) -> crate::ZResult<Self, crate::ZCodecError> {
        let mut dst = [0u8; N];
        r.read(dst.as_mut_slice())?;
        Ok(dst)
    }
}

impl<const N: usize> ZLen for [u8; N] {
    fn z_len(&self) -> usize {
        <Self as ZBodyLen>::z_body_len(self)
    }
}

impl<const N: usize> ZEncode for [u8; N] {
    fn z_encode(&self, w: &mut impl crate::ZWrite) -> crate::ZResult<(), crate::ZCodecError> {
        <Self as ZBodyEncode>::z_body_encode(self, w)
    }
}

impl<'a, const N: usize> ZDecode<'a> for [u8; N] {
    fn z_decode(r: &mut impl crate::ZRead<'a>) -> crate::ZResult<Self, crate::ZCodecError> {
        <Self as ZBodyDecode>::z_body_decode(r, ())
    }
}
