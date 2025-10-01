use uhlc::{ID, NTP64};
use zenoh_buffer::ZBuf;
use zenoh_protocol::core::Timestamp;
use zenoh_result::{zerr, ZE};

use crate::{RCodec, WCodec, Zenoh080};

impl<'a> WCodec<'a, &Timestamp> for Zenoh080 {
    fn write(
        &self,
        message: &Timestamp,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let time = message.get_time().as_u64();
        let id = message.get_id();
        let bytes = &id.to_le_bytes()[..id.size()];

        self.write(time, writer)?;
        self.write(ZBuf(bytes), writer)?;

        Ok(())
    }
}

impl<'a> WCodec<'a, Timestamp> for Zenoh080 {
    fn write(
        &self,
        message: Timestamp,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        self.write(&message, writer)
    }
}

impl<'a> RCodec<'a, Timestamp> for Zenoh080 {
    fn read(&self, reader: &mut zenoh_buffer::ZBufReader<'a>) -> zenoh_result::ZResult<Timestamp> {
        let time: u64 = self.read(reader)?;
        let zbuf: ZBuf<'a> = self.read(reader)?;
        let id = ID::try_from(zbuf.as_bytes()).map_err(|_| zerr!(ZE::ReadFailure))?;

        let time = NTP64(time);
        Ok(Timestamp::new(time, id))
    }
}
