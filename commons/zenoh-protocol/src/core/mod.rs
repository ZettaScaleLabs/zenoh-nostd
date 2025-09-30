use core::{
    convert::{From, TryFrom, TryInto},
    fmt::{self, Display},
    hash::Hash,
    ops::{Deref, RangeInclusive},
    str::FromStr,
};

use heapless::String;
pub use uhlc::{Timestamp, NTP64};
use zenoh_keyexpr::key_expr::OwnedKeyExpr;
use zenoh_result::{zbail, zerr, ZError, ZE};

/// The unique Id of the [`HLC`](uhlc::HLC) that generated the concerned [`Timestamp`].
pub type TimestampId = uhlc::ID;

/// Constants and helpers for zenoh `whatami` flags.
pub mod whatami;

pub mod wire_expr;

pub mod encoding;

pub mod locator;

pub mod endpoint;

pub mod resolution;

/// The global unique id of a zenoh peer.
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

    pub fn into_keyexpr(self) -> OwnedKeyExpr<32> {
        self.into()
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
            type Error = ZError;
            fn try_from(val: $T) -> Result<Self, Self::Error> {
                match val.try_into() {
                    Ok(ok) => Ok(Self(ok)),
                    Err(_) => Err(zerr!(ZE::InvalidArgument)),
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
    type Err = ZError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(|c: char| c.is_ascii_uppercase()) {
            zbail!(ZE::InvalidId);
        }

        let u: uhlc::ID = s.parse().map_err(|_| zerr!(ZE::InvalidId))?;

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

// A PeerID can be converted into a Timestamp's ID
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

impl From<ZenohIdProto> for OwnedKeyExpr<32> {
    fn from(zid: ZenohIdProto) -> Self {
        let id = u128::from_le_bytes(zid.0.to_le_bytes());

        const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";
        let mut leading_zeros = true;

        let mut str = String::<32>::new();
        for i in (0..32).rev() {
            let nibble = ((id >> (i * 4)) & 0xF) as usize;
            let digit = HEX_DIGITS[nibble];

            if digit == b'0' && leading_zeros && i > 0 {
                continue;
            }

            leading_zeros = false;

            str.push(digit as char).unwrap();
        }

        if leading_zeros {
            str.push('0').unwrap();
        }

        // SAFETY: zid.to_string() returns an stringified hexadecimal
        // representation of the zid. Therefore, building a OwnedKeyExpr
        // by calling from_string_unchecked() is safe because it is
        // guaranteed that no wildcards nor reserved chars will be present.
        unsafe { OwnedKeyExpr::from_string_unchecked(str) }
    }
}

impl From<&ZenohIdProto> for OwnedKeyExpr<32> {
    fn from(zid: &ZenohIdProto) -> Self {
        (*zid).into()
    }
}

/// The unique id of a zenoh entity inside it's parent `Session`.
pub type EntityId = u32;

/// The global unique id of a zenoh entity.
#[derive(Debug, Default, Copy, Clone, Eq, Hash, PartialEq)]
pub struct EntityGlobalIdProto {
    pub zid: ZenohIdProto,
    pub eid: EntityId,
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
/// A [`Priority`] range bounded inclusively below and above.
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

    /// Returns `true` if `self` is a superset of `other`.
    pub fn includes(&self, other: &PriorityRange) -> bool {
        self.start() <= other.start() && other.end() <= self.end()
    }

    pub fn len(&self) -> usize {
        *self.end() as usize - *self.start() as usize + 1
    }

    pub fn is_empty(&self) -> bool {
        false
    }
}

impl Display for PriorityRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", *self.start() as u8, *self.end() as u8)
    }
}

impl FromStr for PriorityRange {
    type Err = ZError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const SEPARATOR: &str = "-";
        let mut metadata = s.split(SEPARATOR);

        let start = metadata
            .next()
            .ok_or_else(|| zerr!(ZE::InvalidPriorityValue))?
            .parse::<u8>()
            .map(Priority::try_from)
            .map_err(|_| zerr!(ZE::InvalidPriorityValue))?
            .map_err(|_| zerr!(ZE::InvalidPriorityValue))?;

        match metadata.next() {
            Some(slice) => {
                let end = slice
                    .parse::<u8>()
                    .map(Priority::try_from)
                    .map_err(|_| zerr!(ZE::InvalidPriorityValue))?
                    .map_err(|_| zerr!(ZE::InvalidPriorityValue))?;

                if metadata.next().is_some() {
                    zbail!(ZE::InvalidPriorityValue);
                };

                Ok(PriorityRange::new(start..=end))
            }
            None => Ok(PriorityRange::new(start..=start)),
        }
    }
}

impl Priority {
    /// Default
    pub const DEFAULT: Self = Self::Data;
    /// The lowest Priority
    pub const MIN: Self = Self::Background;
    /// The highest Priority
    pub const MAX: Self = Self::Control;
    /// The number of available priorities
    pub const NUM: usize = 1 + Self::MIN as usize - Self::MAX as usize;
}

impl TryFrom<u8> for Priority {
    type Error = ZError;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Priority::Control),
            1 => Ok(Priority::RealTime),
            2 => Ok(Priority::InteractiveHigh),
            3 => Ok(Priority::InteractiveLow),
            4 => Ok(Priority::DataHigh),
            5 => Ok(Priority::Data),
            6 => Ok(Priority::DataLow),
            7 => Ok(Priority::Background),
            _ => zbail!(ZE::InvalidPriorityValue),
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
    type Err = ZError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Ok(desc) = s.parse::<u8>() else {
            zbail!(ZE::InvalidReliabilityValue);
        };

        if desc == Reliability::BestEffort as u8 {
            Ok(Reliability::BestEffort)
        } else if desc == Reliability::Reliable as u8 {
            Ok(Reliability::Reliable)
        } else {
            Err(zerr!(ZE::InvalidReliabilityValue))
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

/// Congestion control strategy.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CongestionControl {
    #[default]
    /// When transmitting a message in a node with a full queue, the node may drop the message.
    Drop = 0,
    /// When transmitting a message in a node with a full queue, the node will wait for queue to
    /// progress.
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

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use crate::core::{Priority, PriorityRange};

    #[test]
    fn test_priority_range() {
        assert_eq!(
            PriorityRange::from_str("2-3"),
            Ok(PriorityRange::new(
                Priority::InteractiveHigh..=Priority::InteractiveLow
            ))
        );

        assert_eq!(
            PriorityRange::from_str("7"),
            Ok(PriorityRange::new(
                Priority::Background..=Priority::Background
            ))
        );

        assert!(PriorityRange::from_str("1-").is_err());
        assert!(PriorityRange::from_str("-5").is_err());
    }
}
