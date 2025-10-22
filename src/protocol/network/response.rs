use crate::{
    protocol::{
        ZCodecError,
        codec::{decode_u8, decode_u32, encode_u8, encode_u32},
        common::{
            extension::{self, iext},
            imsg,
        },
        core::wire_expr::WireExpr,
        network::{Mapping, id, request::RequestId},
        zenoh::ResponseBody,
    },
    result::ZResult,
    zbail,
    zbuf::{ZBufReader, ZBufWriter},
};

pub(crate) mod flag {
    pub(crate) const N: u8 = 1 << 5;
    pub(crate) const M: u8 = 1 << 6;
    pub(crate) const Z: u8 = 1 << 7;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Response<'a> {
    pub(crate) rid: RequestId,
    pub(crate) wire_expr: WireExpr<'a>,
    pub(crate) payload: ResponseBody<'a>,
    pub(crate) ext_qos: ext::QoSType,
    pub(crate) ext_tstamp: Option<ext::TimestampType>,
    pub(crate) ext_respid: Option<ext::ResponderIdType>,
}

impl<'a> Response<'a> {
    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        let mut header = id::RESPONSE;
        let mut n_exts = ((self.ext_qos != ext::QoSType::DEFAULT) as u8)
            + (self.ext_tstamp.is_some() as u8)
            + (self.ext_respid.is_some() as u8);

        if n_exts != 0 {
            header |= flag::Z;
        }

        if self.wire_expr.mapping != Mapping::DEFAULT {
            header |= flag::M;
        }

        if self.wire_expr.has_suffix() {
            header |= flag::N;
        }

        encode_u8(writer, header)?;
        encode_u32(writer, self.rid)?;
        self.wire_expr.encode(writer)?;

        if self.ext_qos != ext::QoSType::DEFAULT {
            n_exts -= 1;
            self.ext_qos.encode(writer, n_exts != 0)?;
        }

        if let Some(ts) = self.ext_tstamp.as_ref() {
            n_exts -= 1;
            ts.encode(writer, n_exts != 0)?;
        }

        if let Some(ri) = self.ext_respid.as_ref() {
            n_exts -= 1;
            ri.encode(writer, n_exts != 0)?;
        }

        self.payload.encode(writer)
    }

    pub(crate) fn decode(reader: &mut ZBufReader<'a>, header: u8) -> ZResult<Self, ZCodecError> {
        if imsg::mid(header) != id::RESPONSE {
            zbail!(ZCodecError::CouldNotRead);
        }

        let rid = decode_u32(reader)?;
        let mut wire_expr: WireExpr<'_> =
            WireExpr::decode(imsg::has_flag(header, flag::N), reader)?;

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
            let ext = decode_u8(reader)?;
            match iext::eheader(ext) {
                ext::QoS::ID => {
                    let (q, ext) = ext::QoSType::decode(reader, ext)?;
                    ext_qos = q;
                    has_ext = ext;
                }
                ext::Timestamp::ID => {
                    let (t, ext) = ext::TimestampType::decode(reader, ext)?;
                    ext_tstamp = Some(t);
                    has_ext = ext;
                }
                ext::ResponderId::ID => {
                    let (t, ext) = ext::ResponderIdType::decode(reader, ext)?;
                    ext_respid = Some(t);
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip("Response", reader, ext)?;
                }
            }
        }

        let payload = ResponseBody::decode(reader)?;

        Ok(Response {
            rid,
            wire_expr,
            payload,
            ext_qos,
            ext_tstamp,
            ext_respid,
        })
    }

    #[cfg(test)]
    pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let rid: RequestId = rng.r#gen();
        let wire_expr = WireExpr::rand(zbuf);
        let payload = ResponseBody::rand(zbuf);
        let ext_qos = ext::QoSType::rand();
        let ext_tstamp = rng.gen_bool(0.5).then(ext::TimestampType::rand);
        let ext_respid = rng.gen_bool(0.5).then(ext::ResponderIdType::rand);

        Self {
            rid,
            wire_expr,
            payload,
            ext_qos,
            ext_tstamp,
            ext_respid,
        }
    }
}

pub(crate) mod ext {
    pub(crate) type QoS = crate::zextz64!(0x1, false);
    pub(crate) type QoSType = crate::protocol::network::ext::QoSType<{ QoS::ID }>;

    pub(crate) type Timestamp<'a> = crate::zextzbuf!('a, 0x2, false);
    pub(crate) type TimestampType = crate::protocol::network::ext::TimestampType<{ Timestamp::ID }>;

    pub(crate) type ResponderId<'a> = crate::zextzbuf!('a, 0x3, false);
    pub(crate) type ResponderIdType =
        crate::protocol::network::ext::EntityGlobalIdType<{ ResponderId::ID }>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResponseFinal {
    pub(crate) rid: RequestId,
    pub(crate) ext_qos: ext::QoSType,
    pub(crate) ext_tstamp: Option<ext::TimestampType>,
}

impl ResponseFinal {
    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        let mut header = id::RESPONSE_FINAL;
        let mut n_exts =
            ((self.ext_qos != ext::QoSType::DEFAULT) as u8) + (self.ext_tstamp.is_some() as u8);

        if n_exts != 0 {
            header |= flag::Z;
        }

        encode_u8(writer, header)?;
        encode_u32(writer, self.rid)?;

        if self.ext_qos != ext::QoSType::DEFAULT {
            n_exts -= 1;
            self.ext_qos.encode(writer, n_exts != 0)?;
        }

        if let Some(ts) = self.ext_tstamp.as_ref() {
            n_exts -= 1;
            ts.encode(writer, n_exts != 0)?;
        }

        Ok(())
    }

    pub(crate) fn decode(reader: &mut ZBufReader<'_>, header: u8) -> ZResult<Self, ZCodecError> {
        if imsg::mid(header) != id::RESPONSE_FINAL {
            zbail!(ZCodecError::CouldNotRead)
        }

        let rid: RequestId = decode_u32(reader)?;

        let mut ext_qos = ext::QoSType::DEFAULT;
        let mut ext_tstamp = None;

        let mut has_ext = imsg::has_flag(header, flag::Z);
        while has_ext {
            let ext: u8 = decode_u8(reader)?;
            match iext::eheader(ext) {
                ext::QoS::ID => {
                    let (q, ext) = ext::QoSType::decode(reader, ext)?;
                    ext_qos = q;
                    has_ext = ext;
                }
                ext::Timestamp::ID => {
                    let (t, ext) = ext::TimestampType::decode(reader, ext)?;
                    ext_tstamp = Some(t);
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip("ResponseFinal", reader, ext)?;
                }
            }
        }

        Ok(ResponseFinal {
            rid,
            ext_qos,
            ext_tstamp,
        })
    }

    #[cfg(test)]
    pub(crate) fn rand(_: &mut ZBufWriter<'_>) -> Self {
        use rand::Rng;

        let mut rng = rand::thread_rng();
        let rid: RequestId = rng.r#gen();
        let ext_qos = ext::QoSType::rand();
        let ext_tstamp = rng.gen_bool(0.5).then(ext::TimestampType::rand);

        Self {
            rid,
            ext_qos,
            ext_tstamp,
        }
    }
}
