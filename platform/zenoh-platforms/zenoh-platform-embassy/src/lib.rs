#![no_std]

use core::future::Future;

use embassy_net::{tcp::TcpSocket, IpAddress, IpEndpoint, Stack};
use embassy_time::Duration;
use static_cell::StaticCell;
use zenoh_platform::{ws::DummyPlatformWSStream, Platform};
use zenoh_result::{bail, zctx, zerr, WithContext, ZResult, ZE};

use crate::tcp::PlatformEmbassyTcpStream;

pub mod tcp;

pub struct PlatformEmbassy {
    pub stack: Stack<'static>,
}

impl Platform for PlatformEmbassy {
    type PlatformTcpStream = PlatformEmbassyTcpStream;
    type PlatformWSStream = DummyPlatformWSStream;

    fn new_tcp_stream(
        &mut self,
        addr: &core::net::SocketAddr,
    ) -> impl Future<Output = ZResult<Self::PlatformTcpStream>> {
        async move {
            static RX_BUF: StaticCell<[u8; 1024]> = StaticCell::new();
            static TX_BUF: StaticCell<[u8; 1024]> = StaticCell::new();
            let (rx_buf, tx_buf) = (RX_BUF.init([0; 1024]), TX_BUF.init([0; 1024]));

            let mut socket: TcpSocket<'static> = TcpSocket::new(self.stack, rx_buf, tx_buf);

            socket.set_timeout(Some(Duration::from_secs(10)));

            let address: IpAddress = match addr.ip() {
                core::net::IpAddr::V4(v4) => IpAddress::Ipv4(v4),
                core::net::IpAddr::V6(_) => bail!(ZE::UnsupportedPlatform),
            };

            let ip_endpoint = IpEndpoint::new(address, addr.port());
            defmt::info!("connecting to {:?}...", ip_endpoint);

            socket
                .connect(ip_endpoint)
                .await
                .map_err(|_| zerr!(ZE::ConnectionRefused))
                .context(zctx!("connecting embassy net socket to address"))?;

            let local_addr = match socket.local_endpoint().unwrap().addr {
                IpAddress::Ipv4(v4) => core::net::SocketAddr::new(
                    core::net::IpAddr::V4(v4),
                    socket.local_endpoint().unwrap().port,
                ),
            };

            Ok(Self::PlatformTcpStream {
                socket,
                local_addr: local_addr,
                peer_addr: *addr,
            })
        }
    }
}
