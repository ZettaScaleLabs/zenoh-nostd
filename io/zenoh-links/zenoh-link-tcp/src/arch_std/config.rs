use std::net::SocketAddr;

use async_net::{TcpListener, TcpStream};
use zenoh_link_commons::{TCP_SO_RCV_BUF, TCP_SO_SND_BUF};
use zenoh_protocol::core::Config;
use zenoh_result::{zerror, ZResult};

pub struct TcpSocketConfig {
    pub tx_buffer_size: Option<u32>,
    pub rx_buffer_size: Option<u32>,
    pub bind_socket: Option<SocketAddr>,
}

impl TcpSocketConfig {
    pub fn new(
        tx_buffer_size: Option<u32>,
        rx_buffer_size: Option<u32>,
        bind_socket: Option<SocketAddr>,
    ) -> Self {
        Self {
            tx_buffer_size,
            rx_buffer_size,
            bind_socket,
        }
    }

    /// Build a new TCPListener bound to `addr` with the given configuration parameters
    pub async fn new_listener(&self, addr: &SocketAddr) -> ZResult<(TcpListener, SocketAddr)> {
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| zerror!("{}: {}", addr, e))?;

        let local_addr = listener
            .local_addr()
            .map_err(|e| zerror!("{}: {}", addr, e))?;

        Ok((listener, local_addr))
    }

    /// Connect to a TCP socket address at `dst_addr` with the given configuration parameters
    pub async fn new_link(
        &self,
        dst_addr: &SocketAddr,
    ) -> ZResult<(TcpStream, SocketAddr, SocketAddr)> {
        // Build a TcpStream from TcpSocket
        // https://docs.rs/tokio/latest/tokio/net/struct.TcpSocket.html
        let stream = TcpStream::connect(dst_addr)
            .await
            .map_err(|e| zerror!("{}: {}", dst_addr, e))?;

        let src_addr = stream
            .local_addr()
            .map_err(|e| zerror!("{}: {}", dst_addr, e))?;

        let dst_addr = stream
            .peer_addr()
            .map_err(|e| zerror!("{}: {}", dst_addr, e))?;

        Ok((stream, src_addr, dst_addr))
    }
}

pub struct TcpLinkConfig {
    pub(crate) rx_buffer_size: Option<u32>,
    pub(crate) tx_buffer_size: Option<u32>,
    pub(crate) bind_socket: Option<SocketAddr>,
}

impl TcpLinkConfig {
    pub async fn new(config: &Config<'_>) -> ZResult<Self> {
        let mut tcp_config = Self {
            rx_buffer_size: None,
            tx_buffer_size: None,
            bind_socket: None,
        };

        if let Some(size) = config.get(TCP_SO_RCV_BUF) {
            tcp_config.rx_buffer_size = Some(
                size.parse()
                    .map_err(|_| zerror!("Unknown TCP read buffer size argument: {}", size))?,
            );
        };
        if let Some(size) = config.get(TCP_SO_SND_BUF) {
            tcp_config.tx_buffer_size = Some(
                size.parse()
                    .map_err(|_| zerror!("Unknown TCP write buffer size argument: {}", size))?,
            );
        };

        Ok(tcp_config)
    }
}

impl From<TcpLinkConfig> for TcpSocketConfig {
    fn from(value: TcpLinkConfig) -> Self {
        Self::new(
            value.tx_buffer_size,
            value.rx_buffer_size,
            value.bind_socket,
        )
    }
}
