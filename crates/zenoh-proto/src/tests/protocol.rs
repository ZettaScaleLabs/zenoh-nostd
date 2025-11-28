use crate::{exts::*, fields::*, msgs::*, *};

use {
    crate::ZWriterExt,
    ::core::time::Duration,
    rand::{
        Rng,
        distributions::{Alphanumeric, DistString},
        thread_rng,
    },
};

mod ke;
mod msgs;

macro_rules! roundtrip {
    ($ty:ty) => {{
        let mut rand_data = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
        let mut rand_writer = rand_data.as_mut_slice();

        let mut data = [0u8; MAX_PAYLOAD_SIZE];
        for _ in 0..NUM_ITER {
            let mut writer = data.as_mut_slice();
            let value = <$ty>::rand(&mut rand_writer);
            let len = <_ as $crate::ZLen>::z_len(&value);
            <_ as $crate::ZEncode>::z_encode(&value, &mut writer).unwrap();
            let mut reader = data.as_slice();
            let ret = <$ty as $crate::ZDecode>::z_decode(
                &mut <_ as $crate::ZReaderExt>::sub(&mut reader, len).unwrap(),
            )
            .unwrap();
            assert_eq!(ret, value);
        }
    }};

    (ext, $ty:ty) => {{
        let mut rand_data = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
        let mut rand_writer = rand_data.as_mut_slice();

        let mut data = [0u8; MAX_PAYLOAD_SIZE];
        for _ in 0..NUM_ITER {
            let mut writer = data.as_mut_slice();
            let value = <$ty>::rand(&mut rand_writer);
            $crate::zext_encode::<_, 0x1, true>(&value, &mut writer, false).unwrap();
            let mut reader = data.as_slice();
            let ret = $crate::zext_decode::<$ty>(&mut reader).unwrap();
            assert_eq!(ret, value);
        }
    }};
}
pub(crate) use roundtrip;

macro_rules! roundtrips {
    (ext, $namespace:ident, $($ty:ty),* $(,)?) => {
        $(
            paste::paste! {
                #[test]
                fn [<$namespace _proto_ext_ $ty:lower>]() {
                    $crate::tests::protocol::roundtrip!(ext, $ty);
                }
            }
        )*
    };

    ($namespace:ident, $($ty:ty),* $(,)?) => {
        $(
            paste::paste! {
                #[test]
                fn [<$namespace _proto_ $ty:lower>]() {
                    $crate::tests::protocol::roundtrip!($ty);
                }
            }
        )*
    };
}

pub(crate) use roundtrips;

impl ZenohIdProto {
    #[cfg(test)]
    pub fn rand(_: &mut ZWriter) -> ZenohIdProto {
        ZenohIdProto(uhlc::ID::rand())
    }
}

impl Resolution {
    #[cfg(test)]
    pub fn rand() -> Self {
        use rand::Rng;

        let mut rng = rand::thread_rng();
        let v: u8 = rng.r#gen();
        Self(v & 0b0000_1111)
    }
}

impl<'a> Encoding<'a> {
    #[cfg(test)]
    pub fn rand(w: &mut ZWriter<'a>) -> Self {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        const MIN: usize = 0;
        const MAX: usize = 16;

        let id: u16 = rng.r#gen();
        let schema = if rng.gen_bool(0.5) {
            use crate::ZWriterExt;

            Some(
                w.write_slot(rng.gen_range(MIN..MAX), |b: &mut [u8]| {
                    rng.fill(b);
                    b.len()
                })
                .unwrap(),
            )
        } else {
            None
        };

        Encoding { id, schema }
    }
}

impl<'a> WireExpr<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let scope = thread_rng().r#gen();
        let mapping = Mapping::rand(w);

        let suffix = if thread_rng().gen_bool(0.5) {
            let suffix =
                Alphanumeric.sample_string(&mut thread_rng(), thread_rng().gen_range(1..16));
            w.write_str(&suffix).unwrap()
        } else {
            ""
        };

        Self {
            scope,
            mapping,
            suffix,
        }
    }
}
impl<'a> Err<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        let encoding = if thread_rng().gen_bool(0.5) {
            Encoding::rand(w)
        } else {
            Encoding::DEFAULT
        };

        let sinfo = thread_rng().gen_bool(0.5).then_some(SourceInfo::rand(w));
        let payload = w
            .write_slot(thread_rng().gen_range(0..=64), |b: &mut [u8]| {
                thread_rng().fill(b);
                b.len()
            })
            .unwrap();

        Self {
            encoding,
            sinfo,
            payload,
        }
    }
}

impl<'a> Put<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        let timestamp = thread_rng().gen_bool(0.5).then_some({
            let time = uhlc::NTP64(thread_rng().r#gen());
            let id = uhlc::ID::try_from(ZenohIdProto::default().as_le_bytes()).unwrap();
            Timestamp::new(time, id)
        });

        let encoding = if thread_rng().gen_bool(0.5) {
            Encoding::rand(w)
        } else {
            Encoding::DEFAULT
        };

        let sinfo = thread_rng().gen_bool(0.5).then_some(SourceInfo::rand(w));
        let attachment = thread_rng().gen_bool(0.5).then_some(Attachment::rand(w));
        let payload = w
            .write_slot(thread_rng().gen_range(1..=64), |b| {
                thread_rng().fill(b);
                b.len()
            })
            .unwrap();

        Self {
            timestamp,
            encoding,
            sinfo,
            attachment,
            payload,
        }
    }
}
impl<'a> Query<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        const MIN: usize = 1;
        const MAX: usize = 16;

        let consolidation = if thread_rng().gen_bool(0.5) {
            ConsolidationMode::rand(w)
        } else {
            ConsolidationMode::default()
        };

        let parameters = if thread_rng().gen_bool(0.5) {
            let len = thread_rng().gen_range(MIN..MAX);
            let proto = Alphanumeric.sample_string(&mut thread_rng(), len);
            w.write_str(proto.as_str()).unwrap()
        } else {
            ""
        };
        let sinfo = thread_rng().gen_bool(0.5).then_some(SourceInfo::rand(w));
        let body = thread_rng().gen_bool(0.5).then_some(Value::rand(w));
        let attachment = thread_rng().gen_bool(0.5).then_some(Attachment::rand(w));

        Self {
            consolidation,
            parameters,

            sinfo,
            body,
            attachment,
        }
    }
}

impl<'a> Reply<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        let payload = PushBody::rand(w);

        let consolidation = if thread_rng().gen_bool(0.5) {
            ConsolidationMode::rand(w)
        } else {
            ConsolidationMode::default()
        };

        Self {
            consolidation,
            payload,
        }
    }
}

impl EntityGlobalId {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter) -> Self {
        let zid = ZenohIdProto::rand(w);
        let eid: u32 = thread_rng().r#gen();

        Self { zid, eid }
    }
}

impl SourceInfo {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter) -> Self {
        let id = EntityGlobalId::rand(w);
        let sn: u32 = thread_rng().r#gen();

        Self { id, sn }
    }
}

impl<'a> Value<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        let encoding = Encoding::rand(w);
        let payload = w
            .write_slot(thread_rng().gen_range(0..=64), |b: &mut [u8]| {
                thread_rng().fill(b);
                b.len()
            })
            .unwrap();

        Self { encoding, payload }
    }
}

impl<'a> Attachment<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        let buffer = w
            .write_slot(thread_rng().gen_range(0..=64), |b: &mut [u8]| {
                thread_rng().fill(b);
                b.len()
            })
            .unwrap();

        Self { buffer }
    }
}

impl QoS {
    #[cfg(test)]
    pub(crate) fn rand(_: &mut ZWriter) -> Self {
        let inner: u8 = thread_rng().r#gen();
        Self { inner }
    }
}
impl NodeId {
    #[cfg(test)]
    pub(crate) fn rand(_: &mut ZWriter) -> Self {
        let node_id: u16 = thread_rng().r#gen();
        Self { node_id }
    }
}

impl Budget {
    #[cfg(test)]
    pub(crate) fn rand(_: &mut ZWriter) -> Self {
        let budget: u32 = thread_rng().r#gen();
        Self { budget }
    }
}

impl QueryableInfo {
    #[cfg(test)]
    pub(crate) fn rand(_: &mut ZWriter) -> Self {
        let complete = thread_rng().gen_bool(0.5);
        let distance: u16 = thread_rng().r#gen();
        Self { complete, distance }
    }
}

impl<'a> Declare<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let id = if rand::thread_rng().gen_bool(0.5) {
            Some(rand::thread_rng().r#gen())
        } else {
            None
        };

        let qos = if rand::thread_rng().gen_bool(0.5) {
            QoS::rand(w)
        } else {
            QoS::DEFAULT
        };

        let timestamp = rand::thread_rng().gen_bool(0.5).then_some({
            let time = uhlc::NTP64(rand::thread_rng().r#gen());
            let id = uhlc::ID::try_from(ZenohIdProto::default().as_le_bytes()).unwrap();
            Timestamp::new(time, id)
        });

        let nodeid = if rand::thread_rng().gen_bool(0.5) {
            NodeId::rand(w)
        } else {
            NodeId::DEFAULT
        };

        let body = DeclareBody::DeclareKeyExpr(DeclareKeyExpr::rand(w));

        Self {
            id,
            qos,
            timestamp,
            nodeid,
            body,
        }
    }
}

impl<'a> DeclareKeyExpr<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let id = rand::thread_rng().r#gen();
        let wire_expr = WireExpr::rand(w);
        Self { id, wire_expr }
    }
}

impl UndeclareKeyExpr {
    #[cfg(test)]
    pub(crate) fn rand(_: &mut crate::ZWriter) -> Self {
        let id = rand::thread_rng().r#gen();
        Self { id }
    }
}

impl<'a> DeclareSubscriber<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let id = rand::thread_rng().r#gen();
        let wire_expr = WireExpr::rand(w);
        Self { id, wire_expr }
    }
}

impl<'a> UndeclareSubscriber<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let id = rand::thread_rng().r#gen();
        let wire_expr = if rand::thread_rng().gen_bool(0.5) {
            Some(WireExpr::rand(w))
        } else {
            None
        };
        Self { id, wire_expr }
    }
}

impl<'a> DeclareQueryable<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let id = rand::thread_rng().r#gen();
        let wire_expr = WireExpr::rand(w);
        let qinfo = if rand::thread_rng().gen_bool(0.5) {
            QueryableInfo::rand(w)
        } else {
            QueryableInfo::DEFAULT
        };
        Self {
            id,
            wire_expr,
            qinfo,
        }
    }
}

impl<'a> UndeclareQueryable<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let id = rand::thread_rng().r#gen();
        let wire_expr = if rand::thread_rng().gen_bool(0.5) {
            Some(WireExpr::rand(w))
        } else {
            None
        };
        Self { id, wire_expr }
    }
}

impl<'a> DeclareToken<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let id = rand::thread_rng().r#gen();
        let wire_expr = WireExpr::rand(w);
        Self { id, wire_expr }
    }
}

impl<'a> UndeclareToken<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let id = rand::thread_rng().r#gen();
        let wire_expr = if rand::thread_rng().gen_bool(0.5) {
            Some(WireExpr::rand(w))
        } else {
            None
        };
        Self { id, wire_expr }
    }
}

impl DeclareFinal {
    #[cfg(test)]
    pub(crate) fn rand(_: &mut crate::ZWriter) -> Self {
        Self {}
    }
}

impl<'a> Push<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        let wire_expr = WireExpr::rand(w);
        let qos = if thread_rng().gen_bool(0.5) {
            QoS::rand(w)
        } else {
            QoS::DEFAULT
        };

        let timestamp = thread_rng().gen_bool(0.5).then_some({
            let time = uhlc::NTP64(thread_rng().r#gen());
            let id = uhlc::ID::try_from(ZenohIdProto::default().as_le_bytes()).unwrap();
            Timestamp::new(time, id)
        });

        let nodeid = if thread_rng().gen_bool(0.5) {
            NodeId::rand(w)
        } else {
            NodeId::DEFAULT
        };

        let payload = PushBody::rand(w);

        Self {
            wire_expr,
            qos,
            timestamp,
            nodeid,
            payload,
        }
    }
}

impl<'a> Request<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        use ::core::time::Duration;

        let id = thread_rng().r#gen();
        let wire_expr = WireExpr::rand(w);

        let qos = if thread_rng().gen_bool(0.5) {
            QoS::rand(w)
        } else {
            QoS::DEFAULT
        };

        let timestamp = thread_rng().gen_bool(0.5).then_some({
            let time = uhlc::NTP64(thread_rng().r#gen());
            let id = uhlc::ID::try_from(ZenohIdProto::default().as_le_bytes()).unwrap();
            Timestamp::new(time, id)
        });

        let nodeid = if thread_rng().gen_bool(0.5) {
            NodeId::rand(w)
        } else {
            NodeId::DEFAULT
        };

        let target = if thread_rng().gen_bool(0.5) {
            QueryTarget::rand(w)
        } else {
            QueryTarget::DEFAULT
        };

        trait RandDuration {
            fn rand(w: &mut crate::ZWriter) -> Self;
        }

        impl RandDuration for Duration {
            fn rand(_: &mut ZWriter) -> Self {
                Duration::from_millis(thread_rng().gen_range(0..10_000))
            }
        }

        let budget = thread_rng().gen_bool(0.5).then_some(Budget::rand(w));
        let timeout = thread_rng().gen_bool(0.5).then_some(Duration::rand(w));

        let payload = RequestBody::rand(w);

        Self {
            id,
            wire_expr,
            qos,
            timestamp,
            nodeid,
            target,
            budget,
            timeout,
            payload,
        }
    }
}

impl<'a> Response<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        let rid = thread_rng().r#gen();
        let wire_expr = WireExpr::rand(w);

        let qos = if thread_rng().gen_bool(0.5) {
            QoS::rand(w)
        } else {
            QoS::DEFAULT
        };

        let timestamp = thread_rng().gen_bool(0.5).then_some({
            let time = uhlc::NTP64(thread_rng().r#gen());
            let id = uhlc::ID::try_from(ZenohIdProto::default().as_le_bytes()).unwrap();
            Timestamp::new(time, id)
        });

        let respid = thread_rng()
            .gen_bool(0.5)
            .then_some(EntityGlobalId::rand(w));

        let payload = ResponseBody::rand(w);

        Self {
            rid,
            wire_expr,
            qos,
            timestamp,
            respid,
            payload,
        }
    }
}

impl ResponseFinal {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter) -> Self {
        let rid = thread_rng().r#gen();

        let qos = if thread_rng().gen_bool(0.5) {
            QoS::rand(w)
        } else {
            QoS::DEFAULT
        };

        let timestamp = thread_rng().gen_bool(0.5).then_some({
            let time = uhlc::NTP64(thread_rng().r#gen());
            let id = uhlc::ID::try_from(ZenohIdProto::default().as_le_bytes()).unwrap();
            Timestamp::new(time, id)
        });

        Self {
            rid,
            qos,
            timestamp,
        }
    }
}

impl InterestOptions {
    #[cfg(test)]
    pub fn rand() -> Self {
        let mut s = Self { options: 0 };
        if thread_rng().gen_bool(0.5) {
            s.options |= Self::KEYEXPRS.options;
        }
        if thread_rng().gen_bool(0.5) {
            s.options |= Self::SUBSCRIBERS.options;
        }
        if thread_rng().gen_bool(0.5) {
            s.options |= Self::TOKENS.options;
        }
        if thread_rng().gen_bool(0.5) {
            s.options |= Self::AGGREGATE.options;
        }
        s
    }
}

impl<'a> InterestInner<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        let options = InterestOptions::rand().options;
        let wire_expr = if thread_rng().gen_bool(0.5) {
            Some(WireExpr::rand(w))
        } else {
            None
        };

        Self { options, wire_expr }
    }
}

impl InterestExt {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter) -> Self {
        let qos = if thread_rng().gen_bool(0.5) {
            QoS::rand(w)
        } else {
            QoS::DEFAULT
        };

        let timestamp = thread_rng().gen_bool(0.5).then_some({
            let time = uhlc::NTP64(thread_rng().r#gen());
            let id = uhlc::ID::try_from(ZenohIdProto::default().as_le_bytes()).unwrap();
            Timestamp::new(time, id)
        });

        let nodeid = if thread_rng().gen_bool(0.5) {
            NodeId::rand(w)
        } else {
            NodeId::DEFAULT
        };

        Self {
            qos,
            timestamp,
            nodeid,
        }
    }
}

impl<'a> Interest<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        let id = thread_rng().r#gen();
        let mode = InterestMode::rand(w);

        let inner = if mode != InterestMode::Final {
            InterestInner::rand(w)
        } else {
            InterestInner {
                options: 0,
                wire_expr: None,
            }
        };

        let ext = InterestExt::rand(w);

        Self {
            id,
            mode,
            inner,
            ext,
        }
    }
}

impl<'a> Auth<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let payload = w
            .write_slot(thread_rng().gen_range(0..=64), |b: &mut [u8]| {
                thread_rng().fill(b);
                b.len()
            })
            .unwrap();

        Self { payload }
    }
}

impl<'a> MultiLink<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let payload = w
            .write_slot(thread_rng().gen_range(0..=64), |b: &mut [u8]| {
                thread_rng().fill(b);
                b.len()
            })
            .unwrap();

        Self { payload }
    }
}

impl<'a> MultiLinkSyn<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let payload = w
            .write_slot(thread_rng().gen_range(0..=64), |b: &mut [u8]| {
                thread_rng().fill(b);
                b.len()
            })
            .unwrap();

        Self { payload }
    }
}

impl Patch {
    #[cfg(test)]
    pub(crate) fn rand(_: &mut ZWriter) -> Self {
        Self {
            int: thread_rng().r#gen(),
        }
    }
}

impl Close {
    #[cfg(test)]
    pub(crate) fn rand(_: &mut ZWriter) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let reason: u8 = rng.r#gen();
        let behaviour = if rng.gen_bool(0.5) {
            CloseBehaviour::Link
        } else {
            CloseBehaviour::Session
        };
        Self { reason, behaviour }
    }
}

impl FrameHeader {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter) -> Self {
        let reliability = Reliability::rand(w);
        let sn = rand::thread_rng().r#gen();
        let qos = QoS::rand(w);
        Self {
            reliability,
            sn,
            qos,
        }
    }
}

impl InitIdentifier {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter) -> Self {
        let whatami = WhatAmI::rand(w);
        let zid = ZenohIdProto::rand(w);
        Self { whatami, zid }
    }
}

impl InitResolution {
    #[cfg(test)]
    pub(crate) fn rand(_: &mut ZWriter) -> Self {
        let resolution = Resolution::rand();
        let batch_size = BatchSize(rand::thread_rng().r#gen());
        Self {
            resolution,
            batch_size,
        }
    }
}

impl QoSLink {
    #[cfg(test)]
    pub(crate) fn rand(_: &mut ZWriter) -> Self {
        Self {
            qos: thread_rng().r#gen(),
        }
    }
}

impl<'a> InitExt<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        let qos = if rand::thread_rng().gen_bool(0.5) {
            Some(HasQoS {})
        } else {
            None
        };
        let qos_link = if rand::thread_rng().gen_bool(0.5) {
            Some(QoSLink::rand(w))
        } else {
            None
        };
        let auth = if rand::thread_rng().gen_bool(0.5) {
            Some(Auth::rand(w))
        } else {
            None
        };
        let mlink = if rand::thread_rng().gen_bool(0.5) {
            Some(MultiLink::rand(w))
        } else {
            None
        };
        let lowlatency = if rand::thread_rng().gen_bool(0.5) {
            Some(HasLowLatency {})
        } else {
            None
        };
        let compression = if rand::thread_rng().gen_bool(0.5) {
            Some(HasCompression {})
        } else {
            None
        };
        let patch = if rand::thread_rng().gen_bool(0.5) {
            Patch::rand(w)
        } else {
            Patch::NONE
        };

        Self {
            qos,
            qos_link,
            auth,
            mlink,
            lowlatency,
            compression,
            patch,
        }
    }
}

impl<'a> InitSyn<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        let version = rand::thread_rng().r#gen();
        let identifier = InitIdentifier::rand(w);
        let resolution = if rand::thread_rng().gen_bool(0.5) {
            InitResolution::rand(w)
        } else {
            InitResolution::DEFAULT
        };
        let ext = InitExt::rand(w);

        Self {
            version,
            identifier,
            resolution,
            ext,
        }
    }
}

impl<'a> InitAck<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        let version = rand::thread_rng().r#gen();
        let identifier = InitIdentifier::rand(w);
        let resolution = if rand::thread_rng().gen_bool(0.5) {
            InitResolution::rand(w)
        } else {
            InitResolution::DEFAULT
        };
        let cookie_len = rand::thread_rng().gen_range(0..16);
        let cookie = w
            .write_slot(cookie_len, |b: &mut [u8]| {
                rand::thread_rng().fill(b);
                b.len()
            })
            .unwrap();
        let ext = InitExt::rand(w);

        Self {
            version,
            identifier,
            resolution,
            cookie,
            ext,
        }
    }
}

impl KeepAlive {
    #[cfg(test)]
    pub(crate) fn rand(_: &mut crate::ZWriter) -> Self {
        Self {}
    }
}

impl<'a> OpenExt<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let qos = if rand::thread_rng().gen_bool(0.5) {
            Some(HasQoS {})
        } else {
            None
        };

        let auth = if rand::thread_rng().gen_bool(0.5) {
            Some(Auth::rand(w))
        } else {
            None
        };

        let mlink_syn = if rand::thread_rng().gen_bool(0.5) {
            Some(MultiLinkSyn::rand(w))
        } else {
            None
        };

        let mlink_ack = if rand::thread_rng().gen_bool(0.5) {
            Some(HasMultiLinkAck {})
        } else {
            None
        };

        let lowlatency = if rand::thread_rng().gen_bool(0.5) {
            Some(HasLowLatency {})
        } else {
            None
        };

        let compression = if rand::thread_rng().gen_bool(0.5) {
            Some(HasCompression {})
        } else {
            None
        };

        OpenExt {
            qos,
            auth,
            mlink_syn,
            mlink_ack,
            lowlatency,
            compression,
        }
    }
}

impl<'a> OpenSyn<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let lease = Duration::from_secs(rand::thread_rng().gen_range(1..=3600));
        let sn: u32 = rand::thread_rng().r#gen();
        let cookie = w
            .write_slot(rand::thread_rng().gen_range(0..=64), |b: &mut [u8]| {
                rand::thread_rng().fill(b);
                b.len()
            })
            .unwrap();
        let ext = OpenExt::rand(w);

        Self {
            lease,
            sn,
            cookie,
            ext,
        }
    }
}

impl<'a> OpenAck<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let lease = Duration::from_secs(rand::thread_rng().gen_range(1..=3600));
        let sn: u32 = rand::thread_rng().r#gen();
        let ext = OpenExt::rand(w);

        Self { lease, sn, ext }
    }
}
