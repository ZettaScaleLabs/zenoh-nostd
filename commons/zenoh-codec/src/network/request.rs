use zenoh_protocol::{
    common::{extension::iext, imsg},
    core::wire_expr::WireExpr,
    network::{
        id,
        request::{
            ext::{self},
            flag, Request, RequestId,
        },
        Mapping,
    },
    zenoh::RequestBody,
};
use zenoh_result::{zbail, zctx, WithContext, ZE};

use crate::{common::extension, RCodec, WCodec, ZCodec};

impl<'a> WCodec<'a, (&ext::QueryTarget, bool)> for ZCodec {
    fn write(
        &self,
        message: (&ext::QueryTarget, bool),
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let (x, more) = message;

        let v = match x {
            ext::QueryTarget::BestMatching => 0,
            ext::QueryTarget::All => 1,
            ext::QueryTarget::AllComplete => 2,
        };

        let ext = ext::Target::new(v);

        self.write((&ext, more), writer).ctx(zctx!())
    }
}

impl<'a> RCodec<'a, (ext::QueryTarget, bool)> for ZCodec {
    fn read_knowing_header(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
        header: u8,
    ) -> zenoh_result::ZResult<(ext::QueryTarget, bool)> {
        let (ext, more): (ext::Target, bool) =
            self.read_knowing_header(reader, header).ctx(zctx!())?;

        let rt = match ext.value {
            0 => ext::QueryTarget::BestMatching,
            1 => ext::QueryTarget::All,
            2 => ext::QueryTarget::AllComplete,
            _ => zbail!(ZE::ReadFailure),
        };

        Ok((rt, more))
    }
}

impl<'a> WCodec<'a, &Request<'_>> for ZCodec {
    fn write(
        &self,
        message: &Request<'_>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let Request {
            id,
            wire_expr,
            ext_qos,
            ext_tstamp,
            ext_nodeid,
            ext_target,
            ext_budget,
            ext_timeout,
            payload,
        } = message;

        let mut header = id::REQUEST;
        let mut n_exts = ((ext_qos != &ext::QoSType::DEFAULT) as u8)
            + (ext_tstamp.is_some() as u8)
            + ((ext_target != &ext::QueryTarget::DEFAULT) as u8)
            + (ext_budget.is_some() as u8)
            + (ext_timeout.is_some() as u8)
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
        self.write(*id, writer).ctx(zctx!())?;
        self.write(wire_expr, writer).ctx(zctx!())?;

        if ext_qos != &ext::QoSType::DEFAULT {
            n_exts -= 1;
            self.write((*ext_qos, n_exts != 0), writer).ctx(zctx!())?;
        }

        if let Some(ts) = ext_tstamp.as_ref() {
            n_exts -= 1;
            self.write((ts, n_exts != 0), writer).ctx(zctx!())?;
        }

        if ext_target != &ext::QueryTarget::DEFAULT {
            n_exts -= 1;
            self.write((ext_target, n_exts != 0), writer).ctx(zctx!())?;
        }

        if let Some(l) = ext_budget.as_ref() {
            n_exts -= 1;
            let e = ext::Budget::new(l.get() as u64);
            self.write((&e, n_exts != 0), writer).ctx(zctx!())?;
        }

        if let Some(to) = ext_timeout.as_ref() {
            n_exts -= 1;
            let e = ext::Timeout::new(to.as_millis() as u64);
            self.write((&e, n_exts != 0), writer).ctx(zctx!())?;
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

impl<'a> RCodec<'a, Request<'a>> for ZCodec {
    fn read_knowing_header(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
        header: u8,
    ) -> zenoh_result::ZResult<Request<'a>> {
        if imsg::mid(header) != id::REQUEST {
            zbail!(ZE::ReadFailure);
        }

        let id: RequestId = self.read(reader).ctx(zctx!())?;
        let mut wire_expr: WireExpr<'_> = self
            .read_with_condition(reader, imsg::has_flag(header, flag::N))
            .ctx(zctx!())?;

        wire_expr.mapping = if imsg::has_flag(header, flag::M) {
            Mapping::Sender
        } else {
            Mapping::Receiver
        };

        // Extensions
        let mut ext_qos = ext::QoSType::DEFAULT;
        let mut ext_tstamp = None;
        let mut ext_nodeid = ext::NodeIdType::DEFAULT;
        let mut ext_target = ext::QueryTarget::DEFAULT;
        let mut ext_limit = None;
        let mut ext_timeout = None;

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
                ext::Timestamp::ID => {
                    let (t, ext): (ext::TimestampType, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_tstamp = Some(t);
                    has_ext = ext;
                }
                ext::NodeId::ID => {
                    let (nid, ext): (ext::NodeIdType, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_nodeid = nid;
                    has_ext = ext;
                }
                ext::Target::ID => {
                    let (rt, ext): (ext::QueryTarget, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_target = rt;
                    has_ext = ext;
                }
                ext::Budget::ID => {
                    let (l, ext): (ext::Budget, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_limit = ext::BudgetType::new(l.value as u32);
                    has_ext = ext;
                }
                ext::Timeout::ID => {
                    let (to, ext): (ext::Timeout, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_timeout = Some(ext::TimeoutType::from_millis(to.value));
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip(reader, "Request", ext).ctx(zctx!())?;
                }
            }
        }

        let payload: RequestBody<'_> = self.read(reader).ctx(zctx!())?;

        Ok(Request {
            id,
            wire_expr,
            payload,
            ext_qos,
            ext_tstamp,
            ext_nodeid,
            ext_target,
            ext_budget: ext_limit,
            ext_timeout,
        })
    }

    fn read(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
    ) -> zenoh_result::ZResult<Request<'a>> {
        let header = self.read(reader).ctx(zctx!())?;
        self.read_knowing_header(reader, header).ctx(zctx!())
    }
}
