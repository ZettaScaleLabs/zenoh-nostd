use core::fmt;

use crate::{
    protocol::{
        ZCodecError,
        common::imsg,
        core::Reliability,
        network::{
            declare::Declare,
            interest::Interest,
            push::Push,
            request::Request,
            response::{Response, ResponseFinal},
        },
        zcodec::decode_u8,
    },
    result::ZResult,
    zbail,
    zbuf::{ZBufReader, ZBufWriter},
};

pub(crate) mod declare;
pub(crate) mod interest;
pub(crate) mod push;
pub(crate) mod request;
pub(crate) mod response;

pub(crate) mod id {
    pub(crate) const DECLARE: u8 = 0x1e;
    pub(crate) const PUSH: u8 = 0x1d;
    pub(crate) const REQUEST: u8 = 0x1c;
    pub(crate) const RESPONSE: u8 = 0x1b;
    pub(crate) const RESPONSE_FINAL: u8 = 0x1a;
    pub(crate) const INTEREST: u8 = 0x19;
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub(crate) enum Mapping {
    #[default]
    Receiver = 0,
    Sender = 1,
}

impl Mapping {
    pub(crate) const DEFAULT: Self = Self::Receiver;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NetworkBody<'a> {
    Push(Push<'a>),
    Request(Request<'a>),
    Response(Response<'a>),
    ResponseFinal(ResponseFinal),
    Interest(Interest<'a>),
    Declare(Declare<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NetworkMessage<'a> {
    pub(crate) body: NetworkBody<'a>,
    pub(crate) reliability: Reliability,
}

impl<'a> NetworkMessage<'a> {
    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        match &self.body {
            NetworkBody::Declare(m) => m.encode(writer),
            NetworkBody::Push(m) => m.encode(writer),
            NetworkBody::Response(m) => m.encode(writer),
            NetworkBody::Request(m) => m.encode(writer),
            NetworkBody::Interest(m) => m.encode(writer),
            NetworkBody::ResponseFinal(m) => m.encode(writer),
        }
    }

    pub(crate) fn decode(
        reliability: Reliability,
        reader: &mut ZBufReader<'a>,
    ) -> ZResult<Self, ZCodecError> {
        let header = decode_u8(reader)?;

        let body = match imsg::mid(header) {
            id::PUSH => NetworkBody::Push(Push::decode(header, reader)?),
            id::REQUEST => NetworkBody::Request(Request::decode(header, reader)?),
            id::RESPONSE => NetworkBody::Response(Response::decode(header, reader)?),
            id::RESPONSE_FINAL => {
                NetworkBody::ResponseFinal(ResponseFinal::decode(header, reader)?)
            }

            id::INTEREST => NetworkBody::Interest(Interest::decode(header, reader)?),
            id::DECLARE => NetworkBody::Declare(Declare::decode(header, reader)?),
            _ => zbail!(ZCodecError::CouldNotRead),
        };

        Ok(Self { reliability, body })
    }

    #[cfg(test)]
    pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        let body = match rng.gen_range(0..5) {
            0 => NetworkBody::Push(Push::rand(zbuf)),
            1 => NetworkBody::Request(Request::rand(zbuf)),
            2 => NetworkBody::Response(Response::rand(zbuf)),
            3 => NetworkBody::ResponseFinal(ResponseFinal::rand()),
            4 => NetworkBody::Declare(Declare::rand(zbuf)),
            _ => unreachable!(),
        };

        body.into()
    }
}

impl fmt::Display for NetworkMessage<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.body {
            NetworkBody::Push(_) => write!(f, "Push"),
            NetworkBody::Request(_) => write!(f, "Request"),
            NetworkBody::Response(_) => write!(f, "Response"),
            NetworkBody::ResponseFinal(_) => write!(f, "ResponseFinal"),
            NetworkBody::Interest(_) => write!(f, "Interest"),
            NetworkBody::Declare(_) => write!(f, "Declare"),
        }
    }
}

impl<'a> From<NetworkBody<'a>> for NetworkMessage<'a> {
    #[inline]
    fn from(body: NetworkBody<'a>) -> Self {
        Self {
            body,
            reliability: Reliability::DEFAULT,
        }
    }
}

pub(crate) mod ext {
    use core::fmt;

    use crate::{
        protocol::{
            ZCodecError,
            common::{
                extension::{ZExtZ64, ZExtZBufHeader},
                imsg,
            },
            core::{CongestionControl, EntityId, Priority, ZenohIdProto},
            zcodec::{
                decode_timestamp, decode_u8, decode_u32, encode_timestamp, encode_u8, encode_u32,
                encoded_len_timestamp, encoded_len_u32,
            },
        },
        result::ZResult,
        zbuf::{ZBufReader, ZBufWriter},
    };

    #[repr(transparent)]
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub(crate) struct QoSType<const ID: u8> {
        inner: u8,
    }

    impl<const ID: u8> QoSType<{ ID }> {
        const P_MASK: u8 = 0b00000111;
        const D_FLAG: u8 = 0b00001000;
        const E_FLAG: u8 = 0b00010000;
        const F_FLAG: u8 = 0b00100000;

        pub(crate) const DEFAULT: Self =
            Self::new(Priority::DEFAULT, CongestionControl::DEFAULT, false);

        pub(crate) const DECLARE: Self =
            Self::new(Priority::Control, CongestionControl::DEFAULT_DECLARE, false);

        pub(crate) const fn new(
            priority: Priority,
            congestion_control: CongestionControl,
            is_express: bool,
        ) -> Self {
            let mut inner = priority as u8;
            #[allow(clippy::single_match)]
            match congestion_control {
                CongestionControl::Block => inner |= Self::D_FLAG,
                _ => {}
            }
            if is_express {
                inner |= Self::E_FLAG;
            }
            Self { inner }
        }

        pub(crate) const fn get_priority(&self) -> Priority {
            unsafe { core::mem::transmute(self.inner & Self::P_MASK) }
        }

        pub(crate) const fn get_congestion_control(&self) -> CongestionControl {
            match (
                imsg::has_flag(self.inner, Self::D_FLAG),
                imsg::has_flag(self.inner, Self::F_FLAG),
            ) {
                (false, false) => CongestionControl::Drop,
                (false, true) => CongestionControl::Drop,
                (true, _) => CongestionControl::Block,
            }
        }

        pub(crate) const fn is_express(&self) -> bool {
            imsg::has_flag(self.inner, Self::E_FLAG)
        }

        pub(crate) fn encode(
            &self,
            more: bool,
            writer: &mut ZBufWriter<'_>,
        ) -> ZResult<(), ZCodecError> {
            let ext: ZExtZ64<{ ID }> = (*self).into();
            ext.encode(more, writer)
        }

        pub(crate) fn decode(
            header: u8,
            reader: &mut ZBufReader<'_>,
        ) -> ZResult<(Self, bool), ZCodecError> {
            let (ext, more) = ZExtZ64::<{ ID }>::decode(header, reader)?;
            Ok((ext.into(), more))
        }

        #[cfg(test)]
        pub(crate) fn rand() -> Self {
            use rand::Rng;
            let mut rng = rand::thread_rng();

            let inner: u8 = rng.r#gen();
            Self { inner }
        }
    }

    impl<const ID: u8> Default for QoSType<{ ID }> {
        fn default() -> Self {
            Self::new(Priority::DEFAULT, CongestionControl::DEFAULT, false)
        }
    }

    impl<const ID: u8> From<ZExtZ64<{ ID }>> for QoSType<{ ID }> {
        fn from(ext: ZExtZ64<{ ID }>) -> Self {
            Self {
                inner: ext.value as u8,
            }
        }
    }

    impl<const ID: u8> From<QoSType<{ ID }>> for ZExtZ64<{ ID }> {
        fn from(ext: QoSType<{ ID }>) -> Self {
            ZExtZ64::new(ext.inner as u64)
        }
    }

    impl<const ID: u8> fmt::Debug for QoSType<{ ID }> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("QoS")
                .field("priority", &self.get_priority())
                .field("congestion", &self.get_congestion_control())
                .field("express", &self.is_express())
                .finish()
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct TimestampType<const ID: u8> {
        pub(crate) timestamp: uhlc::Timestamp,
    }

    impl<const ID: u8> TimestampType<{ ID }> {
        pub(crate) fn encode(
            &self,
            more: bool,
            writer: &mut ZBufWriter<'_>,
        ) -> ZResult<(), ZCodecError> {
            let header: ZExtZBufHeader<{ ID }> =
                ZExtZBufHeader::new(encoded_len_timestamp(&self.timestamp));
            header.encode(more, writer)?;
            encode_timestamp(writer, &self.timestamp)
        }

        pub(crate) fn decode(
            header: u8,
            reader: &mut ZBufReader<'_>,
        ) -> ZResult<(Self, bool), ZCodecError> {
            let (_, more) = ZExtZBufHeader::<{ ID }>::decode(header, reader)?;
            let timestamp = decode_timestamp(reader)?;
            Ok((Self { timestamp }, more))
        }

        #[cfg(test)]
        pub(crate) fn rand() -> Self {
            use rand::Rng;
            let mut rng = rand::thread_rng();

            let time = uhlc::NTP64(rng.r#gen());
            let id = uhlc::ID::try_from(ZenohIdProto::rand().as_le_bytes()).unwrap();
            let timestamp = uhlc::Timestamp::new(time, id);
            Self { timestamp }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct NodeIdType<const ID: u8> {
        pub(crate) node_id: u16,
    }

    impl<const ID: u8> NodeIdType<{ ID }> {
        pub(crate) const DEFAULT: Self = Self { node_id: 0 };

        pub(crate) fn encode(
            &self,
            more: bool,
            writer: &mut ZBufWriter<'_>,
        ) -> ZResult<(), ZCodecError> {
            let ext: ZExtZ64<{ ID }> = (*self).into();
            ext.encode(more, writer)
        }

        pub(crate) fn decode(
            header: u8,
            reader: &mut ZBufReader<'_>,
        ) -> ZResult<(Self, bool), ZCodecError> {
            let (ext, more) = ZExtZ64::<{ ID }>::decode(header, reader)?;
            Ok((ext.into(), more))
        }

        #[cfg(test)]
        pub(crate) fn rand() -> Self {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let node_id = rng.r#gen();
            Self { node_id }
        }
    }

    impl<const ID: u8> Default for NodeIdType<{ ID }> {
        fn default() -> Self {
            Self::DEFAULT
        }
    }

    impl<const ID: u8> From<ZExtZ64<{ ID }>> for NodeIdType<{ ID }> {
        fn from(ext: ZExtZ64<{ ID }>) -> Self {
            Self {
                node_id: ext.value as u16,
            }
        }
    }

    impl<const ID: u8> From<NodeIdType<{ ID }>> for ZExtZ64<{ ID }> {
        fn from(ext: NodeIdType<{ ID }>) -> Self {
            ZExtZ64::new(ext.node_id as u64)
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct EntityGlobalIdType<const ID: u8> {
        pub(crate) zid: ZenohIdProto,
        pub(crate) eid: EntityId,
    }

    impl<const ID: u8> EntityGlobalIdType<{ ID }> {
        pub(crate) fn encoded_len(&self) -> usize {
            1 + self.zid.encoded_len(false) + encoded_len_u32(self.eid)
        }

        pub(crate) fn encode(
            &self,
            more: bool,
            writer: &mut ZBufWriter<'_>,
        ) -> ZResult<(), ZCodecError> {
            let header = ZExtZBufHeader::<{ ID }>::new(self.encoded_len());
            header.encode(more, writer)?;

            let flags: u8 = (self.zid.size() as u8 - 1) << 4;
            encode_u8(writer, flags)?;
            self.zid.encode(false, writer)?;
            encode_u32(writer, self.eid)
        }

        pub(crate) fn decode(
            header: u8,
            reader: &mut ZBufReader<'_>,
        ) -> ZResult<(Self, bool), ZCodecError> {
            let (_, more) = ZExtZBufHeader::<{ ID }>::decode(header, reader)?;

            let flags = decode_u8(reader)?;
            let length = 1 + ((flags >> 4) as usize);
            let zid = ZenohIdProto::decode(Some(length), reader)?;
            let eid = decode_u32(reader)?;

            Ok((Self { zid, eid }, more))
        }

        #[cfg(test)]
        pub(crate) fn rand() -> Self {
            use rand::Rng;
            let mut rng = rand::thread_rng();

            let zid = ZenohIdProto::rand();
            let eid: EntityId = rng.r#gen();
            Self { zid, eid }
        }
    }
}
