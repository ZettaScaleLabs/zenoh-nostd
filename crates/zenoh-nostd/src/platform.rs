use core::net::SocketAddr;

pub mod tcp;
pub mod udp;
pub mod ws;

pub trait ZPlatform {
    type TcpStream: tcp::ZTcpStream;
    type UdpSocket: udp::ZUdpSocket;
    type WebSocket: ws::ZWebSocket;

    fn connect_tcp(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = core::result::Result<Self::TcpStream, crate::ConnectionError>> {
        let _ = addr;
        async { Err(crate::ConnectionError::CouldNotConnect) }
    }

    fn listen_tcp(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = core::result::Result<Self::TcpStream, crate::ConnectionError>> {
        let _ = addr;
        async { Err(crate::ConnectionError::CouldNotConnect) }
    }

    fn connect_udp(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = core::result::Result<Self::UdpSocket, crate::ConnectionError>> {
        let _ = addr;
        async { Err(crate::ConnectionError::CouldNotConnect) }
    }

    fn connect_websocket(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = core::result::Result<Self::WebSocket, crate::ConnectionError>> {
        let _ = addr;
        async { Err(crate::ConnectionError::CouldNotConnect) }
    }
}
