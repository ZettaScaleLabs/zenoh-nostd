use crate::{ZCodecError, ZCodecResult, ZStruct};

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "_:2|S|ID:5=0x03")]
pub struct Close {
    pub reason: u8,

    #[zenoh(header = S)]
    pub behaviour: CloseBehaviour,
}

#[repr(u8)]
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum CloseBehaviour {
    #[default]
    Link = 0,
    Session = 1,
}

impl From<CloseBehaviour> for u8 {
    fn from(value: CloseBehaviour) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for CloseBehaviour {
    type Error = ZCodecError;

    fn try_from(value: u8) -> ZCodecResult<Self> {
        match value {
            0 => Ok(CloseBehaviour::Link),
            1 => Ok(CloseBehaviour::Session),
            _ => Err(ZCodecError::CouldNotParse),
        }
    }
}
