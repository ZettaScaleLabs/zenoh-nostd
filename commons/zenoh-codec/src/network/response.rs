use zenoh_protocol::{
    common::{extension::iext, imsg},
    core::wire_expr::WireExpr,
    network::{
        id,
        request::RequestId,
        response::{ext, flag, Response, ResponseFinal},
        Mapping,
    },
    zenoh::ResponseBody,
};
use zenoh_result::{zbail, zctx, WithContext, ZE};

use crate::{common::extension, RCodec, WCodec, Zenoh080};

impl<'a, const MAX_EXT_UNKNOWN: usize> WCodec<'a, &Response<'_, MAX_EXT_UNKNOWN>> for Zenoh080 {
    fn write(
        &self,
        message: &Response<'_, MAX_EXT_UNKNOWN>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let Response {
            rid,
            wire_expr,
            payload,
            ext_qos,
            ext_tstamp,
            ext_respid,
        } = message;

        let mut header = id::RESPONSE;
        let mut n_exts = ((ext_qos != &ext::QoSType::DEFAULT) as u8)
            + (ext_tstamp.is_some() as u8)
            + (ext_respid.is_some() as u8);

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
        self.write(*rid, writer).ctx(zctx!())?;
        self.write(wire_expr, writer).ctx(zctx!())?;

        if ext_qos != &ext::QoSType::DEFAULT {
            n_exts -= 1;
            self.write((*ext_qos, n_exts != 0), writer).ctx(zctx!())?;
        }

        if let Some(ts) = ext_tstamp.as_ref() {
            n_exts -= 1;
            self.write((ts, n_exts != 0), writer).ctx(zctx!())?;
        }

        if let Some(ri) = ext_respid.as_ref() {
            n_exts -= 1;
            self.write((ri, n_exts != 0), writer).ctx(zctx!())?;
        }

        self.write(payload, writer).ctx(zctx!())?;

        Ok(())
    }
}

impl<'a, const MAX_EXT_UNKNOWN: usize> RCodec<'a, Response<'a, MAX_EXT_UNKNOWN>> for Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
        header: u8,
    ) -> zenoh_result::ZResult<Response<'a, MAX_EXT_UNKNOWN>> {
        if imsg::mid(header) != id::RESPONSE {
            zbail!(ZE::ReadFailure);
        }

        let rid: RequestId = self.read(reader).ctx(zctx!())?;
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
        let mut ext_respid = None;

        let mut has_ext = imsg::has_flag(header, flag::Z);
        while has_ext {
            let ext: u8 = self.read(reader)?;
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
                ext::ResponderId::ID => {
                    let (t, ext): (ext::ResponderIdType, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_respid = Some(t);
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip(reader, "Response", ext)?;
                }
            }
        }

        let payload: ResponseBody<'_, MAX_EXT_UNKNOWN> = self.read(reader)?;

        Ok(Response {
            rid,
            wire_expr,
            payload,
            ext_qos,
            ext_tstamp,
            ext_respid,
        })
    }

    fn read(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
    ) -> zenoh_result::ZResult<Response<'a, MAX_EXT_UNKNOWN>> {
        let header: u8 = self.read(reader).ctx(zctx!())?;
        self.read_knowing_header(reader, header).ctx(zctx!())
    }
}

impl<'a> WCodec<'a, &ResponseFinal> for Zenoh080 {
    fn write(
        &self,
        message: &ResponseFinal,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let ResponseFinal {
            rid,
            ext_qos,
            ext_tstamp,
        } = message;

        let mut header = id::RESPONSE_FINAL;
        let mut n_exts = ((ext_qos != &ext::QoSType::DEFAULT) as u8) + (ext_tstamp.is_some() as u8);

        if n_exts != 0 {
            header |= flag::Z;
        }

        self.write(header, writer).ctx(zctx!())?;
        self.write(*rid, writer).ctx(zctx!())?;

        if ext_qos != &ext::QoSType::DEFAULT {
            n_exts -= 1;
            self.write((*ext_qos, n_exts != 0), writer)?;
        }

        if let Some(ts) = ext_tstamp.as_ref() {
            n_exts -= 1;
            self.write((ts, n_exts != 0), writer)?;
        }

        Ok(())
    }
}

impl<'a> RCodec<'a, ResponseFinal> for Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
        header: u8,
    ) -> zenoh_result::ZResult<ResponseFinal> {
        if imsg::mid(header) != id::RESPONSE_FINAL {
            zbail!(ZE::ReadFailure);
        }

        let rid: RequestId = self.read(reader).ctx(zctx!())?;

        let mut ext_qos = ext::QoSType::DEFAULT;
        let mut ext_tstamp = None;

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
                _ => {
                    has_ext = extension::skip(reader, "ResponseFinal", ext)?;
                }
            }
        }

        Ok(ResponseFinal {
            rid,
            ext_qos,
            ext_tstamp,
        })
    }

    fn read(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
    ) -> zenoh_result::ZResult<ResponseFinal> {
        let header: u8 = self.read(reader)?;
        self.read_knowing_header(reader, header)
    }
}
