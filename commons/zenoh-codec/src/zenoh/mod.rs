use zenoh_buffer::ZBuf;
use zenoh_protocol::{
    common::{extension::ZExtZBufHeader, imsg},
    core::{encoding::Encoding, EntityGlobalIdProto, EntityId, ZenohIdProto},
    zenoh::{ext, id, PushBody, RequestBody, ResponseBody},
};
use zenoh_result::{zbail, zctx, WithContext, ZE};

use crate::{LCodec, RCodec, WCodec, Zenoh080};

pub mod err;
pub mod put;
pub mod query;
pub mod reply;

impl<'a> WCodec<'a, &PushBody<'_>> for Zenoh080 {
    fn write(
        &self,
        message: &PushBody<'_>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        match message {
            PushBody::Put(put) => self.write(put, writer).ctx(zctx!()),
        }
    }
}

impl<'a> RCodec<'a, PushBody<'a>> for Zenoh080 {
    fn read(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
    ) -> zenoh_result::ZResult<PushBody<'a>> {
        let header = self.read(reader).ctx(zctx!())?;

        match imsg::mid(header) {
            id::PUT => Ok(PushBody::Put(
                self.read_knowing_header(reader, header).ctx(zctx!())?,
            )),
            _ => zbail!(ZE::ReadFailure),
        }
    }
}

impl<'a> WCodec<'a, &RequestBody<'_>> for Zenoh080 {
    fn write(
        &self,
        message: &RequestBody<'_>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        match message {
            RequestBody::Query(query) => self.write(query, writer).ctx(zctx!()),
        }
    }
}

impl<'a> RCodec<'a, RequestBody<'a>> for Zenoh080 {
    fn read(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
    ) -> zenoh_result::ZResult<RequestBody<'a>> {
        let header = self.read(reader).ctx(zctx!())?;

        match imsg::mid(header) {
            id::QUERY => Ok(RequestBody::Query(
                self.read_knowing_header(reader, header).ctx(zctx!())?,
            )),
            _ => zbail!(ZE::ReadFailure),
        }
    }
}

impl<'a> WCodec<'a, &ResponseBody<'_>> for Zenoh080 {
    fn write(
        &self,
        message: &ResponseBody<'_>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        match message {
            ResponseBody::Reply(reply) => self.write(reply, writer).ctx(zctx!()),
            ResponseBody::Err(err) => self.write(err, writer).ctx(zctx!()),
        }
    }
}

impl<'a> RCodec<'a, ResponseBody<'a>> for Zenoh080 {
    fn read(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
    ) -> zenoh_result::ZResult<ResponseBody<'a>> {
        let header = self.read(reader).ctx(zctx!())?;

        match imsg::mid(header) {
            id::REPLY => Ok(ResponseBody::Reply(
                self.read_knowing_header(reader, header).ctx(zctx!())?,
            )),
            id::ERR => Ok(ResponseBody::Err(
                self.read_knowing_header(reader, header).ctx(zctx!())?,
            )),
            _ => zbail!(ZE::ReadFailure),
        }
    }
}

impl<'a, const ID: u8> LCodec<'a, &ext::SourceInfoType<{ ID }>> for Zenoh080 {
    fn w_len(&self, message: &ext::SourceInfoType<{ ID }>) -> usize {
        let ext::SourceInfoType { id, sn } = message;

        1 + self.w_len(&id.zid) + self.w_len(id.eid) + self.w_len(*sn)
    }
}

impl<'a, const ID: u8> WCodec<'a, (&ext::SourceInfoType<{ ID }>, bool)> for Zenoh080 {
    fn write(
        &self,
        message: (&ext::SourceInfoType<{ ID }>, bool),
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let (x, more) = message;
        let ext::SourceInfoType { id, sn } = x;

        let header: ZExtZBufHeader<{ ID }> = ZExtZBufHeader::new(self.w_len(x));
        self.write((&header, more), writer).ctx(zctx!())?;

        let flags: u8 = (id.zid.size() as u8 - 1) << 4;
        self.write(flags, writer).ctx(zctx!())?;

        self.write_without_length(&id.zid, writer).ctx(zctx!())?;

        self.write(id.eid, writer).ctx(zctx!())?;
        self.write(*sn, writer).ctx(zctx!())?;

        Ok(())
    }
}

impl<'a, const ID: u8> RCodec<'a, (ext::SourceInfoType<{ ID }>, bool)> for Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
        header: u8,
    ) -> zenoh_result::ZResult<(ext::SourceInfoType<{ ID }>, bool)> {
        let (_, more): (ZExtZBufHeader<{ ID }>, bool) =
            self.read_knowing_header(reader, header).ctx(zctx!())?;

        let flags: u8 = self.read(reader).ctx(zctx!())?;
        let length = 1 + ((flags >> 4) as usize);

        let zid: ZenohIdProto = self.read_knowing_length(reader, length).ctx(zctx!())?;

        let eid: EntityId = self.read(reader).ctx(zctx!())?;
        let sn: u32 = self.read(reader).ctx(zctx!())?;

        Ok((
            ext::SourceInfoType {
                id: EntityGlobalIdProto { zid, eid },
                sn,
            },
            more,
        ))
    }
}

impl<'a, const VID: u8, const SID: u8> WCodec<'a, (&ext::ValueType<'_, { VID }, { SID }>, bool)>
    for Zenoh080
{
    fn write(
        &self,
        message: (&ext::ValueType<'_, { VID }, { SID }>, bool),
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let (x, more) = message;
        let ext::ValueType { encoding, payload } = x;

        let mut len = self.w_len(encoding);
        len += self.w_len(payload);

        let header: ZExtZBufHeader<{ VID }> = ZExtZBufHeader::new(len);
        self.write((&header, more), writer).ctx(zctx!())?;

        self.write(encoding, writer).ctx(zctx!())?;

        self.write_without_length(payload, writer)
            .ctx(zctx!())
            .ctx(zctx!())?;

        Ok(())
    }
}

impl<'a, const VID: u8, const SID: u8> RCodec<'a, (ext::ValueType<'a, { VID }, { SID }>, bool)>
    for Zenoh080
{
    fn read_knowing_header(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
        header: u8,
    ) -> zenoh_result::ZResult<(ext::ValueType<'a, { VID }, { SID }>, bool)> {
        let (h, more): (ZExtZBufHeader<{ VID }>, bool) =
            self.read_knowing_header(reader, header).ctx(zctx!())?;

        let start = reader.remaining();
        let encoding: Encoding = self.read(reader).ctx(zctx!())?;
        let end = reader.remaining();

        let len = h.len - (start - end);
        let payload: ZBuf<'a> = self.read_knowing_length(reader, len).ctx(zctx!())?;

        Ok((ext::ValueType { encoding, payload }, more))
    }
}

impl<'a, const ID: u8> WCodec<'a, (&ext::AttachmentType<'_, { ID }>, bool)> for Zenoh080 {
    fn write(
        &self,
        message: (&ext::AttachmentType<'_, { ID }>, bool),
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let (x, more) = message;
        let ext::AttachmentType { buffer } = x;

        let header: ZExtZBufHeader<{ ID }> = ZExtZBufHeader::new(self.w_len(buffer));
        self.write((&header, more), writer).ctx(zctx!())?;
        self.write_without_length(buffer, writer).ctx(zctx!())?;

        Ok(())
    }
}

impl<'a, const ID: u8> RCodec<'a, (ext::AttachmentType<'a, { ID }>, bool)> for Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
        header: u8,
    ) -> zenoh_result::ZResult<(ext::AttachmentType<'a, { ID }>, bool)> {
        let (h, more): (ZExtZBufHeader<{ ID }>, bool) =
            self.read_knowing_header(reader, header).ctx(zctx!())?;

        let buffer: ZBuf<'a> = self.read_knowing_length(reader, h.len).ctx(zctx!())?;

        Ok((ext::AttachmentType { buffer }, more))
    }
}
