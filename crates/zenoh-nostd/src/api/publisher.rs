use zenoh_proto::{msgs::*, *};

use crate::{SessionDriver, platform::Platform};

pub struct ZPublisher<T: Platform + 'static> {
    ke: &'static keyexpr,
    driver: &'static SessionDriver<T>,
}

impl<T: Platform + 'static> ZPublisher<T> {
    pub(crate) fn new(ke: &'static keyexpr, driver: &'static SessionDriver<T>) -> Self {
        Self { ke, driver }
    }

    pub fn keyexpr(&self) -> &'static keyexpr {
        self.ke
    }

    pub async fn put(&self, bytes: &[u8]) -> zenoh_proto::ZResult<()> {
        let msg = Push {
            wire_expr: WireExpr::from(self.ke),
            payload: PushBody::Put(Put {
                payload: bytes,
                ..Default::default()
            }),
            ..Default::default()
        };

        self.driver.send(msg).await
    }
}
