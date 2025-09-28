use zenoh_buffers::{reader::Reader, writer::Writer};
use zenoh_protocol::{
    common::imsg,
    transport::{
        id,
        keepalive::{flag, KeepAlive},
    },
};
use zenoh_result::{bail, ZError, ZResult, ZE};

use crate::{common::extension, RCodec, WCodec, Zenoh080, Zenoh080Header};

impl<W> WCodec<&KeepAlive, &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: &KeepAlive) -> Self::Output {
        let KeepAlive = x;

        // Header
        let header = id::KEEP_ALIVE;
        self.write(&mut *writer, header)?;
        Ok(())
    }
}

impl<R> RCodec<KeepAlive, &mut R> for Zenoh080
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<KeepAlive> {
        let header: u8 = self.read(&mut *reader)?;
        let codec = Zenoh080Header::new(header);
        codec.read(reader)
    }
}

impl<R> RCodec<KeepAlive, &mut R> for Zenoh080Header
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<KeepAlive> {
        if imsg::mid(self.header) != id::KEEP_ALIVE {
            bail!(ZE::DidntRead);
        }

        // Extensions
        let has_ext = imsg::has_flag(self.header, flag::Z);
        if has_ext {
            extension::skip_all::<_, 1, 32>(reader, "Unknown KeepAlive ext")?;
        }

        Ok(KeepAlive)
    }
}
