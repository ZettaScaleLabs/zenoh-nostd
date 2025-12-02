use zenoh_proto::{
    WireExpr, keyexpr,
    msgs::{Push, PushBody, Put},
};

use crate::{
    api::driver::{Driver, DriverTx},
    io::transport::TransportTx,
    platform::ZPlatform,
};

impl<'a, Platform, TxBuf, Rx>
    super::Session<'a, Driver<DriverTx<TxBuf, TransportTx<'a, Platform>>, Rx>>
where
    Platform: ZPlatform,
    TxBuf: AsMut<[u8]>,
{
    pub async fn put(&self, ke: &keyexpr, bytes: &[u8]) -> crate::ZResult<()> {
        let msg = Push {
            wire_expr: WireExpr::from(ke),
            payload: PushBody::Put(Put {
                payload: bytes,
                ..Default::default()
            }),
            ..Default::default()
        };

        self.driver.send(msg).await?;

        Ok(())
    }
}
