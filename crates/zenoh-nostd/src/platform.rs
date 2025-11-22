use core::net::SocketAddr;

use zenoh_proto::ZResult;

pub mod tcp;

pub trait Platform {
    type AbstractedTcpStream: tcp::AbstractedTcpStream;

    fn new_tcp_stream(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = ZResult<Self::AbstractedTcpStream, ZConnectionError>>;
}

zenoh_proto::make_zerr! {
    /// Errors related to connections.
    #[err = "connection error"]
    enum ZConnectionError {
        CouldNotGetAddrInfo,
        CouldNotConnect,
        CouldNotWrite,
        CouldNotRead
    }
}
