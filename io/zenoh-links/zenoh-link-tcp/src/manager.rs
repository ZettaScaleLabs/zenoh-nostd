use std::{net::SocketAddr, str::FromStr};

use zenoh_protocol::core::EndPoint;
use zenoh_result::{bail, zerr, ZResult, ZE};

use crate::{config::TcpSocketConfig, unicast::LinkUnicastTcp};

pub struct LinkManagerUnicastTcp {}

impl Default for LinkManagerUnicastTcp {
    fn default() -> Self {
        Self::new()
    }
}

impl LinkManagerUnicastTcp {
    pub fn new() -> Self {
        LinkManagerUnicastTcp {}
    }

    pub async fn new_link<const N: usize, const S: usize, const D: usize>(
        &self,
        endpoint: &EndPoint<N>,
    ) -> ZResult<LinkUnicastTcp<S, D>> {
        let dst_addr = SocketAddr::from_str(endpoint.address().as_str())
            .map_err(|_| zerr!(ZE::InvalidAddress))?;

        let config = endpoint.config();

        if let (Some(_), Some(_)) = (config.get("iface"), config.get("bind")) {
            bail!(ZE::InvalidConfiguration)
        }

        let socket_config = TcpSocketConfig::new();

        let (stream, src_addr, dst_addr) = socket_config.new_link(&dst_addr).await?;

        Ok(LinkUnicastTcp::new(stream, src_addr, dst_addr))
    }
}
