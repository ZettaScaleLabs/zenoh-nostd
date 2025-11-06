#[cfg(test)]
use crate::ZWriter;
#[cfg(test)]
use rand::{Rng, thread_rng};

use crate::{ZCodecError, ZCodecResult, ZExt, zbail};

pub mod declare;
pub mod interest;
pub mod push;
pub mod request;
pub mod response;

#[derive(ZExt, Debug, PartialEq)]
pub struct QoS {
    inner: u8,
}

impl QoS {
    #[cfg(test)]
    pub(crate) fn rand(_: &mut ZWriter) -> Self {
        let inner: u8 = thread_rng().r#gen();
        Self { inner }
    }
}

#[derive(ZExt, Debug, PartialEq)]
pub struct NodeId {
    pub node_id: u16,
}

impl NodeId {
    #[cfg(test)]
    pub(crate) fn rand(_: &mut ZWriter) -> Self {
        let node_id: u16 = thread_rng().r#gen();
        Self { node_id }
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

impl Priority {
    pub const DEFAULT: Self = Self::Data;
}

impl TryFrom<u8> for Priority {
    type Error = ZCodecError;

    fn try_from(v: u8) -> ZCodecResult<Self> {
        match v {
            0 => Ok(Priority::Control),
            1 => Ok(Priority::RealTime),
            2 => Ok(Priority::InteractiveHigh),
            3 => Ok(Priority::InteractiveLow),
            4 => Ok(Priority::DataHigh),
            5 => Ok(Priority::Data),
            6 => Ok(Priority::DataLow),
            7 => Ok(Priority::Background),
            _ => zbail!(ZCodecError::CouldNotParse),
        }
    }
}

// #[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
// #[repr(u8)]
// pub enum Reliability {
//     BestEffort = 0,
//     #[default]
//     Reliable = 1,
// }

// impl Reliability {
//     pub const DEFAULT: Self = Self::Reliable;

//     #[cfg(test)]
//     pub fn rand() -> Self {
//         use rand::Rng;

//         let mut rng = rand::thread_rng();

//         if rng.gen_bool(0.5) {
//             Reliability::Reliable
//         } else {
//             Reliability::BestEffort
//         }
//     }
// }

// impl From<bool> for Reliability {
//     fn from(value: bool) -> Self {
//         if value {
//             Reliability::Reliable
//         } else {
//             Reliability::BestEffort
//         }
//     }
// }

// impl From<Reliability> for bool {
//     fn from(value: Reliability) -> Self {
//         match value {
//             Reliability::BestEffort => false,
//             Reliability::Reliable => true,
//         }
//     }
// }

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CongestionControl {
    #[default]
    Drop = 0,
    Block = 1,
}

impl CongestionControl {
    pub const DEFAULT: Self = Self::Drop;
    pub const DEFAULT_DECLARE: Self = Self::Block;
}
