use crate::zbuf::ZIOError;

pub mod common;
pub mod core;

pub mod zenoh;

pub mod network;
pub mod transport;

pub mod zcodec;

pub const VERSION: u8 = 0x09;

crate::__internal_zerr! {
    /// Errors related to Zenoh protocol
    #[err = "zenoh protocol error"]
    enum ZProtocolError {
        Invalid,
    }

    /// Errors related to encoding/decoding Zenoh protocol messages
    #[err = "zenoh codec error"]
    enum ZCodecError {
        DidNotRead,
        DidNotWrite,
        Invalid,
        Overflow
    }
}

impl From<ZProtocolError> for ZCodecError {
    fn from(e: ZProtocolError) -> Self {
        match e {
            ZProtocolError::Invalid => ZCodecError::Invalid,
        }
    }
}

impl From<ZIOError> for ZCodecError {
    fn from(e: ZIOError) -> Self {
        match e {
            ZIOError::DidNotRead => ZCodecError::DidNotRead,
            ZIOError::DidNotWrite => ZCodecError::DidNotWrite,
            ZIOError::Invalid => ZCodecError::Invalid,
        }
    }
}
