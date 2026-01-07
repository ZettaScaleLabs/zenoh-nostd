use core::net::SocketAddr;

pub mod tcp;
pub mod udp;
pub mod ws;

pub trait ZPlatform {
    type TcpStream: tcp::ZTcpStream;
    type UdpSocket: udp::ZUdpSocket;
    type WebSocket: ws::ZWebSocket;

    fn new_tcp_stream(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = core::result::Result<Self::TcpStream, crate::ConnectionError>> {
        let _ = addr;
        async { Err(crate::ConnectionError::CouldNotConnect) }
    }

    fn new_udp_socket(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = core::result::Result<Self::UdpSocket, crate::ConnectionError>> {
        let _ = addr;
        async { Err(crate::ConnectionError::CouldNotConnect) }
    }

    fn new_websocket_stream(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = core::result::Result<Self::WebSocket, crate::ConnectionError>> {
        let _ = addr;
        async { Err(crate::ConnectionError::CouldNotConnect) }
    }
}
