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
    ) -> impl Future<Output = ZResult<Self::AbstractedTcpStream, crate::ZConnectionError>> {
        let _ = addr;
        async { Err(crate::ZConnectionError::CouldNotConnect) }
    }

    fn new_websocket_stream(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = ZResult<Self::AbstractedWsStream, crate::ZConnectionError>> {
        let _ = addr;
        async { Err(crate::ZConnectionError::CouldNotConnect) }
    }
}
