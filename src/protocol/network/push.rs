use crate::{
    protocol::{
        ZCodecError,
        common::{
            extension::{self, iext},
            imsg,
        },
        core::wire_expr::WireExpr,
        network::Mapping,
        zcodec::{decode_u8, encode_u8},
        zenoh::PushBody,
    },
    result::ZResult,
    zbail,
    zbuf::{ZBufReader, ZBufWriter},
};

// pub(crate) mod flag {
//     /// Indicates the presence of a suffix in the Wire Expression
//     pub(crate) const N: u8 = 1 << 5;
//     /// Indicates that the mapping is from the sender's perspective
//     pub(crate) const M: u8 = 1 << 6;
//     /// Indicates the presence of extensions
//     pub(crate) const Z: u8 = 1 << 7;
// }

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Push<'a> {
    pub(crate) wire_expr: WireExpr<'a>,

    pub(crate) ext_qos: ext::QoSType,
    pub(crate) ext_tstamp: Option<ext::TimestampType>,
    pub(crate) ext_nodeid: ext::NodeIdType,

    pub(crate) payload: PushBody<'a>,
}

impl<'a> Push<'a> {
    // ---------- Header for Push Message ----------------

    /// Message ID for Push messages
    pub(crate) const ID: u8 = 0x1d;

    /// Indicates the presence of a suffix in the Wire Expression
    pub(crate) const FLAG_N: u8 = 1 << 5;
    /// Indicates that the mapping is from the sender's perspective
    pub(crate) const FLAG_M: u8 = 1 << 6;
    /// Indicates the presence of extensions
    pub(crate) const FLAG_Z: u8 = 1 << 7;

    // ----------------------------------------------------
}

impl<'a> Push<'a> {
    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        let mut header = Self::ID;
        let mut n_exts = ((self.ext_qos != ext::QoSType::DEFAULT) as u8)
            + (self.ext_tstamp.is_some() as u8)
            + ((self.ext_nodeid != ext::NodeIdType::DEFAULT) as u8);

        if n_exts != 0 {
            header |= Self::FLAG_Z;
        }

        if self.wire_expr.mapping != Mapping::DEFAULT {
            header |= Self::FLAG_M;
        }

        if self.wire_expr.has_suffix() {
            header |= Self::FLAG_N;
        }

        encode_u8(writer, header)?;
        self.wire_expr.encode(writer)?;

        if self.ext_qos != ext::QoSType::DEFAULT {
            n_exts -= 1;
            self.ext_qos.encode(n_exts != 0, writer)?;
        }

        if let Some(ts) = self.ext_tstamp.as_ref() {
            n_exts -= 1;
            ts.encode(n_exts != 0, writer)?;
        }

        if self.ext_nodeid != ext::NodeIdType::DEFAULT {
            n_exts -= 1;
            self.ext_nodeid.encode(n_exts != 0, writer)?;
        }

        self.payload.encode(writer)
    }

    pub(crate) fn decode(header: u8, reader: &mut ZBufReader<'a>) -> ZResult<Self, ZCodecError> {
        if imsg::mid(header) != Self::ID {
            zbail!(ZCodecError::CouldNotRead);
        }

        let mut wire_expr: WireExpr<'_> =
            WireExpr::decode(imsg::has_flag(header, Self::FLAG_N), reader)?;

        wire_expr.mapping = if imsg::has_flag(header, Self::FLAG_M) {
            Mapping::Sender
        } else {
            Mapping::Receiver
        };

        let mut ext_qos = ext::QoSType::DEFAULT;
        let mut ext_tstamp = None;
        let mut ext_nodeid = ext::NodeIdType::DEFAULT;

        let mut has_ext = imsg::has_flag(header, Self::FLAG_Z);
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
                _ => {
                    has_ext = extension::skip("Push", ext, reader)?;
                }
            }
        }

        let payload = PushBody::decode(reader)?;

        Ok(Push {
            wire_expr,
            payload,
            ext_qos,
            ext_tstamp,
            ext_nodeid,
        })
    }

    #[cfg(test)]
    pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        use rand::Rng;

        let mut rng = rand::thread_rng();
        let wire_expr = WireExpr::rand(zbuf);
        let payload = PushBody::rand(zbuf);
        let ext_qos = ext::QoSType::rand();
        let ext_tstamp = rng.gen_bool(0.5).then(ext::TimestampType::rand);
        let ext_nodeid = ext::NodeIdType::rand();

        Self {
            wire_expr,
            payload,
            ext_tstamp,
            ext_qos,
            ext_nodeid,
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
