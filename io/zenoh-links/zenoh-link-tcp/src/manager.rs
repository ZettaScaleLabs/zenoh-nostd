use core::{net::SocketAddr, str::FromStr};

use zenoh_platform::{tcp::PlatformTcpStream, Platform};
use zenoh_protocol::core::EndPoint;
use zenoh_result::{bail, zctx, zerr, WithContext, ZResult, ZE};

use crate::unicast::LinkUnicastTcp;

pub struct LinkManagerUnicastTcp<'a, T: Platform> {
    platform: &'a mut T,
}

impl<'a, T: Platform> LinkManagerUnicastTcp<'a, T> {
    pub fn new(platform: &'a mut T) -> Self {
        Self { platform }
    }

    pub async fn new_link<const N: usize, const S: usize, const D: usize>(
        &mut self,
        endpoint: &EndPoint<N>,
    ) -> ZResult<LinkUnicastTcp<T::PlatformTcpStream, S, D>> {
        let dst_addr = SocketAddr::from_str(endpoint.address().as_str())
            .map_err(|_| zerr!(ZE::InvalidAddress))
            .context("parsing endpoint address")?;

        let config = endpoint.config();

        if let (Some(_), Some(_)) = (config.get("iface"), config.get("bind")) {
            bail!(ZE::InvalidConfiguration)
        }

        let stream = self
            .platform
            .new_tcp_stream(&dst_addr)
            .await
            .context(zctx!("creating platform tcp stream"))?;

        let src_addr = stream
            .local_addr()
            .context(zctx!("getting local address"))?;

        let dst_addr = stream.peer_addr().context(zctx!("getting peer address"))?;

        Ok(LinkUnicastTcp::new(stream, src_addr, dst_addr))
    }
}
