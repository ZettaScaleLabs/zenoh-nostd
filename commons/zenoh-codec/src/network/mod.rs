use zenoh_buffer::{ZBufReader, ZBufWriter};
use zenoh_protocol::{
    common::extension::{ZExtZ64, ZExtZBufHeader},
    core::{EntityId, ZenohIdProto},
    network::ext::{self, EntityGlobalIdType},
};
use zenoh_result::{zctx, WithContext, ZResult};

use crate::{LCodec, RCodec, WCodec, Zenoh080};

pub mod declare;
pub mod interest;
pub mod push;
pub mod request;
pub mod response;

impl<'a, const ID: u8> WCodec<'a, (ext::QoSType<{ ID }>, bool)> for Zenoh080 {
    fn write(
        &self,
        message: (ext::QoSType<{ ID }>, bool),
        writer: &mut ZBufWriter<'a>,
    ) -> ZResult<()> {
        let (x, more) = message;
        let ext: ZExtZ64<{ ID }> = x.into();
        self.write((&ext, more), writer).ctx(zctx!())
    }
}

impl<'a, const ID: u8> RCodec<'a, (ext::QoSType<{ ID }>, bool)> for Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut ZBufReader<'a>,
        header: u8,
    ) -> ZResult<(ext::QoSType<{ ID }>, bool)> {
        let (ext, more): (ZExtZ64<{ ID }>, bool) =
            self.read_knowing_header(reader, header).ctx(zctx!())?;
        Ok((ext.into(), more))
    }

    fn read(&self, reader: &mut ZBufReader<'a>) -> ZResult<(ext::QoSType<{ ID }>, bool)> {
        let header: u8 = self.read(reader).ctx(zctx!())?;

        self.read_knowing_header(reader, header).ctx(zctx!())
    }
}

impl<'a, const ID: u8> WCodec<'a, (&ext::TimestampType<{ ID }>, bool)> for Zenoh080 {
    fn write(
        &self,
        message: (&ext::TimestampType<{ ID }>, bool),
        writer: &mut ZBufWriter<'a>,
    ) -> ZResult<()> {
        let (tstamp, more) = message;
        let header: ZExtZBufHeader<{ ID }> = ZExtZBufHeader::new(self.w_len(&tstamp.timestamp));
        self.write((&header, more), writer).ctx(zctx!())?;
        self.write(&tstamp.timestamp, writer).ctx(zctx!())
    }
}

impl<'a, const ID: u8> RCodec<'a, (ext::TimestampType<{ ID }>, bool)> for Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut ZBufReader<'a>,
        header: u8,
    ) -> ZResult<(ext::TimestampType<{ ID }>, bool)> {
        let (_, more): (ZExtZBufHeader<{ ID }>, bool) =
            self.read_knowing_header(reader, header).ctx(zctx!())?;
        let timestamp: uhlc::Timestamp = self.read(reader).ctx(zctx!())?;
        Ok((ext::TimestampType { timestamp }, more))
    }

    fn read(&self, reader: &mut ZBufReader<'a>) -> ZResult<(ext::TimestampType<{ ID }>, bool)> {
        let header: u8 = self.read(reader).ctx(zctx!())?;
        self.read_knowing_header(reader, header).ctx(zctx!())
    }
}

impl<'a, const ID: u8> WCodec<'a, (ext::NodeIdType<{ ID }>, bool)> for Zenoh080 {
    fn write(
        &self,
        message: (ext::NodeIdType<{ ID }>, bool),
        writer: &mut ZBufWriter<'a>,
    ) -> ZResult<()> {
        let (x, more) = message;
        let ext: ZExtZ64<{ ID }> = x.into();
        self.write((&ext, more), writer).ctx(zctx!())
    }
}

impl<'a, const ID: u8> RCodec<'a, (ext::NodeIdType<{ ID }>, bool)> for Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut ZBufReader<'a>,
        header: u8,
    ) -> ZResult<(ext::NodeIdType<{ ID }>, bool)> {
        let (ext, more): (ZExtZ64<{ ID }>, bool) =
            self.read_knowing_header(reader, header).ctx(zctx!())?;
        Ok((ext.into(), more))
    }

    fn read(&self, reader: &mut ZBufReader<'a>) -> ZResult<(ext::NodeIdType<{ ID }>, bool)> {
        let header: u8 = self.read(reader).ctx(zctx!())?;
        self.read_knowing_header(reader, header).ctx(zctx!())
    }
}

impl<'a, const ID: u8> LCodec<'a, &ext::EntityGlobalIdType<{ ID }>> for Zenoh080 {
    fn w_len(&self, message: &ext::EntityGlobalIdType<{ ID }>) -> usize {
        let EntityGlobalIdType { zid, eid } = message;

        1 + self.w_len(zid) + self.w_len(*eid)
    }
}

impl<'a, const ID: u8> WCodec<'a, (&ext::EntityGlobalIdType<{ ID }>, bool)> for Zenoh080 {
    fn write(
        &self,
        message: (&ext::EntityGlobalIdType<{ ID }>, bool),
        writer: &mut ZBufWriter<'a>,
    ) -> ZResult<()> {
        let (x, more) = message;
        let header: ZExtZBufHeader<{ ID }> = ZExtZBufHeader::new(self.w_len(x));
        self.write((&header, more), writer).ctx(zctx!())?;

        let flags: u8 = (x.zid.size() as u8 - 1) << 4;
        self.write(flags, writer).ctx(zctx!())?;

        self.write_without_length(message, writer).ctx(zctx!())?;

        self.write(x.eid, writer).ctx(zctx!())?;
        Ok(())
    }
}

impl<'a, const ID: u8> RCodec<'a, (ext::EntityGlobalIdType<{ ID }>, bool)> for Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut ZBufReader<'a>,
        header: u8,
    ) -> ZResult<(ext::EntityGlobalIdType<{ ID }>, bool)> {
        let (_, more): (ZExtZBufHeader<{ ID }>, bool) =
            self.read_knowing_header(reader, header).ctx(zctx!())?;

        let flags: u8 = self.read(reader).ctx(zctx!())?;
        let length = 1 + ((flags >> 4) as usize);

        let zid: ZenohIdProto = self.read_knowing_length(reader, length)?;

        let eid: EntityId = self.read(reader).ctx(zctx!())?;

        Ok((ext::EntityGlobalIdType { zid, eid }, more))
    }
}
