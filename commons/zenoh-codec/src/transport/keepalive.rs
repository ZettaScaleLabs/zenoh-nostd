use zenoh_protocol::{
    common::imsg,
    transport::{
        id,
        keepalive::{flag, KeepAlive},
    },
};
use zenoh_result::{zbail, zctx, WithContext, ZE};

use crate::{common::extension, RCodec, WCodec, ZCodec};

impl<'a> WCodec<'a, KeepAlive> for ZCodec {
    fn write(
        &self,
        _: KeepAlive,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let header = id::KEEP_ALIVE;

        self.write(header, writer).ctx(zctx!())
    }
}

impl<'a> RCodec<'a, KeepAlive> for ZCodec {
    fn read_knowing_header(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
        header: u8,
    ) -> zenoh_result::ZResult<KeepAlive> {
        if imsg::mid(header) != id::KEEP_ALIVE {
            zbail!(ZE::ReadFailure);
        }

        let has_ext = imsg::has_flag(header, flag::Z);
        if has_ext {
            extension::skip_all(reader, "Unknown KeepAlive ext")?;
        }

        Ok(KeepAlive)
    }

    fn read(&self, reader: &mut zenoh_buffer::ZBufReader<'a>) -> zenoh_result::ZResult<KeepAlive> {
        let header: u8 = self.read(reader).ctx(zctx!())?;
        self.read_knowing_header(reader, header).ctx(zctx!())
    }
}
