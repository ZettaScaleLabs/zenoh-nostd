#![no_std]

use embassy_net::{IpAddress, IpEndpoint, Stack, tcp::TcpSocket};
use zenoh_nostd::{platform::ZPlatform, zbail};

pub mod tcp;

/// Platform implementation for Embassy OS. Be careful, it is only safe to create
/// one instance of this struct and to only make a single call to `new_tcp_stream`.
pub struct PlatformEmbassy {
    pub stack: Stack<'static>,

    pub tcp: fn() -> (&'static mut [u8], &'static mut [u8]),
}

impl ZPlatform for PlatformEmbassy {
    type TcpStream = tcp::EmbassyTcpStream;
    type WebSocket = zenoh_nostd::platform::ws::DummyWsStream;

    async fn new_tcp_stream(
        &self,
        addr: &core::net::SocketAddr,
    ) -> core::result::Result<Self::TcpStream, zenoh_nostd::ConnectionError> {
        let (tx, rx) = (self.tcp)();
        let mtu = rx.len() as u16;

        let mut socket: TcpSocket<'static> = TcpSocket::new(self.stack, tx, rx);

        let address: IpAddress = match addr.ip() {
            core::net::IpAddr::V4(v4) => IpAddress::Ipv4(v4),
            core::net::IpAddr::V6(_) => zbail!(zenoh_nostd::ConnectionError::CouldNotConnect),
        };

        let ip_endpoint = IpEndpoint::new(address, addr.port());

        socket.connect(ip_endpoint).await.map_err(|e| {
            zenoh_nostd::error!("Could not connect to {:?}: {:?}", addr, e);
            zenoh_nostd::ConnectionError::CouldNotConnect
        })?;

        Ok(Self::TcpStream { socket, mtu })
    }
}
