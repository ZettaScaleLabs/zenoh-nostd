use {
    async_net::TcpStream,
    wtx::{misc::Uri, web_socket::WebSocketConnector},
    zenoh_nostd::platform::ZPlatform,
};

pub(crate) mod tcp;
pub(crate) mod ws;

pub struct PlatformStd;

impl ZPlatform for PlatformStd {
    type ZTcpStream = tcp::StdTcpStream;
    type ZWebSocket = ws::StdWsStream;

    async fn new_websocket_stream(
        &self,
        addr: &std::net::SocketAddr,
    ) -> core::result::Result<Self::ZWebSocket, zenoh_nostd::ConnectionError> {
        let uri = Uri::new(format!("ws://{}", addr));

        let tcp_stream = TcpStream::connect(uri.hostname_with_implied_port())
            .await
            .map_err(|_| {
                zenoh_nostd::error!("Could not connect to TcpStream");
                zenoh_nostd::ConnectionError::CouldNotConnect
            })?;

        tcp_stream.set_nodelay(true).map_err(|_| {
            zenoh_nostd::error!("Could not set nodelay on TcpStream");
            zenoh_nostd::ConnectionError::CouldNotConnect
        })?;

        let stream = WebSocketConnector::default()
            .connect(tcp_stream, &uri.to_ref())
            .await
            .map_err(|_| {
                zenoh_nostd::error!("Could not connect to WebSocket");
                zenoh_nostd::ConnectionError::CouldNotConnect
            })?;

        let peer_addr = *addr;

        Ok(ws::StdWsStream::new(peer_addr, stream))
    }

    async fn new_tcp_stream(
        &self,
        addr: &core::net::SocketAddr,
    ) -> core::result::Result<Self::ZTcpStream, zenoh_nostd::ConnectionError> {
        let socket = TcpStream::connect(addr)
            .await
            .map_err(|_| zenoh_nostd::ConnectionError::CouldNotConnect)?;

        socket.set_nodelay(true).map_err(|_| {
            zenoh_nostd::error!("Could not set nodelay on TcpStream");
            zenoh_nostd::ConnectionError::CouldNotConnect
        })?;

        let header = match socket
            .local_addr()
            .map_err(|_| zenoh_nostd::ConnectionError::CouldNotGetAddrInfo)?
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
