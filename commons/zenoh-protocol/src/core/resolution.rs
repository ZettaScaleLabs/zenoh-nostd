use core::{fmt, str::FromStr};

use zenoh_result::{bail, ZError, ZE};

use crate::{network::request::RequestId, transport::TransportSn};

#[repr(u8)]
// The value represents the 2-bit encoded value
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Bits {
    U8 = 0b00,
    U16 = 0b01,
    U32 = 0b10,
    U64 = 0b11,
}

impl Bits {
    const S8: &'static str = "8bit";
    const S16: &'static str = "16bit";
    const S32: &'static str = "32bit";
    const S64: &'static str = "64bit";

    pub const fn bits(&self) -> u32 {
        match self {
            Bits::U8 => u8::BITS,
            Bits::U16 => u16::BITS,
            Bits::U32 => u32::BITS,
            Bits::U64 => u64::BITS,
        }
    }

    pub const fn mask(&self) -> u64 {
        match self {
            Bits::U8 => u8::MAX as u64,
            Bits::U16 => u16::MAX as u64,
            Bits::U32 => u32::MAX as u64,
            Bits::U64 => u64::MAX,
        }
    }

    pub const fn to_str(self) -> &'static str {
        match self {
            Bits::U8 => Self::S8,
            Bits::U16 => Self::S16,
            Bits::U32 => Self::S32,
            Bits::U64 => Self::S64,
        }
    }
}

impl From<u8> for Bits {
    fn from(_: u8) -> Self {
        Self::U8
    }
}

impl From<u16> for Bits {
    fn from(_: u16) -> Self {
        Self::U16
    }
}

impl From<u32> for Bits {
    fn from(_: u32) -> Self {
        Self::U32
    }
}

impl From<u64> for Bits {
    fn from(_: u64) -> Self {
        Self::U64
    }
}

impl FromStr for Bits {
    type Err = ZError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            Bits::S8 => Ok(Bits::U8),
            Bits::S16 => Ok(Bits::U16),
            Bits::S32 => Ok(Bits::U32),
            Bits::S64 => Ok(Bits::U64),
            _ => bail!(ZE::InvalidBits),
        }
    }
}

impl fmt::Display for Bits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_str())
    }
}

#[repr(u8)]
// The value indicates the bit offset
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    FrameSN = 0,
    RequestID = 2,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Resolution(u8);

impl Resolution {
    pub const fn as_u8(&self) -> u8 {
        self.0
    }

    pub const fn get(&self, field: Field) -> Bits {
        let value = (self.0 >> (field as u8)) & 0b11;
        unsafe { core::mem::transmute(value) }
    }

    pub fn set(&mut self, field: Field, bits: Bits) {
        self.0 &= !(0b11 << field as u8); // Clear bits
        self.0 |= (bits as u8) << (field as u8); // Set bits
    }
}

impl Default for Resolution {
    fn default() -> Self {
        let frame_sn = Bits::from(TransportSn::MAX) as u8;
        let request_id = (Bits::from(RequestId::MAX) as u8) << 2;
        Self(frame_sn | request_id)
    }
}

impl From<u8> for Resolution {
    fn from(v: u8) -> Self {
        Self(v)
    }
}
