// mod declare;
// mod interest;
// mod oam;
mod push;
// mod request;
// mod response;

use zenoh_buffers::{reader::Reader, writer::Writer, zslice::ZSliceLen};
use zenoh_protocol::{
    common::{imsg, ZExtZ64, ZExtZBufHeader},
    core::{EntityId, Reliability, ZenohIdProto},
    network::{
        ext::{self, EntityGlobalIdType},
        id, NetworkBody, NetworkBodyRef, NetworkMessage, NetworkMessageExt, NetworkMessageRef,
    },
};
use zenoh_result::{bail, ZError, ZResult, ZE};

use crate::{
    LCodec, RCodec, WCodec, Zenoh080, Zenoh080Header, Zenoh080Length, Zenoh080Reliability,
};

// NetworkMessage
impl<W> WCodec<NetworkMessageRef<'_>, &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: NetworkMessageRef) -> Self::Output {
        let NetworkMessageRef { body, .. } = x;

        match body {
            NetworkBodyRef::Push(b) => self.write(&mut *writer, b),
            // NetworkBodyRef::Request(b) => self.write(&mut *writer, b),
            // NetworkBodyRef::Response(b) => self.write(&mut *writer, b),
            // NetworkBodyRef::ResponseFinal(b) => self.write(&mut *writer, b),
            // NetworkBodyRef::Interest(b) => self.write(&mut *writer, b),
            // NetworkBodyRef::Declare(b) => self.write(&mut *writer, b),
            // NetworkBodyRef::OAM(b) => self.write(&mut *writer, b),
        }
    }
}

impl<W> WCodec<&NetworkMessage, &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: &NetworkMessage) -> Self::Output {
        self.write(writer, x.as_ref())
    }
}

impl<R, const L: usize> RCodec<NetworkMessage, &mut R> for (Zenoh080, ZSliceLen<L>)
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<NetworkMessage> {
        let codec = (
            Zenoh080Reliability::new(Reliability::DEFAULT),
            ZSliceLen::<L>,
        );
        codec.read(reader)
    }
}

impl<R, const L: usize> RCodec<NetworkMessage, &mut R> for (Zenoh080Reliability, ZSliceLen<L>)
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<NetworkMessage> {
        let header: u8 = self.0.codec.read(&mut *reader)?;

        let codec = (Zenoh080Header::new(header), ZSliceLen::<L>);
        let mut msg: NetworkMessage = codec.read(&mut *reader)?;
        msg.reliability = self.0.reliability;
        Ok(msg)
    }
}

impl<R, const L: usize> RCodec<NetworkMessage, &mut R> for (Zenoh080Header, ZSliceLen<L>)
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<NetworkMessage> {
        let body = match imsg::mid(self.0.header) {
            id::PUSH => NetworkBody::Push(self.read(&mut *reader)?),
            // id::REQUEST => NetworkBody::Request(self.read(&mut *reader)?),
            // id::RESPONSE => NetworkBody::Response(self.read(&mut *reader)?),
            // id::RESPONSE_FINAL => NetworkBody::ResponseFinal(self.read(&mut *reader)?),
            // id::INTEREST => NetworkBody::Interest(self.read(&mut *reader)?),
            // id::DECLARE => NetworkBody::Declare(self.read(&mut *reader)?),
            // id::OAM => NetworkBody::OAM(self.read(&mut *reader)?),
            _ => bail!(ZE::DidntRead),
        };

        Ok(body.into())
    }
}

// pub struct NetworkMessageIter<R> {
//     codec: Zenoh080Reliability,
//     reader: R,
// }

// impl<R> NetworkMessageIter<R> {
//     pub fn new(reliability: Reliability, reader: R) -> Self {
//         let codec = Zenoh080Reliability::new(reliability);
//         Self { codec, reader }
//     }
// }

// impl<R: Reader> Iterator for NetworkMessageIter<R> {
//     type Item = NetworkMessage;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.codec.read(&mut self.reader).ok()
//     }
// }

// Extensions: QoS
impl<W, const ID: u8> WCodec<(ext::QoSType<{ ID }>, bool), &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: (ext::QoSType<{ ID }>, bool)) -> Self::Output {
        let (x, more) = x;
        let ext: ZExtZ64<{ ID }> = x.into();
        self.write(&mut *writer, (&ext, more))
    }
}

impl<R, const ID: u8> RCodec<(ext::QoSType<{ ID }>, bool), &mut R> for Zenoh080
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<(ext::QoSType<{ ID }>, bool)> {
        let header: u8 = self.read(&mut *reader)?;
        let codec = Zenoh080Header::new(header);
        codec.read(reader)
    }
}

impl<R, const ID: u8> RCodec<(ext::QoSType<{ ID }>, bool), &mut R> for Zenoh080Header
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<(ext::QoSType<{ ID }>, bool)> {
        let (ext, more): (ZExtZ64<{ ID }>, bool) = self.read(&mut *reader)?;
        Ok((ext.into(), more))
    }
}

// Extensions: Timestamp
impl<W, const ID: u8> WCodec<(&ext::TimestampType<{ ID }>, bool), &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: (&ext::TimestampType<{ ID }>, bool)) -> Self::Output {
        let (tstamp, more) = x;
        let header: ZExtZBufHeader<{ ID }> = ZExtZBufHeader::new(self.w_len(&tstamp.timestamp));
        self.write(&mut *writer, (&header, more))?;
        self.write(&mut *writer, &tstamp.timestamp)
    }
}

impl<R, const ID: u8> RCodec<(ext::TimestampType<{ ID }>, bool), &mut R> for Zenoh080
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<(ext::TimestampType<{ ID }>, bool)> {
        let header: u8 = self.read(&mut *reader)?;
        let codec = Zenoh080Header::new(header);
        codec.read(reader)
    }
}

impl<R, const ID: u8> RCodec<(ext::TimestampType<{ ID }>, bool), &mut R> for Zenoh080Header
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<(ext::TimestampType<{ ID }>, bool)> {
        let (_, more): (ZExtZBufHeader<{ ID }>, bool) = self.read(&mut *reader)?;
        let timestamp: uhlc::Timestamp = self.codec.read(&mut *reader)?;
        Ok((ext::TimestampType { timestamp }, more))
    }
}

// Extensions: NodeId
impl<W, const ID: u8> WCodec<(ext::NodeIdType<{ ID }>, bool), &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: (ext::NodeIdType<{ ID }>, bool)) -> Self::Output {
        let (x, more) = x;
        let ext: ZExtZ64<{ ID }> = x.into();
        self.write(&mut *writer, (&ext, more))
    }
}

impl<R, const ID: u8> RCodec<(ext::NodeIdType<{ ID }>, bool), &mut R> for Zenoh080
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<(ext::NodeIdType<{ ID }>, bool)> {
        let header: u8 = self.read(&mut *reader)?;
        let codec = Zenoh080Header::new(header);
        codec.read(reader)
    }
}

impl<R, const ID: u8> RCodec<(ext::NodeIdType<{ ID }>, bool), &mut R> for Zenoh080Header
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<(ext::NodeIdType<{ ID }>, bool)> {
        let (ext, more): (ZExtZ64<{ ID }>, bool) = self.read(&mut *reader)?;
        Ok((ext.into(), more))
    }
}

// Extension: EntityId
impl<const ID: u8> LCodec<&ext::EntityGlobalIdType<{ ID }>> for Zenoh080 {
    fn w_len(self, x: &ext::EntityGlobalIdType<{ ID }>) -> usize {
        let EntityGlobalIdType { zid, eid } = x;

        1 + self.w_len(zid) + self.w_len(*eid)
    }
}

impl<W, const ID: u8> WCodec<(&ext::EntityGlobalIdType<{ ID }>, bool), &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: (&ext::EntityGlobalIdType<{ ID }>, bool)) -> Self::Output {
        let (x, more) = x;
        let header: ZExtZBufHeader<{ ID }> = ZExtZBufHeader::new(self.w_len(x));
        self.write(&mut *writer, (&header, more))?;

        let flags: u8 = (x.zid.size() as u8 - 1) << 4;
        self.write(&mut *writer, flags)?;

        let lodec = Zenoh080Length::new(x.zid.size());
        lodec.write(&mut *writer, &x.zid)?;

        self.write(&mut *writer, x.eid)?;
        Ok(())
    }
}

impl<R, const ID: u8> RCodec<(ext::EntityGlobalIdType<{ ID }>, bool), &mut R> for Zenoh080Header
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<(ext::EntityGlobalIdType<{ ID }>, bool)> {
        let (_, more): (ZExtZBufHeader<{ ID }>, bool) = self.read(&mut *reader)?;

        let flags: u8 = self.codec.read(&mut *reader)?;
        let length = 1 + ((flags >> 4) as usize);

        let lodec = Zenoh080Length::new(length);
        let zid: ZenohIdProto = lodec.read(&mut *reader)?;

        let eid: EntityId = self.codec.read(&mut *reader)?;

        Ok((ext::EntityGlobalIdType { zid, eid }, more))
    }
}
