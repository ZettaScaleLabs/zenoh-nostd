use {
    async_net::TcpStream,
    wtx::{misc::Uri, web_socket::WebSocketConnector},
    zenoh_nostd::{
        ZResult,
        platform::{Platform, ZConnectionError},
    },
};

pub(crate) mod tcp;
pub(crate) mod ws;

pub struct PlatformStd;

impl Platform for PlatformStd {
    type AbstractedTcpStream = tcp::StdTcpStream;
    type AbstractedWsStream = ws::StdWsStream;

    async fn new_websocket_stream(
        &self,
        addr: &std::net::SocketAddr,
    ) -> ZResult<Self::AbstractedWsStream, ZConnectionError> {
        let uri = Uri::new(format!("ws://{}", addr));
        let stream = WebSocketConnector::default()
            .connect(
                TcpStream::connect(uri.hostname_with_implied_port())
                    .await
                    .map_err(|_| {
                        zenoh_nostd::error!("Could not connect to TcpStream");
                        ZConnectionError::CouldNotConnect
                    })?,
                &uri.to_ref(),
            )
            .await
            .map_err(|_| {
                zenoh_nostd::error!("Could not connect to WebSocket");
                ZConnectionError::CouldNotConnect
            })?;
        let peer_addr = *addr;
        Ok(Self::AbstractedWsStream::new(peer_addr, stream))
    }

    async fn new_tcp_stream(
        &self,
        addr: &core::net::SocketAddr,
    ) -> ZResult<Self::AbstractedTcpStream, ZConnectionError> {
        let socket = async_net::TcpStream::connect(addr)
            .await
            .map_err(|_| ZConnectionError::CouldNotConnect)?;

        let header = match socket
            .local_addr()
            .map_err(|_| ZConnectionError::CouldNotGetAddrInfo)?
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

        Ok(tcp::StdTcpStream::new(socket, mtu))
    }
}
