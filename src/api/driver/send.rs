use core::ops::DerefMut;

use embassy_time::Instant;

use crate::{
    api::driver::SessionDriver,
    platform::{Platform, ZCommunicationError},
    protocol::{
        network::NetworkMessage,
        transport::{
            self, TransportMessage,
            frame::Frame,
        },
    },
    result::ZResult,
};

impl<T: Platform> SessionDriver<T> {
    pub async fn send(&self, msg: NetworkMessage<'_>) -> ZResult<(), ZCommunicationError> {
        let mut tx_guard = self.tx.lock().await;
        let tx = tx_guard.deref_mut();

        let frame = Frame {
            reliability: msg.reliability,
            sn: tx.sn,
            ext_qos: transport::frame::ext::QoSType::DEFAULT,
            payload: &[msg],
        };
        let tmsg = TransportMessage {
            body: transport::TransportBody::Frame(frame),
        };

        tx.tx.send(tx.tx_zbuf, &tmsg).await?;

        tx.sn = tx.sn.wrapping_add(1); // TODO wrap around Resolution
        tx.next_keepalive = Instant::now()
            + (self.config.mine_config.mine_lease / (self.config.mine_config.keep_alive as u32))
                .try_into()
                .unwrap();

        Ok(())
    }
}
