use core::ops::DerefMut;

use embassy_time::Instant;

use crate::{
    Reliability,
    api::driver::SessionDriver,
    network::{NetworkBody, QoS},
    platform::Platform,
    result::ZResult,
};

impl<T: Platform> SessionDriver<T> {
    pub(crate) async fn send(&self, msg: NetworkBody<'_>) -> ZResult<()> {
        let mut tx_guard = self.tx.lock().await;
        let tx = tx_guard.deref_mut();

        tx.tx
            .send(tx.tx_zbuf, &mut tx.sn, |batch| {
                batch.write_msg(&msg, Reliability::Reliable, QoS::DEFAULT)
            })
            .await?;

        tx.next_keepalive = Instant::now()
            + (self.config.mine_config.mine_lease / (self.config.mine_config.keep_alive as u32))
                .try_into()
                .unwrap();

        Ok(())
    }
}
