use core::ops::DerefMut;

use embassy_time::Instant;
use zenoh_proto::{exts::*, fields::*, *};

use crate::{api::driver::SessionDriver, platform::Platform};

impl<T: Platform> SessionDriver<T> {
    pub(crate) async fn send(&'static self, x: impl Framed) -> ZResult<()> {
        let mut tx_guard = self.tx.lock().await;
        let tx = tx_guard.deref_mut();

        tx.tx
            .send(tx.tx_zbuf, &mut tx.sn, |batch| {
                batch.frame(&x, Reliability::Reliable, QoS::DEFAULT)
            })
            .await?;

        tx.next_keepalive = Instant::now()
            + (self.config.mine_config.mine_lease / (self.config.mine_config.keep_alive as u32))
                .try_into()
                .unwrap();

        Ok(())
    }
}
