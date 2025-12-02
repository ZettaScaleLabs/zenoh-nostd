use zenoh_proto::{msgs::*, *};

use crate::{SessionDriver, platform::Platform};

pub struct ZPublisher<T: Platform + 'static, TX: AsMut<[u8]> + 'static> {
    ke: &'static keyexpr,
    driver: &'static SessionDriver<T, TX>,
}

impl<T: Platform + 'static, TX: AsMut<[u8]>> ZPublisher<T, TX> {
    pub(crate) fn new(ke: &'static keyexpr, driver: &'static SessionDriver<T, TX>) -> Self {
        Self { ke, driver }
    }

    pub fn keyexpr(&self) -> &'static keyexpr {
        self.ke
    }

    pub async fn put(&self, bytes: &[u8]) -> crate::ZResult<()> {
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
