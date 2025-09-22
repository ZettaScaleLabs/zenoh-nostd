//
// Copyright (c) 2023 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//

//! ⚠️ WARNING ⚠️
//!
//! This crate is intended for Zenoh's internal use.
//!
//! [Click here for Zenoh's documentation](https://docs.rs/zenoh/latest/zenoh)
#![no_std]
extern crate alloc;

mod unicast;

use alloc::{
    boxed::Box,
    string::{String, ToString},
};
use core::{cmp::PartialEq, fmt, hash::Hash};

use async_trait::async_trait;
pub use unicast::*;
use zenoh_protocol::{
    core::{Locator, Metadata, PriorityRange, Reliability},
    transport::BatchSize,
};
use zenoh_result::ZResult;

/*************************************/
/*            GENERAL                */
/*************************************/

pub const BIND_SOCKET: &str = "bind";
pub const BIND_INTERFACE: &str = "iface";
pub const TCP_SO_SND_BUF: &str = "so_sndbuf";
pub const TCP_SO_RCV_BUF: &str = "so_rcvbuf";
pub const DSCP: &str = "dscp";

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

#[async_trait]
pub trait LocatorInspector: Default {
    fn protocol(&self) -> &str;
    async fn is_multicast(&self, locator: &Locator) -> ZResult<bool>;
    fn is_reliable(&self, locator: &Locator) -> ZResult<bool>;
}

pub trait ConfigurationInspector<C>: Default {
    fn inspect_config(&self, configuration: &C) -> ZResult<String>;
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} => {}", &self.src, &self.dst)
    }
}

impl Link {
    pub fn new_unicast(
        link: &LinkUnicast,
        priorities: Option<PriorityRange>,
        reliability: Option<Reliability>,
    ) -> Self {
        Link {
            src: Self::to_patched_locator(link.get_src(), priorities.as_ref(), reliability),
            dst: Self::to_patched_locator(link.get_dst(), priorities.as_ref(), reliability),
            group: None,
            mtu: link.get_mtu(),
            is_streamed: link.is_streamed(),
            auth_identifier: link.get_auth_id().clone(),
            priorities,
            reliability,
        }
    }

    /// Updates the metadata of the `locator` with `priorities` and `reliability`.
    fn to_patched_locator(
        locator: &Locator,
        priorities: Option<&PriorityRange>,
        reliability: Option<Reliability>,
    ) -> Locator {
        let mut locator = locator.clone();
        let mut metadata = locator.metadata_mut();
        reliability
            .map(|r| metadata.insert(Metadata::RELIABILITY, r.to_string()))
            .transpose()
            .expect("adding `reliability` to Locator metadata should not fail");
        priorities
            .map(|ps| metadata.insert(Metadata::PRIORITIES, ps.to_string()))
            .transpose()
            .expect("adding `priorities` to Locator metadata should not fail");
        locator
    }
}
