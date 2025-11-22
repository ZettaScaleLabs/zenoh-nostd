use core::net::SocketAddr;

use zenoh_proto::ZResult;

pub mod tcp;
pub mod ws;

pub trait Platform {
    type AbstractedTcpStream: tcp::AbstractedTcpStream;
    type AbstractedWsStream: ws::AbstractedWsStream;

    fn new_tcp_stream(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = ZResult<Self::AbstractedTcpStream, ZConnectionError>>;

    fn new_websocket_stream(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = ZResult<Self::AbstractedWsStream, ZConnectionError>>;
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
