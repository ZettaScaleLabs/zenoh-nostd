use core::{net::SocketAddr, str::FromStr};

use zenoh_platform::tcp::PlatformTcpStream;
use zenoh_protocol::core::EndPoint;
use zenoh_result::{bail, zerr, ZResult, ZE};

use crate::unicast::LinkUnicastTcp;

pub struct LinkManagerUnicastTcp<T: PlatformTcpStream> {
    _marker: core::marker::PhantomData<T>,
}

impl<T: PlatformTcpStream> Default for LinkManagerUnicastTcp<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: PlatformTcpStream> LinkManagerUnicastTcp<T> {
    fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }

    pub async fn new_link<const N: usize, const S: usize, const D: usize>(
        &self,
        endpoint: &EndPoint<N>,
    ) -> ZResult<LinkUnicastTcp<T, S, D>> {
        let dst_addr = SocketAddr::from_str(endpoint.address().as_str())
            .map_err(|_| zerr!(ZE::InvalidAddress))?;

        let config = endpoint.config();

        if let (Some(_), Some(_)) = (config.get("iface"), config.get("bind")) {
            bail!(ZE::InvalidConfiguration)
        }

        let stream = <T as PlatformTcpStream>::connect(&dst_addr).await?;
        let src_addr = stream.local_addr()?;
        let dst_addr = stream.peer_addr()?;

        Ok(LinkUnicastTcp::new(stream, src_addr, dst_addr))
    }
}
