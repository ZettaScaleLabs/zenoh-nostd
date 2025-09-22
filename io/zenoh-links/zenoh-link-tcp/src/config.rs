use core::net::SocketAddr;

use async_net::TcpStream;
use zenoh_result::{zerror, ZResult};

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
