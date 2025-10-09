use core::{
    fmt::{self, Debug},
    ops::{Add, AddAssign, Sub, SubAssign},
    sync::atomic::AtomicU32,
};

use crate::{
    protocol::{
        ZCodecError,
        common::{
            extension::{self, iext},
            imsg::{self, HEADER_BITS},
        },
        core::wire_expr::WireExpr,
        network::{Mapping, declare, id, interest},
        zcodec::{decode_u32, encode_u8, encode_u32},
    },
    result::ZResult,
    zbail,
    zbuf::{ZBufReader, ZBufWriter},
};

pub type InterestId = u32;

pub mod flag {
    pub const Z: u8 = 1 << 7;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interest<'a> {
    pub id: InterestId,
    pub mode: InterestMode,
    pub options: InterestOptions,
    pub wire_expr: Option<WireExpr<'a>>,
    pub ext_qos: ext::QoSType,
    pub ext_tstamp: Option<ext::TimestampType>,
    pub ext_nodeid: ext::NodeIdType,
}

impl<'a> Interest<'a> {
    pub fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        let mut header = id::INTEREST;
        header |= match self.mode {
            InterestMode::Final => 0b00,
            InterestMode::Current => 0b01,
            InterestMode::Future => 0b10,
            InterestMode::CurrentFuture => 0b11,
        } << HEADER_BITS;

        let mut n_exts = ((self.ext_qos != declare::ext::QoSType::DEFAULT) as u8)
            + (self.ext_tstamp.is_some() as u8)
            + ((self.ext_nodeid != declare::ext::NodeIdType::DEFAULT) as u8);

        if n_exts != 0 {
            header |= declare::flag::Z;
        }

        crate::protocol::zcodec::encode_u8(header, writer)?;
        encode_u32(self.id, writer)?;

        if self.mode != InterestMode::Final {
            encode_u8(self.options(), writer)?;
            if let Some(we) = self.wire_expr.as_ref() {
                we.encode(writer)?;
            }
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

        Ok(())
    }

    pub fn decode(header: u8, reader: &mut ZBufReader<'a>) -> ZResult<Self, ZCodecError> {
        if imsg::mid(header) != id::INTEREST {
            zbail!(ZCodecError::Invalid);
        }

        let id = decode_u32(reader)?;
        let mode = match (header >> HEADER_BITS) & 0b11 {
            0b00 => InterestMode::Final,
            0b01 => InterestMode::Current,
            0b10 => InterestMode::Future,
            0b11 => InterestMode::CurrentFuture,
            _ => zbail!(ZCodecError::Invalid),
        };

        let mut options = InterestOptions::empty();
        let mut wire_expr = None;
        if mode != InterestMode::Final {
            let options_byte = crate::protocol::zcodec::decode_u8(reader)?;
            options = InterestOptions::from(options_byte);
            if options.restricted() {
                let mut we: WireExpr<'_> = WireExpr::decode(options.named(), reader)?;
                we.mapping = if options.mapping() {
                    Mapping::Sender
                } else {
                    Mapping::Receiver
                };
                wire_expr = Some(we);
            }
        }

        let mut ext_qos = declare::ext::QoSType::DEFAULT;
        let mut ext_tstamp = None;
        let mut ext_nodeid = declare::ext::NodeIdType::DEFAULT;

        let mut has_ext = imsg::has_flag(header, declare::flag::Z);
        while has_ext {
            let ext = crate::protocol::zcodec::decode_u8(reader)?;
            match iext::eid(ext) {
                declare::ext::QoS::ID => {
                    let (q, ext) = interest::ext::QoSType::decode(ext, reader)?;

                    ext_qos = q;
                    has_ext = ext;
                }
                declare::ext::Timestamp::ID => {
                    let (t, ext) = interest::ext::TimestampType::decode(ext, reader)?;
                    ext_tstamp = Some(t);
                    has_ext = ext;
                }
                declare::ext::NodeId::ID => {
                    let (nid, ext) = interest::ext::NodeIdType::decode(ext, reader)?;
                    ext_nodeid = nid;
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip("Declare", ext, reader)?;
                }
            }
        }

        Ok(Interest {
            id,
            mode,
            options,
            wire_expr,
            ext_qos,
            ext_tstamp,
            ext_nodeid,
        })
    }

    #[cfg(test)]
    pub fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let id = rng.r#gen::<InterestId>();
        let mode = InterestMode::rand();
        let options = if mode == InterestMode::Final {
            InterestOptions::empty()
        } else {
            InterestOptions::rand()
        };
        let wire_expr = options.restricted().then_some(WireExpr::rand(zbuf));
        let ext_qos = ext::QoSType::rand();
        let ext_tstamp = rng.gen_bool(0.5).then(ext::TimestampType::rand);
        let ext_nodeid = ext::NodeIdType::rand();

        Self {
            id,
            mode,
            wire_expr,
            options,
            ext_qos,
            ext_tstamp,
            ext_nodeid,
        }
    }
}

pub type DeclareRequestId = u32;
pub type AtomicDeclareRequestId = AtomicU32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterestMode {
    Final,
    Current,
    Future,
    CurrentFuture,
}

impl InterestMode {
    #[cfg(test)]
    pub fn rand() -> Self {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        match rng.gen_range(0..4) {
            0 => InterestMode::Final,
            1 => InterestMode::Current,
            2 => InterestMode::Future,
            3 => InterestMode::CurrentFuture,
            _ => unreachable!(),
        }
    }
}

pub mod ext {
    pub type QoS = crate::zextz64!(0x1, false);
    pub type QoSType = crate::protocol::network::ext::QoSType<{ QoS::ID }>;

    pub type Timestamp<'a> = crate::zextzbuf!('a, 0x2, false);
    pub type TimestampType = crate::protocol::network::ext::TimestampType<{ Timestamp::ID }>;

    pub type NodeId = crate::zextz64!(0x3, true);
    pub type NodeIdType = crate::protocol::network::ext::NodeIdType<{ NodeId::ID }>;
}

impl Interest<'_> {
    pub fn options(&self) -> u8 {
        let mut interest = self.options;
        if let Some(we) = self.wire_expr.as_ref() {
            interest += InterestOptions::RESTRICTED;
            if we.has_suffix() {
                interest += InterestOptions::NAMED;
            }
            if let Mapping::Sender = we.mapping {
                interest += InterestOptions::MAPPING;
            }
        }
        interest.options
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct InterestOptions {
    options: u8,
}

impl InterestOptions {
    pub const KEYEXPRS: InterestOptions = InterestOptions::options(1);
    pub const SUBSCRIBERS: InterestOptions = InterestOptions::options(1 << 1);
    pub const QUERYABLES: InterestOptions = InterestOptions::options(1 << 2);
    pub const TOKENS: InterestOptions = InterestOptions::options(1 << 3);
    const RESTRICTED: InterestOptions = InterestOptions::options(1 << 4);
    const NAMED: InterestOptions = InterestOptions::options(1 << 5);
    const MAPPING: InterestOptions = InterestOptions::options(1 << 6);
    pub const AGGREGATE: InterestOptions = InterestOptions::options(1 << 7);
    pub const ALL: InterestOptions = InterestOptions::options(
        InterestOptions::KEYEXPRS.options
            | InterestOptions::SUBSCRIBERS.options
            | InterestOptions::QUERYABLES.options
            | InterestOptions::TOKENS.options,
    );

    const fn options(options: u8) -> Self {
        Self { options }
    }

    pub const fn empty() -> Self {
        Self { options: 0 }
    }

    pub const fn keyexprs(&self) -> bool {
        imsg::has_flag(self.options, Self::KEYEXPRS.options)
    }

    pub const fn subscribers(&self) -> bool {
        imsg::has_flag(self.options, Self::SUBSCRIBERS.options)
    }

    pub const fn queryables(&self) -> bool {
        imsg::has_flag(self.options, Self::QUERYABLES.options)
    }

    pub const fn tokens(&self) -> bool {
        imsg::has_flag(self.options, Self::TOKENS.options)
    }

    pub const fn restricted(&self) -> bool {
        imsg::has_flag(self.options, Self::RESTRICTED.options)
    }

    pub const fn named(&self) -> bool {
        imsg::has_flag(self.options, Self::NAMED.options)
    }

    pub const fn mapping(&self) -> bool {
        imsg::has_flag(self.options, Self::MAPPING.options)
    }

    pub const fn aggregate(&self) -> bool {
        imsg::has_flag(self.options, Self::AGGREGATE.options)
    }

    #[cfg(test)]
    pub fn rand() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let mut s = Self::empty();
        if rng.gen_bool(0.5) {
            s += InterestOptions::KEYEXPRS;
        }
        if rng.gen_bool(0.5) {
            s += InterestOptions::SUBSCRIBERS;
        }
        if rng.gen_bool(0.5) {
            s += InterestOptions::TOKENS;
        }
        if rng.gen_bool(0.5) {
            s += InterestOptions::AGGREGATE;
        }
        s
    }
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

impl Debug for InterestOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Interest {{ ")?;
        if self.keyexprs() {
            write!(f, "K:Y, ")?;
        } else {
            write!(f, "K:N, ")?;
        }
        if self.subscribers() {
            write!(f, "S:Y, ")?;
        } else {
            write!(f, "S:N, ")?;
        }
        if self.queryables() {
            write!(f, "Q:Y, ")?;
        } else {
            write!(f, "Q:N, ")?;
        }
        if self.tokens() {
            write!(f, "T:Y, ")?;
        } else {
            write!(f, "T:N, ")?;
        }
        if self.aggregate() {
            write!(f, "A:Y")?;
        } else {
            write!(f, "A:N")?;
        }
        write!(f, " }}")?;
        Ok(())
    }
}

impl Eq for InterestOptions {}

impl Add for InterestOptions {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            options: self.options | rhs.options,
        }
    }
}

impl AddAssign for InterestOptions {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn add_assign(&mut self, rhs: Self) {
        self.options |= rhs.options;
    }
}

impl Sub for InterestOptions {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            options: self.options & !rhs.options,
        }
    }
}

impl SubAssign for InterestOptions {
    fn sub_assign(&mut self, rhs: Self) {
        self.options &= !rhs.options;
    }
}

impl From<u8> for InterestOptions {
    fn from(options: u8) -> Self {
        Self { options }
    }
}
