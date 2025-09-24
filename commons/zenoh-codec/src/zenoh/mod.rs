// pub mod del;
// pub mod err;
pub mod put;
// pub mod query;
// pub mod reply;

use zenoh_buffers::{reader::Reader, writer::Writer, zbuf::ZBuf, zslice::ZSliceLen};
use zenoh_protocol::{
    common::{imsg, ZExtZBufHeader},
    core::{Encoding, EntityGlobalIdProto, EntityId, ZenohIdProto},
    zenoh::{ext, id, PushBody /*RequestBody, ResponseBody*/},
};
use zenoh_result::{bail, ZError, ZResult, ZE};

use crate::Zenoh080Bounded;
use crate::{LCodec, RCodec, WCodec, Zenoh080, Zenoh080Header, Zenoh080Length};

// Push
impl<W> WCodec<&PushBody, &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: &PushBody) -> Self::Output {
        match x {
            PushBody::Put(b) => self.write(&mut *writer, b),
            // PushBody::Del(b) => self.write(&mut *writer, b),
        }
    }
}

impl<R, const L: usize> RCodec<PushBody, &mut R> for (Zenoh080, ZSliceLen<L>)
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<PushBody> {
        let header: u8 = self.0.read(&mut *reader)?;

        let codec = Zenoh080Header::new(header);
        let body = match imsg::mid(codec.header) {
            id::PUT => PushBody::Put((codec, ZSliceLen::<L>).read(&mut *reader)?),
            // id::DEL => PushBody::Del(codec.read(&mut *reader)?),
            _ => bail!(ZE::DidntRead),
        };

        Ok(body)
    }
}

// Request
// impl<W> WCodec<&RequestBody, &mut W> for Zenoh080
// where
//     W: Writer,
// {
//     type Output = ZResult<()>;

//     fn write(self, writer: &mut W, x: &RequestBody) -> Self::Output {
//         match x {
//             RequestBody::Query(b) => self.write(&mut *writer, b),
//         }
//     }
// }

// impl<R> RCodec<RequestBody, &mut R> for Zenoh080
// where
//     R: Reader,
// {
//     type Error = ZError;

//     fn read(self, reader: &mut R) -> ZResult<RequestBody> {
//         let header: u8 = self.read(&mut *reader)?;

//         let codec = Zenoh080Header::new(header);
//         let body = match imsg::mid(codec.header) {
//             id::QUERY => RequestBody::Query(codec.read(&mut *reader)?),
//             _ => bail!(ZE::DidntRead),
//         };

//         Ok(body)
//     }
// }

// // Response
// impl<W> WCodec<&ResponseBody, &mut W> for Zenoh080
// where
//     W: Writer,
// {
//     type Output = ZResult<()>;

//     fn write(self, writer: &mut W, x: &ResponseBody) -> Self::Output {
//         match x {
//             ResponseBody::Reply(b) => self.write(&mut *writer, b),
//             ResponseBody::Err(b) => self.write(&mut *writer, b),
//         }
//     }
// }

// impl<R> RCodec<ResponseBody, &mut R> for Zenoh080
// where
//     R: Reader,
// {
//     type Error = ZError;

//     fn read(self, reader: &mut R) -> ZResult<ResponseBody> {
//         let header: u8 = self.read(&mut *reader)?;

//         let codec = Zenoh080Header::new(header);
//         let body = match imsg::mid(codec.header) {
//             id::REPLY => ResponseBody::Reply(codec.read(&mut *reader)?),
//             id::ERR => ResponseBody::Err(codec.read(&mut *reader)?),
//             _ => bail!(ZE::DidntRead),
//         };

//         Ok(body)
//     }
// }

// Extension: SourceInfo
impl<const ID: u8> LCodec<&ext::SourceInfoType<{ ID }>> for Zenoh080 {
    fn w_len(self, x: &ext::SourceInfoType<{ ID }>) -> usize {
        let ext::SourceInfoType { id, sn } = x;

        1 + self.w_len(&id.zid) + self.w_len(id.eid) + self.w_len(*sn)
    }
}

impl<W, const ID: u8> WCodec<(&ext::SourceInfoType<{ ID }>, bool), &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: (&ext::SourceInfoType<{ ID }>, bool)) -> Self::Output {
        let (x, more) = x;
        let ext::SourceInfoType { id, sn } = x;

        let header: ZExtZBufHeader<{ ID }> = ZExtZBufHeader::new(self.w_len(x));
        self.write(&mut *writer, (&header, more))?;

        let flags: u8 = (id.zid.size() as u8 - 1) << 4;
        self.write(&mut *writer, flags)?;

        let lodec = Zenoh080Length::new(id.zid.size());
        lodec.write(&mut *writer, &id.zid)?;

        self.write(&mut *writer, id.eid)?;
        self.write(&mut *writer, sn)?;
        Ok(())
    }
}

impl<R, const ID: u8> RCodec<(ext::SourceInfoType<{ ID }>, bool), &mut R> for Zenoh080Header
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<(ext::SourceInfoType<{ ID }>, bool)> {
        let (_, more): (ZExtZBufHeader<{ ID }>, bool) = self.read(&mut *reader)?;

        let flags: u8 = self.codec.read(&mut *reader)?;
        let length = 1 + ((flags >> 4) as usize);

        let lodec = Zenoh080Length::new(length);
        let zid: ZenohIdProto = lodec.read(&mut *reader)?;

        let eid: EntityId = self.codec.read(&mut *reader)?;
        let sn: u32 = self.codec.read(&mut *reader)?;

        Ok((
            ext::SourceInfoType {
                id: EntityGlobalIdProto { zid, eid },
                sn,
            },
            more,
        ))
    }
}

// Extension ValueType
impl<W, const VID: u8, const SID: u8> WCodec<(&ext::ValueType<{ VID }, { SID }>, bool), &mut W>
    for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: (&ext::ValueType<{ VID }, { SID }>, bool)) -> Self::Output {
        let (x, more) = x;
        let ext::ValueType { encoding, payload } = x;

        // Compute extension length
        let mut len = self.w_len(encoding);
        let codec = Zenoh080Bounded::<u32>::new();
        len += codec.w_len(payload);

        // Write ZExtBuf header
        let header: ZExtZBufHeader<{ VID }> = ZExtZBufHeader::new(len);
        self.write(&mut *writer, (&header, more))?;

        // Write encoding
        self.write(&mut *writer, encoding)?;

        // Write payload
        fn write<W>(writer: &mut W, payload: &ZBuf<1, 32>) -> ZResult<()>
        where
            W: Writer,
        {
            // Don't write the length since it is already included in the header
            for s in payload.zslices() {
                writer.write_zslice(s)?;
            }
            Ok(())
        }

        write(&mut *writer, payload)?;

        Ok(())
    }
}

impl<R, const VID: u8, const SID: u8> RCodec<(ext::ValueType<{ VID }, { SID }>, bool), &mut R>
    for Zenoh080Header
where
    R: Reader,
{
    type Error = ZError;

    fn read(
        #[allow(unused_mut)] mut self,
        reader: &mut R,
    ) -> ZResult<(ext::ValueType<{ VID }, { SID }>, bool)> {
        let (header, more): (ZExtZBufHeader<{ VID }>, bool) = self.read(&mut *reader)?;

        // Read encoding
        let start = reader.remaining();
        let encoding: Encoding = (self.codec, ZSliceLen::<32>).read(&mut *reader)?;
        let end = reader.remaining();

        // Read payload
        fn read<R>(reader: &mut R, len: usize) -> ZResult<ZBuf<1, 32>>
        where
            R: Reader,
        {
            let mut payload = ZBuf::empty();
            reader.read_zslices::<_, 32>(len, |s| {
                payload.push_zslice(s).expect("ZBuf push_zslice failed");
            })?;
            Ok(payload)
        }

        // Calculate how many bytes are left in the payload
        let len = header.len - (start - end);

        let payload: ZBuf<1, 32> = { read(&mut *reader, len)? };

        Ok((ext::ValueType { encoding, payload }, more))
    }
}

// Extension: Attachment
impl<W, const ID: u8> WCodec<(&ext::AttachmentType<{ ID }>, bool), &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: (&ext::AttachmentType<{ ID }>, bool)) -> Self::Output {
        let (x, more) = x;
        let ext::AttachmentType { buffer } = x;

        let header: ZExtZBufHeader<{ ID }> = ZExtZBufHeader::new(self.w_len(buffer));
        self.write(&mut *writer, (&header, more))?;
        for s in buffer.zslices() {
            writer.write_zslice(s)?;
        }

        Ok(())
    }
}

impl<R, const ID: u8> RCodec<(ext::AttachmentType<{ ID }>, bool), &mut R> for Zenoh080Header
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<(ext::AttachmentType<{ ID }>, bool)> {
        let (h, more): (ZExtZBufHeader<{ ID }>, bool) = self.read(&mut *reader)?;
        let mut buffer = ZBuf::empty();
        reader.read_zslices::<_, 32>(h.len, |s| {
            buffer.push_zslice(s).expect("ZBuf push_zslice failed");
        })?;

        Ok((ext::AttachmentType { buffer }, more))
    }
}
