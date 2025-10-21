use core::{
    convert::{From, TryFrom, TryInto},
    fmt::{self, Display},
    hash::Hash,
};

use crate::{
    protocol::{
        ZCodecError, ZProtocolError,
        zcodec::{decode_zbuf, encode_zbuf, encoded_len_zbuf},
    },
    result::ZResult,
    zbuf::{ZBufReader, ZBufWriter},
};

pub(crate) mod encoding;
pub(crate) mod endpoint;
pub(crate) mod resolution;
pub(crate) mod whatami;
pub(crate) mod wire_expr;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(crate) struct ZenohIdProto(uhlc::ID);

impl ZenohIdProto {
    #[inline]
    pub(crate) fn size(&self) -> usize {
        self.0.size()
    }

    #[inline]
    pub(crate) fn as_le_bytes(&self) -> [u8; uhlc::ID::MAX_SIZE] {
        self.0.to_le_bytes()
    }

    pub(crate) fn rand() -> ZenohIdProto {
        ZenohIdProto(uhlc::ID::rand())
    }

    pub(crate) fn encoded_len(&self, len: bool) -> usize {
        encoded_len_zbuf(len, &self.as_le_bytes()[..self.size()])
    }

    pub(crate) fn encode(
        &self,
        len: bool,
        writer: &mut ZBufWriter<'_>,
    ) -> ZResult<(), ZCodecError> {
        encode_zbuf(writer, len, &self.as_le_bytes()[..self.size()])
    }

    pub(crate) fn decode(
        len: Option<usize>,
        reader: &mut ZBufReader<'_>,
    ) -> ZResult<Self, ZCodecError> {
        let zbuf = decode_zbuf(reader, len)?;

        Self::try_from(zbuf).map_err(|_| ZCodecError::CouldNotParse)
    }
}

impl Default for ZenohIdProto {
    fn default() -> Self {
        Self::rand()
    }
}

impl TryFrom<&[u8]> for ZenohIdProto {
    type Error = ZProtocolError;

    fn try_from(val: &[u8]) -> crate::result::ZResult<Self, ZProtocolError> {
        match val.try_into() {
            Ok(ok) => Ok(Self(ok)),
            Err(_) => crate::zbail!(ZProtocolError::CouldNotParse),
        }
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

pub(crate) type EntityId = u32;

#[derive(Debug, Default, Copy, Clone, Eq, Hash, PartialEq)]
pub(crate) struct EntityGlobalIdProto {
    pub(crate) zid: ZenohIdProto,
    pub(crate) eid: EntityId,
}

impl EntityGlobalIdProto {
    #[cfg(test)]
    pub(crate) fn rand() -> Self {
        use rand::Rng;
        Self {
            zid: ZenohIdProto::rand(),
            eid: rand::thread_rng().r#gen(),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Copy, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub(crate) enum Priority {
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

impl Priority {
    pub(crate) const DEFAULT: Self = Self::Data;
}

impl TryFrom<u8> for Priority {
    type Error = ZProtocolError;

    fn try_from(v: u8) -> crate::result::ZResult<Self, ZProtocolError> {
        match v {
            0 => Ok(Priority::Control),
            1 => Ok(Priority::RealTime),
            2 => Ok(Priority::InteractiveHigh),
            3 => Ok(Priority::InteractiveLow),
            4 => Ok(Priority::DataHigh),
            5 => Ok(Priority::Data),
            6 => Ok(Priority::DataLow),
            7 => Ok(Priority::Background),
            _ => crate::zbail!(ZProtocolError::CouldNotParse),
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub(crate) enum Reliability {
    BestEffort = 0,
    #[default]
    Reliable = 1,
}

impl Reliability {
    pub(crate) const DEFAULT: Self = Self::Reliable;

    #[cfg(test)]
    pub(crate) fn rand() -> Self {
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

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub(crate) enum CongestionControl {
    #[default]
    Drop = 0,

    Block = 1,
}

impl CongestionControl {
    pub(crate) const DEFAULT: Self = Self::Drop;
    pub(crate) const DEFAULT_DECLARE: Self = Self::Block;
}
