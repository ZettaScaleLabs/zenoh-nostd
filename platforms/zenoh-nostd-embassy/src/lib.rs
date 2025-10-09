#![no_std]


use embassy_net::{IpAddress, IpEndpoint, Stack, tcp::TcpSocket};
use static_cell::StaticCell;
use zenoh_nostd::{
    platform::{Platform, ZCommunicationError, ws::DummyWebSocket},
    result::ZResult,
    zbail,
};

pub mod tcp;

pub struct PlatformEmbassy {
    pub stack: Stack<'static>,
}

impl Platform for PlatformEmbassy {
    type PALTcpStream = tcp::EmbassyTcpStream;
    type PALWebSocket = DummyWebSocket;

    async fn new_tcp_stream(
        &self,
        addr: &core::net::SocketAddr,
    ) -> ZResult<Self::PALTcpStream, ZCommunicationError> {
        static RX_BUF: StaticCell<[u8; 1024]> = StaticCell::new();
        static TX_BUF: StaticCell<[u8; 1024]> = StaticCell::new();
        let (rx_buf, tx_buf) = (RX_BUF.init([0; 1024]), TX_BUF.init([0; 1024]));

        let mut socket: TcpSocket<'static> = TcpSocket::new(self.stack, rx_buf, tx_buf);

        let address: IpAddress = match addr.ip() {
            core::net::IpAddr::V4(v4) => IpAddress::Ipv4(v4),
            core::net::IpAddr::V6(_) => zbail!(ZCommunicationError::Invalid),
        };

        let ip_endpoint = IpEndpoint::new(address, addr.port());

        socket
            .connect(ip_endpoint)
            .await
            .map_err(|_| ZCommunicationError::ConnectionClosed)?;

        let local_addr = match socket.local_endpoint().unwrap().addr {
            IpAddress::Ipv4(v4) => core::net::SocketAddr::new(
                core::net::IpAddr::V4(v4),
                socket.local_endpoint().unwrap().port,
            ),
        };

        Ok(Self::PALTcpStream {
            socket,
            local_addr,
            peer_addr: *addr,
        })
    }

    async fn new_websocket(
        &self,
        _: &core::net::SocketAddr,
    ) -> ZResult<Self::PALWebSocket, ZCommunicationError> { Err(ZCommunicationError::Invalid) }
}
