use core::convert::TryFrom;

use zenoh_buffers::{reader::Reader, writer::Writer};
use zenoh_protocol::core::ZenohIdProto;
use zenoh_result::{bail, zerr, ZError, ZResult, ZE};

use crate::{LCodec, RCodec, WCodec, Zenoh080, Zenoh080Length};

impl LCodec<&ZenohIdProto> for Zenoh080 {
    fn w_len(self, x: &ZenohIdProto) -> usize {
        x.size()
    }
}

impl<W> WCodec<&ZenohIdProto, &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: &ZenohIdProto) -> Self::Output {
        self.write(&mut *writer, &x.to_le_bytes()[..x.size()])
    }
}

impl<R> RCodec<ZenohIdProto, &mut R> for Zenoh080
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<ZenohIdProto> {
        let size: usize = self.read(&mut *reader)?;
        if size > ZenohIdProto::MAX_SIZE {
            bail!(ZE::DidntRead);
        }
        let mut id = [0; ZenohIdProto::MAX_SIZE];
        reader.read_exact(&mut id[..size])?;
        ZenohIdProto::try_from(&id[..size]).map_err(|_| zerr!(ZE::DidntRead))
    }
}

impl<W> WCodec<&ZenohIdProto, &mut W> for Zenoh080Length
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: &ZenohIdProto) -> Self::Output {
        if self.length > ZenohIdProto::MAX_SIZE {
            bail!(ZE::DidntWrite);
        }
        writer.write_exact(&x.to_le_bytes()[..x.size()])
    }
}

impl<R> RCodec<ZenohIdProto, &mut R> for Zenoh080Length
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<ZenohIdProto> {
        if self.length > ZenohIdProto::MAX_SIZE {
            bail!(ZE::DidntRead);
        }
        let mut id = [0; ZenohIdProto::MAX_SIZE];
        reader.read_exact(&mut id[..self.length])?;
        ZenohIdProto::try_from(&id[..self.length]).map_err(|_| zerr!(ZE::DidntRead))
    }
}
