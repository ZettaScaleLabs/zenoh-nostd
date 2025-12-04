use core::net::SocketAddr;

pub mod tcp;
pub mod ws;

pub trait ZPlatform {
    type ZTcpStream: tcp::ZTcpStream;
    type ZWebSocket: ws::ZWebSocket;

    fn new_tcp_stream(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = core::result::Result<Self::ZTcpStream, crate::ConnectionError>> {
        let _ = addr;
        async { Err(crate::ConnectionError::CouldNotConnect) }
    }

    fn new_websocket_stream(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = core::result::Result<Self::ZWebSocket, crate::ConnectionError>> {
        let _ = addr;
        async { Err(crate::ConnectionError::CouldNotConnect) }
    }
}
