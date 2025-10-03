use zenoh_protocol::{
    common::extension::ZExtZ64,
    transport::{ext, TransportBody, TransportMessage},
};
use zenoh_result::{zctx, WithContext, ZResult};

use crate::{RCodec, WCodec, ZCodec};

pub mod batch;
pub mod frame;
pub mod init;
pub mod keepalive;
pub mod open;

impl<'a> WCodec<'a, &TransportMessage<'_>> for ZCodec {
    fn write(
        &self,
        message: &TransportMessage<'_>,
        writer: &mut crate::ZBufWriter<'a>,
    ) -> ZResult<()> {
        match &message.body {
            TransportBody::InitSyn(b) => self.write(b, writer).ctx(zctx!()),
            TransportBody::InitAck(b) => self.write(b, writer).ctx(zctx!()),
            TransportBody::OpenSyn(b) => self.write(b, writer).ctx(zctx!()),
            TransportBody::OpenAck(b) => self.write(b, writer).ctx(zctx!()),
            TransportBody::KeepAlive(b) => self.write(*b, writer).ctx(zctx!()),
            TransportBody::Frame(b) => self.write(b, writer).ctx(zctx!()),
        }
    }
}

impl<'a, const ID: u8> WCodec<'a, (ext::QoSType<{ ID }>, bool)> for ZCodec {
    fn write(
        &self,
        message: (ext::QoSType<{ ID }>, bool),
        writer: &mut crate::ZBufWriter<'a>,
    ) -> ZResult<()> {
        let (x, more) = message;
        let ext: ZExtZ64<{ ID }> = x.into();

        self.write((&ext, more), writer).ctx(zctx!())
    }
}

impl<'a, const ID: u8> RCodec<'a, (ext::QoSType<{ ID }>, bool)> for ZCodec {
    fn read_knowing_header(
        &self,
        reader: &mut crate::ZBufReader<'a>,
        header: u8,
    ) -> ZResult<(ext::QoSType<{ ID }>, bool)> {
        let (ext, more): (ZExtZ64<{ ID }>, bool) =
            self.read_knowing_header(reader, header).ctx(zctx!())?;
        Ok((ext.into(), more))
    }

    fn read(&self, reader: &mut crate::ZBufReader<'a>) -> ZResult<(ext::QoSType<{ ID }>, bool)> {
        let header: u8 = self.read(reader).ctx(zctx!())?;
        self.read_knowing_header(reader, header).ctx(zctx!())
    }
}

impl<'a, const ID: u8> WCodec<'a, (ext::PatchType<{ ID }>, bool)> for ZCodec {
    fn write(
        &self,
        message: (ext::PatchType<{ ID }>, bool),
        writer: &mut crate::ZBufWriter<'a>,
    ) -> ZResult<()> {
        let (x, more) = message;
        let ext: ZExtZ64<{ ID }> = x.into();

        self.write((&ext, more), writer).ctx(zctx!())
    }
}

impl<'a, const ID: u8> RCodec<'a, (ext::PatchType<{ ID }>, bool)> for ZCodec {
    fn read_knowing_header(
        &self,
        reader: &mut crate::ZBufReader<'a>,
        header: u8,
    ) -> ZResult<(ext::PatchType<{ ID }>, bool)> {
        let (ext, more): (ZExtZ64<{ ID }>, bool) =
            self.read_knowing_header(reader, header).ctx(zctx!())?;
        Ok((ext.into(), more))
    }

    fn read(&self, reader: &mut crate::ZBufReader<'a>) -> ZResult<(ext::PatchType<{ ID }>, bool)> {
        let header: u8 = self.read(reader).ctx(zctx!())?;
        self.read_knowing_header(reader, header).ctx(zctx!())
    }
}
