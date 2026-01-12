use core::ops::DerefMut;

use embassy_time::Instant;
use zenoh_proto::{exts::*, fields::*, *};

use crate::{api::ZConfig, io::transport::ZTransportTx};

impl<'transport, Config> super::DriverTx<'transport, Config>
where
    Config: ZConfig,
{
    pub async fn framed(&mut self, x: impl ZFramed) -> crate::ZResult<()> {
        self.tx
            .send(self.tx_buf.as_mut(), &mut self.sn, |batch| {
                batch.framed(&x, Reliability::Reliable, QoS::default())?;
                Ok(())
            })
            .await?;

        self.next_keepalive =
            Instant::now() + (self.config.mine_lease / (self.config.keep_alive as u32));

        Ok(())
    }

    pub async fn unframed(&mut self, x: impl ZUnframed) -> crate::ZResult<()> {
        self.tx
            .send(self.tx_buf.as_mut(), &mut self.sn, |batch| {
                batch.unframed(&x)?;
                Ok(())
            })
            .await?;

        self.next_keepalive =
            Instant::now() + (self.config.mine_lease / (self.config.keep_alive as u32));

        Ok(())
    }

    pub fn next_keepalive(&self) -> Instant {
        self.next_keepalive
    }
}

impl<'transport, Config> super::Driver<'transport, Config>
where
    Config: ZConfig,
{
    pub async fn send(&self, x: impl ZFramed) -> crate::ZResult<()> {
        let mut tx_guard = self.tx.lock().await;
        let tx = tx_guard.deref_mut();

        tx.framed(x).await
    }
}
