use zenoh_buffer::ZBufReader;
use zenoh_protocol::{
    common::{extension::iext, imsg},
    core::Reliability,
    network::NetworkMessage,
    transport::{
        frame::{ext, flag, Frame, FrameHeader},
        id, TransportSn,
    },
};
use zenoh_result::{zbail, zctx, WithContext, ZResult, ZE};

use crate::{common::extension, RCodec, WCodec, ZCodec};

impl<'a> WCodec<'a, &FrameHeader> for ZCodec {
    fn write(&self, message: &FrameHeader, writer: &mut crate::ZBufWriter<'a>) -> ZResult<()> {
        let FrameHeader {
            reliability,
            sn,
            ext_qos,
        } = message;

        let mut header = id::FRAME;

        if let Reliability::Reliable = reliability {
            header |= flag::R;
        }

        if ext_qos != &ext::QoSType::DEFAULT {
            header |= flag::Z;
        }

        self.write(header, writer).ctx(zctx!())?;
        self.write(*sn, writer).ctx(zctx!())?;

        if ext_qos != &ext::QoSType::DEFAULT {
            self.write((*ext_qos, false), writer).ctx(zctx!())?;
        }

        Ok(())
    }
}

impl<'a> RCodec<'a, FrameHeader> for ZCodec {
    fn read_knowing_header(
        &self,
        reader: &mut crate::ZBufReader<'a>,
        header: u8,
    ) -> ZResult<FrameHeader> {
        if imsg::mid(header) != id::FRAME {
            zbail!(ZE::ReadFailure);
        }

        let reliability = match imsg::has_flag(header, flag::R) {
            true => Reliability::Reliable,
            false => Reliability::BestEffort,
        };
        let sn: TransportSn = self.read(reader).ctx(zctx!())?;

        let mut ext_qos = ext::QoSType::DEFAULT;

        let mut has_ext = imsg::has_flag(header, flag::Z);
        while has_ext {
            let ext: u8 = self.read(reader).ctx(zctx!())?;
            match iext::eid(ext) {
                ext::QoS::ID => {
                    let (q, ext): (ext::QoSType, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_qos = q;
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip(reader, "Frame", ext)?;
                }
            }
        }

        Ok(FrameHeader {
            reliability,
            sn,
            ext_qos,
        })
    }

    fn read(&self, reader: &mut crate::ZBufReader<'a>) -> ZResult<FrameHeader> {
        let header: u8 = self.read(reader).ctx(zctx!())?;
        self.read_knowing_header(reader, header).ctx(zctx!())
    }
}

impl<'a> WCodec<'a, &Frame<'_>> for ZCodec {
    fn write(&self, message: &Frame<'_>, writer: &mut zenoh_buffer::ZBufWriter<'a>) -> ZResult<()> {
        let Frame {
            reliability,
            sn,
            payload,
            ext_qos,
        } = message;

        let header = FrameHeader {
            reliability: *reliability,
            sn: *sn,
            ext_qos: *ext_qos,
        };

        self.write(&header, writer).ctx(zctx!())?;
        self.write(payload, writer).ctx(zctx!())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct FrameReader<'a> {
    pub reliability: Reliability,
    pub sn: TransportSn,
    pub ext_qos: ext::QoSType,
    pub reader: ZBufReader<'a>,
    pub codec: ZCodec,
}

impl<'a> Iterator for FrameReader<'a> {
    type Item = NetworkMessage<'a>;

    fn next(&mut self) -> Option<NetworkMessage<'a>> {
        let mark = self.reader.mark();
        let msg: ZResult<NetworkMessage<'a>> = self
            .codec
            .read_with_reliability(&mut self.reader, self.reliability)
            .ctx(zctx!());

        match msg {
            Ok(m) => Some(m),
            Err(_) => {
                self.reader.rewind(mark).unwrap();
                None
            }
        }
    }
}

impl<'a> Drop for FrameReader<'a> {
    fn drop(&mut self) {
        for _ in self {}
    }
}
