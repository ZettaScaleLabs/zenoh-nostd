use core::net::SocketAddr;

use crate::{protocol::ZCodecError, result::ZResult};

pub mod tcp;

#[cfg(feature = "platform-std")]
pub mod platform_std;

pub trait Platform {
    type AbstractedTcpStream: tcp::AbstractedTcpStream;

    fn new_tcp_stream(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = ZResult<Self::AbstractedTcpStream, ZCommunicationError>>;
}

crate::__internal_zerr! {
    /// Errors related to connections.
    #[err = "connection error"]
    enum ZCommunicationError {
        ConnectionClosed,
        DidNotRead,
        DidNotWrite,
        Invalid,
        TimedOut
    }
}

impl From<ZCodecError> for ZCommunicationError {
    fn from(x: ZCodecError) -> Self {
        match x {
            ZCodecError::DidNotRead => ZCommunicationError::DidNotRead,
            ZCodecError::DidNotWrite => ZCommunicationError::DidNotWrite,
            ZCodecError::Invalid => ZCommunicationError::Invalid,
            _ => ZCommunicationError::Invalid,
        }
    }
}
