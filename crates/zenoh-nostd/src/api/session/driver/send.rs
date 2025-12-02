use embassy_time::Instant;
use zenoh_proto::{exts::*, fields::*, *};

use crate::{
    driver::{DriverTx, ZDriverTx},
    io::transport::ZTransportSend,
};

impl<TxBuf: AsMut<[u8]>, Tx: ZTransportSend> ZDriverTx for DriverTx<TxBuf, Tx> {
    async fn frame(&mut self, x: impl Framed) -> ZResult<()> {
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

    async fn unframe(&mut self, x: impl Unframed) -> ZResult<()> {
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

    fn next_keepalive(&self) -> Instant {
        self.next_keepalive
    }
}
