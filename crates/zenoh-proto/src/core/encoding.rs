use crate::*;

#[derive(Debug, PartialEq, Default)]
pub struct Encoding<'a> {
    pub id: u16,

    pub schema: Option<&'a [u8]>,
}

impl Encoding<'_> {
    const FLAG_S: u8 = 0b0000_0001;

    pub const fn bytes() -> Self {
        Self {
            id: 0,
            schema: None,
        }
    }

    pub const fn string() -> Self {
        Self {
            id: 1,
            schema: None,
        }
    }
}
impl ZBodyLen for Encoding<'_> {
    fn z_body_len(&self) -> usize {
        <u32 as ZLen>::z_len(&((self.id as u32) << 1))
            + if let Some(schema) = self.schema.as_ref() {
                let len: usize = <&[u8] as ZLen>::z_len(schema);

                <usize as ZLen>::z_len(&len) + len
            } else {
                0
            }
    }
}

impl ZBodyEncode for Encoding<'_> {
    fn z_body_encode(&self, w: &mut impl crate::ZWrite) -> crate::ZResult<(), crate::ZCodecError> {
        let mut id = (self.id as u32) << 1;

        if self.schema.is_some() {
            id |= Self::FLAG_S as u32;
        }

        <u32 as ZEncode>::z_encode(&id, w)?;

        if let Some(schema) = &self.schema {
            <usize as ZEncode>::z_encode(&schema.len(), w)?;
            <&[u8] as ZEncode>::z_encode(schema, w)?;
        }

        Ok(())
    }
}

impl<'a> ZBodyDecode<'a> for Encoding<'a> {
    type Ctx = ();

    fn z_body_decode(
        r: &mut impl crate::ZRead<'a>,
        _: (),
    ) -> crate::ZResult<Self, crate::ZCodecError> {
        let id = <u32 as ZDecode>::z_decode(r)?;

        let has_schema = (id as u8) & Self::FLAG_S != 0;
        let id = (id >> 1) as u16;

        let schema = if has_schema {
            let len = <usize as ZDecode>::z_decode(r)?;
            let schema: &[u8] = <&[u8] as ZDecode>::z_decode(&mut r.read_slice(len)?)?;
            Some(schema)
        } else {
            None
        };

        Ok(Encoding { id, schema })
    }
}

crate::derive_zstruct_with_body!(lt, Encoding<'a>);
