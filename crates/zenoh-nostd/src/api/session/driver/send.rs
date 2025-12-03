use ::core::ops::DerefMut;

use embassy_time::Instant;
use zenoh_proto::{exts::*, fields::*, *};

use crate::{io::transport::ZTransportTx, platform::ZPlatform};

impl<'a, Platform, TxBuf> super::DriverTx<'a, Platform, TxBuf>
where
    Platform: ZPlatform,
    TxBuf: AsMut<[u8]>,
{
    pub async fn frame(&mut self, x: impl ZFramed) -> crate::ZResult<()> {
        self.tx
            .send(self.tx_buf.as_mut(), &mut self.sn, |batch| {
                batch.frame(&x, Reliability::Reliable, QoS::default())
            })
            .await?;

        self.next_keepalive = Instant::now()
            + (self.config.mine_lease / (self.config.keep_alive as u32))
                .try_into()
                .unwrap();

        Ok(())
    }

    pub async fn unframe(&mut self, x: impl ZUnframed) -> crate::ZResult<()> {
        self.tx
            .send(self.tx_buf.as_mut(), &mut self.sn, |batch| {
                batch.unframe(&x)
            })
            .await?;

        self.next_keepalive = Instant::now()
            + (self.config.mine_lease / (self.config.keep_alive as u32))
                .try_into()
                .unwrap();

        Ok(())
    }

    pub fn next_keepalive(&self) -> Instant {
        self.next_keepalive
    }
}

impl<'a, Platform, TxBuf, RxBuf> super::Driver<'a, Platform, TxBuf, RxBuf>
where
    TxBuf: AsMut<[u8]>,
    Platform: ZPlatform,
{
    pub async fn send(&self, x: impl ZFramed) -> crate::ZResult<()> {
        let mut tx_guard = self.tx.lock().await;
        let tx = tx_guard.deref_mut();

        tx.frame(x).await
    }
}
