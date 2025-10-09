use core::{
    convert::{From, TryFrom, TryInto},
    fmt::{self, Display},
    hash::Hash,
    ops::{Deref, RangeInclusive},
    str::FromStr,
};

pub use uhlc::{NTP64, Timestamp};

use crate::{
    protocol::{
        ZCodecError, ZProtocolError,
        zcodec::{decode_zbuf, encode_zbuf, encoded_len_zbuf},
    },
    result::ZResult,
    zbuf::{ZBufReader, ZBufWriter},
};

pub type TimestampId = uhlc::ID;

pub mod encoding;
pub mod endpoint;
pub mod resolution;
pub mod whatami;
pub mod wire_expr;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ZenohIdProto(uhlc::ID);

impl ZenohIdProto {
    pub const MAX_SIZE: usize = 16;

    #[inline]
    pub fn size(&self) -> usize {
        self.0.size()
    }

    #[inline]
    pub fn to_le_bytes(&self) -> [u8; uhlc::ID::MAX_SIZE] {
        self.0.to_le_bytes()
    }

    pub fn rand() -> ZenohIdProto {
        ZenohIdProto(uhlc::ID::rand())
    }

    pub fn encoded_len(&self, len: bool) -> usize {
        encoded_len_zbuf(len, &self.to_le_bytes()[..self.size()])
    }

    pub fn encode(&self, len: bool, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        encode_zbuf(len, &self.to_le_bytes()[..self.size()], writer)
    }

    pub fn decode(len: Option<usize>, reader: &mut ZBufReader<'_>) -> ZResult<Self, ZCodecError> {
        let zbuf = decode_zbuf(len, reader)?;

        Ok(Self::try_from(zbuf)?)
    }
}

impl Default for ZenohIdProto {
    fn default() -> Self {
        Self::rand()
    }
}

macro_rules! derive_tryfrom {
    ($T: ty) => {
        impl TryFrom<$T> for ZenohIdProto {
            type Error = crate::protocol::ZProtocolError;
            fn try_from(val: $T) -> crate::result::ZResult<Self, crate::protocol::ZProtocolError> {
                match val.try_into() {
                    Ok(ok) => Ok(Self(ok)),
                    Err(_) => crate::zbail!(crate::protocol::ZProtocolError::Invalid),
                }
            }
        }
    };
}

derive_tryfrom!([u8; 1]);
derive_tryfrom!(&[u8; 1]);
derive_tryfrom!([u8; 2]);
derive_tryfrom!(&[u8; 2]);
derive_tryfrom!([u8; 3]);
derive_tryfrom!(&[u8; 3]);
derive_tryfrom!([u8; 4]);
derive_tryfrom!(&[u8; 4]);
derive_tryfrom!([u8; 5]);
derive_tryfrom!(&[u8; 5]);
derive_tryfrom!([u8; 6]);
derive_tryfrom!(&[u8; 6]);
derive_tryfrom!([u8; 7]);
derive_tryfrom!(&[u8; 7]);
derive_tryfrom!([u8; 8]);
derive_tryfrom!(&[u8; 8]);
derive_tryfrom!([u8; 9]);
derive_tryfrom!(&[u8; 9]);
derive_tryfrom!([u8; 10]);
derive_tryfrom!(&[u8; 10]);
derive_tryfrom!([u8; 11]);
derive_tryfrom!(&[u8; 11]);
derive_tryfrom!([u8; 12]);
derive_tryfrom!(&[u8; 12]);
derive_tryfrom!([u8; 13]);
derive_tryfrom!(&[u8; 13]);
derive_tryfrom!([u8; 14]);
derive_tryfrom!(&[u8; 14]);
derive_tryfrom!([u8; 15]);
derive_tryfrom!(&[u8; 15]);
derive_tryfrom!([u8; 16]);
derive_tryfrom!(&[u8; 16]);
derive_tryfrom!(&[u8]);

impl FromStr for ZenohIdProto {
    type Err = crate::protocol::ZProtocolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(|c: char| c.is_ascii_uppercase()) {
            crate::zbail!(crate::protocol::ZProtocolError::Invalid);
        }

        let u: uhlc::ID = s.parse().map_err(|_| ZProtocolError::Invalid)?;

        Ok(ZenohIdProto(u))
    }
}

impl fmt::Debug for ZenohIdProto {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for ZenohIdProto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl From<&ZenohIdProto> for uhlc::ID {
    fn from(zid: &ZenohIdProto) -> Self {
        zid.0
    }
}

impl From<ZenohIdProto> for uhlc::ID {
    fn from(zid: ZenohIdProto) -> Self {
        zid.0
    }
}

pub type EntityId = u32;

#[derive(Debug, Default, Copy, Clone, Eq, Hash, PartialEq)]
pub struct EntityGlobalIdProto {
    pub zid: ZenohIdProto,
    pub eid: EntityId,
}

impl EntityGlobalIdProto {
    #[cfg(test)]
    pub fn rand() -> Self {
        use rand::Rng;
        Self {
            zid: ZenohIdProto::rand(),
            eid: rand::thread_rng().r#gen(),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Copy, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum Priority {
    Control = 0,
    RealTime = 1,
    InteractiveHigh = 2,
    InteractiveLow = 3,
    DataHigh = 4,
    #[default]
    Data = 5,
    DataLow = 6,
    Background = 7,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]

pub struct PriorityRange(RangeInclusive<Priority>);

impl Deref for PriorityRange {
    type Target = RangeInclusive<Priority>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PriorityRange {
    pub fn new(range: RangeInclusive<Priority>) -> Self {
        Self(range)
    }

    pub fn includes(&self, other: &PriorityRange) -> bool {
        self.start() <= other.start() && other.end() <= self.end()
    }

    pub fn len(&self) -> usize {
        *self.end() as usize - *self.start() as usize + 1
    }

    pub fn is_empty(&self) -> bool {
        false
    }

    #[cfg(test)]
    pub fn rand() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let start = rng.gen_range(Priority::MAX as u8..Priority::MIN as u8);
        let end = rng.gen_range((start + 1)..=Priority::MIN as u8);

        Self(Priority::try_from(start).unwrap()..=Priority::try_from(end).unwrap())
    }
}

impl Display for PriorityRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", *self.start() as u8, *self.end() as u8)
    }
}

impl FromStr for PriorityRange {
    type Err = crate::protocol::ZProtocolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const SEPARATOR: &str = "-";
        let mut metadata = s.split(SEPARATOR);

        let start = metadata
            .next()
            .ok_or(ZProtocolError::Invalid)?
            .parse::<u8>()
            .map(Priority::try_from)
            .map_err(|_| ZProtocolError::Invalid)?
            .map_err(|_| ZProtocolError::Invalid)?;

        match metadata.next() {
            Some(slice) => {
                let end = slice
                    .parse::<u8>()
                    .map(Priority::try_from)
                    .map_err(|_| ZProtocolError::Invalid)?
                    .map_err(|_| ZProtocolError::Invalid)?;

                if metadata.next().is_some() {
                    crate::zbail!(crate::protocol::ZProtocolError::Invalid);
                };

                Ok(PriorityRange::new(start..=end))
            }
            None => Ok(PriorityRange::new(start..=start)),
        }
    }
}

impl Priority {
    pub const DEFAULT: Self = Self::Data;

    pub const MIN: Self = Self::Background;

    pub const MAX: Self = Self::Control;

    pub const NUM: usize = 1 + Self::MIN as usize - Self::MAX as usize;
}

impl TryFrom<u8> for Priority {
    type Error = crate::protocol::ZProtocolError;

    fn try_from(v: u8) -> crate::result::ZResult<Self, crate::protocol::ZProtocolError> {
        match v {
            0 => Ok(Priority::Control),
            1 => Ok(Priority::RealTime),
            2 => Ok(Priority::InteractiveHigh),
            3 => Ok(Priority::InteractiveLow),
            4 => Ok(Priority::DataHigh),
            5 => Ok(Priority::Data),
            6 => Ok(Priority::DataLow),
            7 => Ok(Priority::Background),
            _ => crate::zbail!(crate::protocol::ZProtocolError::Invalid),
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Reliability {
    BestEffort = 0,
    #[default]
    Reliable = 1,
}

impl Reliability {
    pub const DEFAULT: Self = Self::Reliable;

    #[cfg(test)]
    pub fn rand() -> Self {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        if rng.gen_bool(0.5) {
            Reliability::Reliable
        } else {
            Reliability::BestEffort
        }
    }
}

impl From<bool> for Reliability {
    fn from(value: bool) -> Self {
        if value {
            Reliability::Reliable
        } else {
            Reliability::BestEffort
        }
    }
}

impl From<Reliability> for bool {
    fn from(value: Reliability) -> Self {
        match value {
            Reliability::BestEffort => false,
            Reliability::Reliable => true,
        }
    }
}

impl Display for Reliability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as u8)
    }
}

impl FromStr for Reliability {
    type Err = crate::protocol::ZProtocolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Ok(desc) = s.parse::<u8>() else {
            crate::zbail!(crate::protocol::ZProtocolError::Invalid);
        };

        if desc == Reliability::BestEffort as u8 {
            Ok(Reliability::BestEffort)
        } else if desc == Reliability::Reliable as u8 {
            Ok(Reliability::Reliable)
        } else {
            Err(ZProtocolError::Invalid)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Channel {
    pub priority: Priority,
    pub reliability: Reliability,
}

impl Channel {
    pub const DEFAULT: Self = Self {
        priority: Priority::DEFAULT,
        reliability: Reliability::DEFAULT,
    };
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CongestionControl {
    #[default]
    Drop = 0,

    Block = 1,
}

impl CongestionControl {
    pub const DEFAULT: Self = Self::Drop;

    pub(crate) const DEFAULT_PUSH: Self = Self::Drop;
    pub(crate) const DEFAULT_REQUEST: Self = Self::Block;
    pub(crate) const DEFAULT_RESPONSE: Self = Self::Block;
    pub(crate) const DEFAULT_DECLARE: Self = Self::Block;
    pub(crate) const DEFAULT_OAM: Self = Self::Block;
}
