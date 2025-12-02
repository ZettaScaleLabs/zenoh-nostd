use zenoh_proto::{
    WireExpr, keyexpr,
    msgs::{Push, PushBody, Put},
};

use crate::{Session, driver::ZDriver};

impl<T: ZDriver> Session<'_, T> {
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
