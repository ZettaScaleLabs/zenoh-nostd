use crate::{
    protocol::{
        ZCodecError,
        common::{
            extension::{self, iext},
            imsg,
        },
        core::wire_expr::WireExpr,
        network::{Mapping, id},
        zcodec::{decode_u8, decode_u32, encode_u8, encode_u32},
        zenoh::RequestBody,
    },
    result::ZResult,
    zbail,
    zbuf::{ZBufReader, ZBufWriter},
};

pub(crate) type RequestId = u32;

pub(crate) mod flag {
    pub(crate) const N: u8 = 1 << 5;
    pub(crate) const M: u8 = 1 << 6;
    pub(crate) const Z: u8 = 1 << 7;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Request<'a> {
    pub(crate) id: RequestId,
    pub(crate) wire_expr: WireExpr<'a>,
    pub(crate) ext_qos: ext::QoSType,
    pub(crate) ext_tstamp: Option<ext::TimestampType>,
    pub(crate) ext_nodeid: ext::NodeIdType,
    pub(crate) ext_target: ext::QueryTarget,
    pub(crate) ext_budget: Option<ext::BudgetType>,
    pub(crate) ext_timeout: Option<ext::TimeoutType>,
    pub(crate) payload: RequestBody<'a>,
}

impl<'a> Request<'a> {
    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        let mut header = id::REQUEST;
        let mut n_exts = ((self.ext_qos != ext::QoSType::DEFAULT) as u8)
            + (self.ext_tstamp.is_some() as u8)
            + ((self.ext_target != ext::QueryTarget::DEFAULT) as u8)
            + (self.ext_budget.is_some() as u8)
            + (self.ext_timeout.is_some() as u8)
            + ((self.ext_nodeid != ext::NodeIdType::DEFAULT) as u8);

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
        encode_u32(writer, self.id)?;
        self.wire_expr.encode(writer)?;

        if self.ext_qos != ext::QoSType::DEFAULT {
            n_exts -= 1;
            self.ext_qos.encode(n_exts != 0, writer)?;
        }

        if let Some(ts) = self.ext_tstamp.as_ref() {
            n_exts -= 1;
            ts.encode(n_exts != 0, writer)?;
        }

        if self.ext_target != ext::QueryTarget::DEFAULT {
            n_exts -= 1;
            self.ext_target.encode(n_exts != 0, writer)?;
        }

        if let Some(l) = self.ext_budget.as_ref() {
            n_exts -= 1;
            let e = ext::Budget::new(l.get() as u64);
            e.encode(n_exts != 0, writer)?;
        }

        if let Some(to) = self.ext_timeout.as_ref() {
            n_exts -= 1;
            let e = ext::Timeout::new(to.as_millis() as u64);
            e.encode(n_exts != 0, writer)?;
        }

        if self.ext_nodeid != ext::NodeIdType::DEFAULT {
            n_exts -= 1;
            self.ext_nodeid.encode(n_exts != 0, writer)?;
        }

        self.payload.encode(writer)
    }

    pub(crate) fn decode(header: u8, reader: &mut ZBufReader<'a>) -> ZResult<Self, ZCodecError> {
        if imsg::mid(header) != id::REQUEST {
            zbail!(ZCodecError::CouldNotRead);
        }

        let id = decode_u32(reader)?;
        let mut wire_expr: WireExpr<'_> =
            WireExpr::decode(imsg::has_flag(header, flag::N), reader)?;

        wire_expr.mapping = if imsg::has_flag(header, flag::M) {
            Mapping::Sender
        } else {
            Mapping::Receiver
        };

        let mut ext_qos = ext::QoSType::DEFAULT;
        let mut ext_tstamp = None;
        let mut ext_nodeid = ext::NodeIdType::DEFAULT;
        let mut ext_target = ext::QueryTarget::DEFAULT;
        let mut ext_limit = None;
        let mut ext_timeout = None;

        let mut has_ext = imsg::has_flag(header, flag::Z);
        while has_ext {
            let ext = decode_u8(reader)?;
            match iext::eheader(ext) {
                ext::QoS::ID => {
                    let (q, ext) = ext::QoSType::decode(ext, reader)?;
                    ext_qos = q;
                    has_ext = ext;
                }
                ext::Timestamp::ID => {
                    let (t, ext) = ext::TimestampType::decode(ext, reader)?;
                    ext_tstamp = Some(t);
                    has_ext = ext;
                }
                ext::NodeId::ID => {
                    let (nid, ext) = ext::NodeIdType::decode(ext, reader)?;
                    ext_nodeid = nid;
                    has_ext = ext;
                }
                ext::Target::ID => {
                    let (rt, ext) = ext::QueryTarget::decode(ext, reader)?;
                    ext_target = rt;
                    has_ext = ext;
                }
                ext::Budget::ID => {
                    let (l, ext) = ext::Budget::decode(ext, reader)?;
                    ext_limit = ext::BudgetType::new(l.value as u32);
                    has_ext = ext;
                }
                ext::Timeout::ID => {
                    let (to, ext) = ext::Timeout::decode(ext, reader)?;
                    ext_timeout = Some(ext::TimeoutType::from_millis(to.value));
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip("Request", ext, reader)?;
                }
            }
        }

        let payload = RequestBody::decode(reader)?;

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

    #[cfg(test)]
    pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        use core::num::NonZeroU32;

        use rand::Rng;

        let mut rng = rand::thread_rng();
        let wire_expr = WireExpr::rand(zbuf);
        let id: RequestId = rng.r#gen();
        let payload = RequestBody::rand(zbuf);
        let ext_qos = ext::QoSType::rand();
        let ext_tstamp = rng.gen_bool(0.5).then(ext::TimestampType::rand);
        let ext_nodeid = ext::NodeIdType::rand();
        let ext_target = ext::QueryTarget::rand();
        let ext_budget = if rng.gen_bool(0.5) {
            NonZeroU32::new(rng.r#gen())
        } else {
            None
        };
        let ext_timeout = if rng.gen_bool(0.5) {
            Some(ext::TimeoutType::from_millis(rng.r#gen()))
        } else {
            None
        };

        Self {
            wire_expr,
            id,
            payload,
            ext_qos,
            ext_tstamp,
            ext_nodeid,
            ext_target,
            ext_budget,
            ext_timeout,
        }
    }
}

pub(crate) mod ext {
    use core::{num::NonZeroU32, time::Duration};

    use crate::{
        protocol::{ZCodecError, network::request::ext},
        result::ZResult,
        zbail,
        zbuf::{ZBufReader, ZBufWriter},
    };

    pub(crate) type QoS = crate::zextz64!(0x1, false);
    pub(crate) type QoSType = crate::protocol::network::ext::QoSType<{ QoS::ID }>;

    pub(crate) type Timestamp<'a> = crate::zextzbuf!('a, 0x2, false);
    pub(crate) type TimestampType = crate::protocol::network::ext::TimestampType<{ Timestamp::ID }>;

    pub(crate) type NodeId = crate::zextz64!(0x3, true);
    pub(crate) type NodeIdType = crate::protocol::network::ext::NodeIdType<{ NodeId::ID }>;

    pub(crate) type Target = crate::zextz64!(0x4, true);

    #[repr(u8)]
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum QueryTarget {
        #[default]
        BestMatching,

        All,

        AllComplete,
    }

    impl QueryTarget {
        pub(crate) const DEFAULT: Self = Self::BestMatching;

        pub(crate) fn encode(
            &self,
            more: bool,
            writer: &mut ZBufWriter<'_>,
        ) -> ZResult<(), ZCodecError> {
            let v = match self {
                ext::QueryTarget::BestMatching => 0,
                ext::QueryTarget::All => 1,
                ext::QueryTarget::AllComplete => 2,
            };

            let ext = ext::Target::new(v);
            ext.encode(more, writer)
        }

        pub(crate) fn decode(
            header: u8,
            reader: &mut ZBufReader<'_>,
        ) -> ZResult<(Self, bool), ZCodecError> {
            let (ext, more) = ext::Target::decode(header, reader)?;

            let v = match ext.value {
                0 => ext::QueryTarget::BestMatching,
                1 => ext::QueryTarget::All,
                2 => ext::QueryTarget::AllComplete,
                _ => zbail!(ZCodecError::CouldNotRead),
            };

            Ok((v, more))
        }

        #[cfg(test)]
        pub(crate) fn rand() -> Self {
            use rand::prelude::*;
            let mut rng = rand::thread_rng();

            *[
                QueryTarget::All,
                QueryTarget::AllComplete,
                QueryTarget::BestMatching,
            ]
            .choose(&mut rng)
            .unwrap()
        }
    }

    pub(crate) type Budget = crate::zextz64!(0x5, false);
    pub(crate) type BudgetType = NonZeroU32;

    pub(crate) type Timeout = crate::zextz64!(0x6, false);
    pub(crate) type TimeoutType = Duration;
}
