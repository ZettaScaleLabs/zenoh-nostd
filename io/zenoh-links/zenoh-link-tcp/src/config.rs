use core::net::SocketAddr;

use async_net::TcpStream;
use zenoh_result::{zerr, ZResult, ZE};

pub struct TcpSocketConfig {}

impl Default for TcpSocketConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl TcpSocketConfig {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn new_link(
        &self,
        dst_addr: &SocketAddr,
    ) -> ZResult<(TcpStream, SocketAddr, SocketAddr)> {
        let stream = TcpStream::connect(*dst_addr)
            .await
            .map_err(|_| zerr!(ZE::ConnectionRefused))?;

        let src_addr = stream
            .local_addr()
            .map_err(|_| zerr!(ZE::ConnectionRefused))?;

        let dst_addr = stream
            .peer_addr()
            .map_err(|_| zerr!(ZE::ConnectionRefused))?;

        Ok((stream, src_addr, dst_addr))
    }
}
