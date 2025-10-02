use uhlc::{ID, NTP64};
use zenoh_buffer::ZBuf;
use zenoh_protocol::core::Timestamp;
use zenoh_result::{zctx, zerr, WithContext, ZE};

use crate::{LCodec, RCodec, WCodec, ZCodec};

impl<'a> LCodec<'a, &Timestamp> for ZCodec {
    fn w_len(&self, message: &Timestamp) -> usize {
        let id = message.get_id();
        let bytes = &id.to_le_bytes()[..id.size()];

        self.w_len(message.get_time().as_u64()) + self.w_len(ZBuf(bytes))
    }
}

impl<'a> WCodec<'a, &Timestamp> for ZCodec {
    fn write(
        &self,
        message: &Timestamp,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let time = message.get_time().as_u64();
        let id = message.get_id();
        let bytes = &id.to_le_bytes()[..id.size()];

        self.write(time, writer).ctx(zctx!())?;
        self.write(ZBuf(bytes), writer).ctx(zctx!())?;

        Ok(())
    }
}

impl<'a> WCodec<'a, Timestamp> for ZCodec {
    fn write(
        &self,
        message: Timestamp,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        self.write(&message, writer).ctx(zctx!())
    }
}

impl<'a> RCodec<'a, Timestamp> for ZCodec {
    fn read(&self, reader: &mut zenoh_buffer::ZBufReader<'a>) -> zenoh_result::ZResult<Timestamp> {
        let time: u64 = self.read(reader).ctx(zctx!())?;
        let zbuf: ZBuf<'a> = self.read(reader).ctx(zctx!())?;
        let id = ID::try_from(zbuf.as_bytes()).map_err(|_| zerr!(ZE::ReadFailure))?;

        let time = NTP64(time);
        Ok(Timestamp::new(time, id))
    }
}
