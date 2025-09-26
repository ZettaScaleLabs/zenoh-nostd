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

#[derive(Debug)]
pub struct TransportManager<'a, T: Platform> {
    pub zid: ZenohIdProto,
    pub whatami: WhatAmI,
    pub resolution: Resolution,
    pub batch_size: BatchSize,
    pub batching: bool,
    pub unicast: TransportManagerUnicast<'a, T>,
}

impl<'a, T: Platform> TransportManager<'a, T> {
    pub fn new(platform: &'a mut T, zid: ZenohIdProto, whatami: WhatAmI) -> Self {
        Self {
            zid,
            whatami,
            resolution: Resolution::default(),
            batch_size: BatchSize::MAX,
            batching: true,
            unicast: TransportManagerUnicast::new(platform),
        }
    }

    pub async fn open_transport_link_unicast<
        const L: usize,
        const N: usize,
        const S: usize,
        const D: usize,
    >(
        &mut self,
        endpoint: &EndPoint<N>,
    ) -> ZResult<(
        TransportLinkUnicast<T, S, D>,
        SendOpenSynOut,
        RecvOpenAckOut,
    )> {
        self.unicast
            .open_transport_link_unicast::<L, _, _, _>(
                endpoint,
                self.batch_size,
                self.resolution,
                self.zid,
                self.whatami,
                self.unicast.lease,
            )
            .await
    }
}
