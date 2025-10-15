use crate::zbuf::ZBufError;

pub(crate) mod common;
pub(crate) mod core;

pub(crate) mod zenoh;

pub(crate) mod network;
pub(crate) mod transport;

pub(crate) mod zcodec;

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
