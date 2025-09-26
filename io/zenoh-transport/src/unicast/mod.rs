use core::time::Duration;

use embassy_futures::select::select;
use embassy_time::Timer;

pub use establishment::*;
use zenoh_link::unicast::LinkManagerUnicast;
use zenoh_platform::Platform;
use zenoh_protocol::{
    core::{EndPoint, Resolution, WhatAmI, ZenohIdProto},
    transport::BatchSize,
};
use zenoh_result::{zctx, zerr, WithContext, ZResult, ZE};

use crate::unicast::{
    link::TransportLinkUnicast,
    open::{RecvOpenAckOut, SendOpenSynOut},
};

pub mod establishment;
pub mod link;

#[derive(Debug)]
pub struct TransportManagerUnicast<'a, T: Platform> {
    pub lease: Duration,
    pub keep_alive: usize,
    pub open_timeout: Duration,
    pub accept_timeout: Duration,
    pub accept_pending: usize,
    pub max_sessions: usize,
    pub is_qos: bool,
    pub is_lowlatency: bool,
    pub platform: &'a mut T,
}

impl<'a, T: Platform> TransportManagerUnicast<'a, T> {
    pub fn new(platform: &'a mut T) -> Self {
        Self {
            platform,
            lease: Duration::from_secs(15),
            keep_alive: 3,
            open_timeout: Duration::from_secs(5),
            accept_timeout: Duration::from_secs(5),
            accept_pending: 10,
            max_sessions: 100,
            is_qos: false,
            is_lowlatency: false,
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
        batch_size: BatchSize,
        resolution: Resolution,
        zid: ZenohIdProto,
        whatami: WhatAmI,
        lease: Duration,
    ) -> ZResult<(
        TransportLinkUnicast<T, S, D>,
        SendOpenSynOut,
        RecvOpenAckOut,
    )> {
        match select(Timer::after(self.open_timeout.try_into().unwrap()), async {
            let mut lm = LinkManagerUnicast::new(self.platform, endpoint)
                .context(zctx!("creating link manager"))?;

            let link = lm
                .new_link(endpoint)
                .await
                .context(zctx!("creating unicast link"))?;

            establishment::open::open_unicast_link::<_, L, _, _>(
                link, batch_size, resolution, zid, whatami, lease,
            )
            .await
        })
        .await
        {
            embassy_futures::select::Either::First(_) => Err(zerr!(ZE::Timeout).into()),
            embassy_futures::select::Either::Second(result) => result,
        }
    }
}
