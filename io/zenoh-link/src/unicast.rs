use zenoh_link_tcp::{manager::LinkManagerUnicastTcp, unicast::LinkUnicastTcp};
use zenoh_protocol::core::EndPoint;
use zenoh_result::{bail, ZResult, ZE};

use crate::LinkKind;

pub enum LinkUnicast<const S: usize, const D: usize> {
    LinkUnicastTcp(LinkUnicastTcp<S, D>),
}

impl<const S: usize, const D: usize> LinkUnicast<S, D> {
    pub fn is_streamed(&self) -> bool {
        match self {
            Self::LinkUnicastTcp(tcp) => tcp.is_streamed(),
        }
    }

    pub fn get_mtu(&self) -> u16 {
        match self {
            Self::LinkUnicastTcp(tcp) => tcp.get_mtu(),
        }
    }

    pub async fn write(&mut self, buffer: &[u8]) -> ZResult<usize> {
        match self {
            Self::LinkUnicastTcp(tcp) => tcp.write(buffer).await,
        }
    }

    pub async fn write_all(&mut self, buffer: &[u8]) -> ZResult<()> {
        match self {
            Self::LinkUnicastTcp(tcp) => tcp.write_all(buffer).await,
        }
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize> {
        match self {
            Self::LinkUnicastTcp(tcp) => tcp.read(buffer).await,
        }
    }

    pub async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<()> {
        match self {
            Self::LinkUnicastTcp(tcp) => tcp.read_exact(buffer).await,
        }
    }
}

pub enum LinkManagerUnicast {
    LinkManagerUnicastTcp(LinkManagerUnicastTcp),
}

impl LinkManagerUnicast {
    pub fn new<const N: usize>(endpoint: &EndPoint<N>) -> ZResult<Self> {
        match LinkKind::try_from(endpoint)? {
            LinkKind::Tcp => Ok(LinkManagerUnicast::LinkManagerUnicastTcp(
                LinkManagerUnicastTcp::new(),
            )),
            _ => bail!(ZE::InvalidProtocol),
        }
    }

    pub async fn new_link<const N: usize, const S: usize, const D: usize>(
        &self,
        endpoint: &EndPoint<N>,
    ) -> ZResult<LinkUnicast<S, D>> {
        match self {
            Self::LinkManagerUnicastTcp(lm) => {
                let link = lm.new_link(endpoint).await?;
                Ok(LinkUnicast::LinkUnicastTcp(link))
            }
        }
    }
}
