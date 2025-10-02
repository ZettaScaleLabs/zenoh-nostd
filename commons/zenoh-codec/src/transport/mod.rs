use zenoh_buffer::ZBufReader;
use zenoh_protocol::{
    common::{extension::ZExtZ64, imsg},
    transport::{
        ext,
        frame::FrameHeader,
        id,
        init::{InitAck, InitSyn},
        keepalive::KeepAlive,
        open::{OpenAck, OpenSyn},
        TransportBody, TransportMessage,
    },
};
use zenoh_result::{zctx, WithContext, ZResult};

use crate::{transport::frame::FrameReader, RCodec, WCodec, Zenoh080};

pub mod frame;
pub mod init;
pub mod keepalive;
pub mod open;

impl<'a> WCodec<'a, &TransportMessage<'_>> for Zenoh080 {
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

#[derive(Debug, PartialEq, Eq)]
pub enum TransportMessageReader<'a> {
    InitSyn(InitSyn<'a>),
    InitAck(InitAck<'a>),
    OpenSyn(OpenSyn<'a>),
    OpenAck(OpenAck<'a>),
    KeepAlive(KeepAlive),
    Frame(FrameReader<'a>),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TransportMessagesReader<'a> {
    reader: ZBufReader<'a>,
    codec: Zenoh080,
}

impl<'a> RCodec<'a, TransportMessagesReader<'a>> for Zenoh080 {
    fn read(
        &self,
        reader: &mut ZBufReader<'a>,
    ) -> zenoh_result::ZResult<TransportMessagesReader<'a>> {
        Ok(TransportMessagesReader {
            reader: reader.clone(),
            codec: self.clone(),
        })
    }
}

impl<'a> Iterator for TransportMessagesReader<'a> {
    type Item = TransportMessageReader<'a>;

    fn next(&mut self) -> Option<TransportMessageReader<'a>> {
        if !self.reader.can_read() {
            return None;
        }

        let header: u8 = self.codec.read(&mut self.reader).ok()?;
        match imsg::mid(header) {
            id::FRAME => {
                let header: FrameHeader = self
                    .codec
                    .read_knowing_header(&mut self.reader, header)
                    .ok()?;

                let FrameHeader {
                    reliability,
                    sn,
                    ext_qos,
                } = header;

                Some(TransportMessageReader::Frame(FrameReader {
                    reliability,
                    sn,
                    ext_qos,
                    reader: self.reader.clone(),
                    codec: self.codec.clone(),
                }))
            }
            id::KEEP_ALIVE => Some(TransportMessageReader::KeepAlive(
                self.codec
                    .read_knowing_header(&mut self.reader, header)
                    .ok()?,
            )),
            id::INIT => {
                if !imsg::has_flag(header, zenoh_protocol::transport::init::flag::A) {
                    Some(TransportMessageReader::InitSyn(
                        self.codec
                            .read_knowing_header(&mut self.reader, header)
                            .ok()?,
                    ))
                } else {
                    Some(TransportMessageReader::InitAck(
                        self.codec
                            .read_knowing_header(&mut self.reader, header)
                            .ok()?,
                    ))
                }
            }
            id::OPEN => {
                if !imsg::has_flag(header, zenoh_protocol::transport::open::flag::A) {
                    Some(TransportMessageReader::OpenSyn(
                        self.codec
                            .read_knowing_header(&mut self.reader, header)
                            .ok()?,
                    ))
                } else {
                    Some(TransportMessageReader::OpenAck(
                        self.codec
                            .read_knowing_header(&mut self.reader, header)
                            .ok()?,
                    ))
                }
            }
            _ => None,
        }
    }
}

impl<'a, const ID: u8> WCodec<'a, (ext::QoSType<{ ID }>, bool)> for Zenoh080 {
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

impl<'a, const ID: u8> RCodec<'a, (ext::QoSType<{ ID }>, bool)> for Zenoh080 {
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

impl<'a, const ID: u8> WCodec<'a, (ext::PatchType<{ ID }>, bool)> for Zenoh080 {
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

impl<'a, const ID: u8> RCodec<'a, (ext::PatchType<{ ID }>, bool)> for Zenoh080 {
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
