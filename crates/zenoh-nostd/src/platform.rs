use ::core::net::SocketAddr;

use crate::ZResult;

pub mod tcp;
pub mod ws;

pub trait ZPlatform {
    type ZTcpStream: tcp::ZTcpStream;
    type ZWebSocket: ws::ZWebSocket;

    fn new_tcp_stream(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = ZResult<Self::ZTcpStream, crate::ZConnectionError>> {
        let _ = addr;
        async { Err(crate::ZConnectionError::CouldNotConnect) }
    }

    fn new_websocket_stream(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = ZResult<Self::ZWebSocket, crate::ZConnectionError>> {
        let _ = addr;
        async { Err(crate::ZConnectionError::CouldNotConnect) }
    }
}
