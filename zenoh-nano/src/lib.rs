#![cfg_attr(not(feature = "arch-std"), no_std)]

use core::future::Future;
use zenoh_protocol::transport::KeepAlive;
use zenoh_result::ZResult;
use zenoh_transport::TransportLinkUnicast;

pub struct Session {
    link: TransportLinkUnicast,
}

impl Session {
    pub fn lease_task(&self) -> impl Future<Output = ZResult<()>> {
        let mut tx = self.link.tx();
        let lease = self.link.config.mine_lease.unwrap();

        async move {
            loop {
                tx.send(&KeepAlive.into()).await?;

                #[cfg(feature = "arch-std")]
                futures_timer::Delay::new(lease / 3).await;
            }
        }
    }
}

pub use zenoh_protocol::core::EndPoint;

pub mod session {
    use zenoh_link::new_link_unicast;
    use zenoh_protocol::core::EndPoint;
    use zenoh_result::ZResult;
    use zenoh_transport::open_link;

    use crate::Session;

    pub async fn open(endpoint: EndPoint) -> ZResult<Session> {
        let link = new_link_unicast(&endpoint).await?;
        let link = open_link(endpoint, link).await?;

        Ok(Session { link })
    }
}
