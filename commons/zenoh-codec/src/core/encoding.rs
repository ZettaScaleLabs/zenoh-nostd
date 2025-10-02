use zenoh_buffer::ZBuf;
use zenoh_protocol::{
    common::imsg,
    core::encoding::{flag, Encoding, EncodingId},
};
use zenoh_result::{zbail, zctx, WithContext};

use crate::{LCodec, RCodec, WCodec, ZCodec};

impl<'a> LCodec<'a, &Encoding<'_>> for ZCodec {
    fn w_len(&self, message: &Encoding<'_>) -> usize {
        let mut len = self.w_len((message.id as u32) << 1);
        if let Some(schema) = message.schema.as_ref() {
            len += self.w_len(schema);
        }
        len
    }
}

impl<'a> WCodec<'a, &Encoding<'_>> for ZCodec {
    fn write(
        &self,
        message: &Encoding<'_>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let mut id = (message.id as u32) << 1;

        if message.schema.is_some() {
            id |= flag::S;
        }

        self.write(id, writer).ctx(zctx!())?;

        if let Some(schema) = &message.schema {
            if schema.len() > 255 {
                zbail!(zenoh_result::ZE::InvalidArgument);
            }

            self.write(schema, writer).ctx(zctx!())?;
        }

        Ok(())
    }
}

impl<'a> WCodec<'a, Encoding<'_>> for ZCodec {
    fn write(
        &self,
        message: Encoding<'_>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        self.write(&message, writer).ctx(zctx!())
    }
}

impl<'a> RCodec<'a, Encoding<'a>> for ZCodec {
    fn read(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
    ) -> zenoh_result::ZResult<Encoding<'a>> {
        let id: u32 = self.read(reader).ctx(zctx!())?;
        let has_schema = imsg::has_flag(id as u8, flag::S as u8);
        let id = (id >> 1) as EncodingId;

        let schema = if has_schema {
            let schema: ZBuf<'a> = self.read(reader).ctx(zctx!())?;
            if schema.len() > 255 {
                zbail!(zenoh_result::ZE::InvalidArgument);
            }

            Some(schema)
        } else {
            None
        };

        Ok(Encoding { id, schema })
    }
}
