#![no_std]

use embassy_net::{IpAddress, IpEndpoint, Stack, tcp::TcpSocket};
use static_cell::StaticCell;
use zenoh_nostd::{platform::Platform, zbail};

pub mod tcp;

pub struct PlatformEmbassy {
    pub stack: Stack<'static>,
}

impl Platform for PlatformEmbassy {
    type AbstractedTcpStream = tcp::EmbassyTcpStream;
    type AbstractedWsStream = zenoh_nostd::platform::ws::DummyWsStream;

    async fn new_tcp_stream(
        &self,
        addr: &core::net::SocketAddr,
    ) -> zenoh_nostd::ZResult<Self::AbstractedTcpStream, zenoh_nostd::ZConnectionError> {
        static RX_BUF: StaticCell<[u8; 2048]> = StaticCell::new();
        static TX_BUF: StaticCell<[u8; 2048]> = StaticCell::new();
        let (rx_buf, tx_buf) = (RX_BUF.init([0; 2048]), TX_BUF.init([0; 2048]));

        let mut socket: TcpSocket<'static> = TcpSocket::new(self.stack, rx_buf, tx_buf);

        let address: IpAddress = match addr.ip() {
            core::net::IpAddr::V4(v4) => IpAddress::Ipv4(v4),
            core::net::IpAddr::V6(_) => zbail!(zenoh_nostd::ZConnectionError::CouldNotConnect),
        };

        let ip_endpoint = IpEndpoint::new(address, addr.port());

        socket
            .connect(ip_endpoint)
            .await
            .map_err(|_| zenoh_nostd::ZConnectionError::CouldNotConnect)?;

        Ok(Self::AbstractedTcpStream { socket, mtu: 1024 })
    }
}
