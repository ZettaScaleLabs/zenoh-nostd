use std::{net::SocketAddr, str::FromStr};

use zenoh_protocol::core::EndPoint;
use zenoh_result::{bail, zerror, ZResult};

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

    pub async fn new_link(&self, endpoint: &EndPoint) -> ZResult<LinkUnicastTcp> {
        let dst_addr = SocketAddr::from_str(endpoint.address().as_str())
            .map_err(|e| zerror!("Can not parse the given address {}: {}", endpoint, e))?;

        let config = endpoint.config();

        if let (Some(_), Some(_)) = (config.get("iface"), config.get("bind")) {
            bail!(
                "Using Config options `iface` and `bind` in conjunction is unsupported at this time {} {:?}",
                "iface",
                "bind"
            )
        }

        let socket_config = TcpSocketConfig::new();

        let (stream, src_addr, dst_addr) = socket_config.new_link(&dst_addr).await?;

        Ok(LinkUnicastTcp::new(stream, src_addr, dst_addr))
    }
}
