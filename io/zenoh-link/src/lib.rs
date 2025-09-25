#![cfg_attr(
    not(any(target_os = "linux", target_os = "macos", target_os = "windows",)),
    no_std
)]

use zenoh_protocol::core::{EndPoint, Locator};
use zenoh_result::{bail, ZError, ZResult, ZE};

pub mod unicast;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LinkKind {
    Serial,
    Tcp,
    Udp,
}

impl<const N: usize> TryFrom<&Locator<N>> for LinkKind {
    type Error = ZError;

    fn try_from(locator: &Locator<N>) -> ZResult<Self> {
        match locator.protocol().as_str() {
            "tcp" => Ok(LinkKind::Tcp),
            "udp" => Ok(LinkKind::Udp),
            "serial" => Ok(LinkKind::Serial),

            _ => bail!(ZE::InvalidProtocol),
        }
    }
}

impl<const N: usize> TryFrom<&EndPoint<N>> for LinkKind {
    type Error = ZError;

    fn try_from(endpoint: &EndPoint<N>) -> ZResult<Self> {
        Self::try_from(&endpoint.to_locator())
    }
}
