#![no_std]

use zenoh_link_tcp::TcpLocatorInspector;
use zenoh_protocol::core::{EndPoint, Locator};
use zenoh_result::{zerror, ZResult};

pub mod unicast;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LinkKind {
    Serial,
    Tcp,
    Udp,
}

impl TryFrom<&Locator> for LinkKind {
    type Error = zenoh_result::Error;

    fn try_from(locator: &Locator) -> ZResult<Self> {
        match locator.protocol().as_str() {
            "tcp" => Ok(LinkKind::Tcp),
            "udp" => Ok(LinkKind::Udp),
            "serial" => Ok(LinkKind::Serial),

            _ => Err(zerror!("Unsupported protocol: {}", locator.protocol()).into()),
        }
    }
}

impl TryFrom<&EndPoint> for LinkKind {
    type Error = zenoh_result::Error;

    fn try_from(endpoint: &EndPoint) -> ZResult<Self> {
        Self::try_from(&endpoint.to_locator())
    }
}

#[derive(Default, Clone)]
pub struct LocatorInspector {
    tcp_inspector: TcpLocatorInspector,
    // udp_inspector: UdpLocatorInspector,
    // serial_inspector: SerialLocatorInspector,
}
impl LocatorInspector {
    pub fn is_reliable(&self, locator: &Locator) -> ZResult<bool> {
        use zenoh_link_commons::LocatorInspector;

        match LinkKind::try_from(locator)? {
            LinkKind::Tcp => self.tcp_inspector.is_reliable(locator),
            // LinkKind::Udp => self.udp_inspector.is_reliable(locator),
            // LinkKind::Serial => self.serial_inspector.is_reliable(locator),
            _ => Err(zerror!("Unsupported protocol: {}", locator.protocol()).into()),
        }
    }
}
