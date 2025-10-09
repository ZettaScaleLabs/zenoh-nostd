use core::fmt::Debug;

use crate::{
    protocol::{
        ZCodecError,
        common::imsg,
        zcodec::{
            decode_u32, decode_zbuf, encode_u32, encode_zbuf, encoded_len_u32, encoded_len_zbuf,
        },
    },
    result::ZResult,
    zbuf::{ZBuf, ZBufReader, ZBufWriter},
};

pub type EncodingId = u16;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Encoding<'a> {
    pub id: EncodingId,
    pub schema: Option<ZBuf<'a>>,
}

pub mod flag {
    pub const S: u32 = 1;
}

impl<'a> Encoding<'a> {
    pub const fn empty() -> Self {
        Self {
            id: 0,
            schema: None,
        }
    }

    pub fn encoded_len(&self) -> usize {
        let mut len = encoded_len_u32((self.id as u32) << 1);

        if let Some(schema) = &self.schema {
            len += encoded_len_zbuf(true, schema);
        }

        len
    }

    pub fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        let mut id = (self.id as u32) << 1;

        if self.schema.is_some() {
            id |= flag::S;
        }

        encode_u32(id, writer)?;

        if let Some(schema) = &self.schema {
            encode_zbuf(true, schema, writer)?;
        }

        Ok(())
    }

    pub fn decode(reader: &mut ZBufReader<'a>) -> ZResult<Self, ZCodecError> {
        let id = decode_u32(reader)?;

        let has_schema = imsg::has_flag(id as u8, flag::S as u8);
        let id = (id >> 1) as EncodingId;

        let schema = if has_schema {
            let schema: ZBuf<'a> = decode_zbuf(None, reader)?;
            Some(schema)
        } else {
            None
        };

        Ok(Encoding { id, schema })
    }

    #[cfg(test)]
    pub fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        const MIN: usize = 0;
        const MAX: usize = 16;

        let id: EncodingId = rng.r#gen();
        let schema = rng.gen_bool(0.5);
        let schema = if schema {
            use crate::zbuf::BufWriterExt;

            Some(
                zbuf.write_slot_return(rng.gen_range(MIN..MAX), |b: &mut [u8]| {
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

impl Default for Encoding<'_> {
    fn default() -> Self {
        Self::empty()
    }
}
