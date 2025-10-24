use crate::zbuf::ZBufError;

pub(crate) mod codec;
pub(crate) mod ext;
pub(crate) mod exts;

pub(crate) mod keyexpr;

pub(crate) mod common;
pub(crate) mod core;

pub(crate) mod zenoh;

pub(crate) mod network;
pub(crate) mod transport;

pub(crate) const VERSION: u8 = 0x09;

crate::__internal_zerr! {
    /// Errors related to Zenoh protocol
    #[err = "zenoh protocol error"]
    enum ZProtocolError {
        NoProtocolSeparator,
        MetadataNotSupported,
        ConfigNotSupported,
        CouldNotParse
    }

    /// Errors related to encoding/decoding Zenoh protocol messages
    #[err = "zenoh codec error"]
    enum ZCodecError {
        CouldNotRead,
        CouldNotWrite,
        CouldNotParse
    }
}

impl From<ZBufError> for ZCodecError {
    fn from(e: ZBufError) -> Self {
        match e {
            ZBufError::CouldNotRead => ZCodecError::CouldNotRead,
            ZBufError::CouldNotWrite => ZCodecError::CouldNotWrite,
            ZBufError::CouldNotParse => ZCodecError::CouldNotParse,
        }
    }
}

const MSG_ID_BITS: u8 = 5;
const MSG_ID_MASK: u8 = !(u8::MAX << MSG_ID_BITS);

pub(crate) const fn msg_id(header: u8) -> u8 {
    header & MSG_ID_MASK
}

pub(crate) const fn has_flag(byte: u8, flag: u8) -> bool {
    byte & flag != 0
}
