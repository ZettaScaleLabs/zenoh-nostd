mod link;
use std::time::Duration;

use embassy_futures::select::select;
use embassy_time::Timer;
pub use link::*;

mod establishment;
pub use establishment::*;
use zenoh_link::unicast::LinkManagerUnicast;
use zenoh_protocol::core::EndPoint;
use zenoh_result::{zerror, ZResult};

use crate::{
    unicast::open::{RecvOpenAckOut, SendOpenSynOut},
    TransportManager,
};

#[derive(Debug, Clone)]
pub struct TransportManagerUnicast {
    pub lease: Duration,
    pub keep_alive: usize,
    pub open_timeout: Duration,
    pub accept_timeout: Duration,
    pub accept_pending: usize,
    pub max_sessions: usize,
    pub is_qos: bool,
    pub is_lowlatency: bool,
}

impl TransportManagerUnicast {
    pub fn new() -> Self {
        Self {
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

    pub async fn open_transport_link_unicast(
        &self,
        endpoint: &EndPoint,
        tm: &TransportManager,
    ) -> ZResult<(TransportLinkUnicast, SendOpenSynOut, RecvOpenAckOut)> {
        match select(Timer::after(self.open_timeout.try_into().unwrap()), async {
            let lm = LinkManagerUnicast::new(endpoint)?;
            let link = lm.new_link(endpoint).await?;

            establishment::open::open_link(endpoint, link, tm).await
        })
        .await
        {
            embassy_futures::select::Either::First(_) => Err(zerror!(
                "Timeout opening transport link unicast to endpoint {}",
                endpoint
            )
            .into()),
            embassy_futures::select::Either::Second(result) => result,
        }
    }
}
