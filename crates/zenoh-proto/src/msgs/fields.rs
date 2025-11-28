use crate::*;

#[repr(u8)]
#[derive(ZRU8, Debug, Default, Clone, PartialEq, Copy)]
pub enum ConsolidationMode {
    #[default]
    Auto = 0,
    None = 1,
    Monotonic = 2,
    Latest = 3,
}

#[derive(Debug, PartialEq)]
pub struct BatchSize(pub u16);

impl ZBodyLen for BatchSize {
    fn z_body_len(&self) -> usize {
        self.0.to_le_bytes().len()
    }
}

impl ZBodyEncode for BatchSize {
    fn z_body_encode(&self, w: &mut ZWriter) -> crate::ZResult<(), crate::ZCodecError> {
        w.write(&self.0.to_le_bytes())?;
        Ok(())
    }
}

impl<'a> ZBodyDecode<'a> for BatchSize {
    type Ctx = ();

    fn z_body_decode(
        r: &mut crate::ZReader<'_>,
        _: (),
    ) -> crate::ZResult<Self, crate::ZCodecError> {
        let mut bytes = u16::MAX.to_le_bytes();
        r.read_into(&mut bytes)?;
        Ok(BatchSize(u16::from_le_bytes(bytes)))
    }
}

crate::derive_zstruct_with_body!(BatchSize);
