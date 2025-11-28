use ::core::convert::TryFrom;

use crate::*;

#[repr(u8)]
#[derive(ZRU8, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WhatAmI {
    #[default]
    Peer = Self::U8_P,
    Router = Self::U8_R,
    Client = Self::U8_C,
}

impl WhatAmI {
    const U8_R: u8 = 0b0000_0000;
    const U8_P: u8 = 0b0000_0001;
    const U8_C: u8 = 0b0000_0010;
}
