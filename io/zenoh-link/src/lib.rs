use zenoh_link_commons::LinkUnicast;
use zenoh_link_tcp::{LinkUnicastTcp, TCP_LOCATOR_PREFIX};
use zenoh_protocol::core::{EndPoint, Locator};
use zenoh_result::{zerror, Error, ZResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LinkKind {
    Tcp,
}

impl TryFrom<&Locator> for LinkKind {
    type Error = Error;

    fn try_from(value: &Locator) -> ZResult<Self> {
        match value.protocol().as_str() {
            TCP_LOCATOR_PREFIX => Ok(LinkKind::Tcp),
            _ => Err(zerror!("Unsupported locator protocol: {}", value.protocol()).into()),
        }
    }
}

impl TryFrom<&EndPoint> for LinkKind {
    type Error = Error;

    fn try_from(value: &EndPoint) -> ZResult<Self> {
        LinkKind::try_from(&value.to_locator())
    }
}

pub async fn new_link_unicast(endpoint: &EndPoint) -> ZResult<LinkUnicast> {
    match LinkKind::try_from(endpoint) {
        Ok(LinkKind::Tcp) => LinkUnicastTcp::new_link(endpoint).await,
        Err(e) => panic!("Unsupported locator protocol: {}", e),
    }
}
