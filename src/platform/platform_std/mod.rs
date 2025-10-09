
use crate::{
    platform::{Platform, ZCommunicationError, platform_std::tcp::StdTcpStream},
    result::ZResult,
};

pub mod tcp;

pub struct PlatformStd;

impl Platform for PlatformStd {
    type PALTcpStream = StdTcpStream;
    type PALWebSocket = crate::platform::ws::DummyWebSocket;

    async fn new_tcp_stream(
        &self,
        addr: &core::net::SocketAddr,
    ) -> ZResult<Self::PALTcpStream, ZCommunicationError> {
        let socket = async_net::TcpStream::connect(addr)
            .await
            .map_err(|_| ZCommunicationError::ConnectionClosed)?;

        let header = match socket
            .local_addr()
            .map_err(|_| ZCommunicationError::Invalid)?
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

        Ok(Self::PALTcpStream { socket, mtu })
    }

    async fn new_websocket(
        &self,
        _addr: &core::net::SocketAddr,
    ) -> ZResult<Self::PALWebSocket, ZCommunicationError> { Err(ZCommunicationError::Invalid) }
}
