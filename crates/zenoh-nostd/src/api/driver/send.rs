use core::ops::DerefMut;

use embassy_time::{Duration, Instant};
use zenoh_proto::msgs::{NetworkBody, NetworkMessageRef};

use crate::{api::ZConfig, io::transport::ZTransportLinkTx};

impl<'res, Config> super::DriverTx<'res, Config>
where
    Config: ZConfig,
{
    async fn send<'a>(
        &mut self,
        msgs: impl Iterator<Item = NetworkBody<'a>>,
    ) -> crate::ZResult<()> {
        self.next_keepalive =
            Instant::now() + (TryInto::<Duration>::try_into(self.tx.transport.lease).unwrap() / 4);

        Ok(self.tx.send(msgs).await?)
    }

    async fn send_ref<'a>(
        &mut self,
        msgs: impl Iterator<Item = NetworkMessageRef<'a>>,
    ) -> crate::ZResult<()> {
        self.next_keepalive =
            Instant::now() + (TryInto::<Duration>::try_into(self.tx.transport.lease).unwrap() / 4);

        Ok(self.tx.send_ref(msgs).await?)
    }

    pub fn next_keepalive(&self) -> Instant {
        self.next_keepalive
    }

    pub async fn keepalive(&mut self) -> crate::ZResult<()> {
        self.next_keepalive =
            Instant::now() + (TryInto::<Duration>::try_into(self.tx.transport.lease).unwrap() / 4);

        Ok(self.tx.keepalive().await?)
    }
}

impl<'res, Config> super::Driver<'res, Config>
where
    Config: ZConfig,
{
    pub async fn send<'a>(
        &self,
        msgs: impl Iterator<Item = NetworkBody<'a>>,
    ) -> crate::ZResult<()> {
        let mut tx_guard = self.tx.lock().await;
        let tx = tx_guard.deref_mut();

        tx.send(msgs).await
    }

    pub async fn send_ref<'a>(
        &self,
        msgs: impl Iterator<Item = NetworkMessageRef<'a>>,
    ) -> crate::ZResult<()> {
        let mut tx_guard = self.tx.lock().await;
        let tx = tx_guard.deref_mut();

        tx.send_ref(msgs).await
    }
}
