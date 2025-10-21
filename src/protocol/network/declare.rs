use crate::{
    protocol::{
        ZCodecError,
        common::{
            extension::{self, iext},
            imsg,
        },
        network::declare::{
            self,
            common::DeclareFinal,
            keyexpr::{DeclareKeyExpr, UndeclareKeyExpr},
            queryable::{DeclareQueryable, UndeclareQueryable},
            subscriber::{DeclareSubscriber, UndeclareSubscriber},
        },
        zcodec::{decode_u8, decode_u32, encode_u8, encode_u32},
    },
    result::ZResult,
    zbail,
    zbuf::{ZBufReader, ZBufWriter},
};

pub(crate) mod flag {
    pub(crate) const I: u8 = 1 << 5;

    pub(crate) const Z: u8 = 1 << 7;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Declare<'a> {
    pub(crate) interest_id: Option<super::interest::InterestId>,
    pub(crate) ext_qos: ext::QoSType,
    pub(crate) ext_tstamp: Option<ext::TimestampType>,
    pub(crate) ext_nodeid: ext::NodeIdType,
    pub(crate) body: DeclareBody<'a>,
}

impl<'a> Declare<'a> {
    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        let mut header = crate::protocol::network::id::DECLARE;

        if self.interest_id.is_some() {
            header |= declare::flag::I;
        }

        let mut n_exts = ((self.ext_qos != declare::ext::QoSType::DEFAULT) as u8)
            + (self.ext_tstamp.is_some() as u8)
            + ((self.ext_nodeid != declare::ext::NodeIdType::DEFAULT) as u8);

        if n_exts != 0 {
            header |= declare::flag::Z;
        }

        encode_u8(writer, header)?;

        if let Some(interest_id) = self.interest_id {
            encode_u32(writer, interest_id)?;
        }

        if self.ext_qos != declare::ext::QoSType::DEFAULT {
            n_exts -= 1;
            self.ext_qos.encode(n_exts != 0, writer)?;
        }
        if let Some(ts) = self.ext_tstamp.as_ref() {
            n_exts -= 1;
            ts.encode(n_exts != 0, writer)?;
        }
        if self.ext_nodeid != declare::ext::NodeIdType::DEFAULT {
            n_exts -= 1;
            self.ext_nodeid.encode(n_exts != 0, writer)?;
        }

        self.body.encode(writer)
    }

    pub(crate) fn decode(header: u8, reader: &mut ZBufReader<'a>) -> ZResult<Self, ZCodecError> {
        if imsg::mid(header) != crate::protocol::network::id::DECLARE {
            zbail!(ZCodecError::CouldNotRead)
        }

        let mut interest_id = None;
        if imsg::has_flag(header, declare::flag::I) {
            interest_id = Some(decode_u32(reader)?);
        }

        let mut ext_qos = declare::ext::QoSType::DEFAULT;
        let mut ext_tstamp = None;
        let mut ext_nodeid = declare::ext::NodeIdType::DEFAULT;

        let mut has_ext = imsg::has_flag(header, declare::flag::Z);
        while has_ext {
            let ext: u8 = crate::protocol::zcodec::decode_u8(reader)?;
            match iext::eheader(ext) {
                declare::ext::QoS::ID => {
                    let (q, ext) = declare::ext::QoSType::decode(ext, reader)?;
                    ext_qos = q;
                    has_ext = ext;
                }
                declare::ext::Timestamp::ID => {
                    let (t, ext) = declare::ext::TimestampType::decode(ext, reader)?;
                    ext_tstamp = Some(t);
                    has_ext = ext;
                }
                declare::ext::NodeId::ID => {
                    let (nid, ext) = declare::ext::NodeIdType::decode(ext, reader)?;
                    ext_nodeid = nid;
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip("Declare", ext, reader)?;
                }
            }
        }

        let body: DeclareBody = DeclareBody::decode(reader)?;

        Ok(Declare {
            interest_id,
            ext_qos,
            ext_tstamp,
            ext_nodeid,
            body,
        })
    }

    #[cfg(test)]
    pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        let interest_id = rng
            .gen_bool(0.5)
            .then_some(rng.r#gen::<super::interest::InterestId>());
        let ext_qos = ext::QoSType::rand();
        let ext_tstamp = rng.gen_bool(0.5).then(ext::TimestampType::rand);
        let ext_nodeid = ext::NodeIdType::rand();
        let body = DeclareBody::rand(zbuf);

        Self {
            interest_id,
            ext_qos,
            ext_tstamp,
            ext_nodeid,
            body,
        }
    }
}

pub(crate) mod ext {
    pub(crate) type QoS = crate::zextz64!(0x1, false);
    pub(crate) type QoSType = crate::protocol::network::ext::QoSType<{ QoS::ID }>;

    pub(crate) type Timestamp<'a> = crate::zextzbuf!('a, 0x2, false);
    pub(crate) type TimestampType = crate::protocol::network::ext::TimestampType<{ Timestamp::ID }>;

    pub(crate) type NodeId = crate::zextz64!(0x3, true);
    pub(crate) type NodeIdType = crate::protocol::network::ext::NodeIdType<{ NodeId::ID }>;
}

pub(crate) mod id {
    pub(crate) const D_KEYEXPR: u8 = 0x00;
    pub(crate) const U_KEYEXPR: u8 = 0x01;

    pub(crate) const D_SUBSCRIBER: u8 = 0x02;
    pub(crate) const U_SUBSCRIBER: u8 = 0x03;

    pub(crate) const D_QUERYABLE: u8 = 0x04;
    pub(crate) const U_QUERYABLE: u8 = 0x05;

    pub(crate) const D_FINAL: u8 = 0x1A;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DeclareBody<'a> {
    DeclareKeyExpr(DeclareKeyExpr<'a>),
    UndeclareKeyExpr(UndeclareKeyExpr),
    DeclareSubscriber(DeclareSubscriber<'a>),
    UndeclareSubscriber(UndeclareSubscriber<'a>),
    DeclareQueryable(DeclareQueryable<'a>),
    UndeclareQueryable(UndeclareQueryable<'a>),
    DeclareFinal(DeclareFinal),
}

impl<'a> DeclareBody<'a> {
    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        match self {
            DeclareBody::DeclareKeyExpr(r) => r.encode(writer),
            DeclareBody::UndeclareKeyExpr(r) => r.encode(writer),
            DeclareBody::DeclareSubscriber(r) => r.encode(writer),
            DeclareBody::UndeclareSubscriber(r) => r.encode(writer),
            DeclareBody::DeclareQueryable(r) => r.encode(writer),
            DeclareBody::UndeclareQueryable(r) => r.encode(writer),
            DeclareBody::DeclareFinal(r) => r.encode(writer),
        }
    }

    pub(crate) fn decode(reader: &mut ZBufReader<'a>) -> ZResult<Self, ZCodecError> {
        let header: u8 = decode_u8(reader)?;

        Ok(match imsg::mid(header) {
            declare::id::D_KEYEXPR => {
                DeclareBody::DeclareKeyExpr(DeclareKeyExpr::decode(header, reader)?)
            }
            declare::id::U_KEYEXPR => {
                DeclareBody::UndeclareKeyExpr(UndeclareKeyExpr::decode(header, reader)?)
            }
            declare::id::D_SUBSCRIBER => {
                DeclareBody::DeclareSubscriber(DeclareSubscriber::decode(header, reader)?)
            }
            declare::id::U_SUBSCRIBER => {
                DeclareBody::UndeclareSubscriber(UndeclareSubscriber::decode(header, reader)?)
            }
            declare::id::D_QUERYABLE => {
                DeclareBody::DeclareQueryable(DeclareQueryable::decode(header, reader)?)
            }
            declare::id::U_QUERYABLE => {
                DeclareBody::UndeclareQueryable(UndeclareQueryable::decode(header, reader)?)
            }
            declare::id::D_FINAL => {
                DeclareBody::DeclareFinal(DeclareFinal::decode(header, reader)?)
            }
            _ => zbail!(ZCodecError::CouldNotRead),
        })
    }

    #[cfg(test)]
    pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        match rng.gen_range(0..7) {
            0 => DeclareBody::DeclareKeyExpr(DeclareKeyExpr::rand(zbuf)),
            1 => DeclareBody::UndeclareKeyExpr(UndeclareKeyExpr::rand()),
            2 => DeclareBody::DeclareSubscriber(DeclareSubscriber::rand(zbuf)),
            3 => DeclareBody::UndeclareSubscriber(UndeclareSubscriber::rand(zbuf)),
            4 => DeclareBody::DeclareQueryable(DeclareQueryable::rand(zbuf)),
            5 => DeclareBody::UndeclareQueryable(UndeclareQueryable::rand(zbuf)),
            6 => DeclareBody::DeclareFinal(DeclareFinal::rand()),
            _ => unreachable!(),
        }
    }
}

pub(crate) mod common {
    use crate::{
        protocol::{
            ZCodecError,
            common::{extension, imsg},
            network::declare::{self, common, token},
            zcodec::encode_u8,
        },
        result::ZResult,
        zbail,
        zbuf::{ZBufReader, ZBufWriter},
    };

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct DeclareFinal;

    impl DeclareFinal {
        pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
            encode_u8(writer, declare::id::D_FINAL)?;

            Ok(())
        }

        pub(crate) fn decode(
            header: u8,
            reader: &mut ZBufReader<'_>,
        ) -> ZResult<Self, ZCodecError> {
            if imsg::mid(header) != declare::id::D_FINAL {
                zbail!(ZCodecError::CouldNotRead);
            }

            let has_ext = imsg::has_flag(header, token::flag::Z);
            if has_ext {
                extension::skip_all("Final", reader)?;
            }

            Ok(common::DeclareFinal)
        }

        #[cfg(test)]
        pub(crate) fn rand() -> Self {
            Self
        }
    }

    pub(crate) mod ext {
        use crate::{
            protocol::{
                ZCodecError,
                common::imsg,
                core::wire_expr::{ExprId, ExprLen, WireExpr},
                network::{Mapping, declare::common},
                zcodec::{decode_u16, encode_str, encode_u8, encode_u16},
            },
            result::ZResult,
            zbuf::{BufReaderExt, BufWriterExt, ZBuf, ZBufExt, ZBufMutExt, ZBufReader, ZBufWriter},
        };

        pub(crate) type WireExprExt<'a> = crate::zextzbuf!('a, 0x0f, true);
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub(crate) struct WireExprType<'a> {
            pub(crate) wire_expr: WireExpr<'a>,
        }

        impl<'a> WireExprType<'a> {
            pub(crate) fn null() -> Self {
                Self {
                    wire_expr: WireExpr {
                        scope: ExprId::MIN,
                        suffix: "",
                        mapping: Mapping::Receiver,
                    },
                }
            }

            pub(crate) fn is_null(&self) -> bool {
                self.wire_expr.is_empty()
            }

            pub(crate) fn encode(
                &self,
                more: bool,
                writer: &mut ZBufWriter<'_>,
            ) -> ZResult<(), ZCodecError> {
                let mut data = [0u8; 256]; // Temporary buffer for the inner encoding, assume max 256 bytes
                let mut value = &mut data[..];
                let mut zriter = value.writer();

                let mut flags: u8 = 0;

                if self.wire_expr.has_suffix() {
                    flags |= 1;
                }

                if let Mapping::Sender = self.wire_expr.mapping {
                    flags |= 1 << 1;
                }

                encode_u8(&mut zriter, flags)?;
                encode_u16(&mut zriter, self.wire_expr.scope)?;
                if self.wire_expr.has_suffix() {
                    encode_str(&mut zriter, false, self.wire_expr.suffix)?;
                }

                let zbuf_len = 256 - zriter.remaining();
                let ext = WireExprExt {
                    value: &data[..zbuf_len],
                };

                ext.encode(more, writer)
            }

            pub(crate) fn decode(
                header: u8,
                reader: &mut ZBufReader<'a>,
            ) -> ZResult<(Self, bool), ZCodecError> {
                let (ext, more) = common::ext::WireExprExt::decode(header, reader)?;

                let mut zeader: ZBufReader<'a> = ext.value.reader();
                let flags = zeader.read_u8()?;

                let scope: ExprLen = decode_u16(&mut zeader)?;
                let suffix: &'a str = if imsg::has_flag(flags, 1) {
                    let len = zeader.remaining();
                    let zbuf: ZBuf<'a> = zeader.read_zbuf(len)?;

                    zbuf.as_str()?
                } else {
                    ""
                };

                let mapping = if imsg::has_flag(flags, 1 << 1) {
                    Mapping::Sender
                } else {
                    Mapping::Receiver
                };

                Ok((
                    common::ext::WireExprType {
                        wire_expr: WireExpr {
                            scope,
                            suffix,
                            mapping,
                        },
                    },
                    more,
                ))
            }

            #[cfg(test)]
            pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
                Self {
                    wire_expr: WireExpr::rand(zbuf),
                }
            }
        }
    }
}

pub(crate) mod keyexpr {
    use crate::{
        protocol::{
            ZCodecError,
            common::{extension, imsg},
            core::wire_expr::{ExprId, WireExpr},
            network::declare::{self, keyexpr},
            zcodec::{decode_u16, encode_u8, encode_u16},
        },
        result::ZResult,
        zbail,
        zbuf::{ZBufReader, ZBufWriter},
    };

    pub(crate) mod flag {
        pub(crate) const N: u8 = 1 << 5;

        pub(crate) const Z: u8 = 1 << 7;
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct DeclareKeyExpr<'a> {
        pub(crate) id: ExprId,
        pub(crate) wire_expr: WireExpr<'a>,
    }

    impl<'a> DeclareKeyExpr<'a> {
        pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
            let mut header = declare::id::D_KEYEXPR;
            if self.wire_expr.has_suffix() {
                header |= keyexpr::flag::N;
            }

            encode_u8(writer, header)?;
            encode_u16(writer, self.id)?;
            self.wire_expr.encode(writer)
        }

        pub(crate) fn decode(
            header: u8,
            reader: &mut ZBufReader<'a>,
        ) -> ZResult<Self, ZCodecError> {
            if imsg::mid(header) != declare::id::D_KEYEXPR {
                zbail!(ZCodecError::CouldNotRead);
            }

            let id: ExprId = decode_u16(reader)?;
            let wire_expr: WireExpr<'_> =
                WireExpr::decode(imsg::has_flag(header, keyexpr::flag::N), reader)?;

            let has_ext = imsg::has_flag(header, keyexpr::flag::Z);
            if has_ext {
                extension::skip_all("DeclareKeyExpr", reader)?;
            }

            Ok(keyexpr::DeclareKeyExpr { id, wire_expr })
        }

        #[cfg(test)]
        pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
            use rand::Rng;
            let mut rng = rand::thread_rng();

            let id: ExprId = rng.r#gen();
            let wire_expr = WireExpr::rand(zbuf);

            Self { id, wire_expr }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct UndeclareKeyExpr {
        pub(crate) id: ExprId,
    }

    impl UndeclareKeyExpr {
        pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
            let header = declare::id::U_KEYEXPR;

            encode_u8(writer, header)?;
            encode_u16(writer, self.id)
        }

        pub(crate) fn decode(
            header: u8,
            reader: &mut ZBufReader<'_>,
        ) -> ZResult<Self, ZCodecError> {
            if imsg::mid(header) != declare::id::U_KEYEXPR {
                zbail!(ZCodecError::CouldNotRead);
            }

            let id: ExprId = decode_u16(reader)?;

            let has_ext = imsg::has_flag(header, keyexpr::flag::Z);
            if has_ext {
                extension::skip_all("UndeclareKeyExpr", reader)?;
            }

            Ok(keyexpr::UndeclareKeyExpr { id })
        }

        #[cfg(test)]
        pub(crate) fn rand() -> Self {
            use rand::Rng;
            let mut rng = rand::thread_rng();

            let id: ExprId = rng.r#gen();

            Self { id }
        }
    }
}

pub(crate) mod subscriber {
    use super::*;
    use crate::{
        protocol::{
            ZCodecError,
            common::{
                extension::{self, iext},
                imsg,
            },
            core::{EntityId, wire_expr::WireExpr},
            network::{Mapping, declare},
            zcodec::{decode_u32, encode_u32},
        },
        result::ZResult,
        zbail,
        zbuf::{ZBufReader, ZBufWriter},
    };

    pub(crate) type SubscriberId = EntityId;

    pub(crate) mod flag {
        pub(crate) const N: u8 = 1 << 5;
        pub(crate) const M: u8 = 1 << 6;
        pub(crate) const Z: u8 = 1 << 7;
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct DeclareSubscriber<'a> {
        pub(crate) id: SubscriberId,
        pub(crate) wire_expr: WireExpr<'a>,
    }

    impl<'a> DeclareSubscriber<'a> {
        pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
            let mut header = declare::id::D_SUBSCRIBER;

            if self.wire_expr.mapping != Mapping::DEFAULT {
                header |= subscriber::flag::M;
            }

            if self.wire_expr.has_suffix() {
                header |= subscriber::flag::N;
            }

            encode_u8(writer, header)?;
            encode_u32(writer, self.id)?;
            self.wire_expr.encode(writer)
        }

        pub(crate) fn decode(
            header: u8,
            reader: &mut ZBufReader<'a>,
        ) -> ZResult<Self, ZCodecError> {
            if imsg::mid(header) != declare::id::D_SUBSCRIBER {
                zbail!(ZCodecError::CouldNotRead);
            }

            let id: subscriber::SubscriberId = decode_u32(reader)?;
            let mut wire_expr: WireExpr<'_> =
                WireExpr::decode(imsg::has_flag(header, subscriber::flag::N), reader)?;

            wire_expr.mapping = if imsg::has_flag(header, subscriber::flag::M) {
                Mapping::Sender
            } else {
                Mapping::Receiver
            };

            let mut has_ext = imsg::has_flag(header, subscriber::flag::Z);
            while has_ext {
                let ext: u8 = decode_u8(reader)?;
                has_ext = extension::skip("DeclareSubscriber", ext, reader)?;
            }

            Ok(subscriber::DeclareSubscriber { id, wire_expr })
        }

        #[cfg(test)]
        pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
            use rand::Rng;
            let mut rng = rand::thread_rng();

            let id: SubscriberId = rng.r#gen();
            let wire_expr = WireExpr::rand(zbuf);

            Self { id, wire_expr }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct UndeclareSubscriber<'a> {
        pub(crate) id: SubscriberId,
        pub(crate) ext_wire_expr: common::ext::WireExprType<'a>,
    }

    impl<'a> UndeclareSubscriber<'a> {
        pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
            let mut header = declare::id::U_SUBSCRIBER;

            if !self.ext_wire_expr.is_null() {
                header |= subscriber::flag::Z;
            }

            encode_u8(writer, header)?;
            encode_u32(writer, self.id)?;

            if !self.ext_wire_expr.is_null() {
                self.ext_wire_expr.encode(false, writer)?;
            }

            Ok(())
        }

        pub(crate) fn decode(
            header: u8,
            reader: &mut ZBufReader<'a>,
        ) -> ZResult<Self, ZCodecError> {
            if imsg::mid(header) != declare::id::U_SUBSCRIBER {
                zbail!(ZCodecError::CouldNotRead);
            }

            let id: subscriber::SubscriberId = decode_u32(reader)?;

            let mut ext_wire_expr = common::ext::WireExprType::null();

            let mut has_ext = imsg::has_flag(header, subscriber::flag::Z);
            while has_ext {
                let ext = decode_u8(reader)?;
                match iext::eheader(ext) {
                    common::ext::WireExprExt::ID => {
                        let (we, ext) = common::ext::WireExprType::decode(ext, reader)?;
                        ext_wire_expr = we;
                        has_ext = ext;
                    }
                    _ => {
                        has_ext = extension::skip("UndeclareSubscriber", ext, reader)?;
                    }
                }
            }

            Ok(subscriber::UndeclareSubscriber { id, ext_wire_expr })
        }

        #[cfg(test)]
        pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
            use rand::Rng;
            let mut rng = rand::thread_rng();

            let id: SubscriberId = rng.r#gen();
            let ext_wire_expr = common::ext::WireExprType::rand(zbuf);

            Self { id, ext_wire_expr }
        }
    }
}

pub(crate) mod queryable {
    use super::*;
    use crate::{
        protocol::{
            ZCodecError,
            common::{
                extension::{self, iext},
                imsg,
            },
            core::{EntityId, wire_expr::WireExpr},
            network::{Mapping, declare},
            zcodec::{decode_u32, encode_u32},
        },
        result::ZResult,
        zbail,
        zbuf::{ZBufReader, ZBufWriter},
    };

    pub(crate) type QueryableId = EntityId;

    pub(crate) mod flag {
        pub(crate) const N: u8 = 1 << 5;
        pub(crate) const M: u8 = 1 << 6;
        pub(crate) const Z: u8 = 1 << 7;
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct DeclareQueryable<'a> {
        pub(crate) id: QueryableId,
        pub(crate) wire_expr: WireExpr<'a>,
        pub(crate) ext_info: ext::QueryableInfoType,
    }

    impl<'a> DeclareQueryable<'a> {
        pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
            let mut header = declare::id::D_QUERYABLE;

            let mut n_exts = (self.ext_info != queryable::ext::QueryableInfoType::DEFAULT) as u8;

            if n_exts != 0 {
                header |= subscriber::flag::Z;
            }

            if self.wire_expr.mapping != Mapping::DEFAULT {
                header |= subscriber::flag::M;
            }

            if self.wire_expr.has_suffix() {
                header |= subscriber::flag::N;
            }

            encode_u8(writer, header)?;
            encode_u32(writer, self.id)?;
            self.wire_expr.encode(writer)?;

            if self.ext_info != queryable::ext::QueryableInfoType::DEFAULT {
                n_exts -= 1;
                self.ext_info.encode(n_exts != 0, writer)?;
            }

            Ok(())
        }

        pub(crate) fn decode(
            header: u8,
            reader: &mut ZBufReader<'a>,
        ) -> ZResult<Self, ZCodecError> {
            if imsg::mid(header) != declare::id::D_QUERYABLE {
                zbail!(ZCodecError::CouldNotRead);
            }

            let id: queryable::QueryableId = decode_u32(reader)?;
            let mut wire_expr: WireExpr<'_> =
                WireExpr::decode(imsg::has_flag(header, queryable::flag::N), reader)?;

            wire_expr.mapping = if imsg::has_flag(header, queryable::flag::M) {
                Mapping::Sender
            } else {
                Mapping::Receiver
            };

            let mut ext_info = queryable::ext::QueryableInfoType::DEFAULT;

            let mut has_ext = imsg::has_flag(header, queryable::flag::Z);
            while has_ext {
                let ext: u8 = decode_u8(reader)?;
                match iext::eheader(ext) {
                    queryable::ext::QueryableInfo::ID => {
                        let (i, ext) = queryable::ext::QueryableInfoType::decode(ext, reader)?;
                        ext_info = i;
                        has_ext = ext;
                    }
                    _ => {
                        has_ext = extension::skip("DeclareQueryable", ext, reader)?;
                    }
                }
            }

            Ok(queryable::DeclareQueryable {
                id,
                wire_expr,
                ext_info,
            })
        }

        #[cfg(test)]
        pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
            use rand::Rng;
            let mut rng = rand::thread_rng();

            let id: QueryableId = rng.r#gen();
            let wire_expr = WireExpr::rand(zbuf);
            let ext_info = ext::QueryableInfoType::rand();

            Self {
                id,
                wire_expr,
                ext_info,
            }
        }
    }

    pub(crate) mod ext {
        use crate::{
            protocol::{ZCodecError, common::imsg, network::declare::queryable},
            result::ZResult,
            zbuf::{ZBufReader, ZBufWriter},
        };

        pub(crate) type QueryableInfo = crate::zextz64!(0x01, false);

        pub(crate) mod flag {
            pub(crate) const C: u8 = 1;
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub(crate) struct QueryableInfoType {
            pub(crate) complete: bool,
            pub(crate) distance: u16,
        }

        impl QueryableInfoType {
            pub(crate) const DEFAULT: Self = Self {
                complete: false,
                distance: 0,
            };

            pub(crate) fn encode(
                &self,
                more: bool,
                writer: &mut ZBufWriter<'_>,
            ) -> ZResult<(), ZCodecError> {
                let mut flags: u8 = 0;
                if self.complete {
                    flags |= queryable::ext::flag::C;
                }
                let v: u64 = (flags as u64) | ((self.distance as u64) << 8);
                let ext = queryable::ext::QueryableInfo::new(v);

                ext.encode(more, writer)
            }

            pub(crate) fn decode(
                header: u8,
                reader: &mut ZBufReader<'_>,
            ) -> ZResult<(Self, bool), ZCodecError> {
                let (ext, more) = queryable::ext::QueryableInfo::decode(header, reader)?;

                let complete = imsg::has_flag(ext.value as u8, queryable::ext::flag::C);
                let distance = (ext.value >> 8) as u16;

                Ok((
                    queryable::ext::QueryableInfoType { complete, distance },
                    more,
                ))
            }

            #[cfg(test)]
            pub(crate) fn rand() -> Self {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let complete: bool = rng.gen_bool(0.5);
                let distance: u16 = rng.r#gen();

                Self { complete, distance }
            }
        }

        impl Default for QueryableInfoType {
            fn default() -> Self {
                Self::DEFAULT
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct UndeclareQueryable<'a> {
        pub(crate) id: QueryableId,
        pub(crate) ext_wire_expr: common::ext::WireExprType<'a>,
    }

    impl<'a> UndeclareQueryable<'a> {
        pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
            let header = declare::id::U_QUERYABLE | queryable::flag::Z;
            encode_u8(writer, header)?;
            encode_u32(writer, self.id)?;
            self.ext_wire_expr.encode(false, writer)
        }

        pub(crate) fn decode(
            header: u8,
            reader: &mut ZBufReader<'a>,
        ) -> ZResult<Self, ZCodecError> {
            if imsg::mid(header) != declare::id::U_QUERYABLE {
                zbail!(ZCodecError::CouldNotRead);
            }

            let id: queryable::QueryableId = decode_u32(reader)?;

            let mut ext_wire_expr = common::ext::WireExprType::null();

            let mut has_ext = imsg::has_flag(header, queryable::flag::Z);
            while has_ext {
                let ext = decode_u8(reader)?;
                match iext::eheader(ext) {
                    common::ext::WireExprExt::ID => {
                        let (we, ext) = common::ext::WireExprType::decode(ext, reader)?;
                        ext_wire_expr = we;
                        has_ext = ext;
                    }
                    _ => {
                        has_ext = extension::skip("UndeclareQueryable", ext, reader)?;
                    }
                }
            }

            Ok(queryable::UndeclareQueryable { id, ext_wire_expr })
        }

        #[cfg(test)]
        pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
            use rand::Rng;
            let mut rng = rand::thread_rng();

            let id: QueryableId = rng.r#gen();
            let ext_wire_expr = common::ext::WireExprType::rand(zbuf);

            Self { id, ext_wire_expr }
        }
    }
}

pub(crate) mod token {
    pub(crate) mod flag {
        pub(crate) const Z: u8 = 1 << 7;
    }
}
