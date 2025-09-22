#![no_std]

use zenoh_protocol::{
    core::{Locator, PriorityRange, Reliability},
    transport::BatchSize,
};
use zenoh_result::ZResult;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Link {
    pub src: Locator,
    pub dst: Locator,
    pub group: Option<Locator>,
    pub mtu: BatchSize,
    pub is_streamed: bool,
    pub auth_identifier: LinkAuthId,
    pub priorities: Option<PriorityRange>,
    pub reliability: Option<Reliability>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum LinkAuthId {
    Tcp,
    Udp,
    Serial,
}

impl LinkAuthId {
    pub fn get_cert_common_name(&self) -> Option<&str> {
        match &self {
            LinkAuthId::Tcp => None,
            LinkAuthId::Udp => None,
            LinkAuthId::Serial => None,
        }
    }
}

pub trait LocatorInspector: Default {
    fn protocol(&self) -> &str;
    fn is_reliable(&self, locator: &Locator) -> ZResult<bool>;
}
