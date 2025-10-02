use zenoh_protocol::{
    common::{extension::iext, imsg},
    core::wire_expr::WireExpr,
    network::{
        id,
        push::{ext, flag, Push},
        Mapping,
    },
    zenoh::PushBody,
};
use zenoh_result::{zbail, zctx, WithContext, ZE};

use crate::{common::extension, RCodec, WCodec, Zenoh080};

impl<'a> WCodec<'a, &Push<'_>> for Zenoh080 {
    fn write(
        &self,
        message: &Push<'_>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let Push {
            wire_expr,
            ext_qos,
            ext_tstamp,
            ext_nodeid,
            payload,
        } = message;

        let mut header = id::PUSH;
        let mut n_exts = ((ext_qos != &ext::QoSType::DEFAULT) as u8)
            + (ext_tstamp.is_some() as u8)
            + ((ext_nodeid != &ext::NodeIdType::DEFAULT) as u8);

        if n_exts != 0 {
            header |= flag::Z;
        }

        if wire_expr.mapping != Mapping::DEFAULT {
            header |= flag::M;
        }

        if wire_expr.has_suffix() {
            header |= flag::N;
        }

        self.write(header, writer).ctx(zctx!())?;
        self.write(wire_expr, writer).ctx(zctx!())?;

        if ext_qos != &ext::QoSType::DEFAULT {
            n_exts -= 1;
            self.write((*ext_qos, n_exts != 0), writer).ctx(zctx!())?;
        }

        if let Some(ts) = ext_tstamp.as_ref() {
            n_exts -= 1;
            self.write((ts, n_exts != 0), writer).ctx(zctx!())?;
        }

        if ext_nodeid != &ext::NodeIdType::DEFAULT {
            n_exts -= 1;
            self.write((*ext_nodeid, n_exts != 0), writer)
                .ctx(zctx!())?;
        }

        self.write(payload, writer).ctx(zctx!())?;

        Ok(())
    }
}

impl<'a> RCodec<'a, Push<'a>> for Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
        header: u8,
    ) -> zenoh_result::ZResult<Push<'a>> {
        if imsg::mid(header) != id::PUSH {
            zbail!(ZE::ReadFailure);
        }

        let mut wire_expr: WireExpr<'_> = self
            .read_with_condition(reader, imsg::has_flag(header, flag::N))
            .ctx(zctx!())?;

        wire_expr.mapping = if imsg::has_flag(header, flag::M) {
            Mapping::Sender
        } else {
            Mapping::Receiver
        };

        let mut ext_qos = ext::QoSType::DEFAULT;
        let mut ext_tstamp = None;
        let mut ext_nodeid = ext::NodeIdType::DEFAULT;

        let mut has_ext = imsg::has_flag(header, flag::Z);
        while has_ext {
            let ext: u8 = self.read(reader).ctx(zctx!())?;
            match iext::eid(ext) {
                ext::QoS::ID => {
                    let (q, ext): (ext::QoSType, bool) =
                        self.read_knowing_header(reader, header).ctx(zctx!())?;
                    ext_qos = q;
                    has_ext = ext;
                }
                ext::Timestamp::ID => {
                    let (t, ext): (ext::TimestampType, bool) =
                        self.read_knowing_header(reader, header).ctx(zctx!())?;
                    ext_tstamp = Some(t);
                    has_ext = ext;
                }
                ext::NodeId::ID => {
                    let (nid, ext): (ext::NodeIdType, bool) =
                        self.read_knowing_header(reader, header).ctx(zctx!())?;
                    ext_nodeid = nid;
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip(reader, "Push", ext)?;
                }
            }
        }

        let payload: PushBody<'_> = self.read(reader).ctx(zctx!())?;

        Ok(Push {
            wire_expr,
            payload,
            ext_qos,
            ext_tstamp,
            ext_nodeid,
        })
    }

    fn read(&self, reader: &mut zenoh_buffer::ZBufReader<'a>) -> zenoh_result::ZResult<Push<'a>> {
        let header: u8 = self.read(reader).ctx(zctx!())?;
        self.read_knowing_header(reader, header).ctx(zctx!())
    }
}
