use core::fmt::Debug;

use crate::{ByteIOResult, ByteReader, ByteReaderExt, ByteWriter, ZStruct};

/// TODO: make the codec capable of handling this silly one.
#[derive(Debug, PartialEq)]
pub struct Encoding<'a> {
    pub id: u16,

    pub schema: Option<&'a [u8]>,
}

impl<'a> Encoding<'a> {
    const FLAG_S: u8 = 0b0000_0001;

    pub const fn empty() -> Self {
        Self {
            id: 0,
            schema: None,
        }
    }

    #[cfg(test)]
    pub fn rand(w: &mut ByteWriter<'a>) -> Self {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        const MIN: usize = 0;
        const MAX: usize = 16;

        let id: u16 = rng.r#gen();
        let schema = if rng.gen_bool(0.5) {
            use crate::ByteWriterExt;

            Some(
                w.write_slot(rng.gen_range(MIN..MAX), |b: &mut [u8]| {
                    rng.fill(b);
                    b.len()
                })
                .unwrap(),
            )
        } else {
            None
        };

        Encoding { id, schema }
    }
}

impl ZStruct for Encoding<'_> {
    fn z_len(&self) -> usize {
        <u32 as ZStruct>::z_len(&((self.id as u32) << 1))
            + if let Some(schema) = self.schema.as_ref() {
                let len: usize = <&[u8] as ZStruct>::z_len(schema);

                <usize as ZStruct>::z_len(&len) + len
            } else {
                0
            }
    }

    fn z_encode(&self, w: &mut ByteWriter) -> ByteIOResult<()> {
        let mut id = (self.id as u32) << 1;

        if self.schema.is_some() {
            id |= Self::FLAG_S as u32;
        }

        <u32 as ZStruct>::z_encode(&id, w)?;

        if let Some(schema) = &self.schema {
            <usize as ZStruct>::z_encode(&schema.len(), w)?;
            <&[u8] as ZStruct>::z_encode(schema, w)?;
        }

        Ok(())
    }

    type ZType<'a> = Encoding<'a>;

    fn z_decode<'a>(r: &mut ByteReader<'a>) -> ByteIOResult<Self::ZType<'a>> {
        let id = <u32 as ZStruct>::z_decode(r)?;

        let has_schema = (id as u8) & Self::FLAG_S != 0;
        let id = (id >> 1) as u16;

        let schema = if has_schema {
            let len = <usize as ZStruct>::z_decode(r)?;
            let schema: &[u8] = <&[u8] as ZStruct>::z_decode(&mut r.sub(len)?)?;
            Some(schema)
        } else {
            None
        };

        Ok(Encoding { id, schema })
    }
}
