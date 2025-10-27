use core::net::SocketAddr;

use crate::result::ZResult;

pub mod tcp;

#[cfg(feature = "platform-std")]
pub mod platform_std;

pub trait Platform {
    type AbstractedTcpStream: tcp::AbstractedTcpStream;

    fn new_tcp_stream(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = ZResult<Self::AbstractedTcpStream, ZConnectionError>>;
}

crate::__internal_zerr! {
    /// Errors related to connections.
    #[err = "connection error"]
    enum ZConnectionError {
        CouldNotGetAddrInfo,
        CouldNotConnect,
        CouldNotWrite,
        CouldNotRead
    }
}
