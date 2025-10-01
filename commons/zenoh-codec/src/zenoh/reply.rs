use heapless::Vec;
use zenoh_protocol::{
    common::imsg,
    zenoh::{
        id,
        query::ConsolidationMode,
        reply::{flag, Reply, ReplyBody},
    },
};
use zenoh_result::{zbail, zctx, zerr, WithContext, ZE};

use crate::{common::extension, RCodec, WCodec};

impl<'a, const MAX_EXT_UNKNOWN: usize> WCodec<'a, &Reply<'_, MAX_EXT_UNKNOWN>> for crate::Zenoh080 {
    fn write(
        &self,
        message: &Reply<'_, MAX_EXT_UNKNOWN>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let Reply {
            consolidation,
            ext_unknown,
            payload,
        } = message;

        let mut header = id::REPLY;

        if consolidation != &ConsolidationMode::DEFAULT {
            header |= flag::C;
        }

        let mut n_exts = ext_unknown.len() as u8;

        if n_exts != 0 {
            header |= flag::Z;
        }

        self.write(header, writer).ctx(zctx!())?;

        if consolidation != &ConsolidationMode::DEFAULT {
            self.write(consolidation, writer).ctx(zctx!())?;
        }

        for u in ext_unknown.iter() {
            n_exts -= 1;
            self.write((u, n_exts != 0), writer).ctx(zctx!())?;
        }

        self.write(payload, writer).ctx(zctx!())?;

        Ok(())
    }
}

impl<'a, const MAX_EXT_UNKNOWN: usize> RCodec<'a, Reply<'a, MAX_EXT_UNKNOWN>> for crate::Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
        header: u8,
    ) -> zenoh_result::ZResult<Reply<'a, MAX_EXT_UNKNOWN>> {
        if imsg::mid(header) != id::REPLY {
            zbail!(ZE::ReadFailure);
        }

        let mut consolidation = ConsolidationMode::DEFAULT;
        if imsg::has_flag(header, flag::C) {
            consolidation = self.read(reader).ctx(zctx!())?;
        }

        let mut ext_unknown = Vec::new();

        let mut has_ext = imsg::has_flag(header, flag::Z);
        while has_ext {
            let ext: u8 = self.read(reader).ctx(zctx!())?;
            let (u, ext) = extension::read(reader, "Reply", ext)?;
            ext_unknown.push(u).map_err(|_| zerr!(ZE::ReadFailure))?;
            has_ext = ext;
        }

        let payload: ReplyBody<'_, _> = self.read(reader).ctx(zctx!())?;

        Ok(Reply {
            consolidation,
            ext_unknown,
            payload,
        })
    }

    fn read(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
    ) -> zenoh_result::ZResult<Reply<'a, MAX_EXT_UNKNOWN>> {
        let header: u8 = self.read(reader).ctx(zctx!())?;

        self.read_knowing_header(reader, header).ctx(zctx!())
    }
}
