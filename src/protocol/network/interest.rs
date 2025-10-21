use core::{
    fmt::{self, Debug},
    ops::{Add, AddAssign, Sub, SubAssign},
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
        zcodec::{decode_u8, decode_u32, encode_u8, encode_u32},
    },
    result::ZResult,
    zbail,
    zbuf::{ZBufReader, ZBufWriter},
};

pub(crate) type InterestId = u32;

pub(crate) mod flag {
    pub(crate) const Z: u8 = 1 << 7;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Interest<'a> {
    pub(crate) id: InterestId,
    pub(crate) mode: InterestMode,
    pub(crate) options: InterestOptions,
    pub(crate) wire_expr: Option<WireExpr<'a>>,
    pub(crate) ext_qos: ext::QoSType,
    pub(crate) ext_tstamp: Option<ext::TimestampType>,
    pub(crate) ext_nodeid: ext::NodeIdType,
}

impl<'a> Interest<'a> {
    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
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
            header |= interest::flag::Z;
        }

        encode_u8(writer, header)?;
        encode_u32(writer, self.id)?;

        if self.mode != InterestMode::Final {
            encode_u8(writer, self.options())?;
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

    pub(crate) fn decode(header: u8, reader: &mut ZBufReader<'a>) -> ZResult<Self, ZCodecError> {
        if imsg::mid(header) != id::INTEREST {
            zbail!(ZCodecError::CouldNotRead);
        }

        let id = decode_u32(reader)?;
        let mode = match (header >> HEADER_BITS) & 0b11 {
            0b00 => InterestMode::Final,
            0b01 => InterestMode::Current,
            0b10 => InterestMode::Future,
            0b11 => InterestMode::CurrentFuture,
            _ => zbail!(ZCodecError::CouldNotRead),
        };

        let mut options = InterestOptions::empty();
        let mut wire_expr = None;
        if mode != InterestMode::Final {
            let options_byte = decode_u8(reader)?;
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

        let mut has_ext = imsg::has_flag(header, interest::flag::Z);
        while has_ext {
            let ext = decode_u8(reader)?;
            match iext::eheader(ext) {
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
    pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InterestMode {
    Final,
    Current,
    Future,
    CurrentFuture,
}

impl InterestMode {
    #[cfg(test)]
    pub(crate) fn rand() -> Self {
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

pub(crate) mod ext {
    pub(crate) type QoS = crate::zextz64!(0x1, false);
    pub(crate) type QoSType = crate::protocol::network::ext::QoSType<{ QoS::ID }>;

    pub(crate) type Timestamp<'a> = crate::zextzbuf!('a, 0x2, false);
    pub(crate) type TimestampType = crate::protocol::network::ext::TimestampType<{ Timestamp::ID }>;

    pub(crate) type NodeId = crate::zextz64!(0x3, true);
    pub(crate) type NodeIdType = crate::protocol::network::ext::NodeIdType<{ NodeId::ID }>;
}

impl Interest<'_> {
    pub(crate) fn options(&self) -> u8 {
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
pub(crate) struct InterestOptions {
    options: u8,
}

impl InterestOptions {
    pub(crate) const KEYEXPRS: InterestOptions = InterestOptions::options(1);
    pub(crate) const SUBSCRIBERS: InterestOptions = InterestOptions::options(1 << 1);
    pub(crate) const QUERYABLES: InterestOptions = InterestOptions::options(1 << 2);
    pub(crate) const TOKENS: InterestOptions = InterestOptions::options(1 << 3);
    const RESTRICTED: InterestOptions = InterestOptions::options(1 << 4);
    const NAMED: InterestOptions = InterestOptions::options(1 << 5);
    const MAPPING: InterestOptions = InterestOptions::options(1 << 6);
    pub(crate) const AGGREGATE: InterestOptions = InterestOptions::options(1 << 7);

    const fn options(options: u8) -> Self {
        Self { options }
    }

    pub(crate) const fn empty() -> Self {
        Self { options: 0 }
    }

    pub(crate) const fn keyexprs(&self) -> bool {
        imsg::has_flag(self.options, Self::KEYEXPRS.options)
    }

    pub(crate) const fn subscribers(&self) -> bool {
        imsg::has_flag(self.options, Self::SUBSCRIBERS.options)
    }

    pub(crate) const fn queryables(&self) -> bool {
        imsg::has_flag(self.options, Self::QUERYABLES.options)
    }

    pub(crate) const fn tokens(&self) -> bool {
        imsg::has_flag(self.options, Self::TOKENS.options)
    }

    pub(crate) const fn restricted(&self) -> bool {
        imsg::has_flag(self.options, Self::RESTRICTED.options)
    }

    pub(crate) const fn named(&self) -> bool {
        imsg::has_flag(self.options, Self::NAMED.options)
    }

    pub(crate) const fn mapping(&self) -> bool {
        imsg::has_flag(self.options, Self::MAPPING.options)
    }

    pub(crate) const fn aggregate(&self) -> bool {
        imsg::has_flag(self.options, Self::AGGREGATE.options)
    }

    #[cfg(test)]
    pub(crate) fn rand() -> Self {
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
