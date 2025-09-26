#![no_std]

use zenoh_platform::Platform;
use zenoh_protocol::{
    core::{EndPoint, Resolution, WhatAmI, ZenohIdProto},
    transport::BatchSize,
};
use zenoh_result::ZResult;

use crate::unicast::{
    link::TransportLinkUnicast,
    open::{RecvOpenAckOut, SendOpenSynOut},
    TransportManagerUnicast,
};

pub mod common;
pub mod unicast;

#[derive(Debug, Clone)]
pub struct TransportManager {
    pub zid: ZenohIdProto,
    pub whatami: WhatAmI,
    pub resolution: Resolution,
    pub batch_size: BatchSize,
    pub batching: bool,
    pub unicast: TransportManagerUnicast,
}

impl TransportManager {
    pub fn new(zid: ZenohIdProto, whatami: WhatAmI) -> Self {
        Self {
            zid,
            whatami,
            resolution: Resolution::default(),
            batch_size: BatchSize::MAX,
            batching: true,
            unicast: TransportManagerUnicast::new(),
        }
    }

    pub async fn open_transport_link_unicast<
        T: Platform,
        const L: usize,
        const N: usize,
        const S: usize,
        const D: usize,
    >(
        &self,
        endpoint: &EndPoint<N>,
    ) -> ZResult<(
        TransportLinkUnicast<T, S, D>,
        SendOpenSynOut,
        RecvOpenAckOut,
    )> {
        self.unicast
            .open_transport_link_unicast::<_, L, _, _, _>(endpoint, self)
            .await
    }
}
