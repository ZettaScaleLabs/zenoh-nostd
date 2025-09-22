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
use std::str::FromStr;

use async_trait::async_trait;
use zenoh_link_commons::LocatorInspector;
use zenoh_protocol::{
    core::{Locator, Metadata, Reliability},
    transport::BatchSize,
};
use zenoh_result::{zerror, ZResult};

#[cfg(feature = "arch-std")]
mod arch_std;
#[cfg(feature = "arch-std")]
pub use arch_std::*;

pub const TCP_MAX_MTU: BatchSize = BatchSize::MAX;

pub const TCP_LOCATOR_PREFIX: &str = "tcp";

const IS_RELIABLE: bool = true;

#[derive(Default, Clone, Copy)]
pub struct TcpLocatorInspector;
#[async_trait]
impl LocatorInspector for TcpLocatorInspector {
    fn protocol(&self) -> &str {
        TCP_LOCATOR_PREFIX
    }

    async fn is_multicast(&self, _locator: &Locator) -> ZResult<bool> {
        Ok(false)
    }

    fn is_reliable(&self, locator: &Locator) -> ZResult<bool> {
        if let Some(reliability) = locator
            .metadata()
            .get(Metadata::RELIABILITY)
            .map(Reliability::from_str)
            .transpose()
            .map_err(|e| zerror!("{}", e))?
        {
            Ok(reliability == Reliability::Reliable)
        } else {
            Ok(IS_RELIABLE)
        }
    }
}

// Default MTU (TCP PDU) in bytes.
pub static TCP_DEFAULT_MTU: BatchSize = TCP_MAX_MTU;
pub static TCP_LINGER_TIMEOUT: i32 = 10;
pub static TCP_ACCEPT_THROTTLE_TIME: u64 = 100_000;
