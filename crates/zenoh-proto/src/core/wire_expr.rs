use crate::{ZExt, keyexpr};

#[repr(u8)]
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum Mapping {
    #[default]
    Receiver = 0,
    Sender = 1,
}

impl From<Mapping> for u8 {
    fn from(val: Mapping) -> u8 {
        val as u8
    }
}

impl TryFrom<u8> for Mapping {
    type Error = crate::ZCodecError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Mapping::Receiver),
            1 => Ok(Mapping::Sender),
            _ => Err(crate::ZCodecError::CouldNotParseField),
        }
    }
}

#[derive(ZExt, Debug, PartialEq)]
#[zenoh(header = "_:6|M|N|")]
pub struct WireExpr<'a> {
    pub scope: u16,

    #[zenoh(header = M)]
    pub mapping: Mapping,

    #[zenoh(presence = header(N), default = "", size = prefixed)]
    pub suffix: &'a str,
}

impl<'a> From<&'a keyexpr> for WireExpr<'a> {
    fn from(ke: &'a keyexpr) -> Self {
        Self {
            scope: 0,
            mapping: Mapping::Sender,
            suffix: ke.as_str(),
        }
    }
}
