use zenoh_protocol::{
    common::imsg,
    zenoh::{
        id,
        query::ConsolidationMode,
        reply::{flag, Reply, ReplyBody},
    },
};
use zenoh_result::{zbail, zctx, WithContext, ZE};

use crate::{common::extension, RCodec, WCodec};

impl<'a> WCodec<'a, &Reply<'_>> for crate::ZCodec {
    fn write(
        &self,
        message: &Reply<'_>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let Reply {
            consolidation,

            payload,
        } = message;

        let mut header = id::REPLY;

        if consolidation != &ConsolidationMode::DEFAULT {
            header |= flag::C;
        }

        self.write(header, writer).ctx(zctx!())?;

        if consolidation != &ConsolidationMode::DEFAULT {
            self.write(consolidation, writer).ctx(zctx!())?;
        }

        self.write(payload, writer).ctx(zctx!())?;

        Ok(())
    }
}

impl<'a> RCodec<'a, Reply<'a>> for crate::ZCodec {
    fn read_knowing_header(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
        header: u8,
    ) -> zenoh_result::ZResult<Reply<'a>> {
        if imsg::mid(header) != id::REPLY {
            zbail!(ZE::ReadFailure);
        }

        let mut consolidation = ConsolidationMode::DEFAULT;
        if imsg::has_flag(header, flag::C) {
            consolidation = self.read(reader).ctx(zctx!())?;
        }

        let mut has_ext = imsg::has_flag(header, flag::Z);
        while has_ext {
            let ext: u8 = self.read(reader).ctx(zctx!())?;
            let (_, ext) = extension::read(reader, "Reply", ext)?;
            has_ext = ext;
        }

        let payload: ReplyBody<'_> = self.read(reader).ctx(zctx!())?;

        Ok(Reply {
            consolidation,

            payload,
        })
    }

    fn read(&self, reader: &mut zenoh_buffer::ZBufReader<'a>) -> zenoh_result::ZResult<Reply<'a>> {
        let header: u8 = self.read(reader).ctx(zctx!())?;

        self.read_knowing_header(reader, header).ctx(zctx!())
    }
}
