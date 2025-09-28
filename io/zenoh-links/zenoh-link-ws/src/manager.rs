use core::{net::SocketAddr, str::FromStr};

use zenoh_platform::{ws::PlatformWSStream, Platform};
use zenoh_protocol::core::EndPoint;
use zenoh_result::{bail, zctx, zerr, WithContext, ZResult, ZE};

use crate::unicast::LinkUnicastWS;

pub struct LinkManagerUnicastWs<'a, T: Platform> {
    platform: &'a mut T,
}

impl<'a, T: Platform> LinkManagerUnicastWs<'a, T> {
    pub fn new(platform: &'a mut T) -> Self {
        Self { platform }
    }

    pub async fn new_link<const N: usize, const S: usize, const D: usize>(
        &mut self,
        endpoint: &EndPoint<N>,
    ) -> ZResult<LinkUnicastWS<T::PlatformWSStream, S, D>> {
        let dst_addr = SocketAddr::from_str(endpoint.address().as_str())
            .map_err(|_| zerr!(ZE::InvalidAddress))?;

        let config = endpoint.config();

        if let (Some(_), Some(_)) = (config.get("iface"), config.get("bind")) {
            bail!(ZE::InvalidConfiguration)
        }

        let stream = self
            .platform
            .new_ws_stream(&dst_addr)
            .await
            .context(zctx!("creating new platform WS stream"))?;

        let src_addr = stream
            .local_addr()
            .context(zctx!("getting local addr from WS stream"))?;

        let dst_addr = stream
            .peer_addr()
            .context(zctx!("getting peer addr from WS stream"))?;

        Ok(LinkUnicastWS::new(stream, src_addr, dst_addr))
    }
}
