use zenoh_buffer::ZBuf;
use zenoh_protocol::{
    common::imsg,
    core::encoding::{flag, Encoding, EncodingId},
};
use zenoh_result::zbail;

use crate::{RCodec, WCodec, Zenoh080};

impl<'a> WCodec<'a, Encoding<'_>> for Zenoh080 {
    fn write(
        &self,
        message: Encoding<'_>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let mut id = (message.id as u32) << 1;

        if message.schema.is_some() {
            id |= flag::S;
        }

        self.write(id, writer)?;

        if let Some(schema) = message.schema {
            if schema.len() > 255 {
                zbail!(zenoh_result::ZE::InvalidArgument);
            }

            self.write(schema, writer)?;
        }

        Ok(())
    }
}

impl<'a> RCodec<'a, Encoding<'a>> for Zenoh080 {
    fn read(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
    ) -> zenoh_result::ZResult<Encoding<'a>> {
        let id: u32 = self.read(reader)?;
        let has_schema = imsg::has_flag(id as u8, flag::S as u8);
        let id = (id >> 1) as EncodingId;

        let schema = if has_schema {
            let schema: ZBuf<'a> = self.read(reader)?;
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
