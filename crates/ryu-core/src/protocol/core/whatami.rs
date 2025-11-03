use core::convert::TryFrom;

use crate::ProtocolError;

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WhatAmI {
    #[default]
    Peer = Self::U8_P,
    Router = Self::U8_R,
    Client = Self::U8_C,
}

impl WhatAmI {
    const U8_R: u8 = 0b0000_0001;
    const U8_P: u8 = 0b0000_0010;
    const U8_C: u8 = 0b0000_0100;

    #[cfg(test)]
    pub fn rand() -> Self {
        use rand::prelude::SliceRandom;
        let mut rng = rand::thread_rng();

        *[Self::Router, Self::Peer, Self::Client]
            .choose(&mut rng)
            .unwrap()
    }
}

impl From<WhatAmI> for u8 {
    fn from(w: WhatAmI) -> Self {
        w as u8
    }
}

impl TryFrom<u8> for WhatAmI {
    type Error = ProtocolError;

    fn try_from(v: u8) -> Result<Self, ProtocolError> {
        match v {
            Self::U8_R => Ok(Self::Router),
            Self::U8_P => Ok(Self::Peer),
            Self::U8_C => Ok(Self::Client),
            _ => Err(ProtocolError::CouldNotParse),
        }
    }
}
