#![no_std]

use embassy_net::{
    IpAddress, IpEndpoint, Stack,
    tcp::TcpSocket,
    udp::{PacketMetadata, UdpSocket},
};
use zenoh_nostd::{platform::ZPlatform, zbail};

pub mod tcp;
pub mod udp;

pub struct PlatformEmbassy {
    pub stack: Stack<'static>,

    /// It is expected that every call to `tcp` generates new buffers. If you do it the naive way
    /// with just a `static BUFF` that is returned every time, it's UB to make multiple links.
    pub buffers: fn() -> (&'static mut [u8], &'static mut [u8]),
    pub metadatas: fn() -> (&'static mut [PacketMetadata], &'static mut [PacketMetadata]),
}

impl ZPlatform for PlatformEmbassy {
    type TcpStream = tcp::EmbassyTcpStream;
    type UdpSocket = udp::EmbassyUdpSocket;
    type WebSocket = zenoh_nostd::platform::ws::DummyWsStream;

    async fn new_tcp_stream(
        &self,
        addr: &core::net::SocketAddr,
    ) -> core::result::Result<Self::TcpStream, zenoh_nostd::ConnectionError> {
        let (tx, rx) = (self.buffers)();
        let mtu = rx.len() as u16;

        let mut socket: TcpSocket<'static> = TcpSocket::new(self.stack, rx, tx);

        let address: IpAddress = match addr.ip() {
            core::net::IpAddr::V4(v4) => IpAddress::Ipv4(v4),
            core::net::IpAddr::V6(_) => zbail!(zenoh_nostd::ConnectionError::CouldNotConnect),
        };

        let ip_endpoint = IpEndpoint::new(address, addr.port());

        socket.connect(ip_endpoint).await.map_err(|e| {
            zenoh_nostd::error!("Could not connect to {:?}: {:?}", addr, e);
            zenoh_nostd::ConnectionError::CouldNotConnect
        })?;

        Ok(Self::TcpStream::new(socket, mtu))
    }

    async fn new_udp_socket(
        &self,
        addr: &core::net::SocketAddr,
    ) -> core::result::Result<Self::UdpSocket, zenoh_nostd::ConnectionError> {
        let (tx, rx) = (self.buffers)();
        let mtu = rx.len() as u16;
        let (tx_meta, rx_meta) = (self.metadatas)();

        let socket: UdpSocket<'static> = UdpSocket::new(self.stack, rx_meta, rx, tx_meta, tx);

        let address: IpAddress = match addr.ip() {
            core::net::IpAddr::V4(v4) => IpAddress::Ipv4(v4),
            core::net::IpAddr::V6(_) => zbail!(zenoh_nostd::ConnectionError::CouldNotConnect),
        };
        let ip_endpoint = IpEndpoint::new(address, addr.port());

        Ok(Self::UdpSocket::new(socket, ip_endpoint.into(), mtu))
    }
}
