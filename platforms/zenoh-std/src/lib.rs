use std::net::SocketAddr;

use async_net::{TcpListener, TcpStream, UdpSocket};
use zenoh_nostd::platform::*;

mod tcp;
mod udp;
// mod ws;

pub struct StdLinkManager;

#[derive(ZLinkInfo, ZLinkTx, ZLinkRx, ZLink)]
#[zenoh(ZLink = (StdLinkTx, StdLinkRx))]
pub enum StdLink {
    Tcp(tcp::StdTcpLink),
    Udp(udp::StdUdpLink),
}

#[derive(ZLinkInfo, ZLinkTx)]
pub enum StdLinkTx {
    Tcp(tcp::StdTcpLinkTx),
    Udp(udp::StdUdpLinkTx),
}

#[derive(ZLinkInfo, ZLinkRx)]
pub enum StdLinkRx {
    Tcp(tcp::StdTcpLinkRx),
    Udp(udp::StdUdpLinkRx),
}

impl ZLinkManager for StdLinkManager {
    type Link<'a>
        = StdLink
    where
        Self: 'a;

    async fn connect(
        &self,
        endpoint: Endpoint<'_>,
    ) -> core::result::Result<Self::Link<'_>, LinkError> {
        let protocol = endpoint.protocol();
        let address = endpoint.address();

        match protocol.as_str() {
            "tcp" => {
                let dst_addr = SocketAddr::try_from(address)?;
                let socket = TcpStream::connect(dst_addr)
                    .await
                    .map_err(|_| LinkError::CouldNotConnect)?;

                socket
                    .set_nodelay(true)
                    .map_err(|_| LinkError::CouldNotConnect)?;

                let header = match socket
                    .local_addr()
                    .map_err(|_| LinkError::CouldNotGetAddrInfo)?
                    .ip()
                {
                    core::net::IpAddr::V4(_) => 40,
                    core::net::IpAddr::V6(_) => 60,
                };

                #[allow(unused_mut)] // mut is not needed when target_family != unix
                let mut mtu = u16::MAX - header;

                // target limitation of socket2: https://docs.rs/socket2/latest/src/socket2/sys/unix.rs.html#1544
                #[cfg(target_family = "unix")]
                {
                    let socket = socket2::SockRef::from(&socket);
                    // Get the MSS and divide it by 2 to ensure we can at least fill half the MSS
                    let mss = socket.tcp_mss().unwrap_or(mtu as u32) / 2;
                    // Compute largest multiple of TCP MSS that is smaller of default MTU
                    let mut tgt = mss;
                    while (tgt + mss) < mtu as u32 {
                        tgt += mss;
                    }
                    mtu = (mtu as u32).min(tgt) as u16;
                }

                Ok(Self::Link::Tcp(tcp::StdTcpLink::new(socket, mtu)))
            }
            "udp" => {
                let dst_addr = SocketAddr::try_from(address)?;
                let socket = UdpSocket::bind("0.0.0.0:0")
                    .await
                    .map_err(|_| LinkError::CouldNotConnect)?;

                socket
                    .connect(dst_addr)
                    .await
                    .map_err(|_| LinkError::CouldNotConnect)?;

                Ok(Self::Link::Udp(udp::StdUdpLink::new(socket, 8192)))
            }
            _ => zenoh::zbail!(LinkError::CouldNotParseProtocol),
        }
    }

    async fn listen(
        &self,
        endpoint: Endpoint<'_>,
    ) -> core::result::Result<Self::Link<'_>, LinkError> {
        let protocol = endpoint.protocol();
        let address = endpoint.address();

        match protocol.as_str() {
            "tcp" => {
                let src_addr = SocketAddr::try_from(address)?;
                let socket = TcpListener::bind(src_addr)
                    .await
                    .map_err(|_| LinkError::CouldNotConnect)?;

                let (socket, _) = socket
                    .accept()
                    .await
                    .map_err(|_| LinkError::CouldNotConnect)?;

                socket
                    .set_nodelay(true)
                    .map_err(|_| LinkError::CouldNotConnect)?;

                let header = match socket
                    .local_addr()
                    .map_err(|_| LinkError::CouldNotGetAddrInfo)?
                    .ip()
                {
                    core::net::IpAddr::V4(_) => 40,
                    core::net::IpAddr::V6(_) => 60,
                };

                #[allow(unused_mut)] // mut is not needed when target_family != unix
                let mut mtu = u16::MAX - header;

                // target limitation of socket2: https://docs.rs/socket2/latest/src/socket2/sys/unix.rs.html#1544
                #[cfg(target_family = "unix")]
                {
                    let socket = socket2::SockRef::from(&socket);
                    // Get the MSS and divide it by 2 to ensure we can at least fill half the MSS
                    let mss = socket.tcp_mss().unwrap_or(mtu as u32) / 2;
                    // Compute largest multiple of TCP MSS that is smaller of default MTU
                    let mut tgt = mss;
                    while (tgt + mss) < mtu as u32 {
                        tgt += mss;
                    }
                    mtu = (mtu as u32).min(tgt) as u16;
                }

                Ok(Self::Link::Tcp(tcp::StdTcpLink::new(socket, mtu)))
            }
            "udp" => {
                let dst_addr = SocketAddr::try_from(address)?;
                let socket = UdpSocket::bind(dst_addr)
                    .await
                    .map_err(|_| LinkError::CouldNotConnect)?;

                Ok(Self::Link::Udp(udp::StdUdpLink::new(socket, 8192)))
            }
            _ => zenoh::zbail!(LinkError::CouldNotParseProtocol),
        }
    }
}
