use zenoh_link_tcp::{manager::LinkManagerUnicastTcp, unicast::LinkUnicastTcp};
use zenoh_link_ws::{manager::LinkManagerUnicastWs, unicast::LinkUnicastWS};
use zenoh_platform::Platform;
use zenoh_protocol::core::EndPoint;
use zenoh_result::{bail, zctx, WithContext, ZResult, ZE};

use crate::LinkKind;

pub enum LinkUnicast<T: Platform, const S: usize, const D: usize> {
    LinkUnicastTcp(LinkUnicastTcp<T::PlatformTcpStream, S, D>),
    LinkUnicastWS(LinkUnicastWS<T::PlatformWSStream, S, D>),
}

impl<T: Platform, const S: usize, const D: usize> LinkUnicast<T, S, D> {
    pub fn is_streamed(&self) -> bool {
        match self {
            Self::LinkUnicastTcp(tcp) => tcp.is_streamed(),
            Self::LinkUnicastWS(ws) => ws.is_streamed(),
        }
    }

    pub fn get_mtu(&self) -> u16 {
        match self {
            Self::LinkUnicastTcp(tcp) => tcp.get_mtu(),
            Self::LinkUnicastWS(ws) => ws.get_mtu(),
        }
    }

    pub async fn write(&mut self, buffer: &[u8]) -> ZResult<usize> {
        match self {
            Self::LinkUnicastTcp(tcp) => tcp.write(buffer).await,
            Self::LinkUnicastWS(ws) => ws.write(buffer).await,
        }
    }

    pub async fn write_all(&mut self, buffer: &[u8]) -> ZResult<()> {
        match self {
            Self::LinkUnicastTcp(tcp) => tcp.write_all(buffer).await,
            Self::LinkUnicastWS(ws) => ws.write_all(buffer).await,
        }
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize> {
        match self {
            Self::LinkUnicastTcp(tcp) => tcp.read(buffer).await,
            Self::LinkUnicastWS(ws) => ws.read(buffer).await,
        }
    }

    pub async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<()> {
        match self {
            Self::LinkUnicastTcp(tcp) => tcp.read_exact(buffer).await,
            Self::LinkUnicastWS(ws) => ws.read_exact(buffer).await,
        }
    }
}

pub enum LinkManagerUnicast<'a, T: Platform> {
    LinkManagerUnicastTcp(LinkManagerUnicastTcp<'a, T>),
    LinkManagerUnicastWs(LinkManagerUnicastWs<'a, T>),
}

impl<'a, T: Platform> LinkManagerUnicast<'a, T> {
    pub fn new<const N: usize>(platform: &'a mut T, endpoint: &EndPoint<N>) -> ZResult<Self> {
        match LinkKind::try_from(endpoint)? {
            LinkKind::Tcp => Ok(LinkManagerUnicast::LinkManagerUnicastTcp(
                LinkManagerUnicastTcp::new(platform),
            )),
            LinkKind::WS => Ok(LinkManagerUnicast::LinkManagerUnicastWs(
                LinkManagerUnicastWs::new(platform),
            )),
            _ => bail!(ZE::InvalidProtocol),
        }
    }

    pub async fn new_link<const N: usize, const S: usize, const D: usize>(
        &mut self,
        endpoint: &EndPoint<N>,
    ) -> ZResult<LinkUnicast<T, S, D>> {
        match self {
            Self::LinkManagerUnicastTcp(lm) => {
                let link = lm
                    .new_link(endpoint)
                    .await
                    .context(zctx!("creating TCP link"))?;
                Ok(LinkUnicast::LinkUnicastTcp(link))
            }
            Self::LinkManagerUnicastWs(lm) => {
                let link = lm
                    .new_link(endpoint)
                    .await
                    .context(zctx!("creating WS link"))?;
                Ok(LinkUnicast::LinkUnicastWS(link))
            }
        }
    }
}
