use async_net::{TcpListener, TcpStream, UdpSocket};
use zenoh_nostd::platform::*;

mod tcp;
mod udp;
// mod ws;

pub struct StdLinkManager;

impl ZLinkManager for StdLinkManager {
    type Tcp<'ext> = tcp::StdTcpLink;
    type Udp<'ext> = udp::StdUdpLink;
    type Ws<'ext> = ();
    type Serial<'ext> = ();

    async fn connect_tcp(
        &self,
        addr: &core::net::SocketAddr,
    ) -> core::result::Result<Link<'_, Self>, LinkError> {
        let socket = TcpStream::connect(addr)
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

        Ok(Link::Tcp(Self::Tcp::new(socket, mtu)))
    }

    async fn listen_tcp(
        &self,
        addr: &std::net::SocketAddr,
    ) -> core::result::Result<Link<'_, Self>, LinkError> {
        let socket = TcpListener::bind(addr)
            .await
            .map_err(|_| LinkError::CouldNotConnect)?;

        let (socket, _) = socket
            .accept()
            .await
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

        Ok(Link::Tcp(Self::Tcp::new(socket, mtu)))
    }

    async fn connect_udp(
        &self,
        addr: &core::net::SocketAddr,
    ) -> core::result::Result<Link<'_, Self>, LinkError> {
        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|_| LinkError::CouldNotConnect)?;

        socket
            .connect(addr)
            .await
            .map_err(|_| LinkError::CouldNotConnect)?;

        Ok(Link::Udp(Self::Udp::new(socket, 8192)))
    }

    async fn listen_udp(
        &self,
        addr: &std::net::SocketAddr,
    ) -> core::result::Result<Link<'_, Self>, LinkError> {
        let socket = UdpSocket::bind(addr)
            .await
            .map_err(|_| LinkError::CouldNotConnect)?;

        Ok(Link::Udp(Self::Udp::new(socket, 8192)))
    }
}
