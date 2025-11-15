use uhlc::Timestamp;

use crate::{ZBodyDecode, ZDecode, ZEncode, ZStruct, ZWriter};
#[cfg(test)]
use rand::{Rng, thread_rng};

use crate::{
    WireExpr, ZBodyEncode, ZBodyLen, ZHeader, ZLen,
    network::{NodeId, QoS},
    zbail,
};

impl InterestInner<'_> {
    const HEADER_SLOT_FULL: u8 = 0b1111_1111;
}

#[derive(ZStruct, Debug)]
#[zenoh(header = "A|M|N|R|T|Q|S|K")]
pub struct InterestInner<'a> {
    #[zenoh(header = FULL)]
    pub options: u8,

    #[zenoh(presence = header(R), flatten, shift = 5)]
    pub wire_expr: Option<WireExpr<'a>>,
}

impl PartialEq for InterestInner<'_> {
    fn eq(&self, other: &Self) -> bool {
        let options = InterestOptions::options(self.options);
        let other_options = InterestOptions::options(other.options);

        options == other_options && self.wire_expr == other.wire_expr
    }
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|_:7")]
pub struct InterestExt {
    #[zenoh(ext = 0x1, default = QoS::DEFAULT)]
    pub qos: QoS,
    #[zenoh(ext = 0x2)]
    pub timestamp: Option<Timestamp>,
    #[zenoh(ext = 0x3, default = NodeId::DEFAULT, mandatory)]
    pub nodeid: NodeId,
}

#[derive(Debug, PartialEq)]
// #[zenoh(header = "Z|MODE|_:5=0x19")]
pub struct Interest<'a> {
    pub id: u32,
    pub mode: InterestMode,

    pub inner: InterestInner<'a>,
    // #[zenoh(flatten)]
    pub ext: InterestExt,
}

impl Interest<'_> {
    const HEADER_BASE: u8 = 25u8;
    pub const ID: u8 = 25u8;
}

impl ZHeader for Interest<'_> {
    fn z_header(&self) -> u8 {
        let mut header: u8 = Self::HEADER_BASE;

        header |= match self.mode {
            InterestMode::Final => 0b00,
            InterestMode::Current => 0b01,
            InterestMode::Future => 0b10,
            InterestMode::CurrentFuture => 0b11,
        } << 5;

        header |= <_ as ZHeader>::z_header(&self.ext);

        header
    }
}

impl ZBodyLen for Interest<'_> {
    fn z_body_len(&self) -> usize {
        <u32 as ZLen>::z_len(&self.id)
            + if self.mode != InterestMode::Final {
                <_ as ZLen>::z_len(&self.inner)
            } else {
                0usize
            }
            + self.ext.z_body_len()
    }
}

impl ZBodyEncode for Interest<'_> {
    fn z_body_encode(&self, w: &mut ZWriter) -> crate::ZCodecResult<()> {
        <u32 as ZBodyEncode>::z_body_encode(&self.id, w)?;

        if self.mode != InterestMode::Final {
            <_ as ZEncode>::z_encode(&self.inner, w)?;
        }

        <_ as ZBodyEncode>::z_body_encode(&self.ext, w)?;

        Ok(())
    }
}

impl<'a> ZBodyDecode<'a> for Interest<'a> {
    type Ctx = u8;

    fn z_body_decode(r: &mut crate::ZReader<'a>, header: u8) -> crate::ZCodecResult<Self> {
        let id = <u32 as ZDecode>::z_decode(r)?;

        let mode = match (header >> 5) & 0b11 {
            0b00 => InterestMode::Final,
            0b01 => InterestMode::Current,
            0b10 => InterestMode::Future,
            0b11 => InterestMode::CurrentFuture,
            _ => zbail!(crate::ZCodecError::CouldNotParse),
        };

        let inner = if mode != InterestMode::Final {
            <_ as ZDecode>::z_decode(r)?
        } else {
            InterestInner {
                options: 0,
                wire_expr: None,
            }
        };

        let ext = <_ as ZBodyDecode>::z_body_decode(r, header)?;

        Ok(Self {
            id,
            mode,
            inner,
            ext,
        })
    }
}

impl ZLen for Interest<'_> {
    fn z_len(&self) -> usize {
        1usize + <Self as ZBodyLen>::z_body_len(self)
    }
}

impl ZEncode for Interest<'_> {
    fn z_encode(&self, w: &mut ZWriter) -> crate::ZCodecResult<()> {
        let header = <Self as ZHeader>::z_header(self);
        <u8 as ZEncode>::z_encode(&header, w)?;
        <Self as ZBodyEncode>::z_body_encode(self, w)
    }
}

impl<'a> ZDecode<'a> for Interest<'a> {
    fn z_decode(r: &mut crate::ZReader<'a>) -> crate::ZCodecResult<Self> {
        let header = <u8 as ZDecode>::z_decode(r)?;
        <Self as ZBodyDecode>::z_body_decode(r, header)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterestMode {
    Final,
    Current,
    Future,
    CurrentFuture,
}

impl InterestMode {
    #[cfg(test)]
    pub fn rand(_: &mut ZWriter) -> Self {
        match thread_rng().gen_range(0..4) {
            0 => InterestMode::Final,
            1 => InterestMode::Current,
            2 => InterestMode::Future,
            3 => InterestMode::CurrentFuture,
            _ => unreachable!(),
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct InterestOptions {
    options: u8,
}

impl PartialEq for InterestOptions {
    fn eq(&self, other: &Self) -> bool {
        self.keyexprs() == other.keyexprs()
            && self.subscribers() == other.subscribers()
            && self.queryables() == other.queryables()
            && self.tokens() == other.tokens()
            && self.aggregate() == other.aggregate()
    }
}

impl InterestOptions {
    pub const KEYEXPRS: InterestOptions = InterestOptions::options(1);
    pub const SUBSCRIBERS: InterestOptions = InterestOptions::options(1 << 1);
    pub const QUERYABLES: InterestOptions = InterestOptions::options(1 << 2);
    pub const TOKENS: InterestOptions = InterestOptions::options(1 << 3);

    pub const AGGREGATE: InterestOptions = InterestOptions::options(1 << 7);

    const fn options(options: u8) -> Self {
        Self { options }
    }

    pub const fn keyexprs(&self) -> bool {
        self.options & Self::KEYEXPRS.options != 0
    }

    pub const fn subscribers(&self) -> bool {
        self.options & Self::SUBSCRIBERS.options != 0
    }

    pub const fn queryables(&self) -> bool {
        self.options & Self::QUERYABLES.options != 0
    }

    pub const fn tokens(&self) -> bool {
        self.options & Self::TOKENS.options != 0
    }

    pub const fn aggregate(&self) -> bool {
        self.options & Self::AGGREGATE.options != 0
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
    pub(crate) fn rand(w: &mut ZWriter) -> Self {
        let qos = if thread_rng().gen_bool(0.5) {
            QoS::rand(w)
        } else {
            QoS::DEFAULT
        };

        let timestamp = thread_rng().gen_bool(0.5).then_some({
            use crate::protocol::core::ZenohIdProto;

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
