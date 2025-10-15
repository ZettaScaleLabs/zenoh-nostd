use core::{convert::TryFrom, fmt, str::FromStr};

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum WhatAmI {
    Router = 0b001,
    #[default]
    Peer = 0b010,
    Client = 0b100,
}

impl WhatAmI {
    const STR_R: &'static str = "router";
    const STR_P: &'static str = "peer";
    const STR_C: &'static str = "client";

    const U8_R: u8 = Self::Router as u8;
    const U8_P: u8 = Self::Peer as u8;
    const U8_C: u8 = Self::Client as u8;

    pub(crate) const fn to_str(self) -> &'static str {
        match self {
            Self::Router => Self::STR_R,
            Self::Peer => Self::STR_P,
            Self::Client => Self::STR_C,
        }
    }

    #[cfg(test)]
    pub(crate) fn rand() -> Self {
        use rand::prelude::SliceRandom;
        let mut rng = rand::thread_rng();

        *[Self::Router, Self::Peer, Self::Client]
            .choose(&mut rng)
            .unwrap()
    }
}

impl TryFrom<u8> for WhatAmI {
    type Error = ();

    fn try_from(v: u8) -> crate::result::ZResult<Self, ()> {
        match v {
            Self::U8_R => Ok(Self::Router),
            Self::U8_P => Ok(Self::Peer),
            Self::U8_C => Ok(Self::Client),
            _ => Err(()),
        }
    }
}

impl FromStr for WhatAmI {
    type Err = crate::protocol::ZProtocolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            Self::STR_R => Ok(Self::Router),
            Self::STR_P => Ok(Self::Peer),
            Self::STR_C => Ok(Self::Client),
            _ => crate::zbail!(crate::protocol::ZProtocolError::Invalid),
        }
    }
}

impl fmt::Display for WhatAmI {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_str())
    }
}

impl From<WhatAmI> for u8 {
    fn from(w: WhatAmI) -> Self {
        w as u8
    }
}
