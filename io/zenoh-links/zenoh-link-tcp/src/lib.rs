#![cfg_attr(not(feature = "arch-std"), no_std)]

use core::str::FromStr;

use zenoh_link_commons::LocatorInspector;
use zenoh_protocol::core::{Locator, Metadata, Reliability};
use zenoh_result::{zerror, ZResult};

pub mod config;
pub mod manager;
pub mod unicast;

#[derive(Default, Clone, Copy)]
pub struct TcpLocatorInspector;

impl LocatorInspector for TcpLocatorInspector {
    fn protocol(&self) -> &str {
        "tcp"
    }

    fn is_reliable(&self, locator: &Locator) -> ZResult<bool> {
        if let Some(reliability) = locator
            .metadata()
            .get(Metadata::RELIABILITY)
            .map(Reliability::from_str)
            .transpose()
            .map_err(|e| zerror!("Invalid reliability in locator metadata: {}", e))?
        {
            Ok(reliability == Reliability::Reliable)
        } else {
            Ok(true)
        }
    }
}
